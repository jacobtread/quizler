use crate::{
    game::{
        GameRef,
        manager::{Games, InitializedMessage},
    },
    msg::{
        AnswerMessage, ClientMessage, ConnectMessage, HostActionMessage, InitializeMessage,
        JoinMessage, KickMessage, ReadyMessage, ResponseMessage, ServerEvent, ServerMessage,
        ServerResponse,
    },
    session_store::{SessionStore, SessionStoreMessage},
    types::{GameToken, RemoveReason, ServerError},
};
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use bytes::Bytes;
use futures_util::{StreamExt, future::BoxFuture, stream::SplitStream};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::mpsc,
    time::{MissedTickBehavior, interval},
};
use uuid::Uuid;

/// Type alias for numbers that represent Session ID's
pub type SessionId = Uuid;

/// Structure of a session connected to the server
pub struct Session {
    /// Unique ID of the session
    id: SessionId,
    /// Token of the current game this session is in
    game: Option<GameRef>,

    /// Last heartbeat received from the client
    hb: Instant,
    /// Current session socket data
    socket: Option<SessionSocket>,

    /// Receiver for receiving server events
    rx: mpsc::UnboundedReceiver<Arc<ServerEvent>>,

    /// Receiver for messages from the session store (Resumptions)
    session_store_rx: mpsc::UnboundedReceiver<SessionStoreMessage>,

    /// Sender for server events
    tx: EventTarget,

    /// Future for when session resumption has timed out, if the
    /// socket is connected this future is a infinitely pending one
    resumption_timeout: BoxFuture<'static, ()>,
}

/// [WebSocket] that has been split into a stream and sink that is
/// automatically fed from messages sent into `ws_tx`
struct SessionSocket {
    /// Sender to send messages to the websocket
    ws_tx: mpsc::UnboundedSender<Message>,
    /// Stream to receive messages from the websocket
    ws_stream: SplitStream<WebSocket>,
}

impl SessionSocket {
    fn new(socket: WebSocket) -> SessionSocket {
        let (ws_tx, ws_rx) = mpsc::unbounded_channel::<Message>();
        let outgoing_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(ws_rx).map(Ok);

        let (ws_sink, ws_stream) = socket.split();

        // Forward all outgoing messages to the outgoing stream
        tokio::spawn(async {
            _ = outgoing_stream.forward(ws_sink).await;
        });

        SessionSocket { ws_tx, ws_stream }
    }
}

// Time intervals to check heartbeats
const HB_INTERVAL: Duration = Duration::from_secs(5);
// Timeout for handling loss of connection
const TIMEOUT: Duration = Duration::from_secs(15);
// Allow sessions to continue existing for 3 minutes after the
// socket connection was lost to allow resuming the session
const SESSION_RESUME_TIMEOUT: Duration = Duration::from_secs(60 * 3);

/// Accepts a message from the socket stream if the socket is available
/// or provides a pending future
async fn socket_recv_or_pending(
    session_socket: Option<&mut SessionSocket>,
) -> Option<Result<Message, axum::Error>> {
    match session_socket {
        Some(socket) => socket.ws_stream.next().await,
        _ => std::future::pending().await,
    }
}

impl Session {
    /// Starts a new session from the provided socket and session store
    pub async fn start(socket: WebSocket) {
        let (mut session, token) = Self::new(socket);
        log::debug!("Starting new session {}", session.id);

        // Notify the session of its resumption token
        _ = session.send(&ServerEvent::ResumptionToken { token });

        session.process().await;
    }

    /// Create a new session instance
    fn new(socket: WebSocket) -> (Self, String) {
        let (tx, rx) = mpsc::unbounded_channel();

        let id = Uuid::new_v4();
        let hb = Instant::now();
        let session_socket = SessionSocket::new(socket);

        let (session_store_tx, session_store_rx) = mpsc::unbounded_channel();

        // Register the message sender with the session store
        let session_store = SessionStore::global();
        let token = session_store.add_session(id, session_store_tx);

        let this = Self {
            id,
            game: None,
            hb,
            socket: Some(session_socket),
            rx,
            tx: EventTarget(tx),
            session_store_rx,
            resumption_timeout: Box::pin(std::future::pending()),
        };

        (this, token)
    }

    /// Handles processing all events for the session
    async fn process(mut self) {
        // Heartbeat interval ticking
        let mut hb_interval = interval(HB_INTERVAL);
        hb_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            select! {
                // Handle incoming server events to send to the client
                // (Impossible to get None on this branch as the session owns a copy of its sender)
                Some(event) = self.rx.recv() => {
                    if self.handle_event(event).is_err() {
                        // Failed to send the response
                        break;
                    }
                }

                // Handle socket messages if the socket is active
                req = socket_recv_or_pending(self.socket.as_mut()), if self.socket.is_some() => {
                    let msg_result = match req {
                        Some(value) => value,
                        // Socket has become closed
                        None => {
                            self.socket_disconnected();
                            continue;
                        },
                    };

                    let msg: Message = match msg_result {
                        Ok(value) => value,
                        // Error while reading body (Skip the message)
                        Err(_) => continue,
                    };

                    // Close messages are handled ahead of time since they directly
                    // affect whether we continue or stop the loop
                    if matches!(&msg, Message::Close(_)) {
                        // In debug close triggers resumption logic to allow testing
                        // session resumption
                        #[cfg(debug_assertions)]
                        {
                            self.socket_disconnected();
                            continue;
                        }


                        // In real world situations a close is treated as the end
                        // of the connection
                        #[cfg(not(debug_assertions))]
                        break;
                    }

                    if self.handle_message(msg).is_err() {
                         break;
                    }
                }

                // If we are disconnected run the resumption timeout future
                _ = &mut self.resumption_timeout, if self.socket.is_none()  => {
                    log::debug!("Session connection lost and exceeded resumption window, closing: {}", self.id);
                    break;
                }

                // If we are disconnected wait for messages from the session store to handle a reconnect
                store_msg = self.session_store_rx.recv(), if self.socket.is_none() => {
                    let socket = match store_msg {
                        Some(SessionStoreMessage::Reconnect { socket }) => socket,
                        // For future variants:
                        // Some(_) => continue,
                        // Session store has closed our channel no possible way to resume
                        None => break,
                    };

                    // We are reconnected
                    self.resume(socket);
                }

                // Heartbeat, only if we are connected
                _ = hb_interval.tick(), if self.socket.is_some() => {
                    if !self.heartbeat() {
                        // Socket heartbeat has failed, consider the socket dead
                        self.socket_disconnected();
                        continue;
                    }
                }
            };
        }
        self.cleanup();
    }

    /// Handles the socket disconnecting
    fn socket_disconnected(&mut self) {
        self.socket = None;
        self.resumption_timeout = Box::pin(tokio::time::sleep(SESSION_RESUME_TIMEOUT));
    }

    /// Resumes the session using a new `socket` connection.
    ///
    /// Resume all information about the session sending all the
    /// current details to the player
    fn resume(&mut self, socket: WebSocket) {
        log::debug!("resumed session with new socket {}", self.id);

        let session_socket = SessionSocket::new(socket);

        self.hb = Instant::now();
        self.socket = Some(session_socket);
        self.resumption_timeout = Box::pin(std::future::pending());

        if let Some(game) = self.game.as_ref() {
            let game = game.read();
            game.resume_player(self.id);
        }
    }

    /// Heartbeat returns false if connection is failed
    fn heartbeat(&mut self) -> bool {
        let socket = match &mut self.socket {
            Some(socket) => socket,
            _ => return false,
        };

        let elapsed = self.hb.elapsed();
        if elapsed >= TIMEOUT {
            // Connection lost timeout
            return false;
        }

        socket.ws_tx.send(Message::Ping(Bytes::new())).is_ok()
    }

    /// Handles cleaning up the session after processing has
    /// terminated.
    ///
    /// Removes the player from its game if its present
    fn cleanup(&mut self) {
        log::debug!("Session stopped: {}", self.id);

        let session_store = SessionStore::global();
        session_store.remove_session(self.id);

        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            let mut lock = game.write();

            // Inform game to remove self
            let _ = lock.remove_player(self.id, self.id, RemoveReason::LostConnection);
        }
    }

    /// Handles server events received by this session, processes the
    /// events then sends them to the client
    ///
    /// # Arguments
    /// * event - The event to handle
    fn handle_event(&mut self, event: Arc<ServerEvent>) -> Result<(), axum::Error> {
        let value = event.as_ref();

        // Ensure we drop our reference to the game when kicked
        if let ServerEvent::Kicked { id, .. } = value
            && self.id.eq(id)
        {
            self.game = None;
        }

        self.send(value)
    }

    /// Handles processing websocket messages, updating heartbeat, and forwarding
    /// along parsed messages to handle_request
    ///
    /// # Arguments
    /// * msg - The websocket message
    fn handle_message(&mut self, msg: Message) -> Result<(), axum::Error> {
        // Update heartbeat
        self.hb = Instant::now();

        // Handle different message types
        match msg {
            Message::Text(text) => {
                let res = match self.handle_text_message(text) {
                    Ok(value) => ServerResponse(value),
                    Err(error) => ServerResponse(ResponseMessage::Error { error }),
                };

                self.send(&res)?;
                Ok(())
            }

            Message::Ping(ping) => {
                self.handle_ping_message(ping)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }

    /// Handles ping messages by responding with a pong
    fn handle_ping_message(&mut self, ping: Bytes) -> Result<(), axum::Error> {
        let socket = self
            .socket
            .as_mut()
            .expect("should never be able to receive a ping message without a socket first");

        _ = socket.ws_tx.send(Message::Pong(ping));

        Ok(())
    }

    /// Handles a text base message
    fn handle_text_message(&mut self, message: Utf8Bytes) -> Result<ResponseMessage, ServerError> {
        let client_message = serde_json::from_str::<ClientMessage>(&message)
            .inspect_err(|err| log::error!("Unable to decode client message: {}", err))
            .map_err(|_| ServerError::MalformedMessage)?;

        client_message.handle(self)
    }

    /// Converts the provided message to JSON writing it as a text frame
    /// to the websocket
    ///
    /// # Arguments
    /// * msg - The message to send
    fn send<S: ServerMessage>(&mut self, msg: &S) -> Result<(), axum::Error> {
        let value = serde_json::to_string(msg).map_err(|err| axum::Error::new(Box::new(err)))?;
        let message = Message::Text(value.into());

        if let Some(socket) = self.socket.as_mut() {
            _ = socket.ws_tx.send(message);
        }

        Ok(())
    }

    /// Disconnects the session from their current game if they
    /// are already in one
    fn disconnect(&mut self) {
        let game = match self.game.take() {
            Some(value) => value,
            None => return,
        };

        // If already in a game inform the game that we've left
        let mut lock = game.write();
        let _ = lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
    }
}

trait SessionMessageHandler {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError>;
}

impl SessionMessageHandler for ClientMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        match self {
            ClientMessage::Initialize(msg) => msg.handle(session),
            ClientMessage::Connect(msg) => msg.handle(session),
            ClientMessage::Join(msg) => msg.handle(session),
            ClientMessage::Ready(msg) => msg.handle(session),
            ClientMessage::HostAction(msg) => msg.handle(session),
            ClientMessage::Answer(msg) => msg.handle(session),
            ClientMessage::Kick(msg) => msg.handle(session),
        }
    }
}

impl SessionMessageHandler for InitializeMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        session.disconnect();

        let msg: InitializedMessage = Games::initialize(self.uuid, session.id, session.tx.clone())?;
        session.game = Some(msg.game);

        Ok(ResponseMessage::Joined {
            id: session.id,
            config: msg.config,
            token: msg.token,
        })
    }
}

impl SessionMessageHandler for ConnectMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        session.disconnect();

        let token: GameToken = self.token.parse()?;
        let game = Games::get_game(&token).ok_or(ServerError::InvalidToken)?;

        session.game = Some(game);
        Ok(ResponseMessage::Ok)
    }
}

impl SessionMessageHandler for JoinMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        let game = session.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write();
        let msg = game.join(session.id, session.tx.clone(), self.name)?;

        Ok(ResponseMessage::Joined {
            id: session.id,
            token: msg.token,
            config: msg.config,
        })
    }
}

impl SessionMessageHandler for ReadyMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        let game = session.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write();
        game.ready(session.id);
        Ok(ResponseMessage::Ok)
    }
}

impl SessionMessageHandler for HostActionMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        let game = session.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write();

        game.host_action(session.id, self.action)?;
        Ok(ResponseMessage::Ok)
    }
}

impl SessionMessageHandler for AnswerMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        let game = session.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write();

        game.answer(session.id, self.answer)?;
        Ok(ResponseMessage::Ok)
    }
}

impl SessionMessageHandler for KickMessage {
    fn handle(self, session: &mut Session) -> Result<ResponseMessage, ServerError> {
        let game = session.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write();

        game.remove_player(session.id, self.id, RemoveReason::RemovedByHost)?;
        Ok(ResponseMessage::Ok)
    }
}

/// Wrapper around the session sender to allow sending server
/// events to the sessions
#[derive(Clone)]
pub struct EventTarget(mpsc::UnboundedSender<Arc<ServerEvent>>);

impl EventTarget {
    /// Sends a server event to the event target
    ///
    /// # Arguments
    /// * event - The server event to send
    #[inline]
    pub fn send(&self, event: Arc<ServerEvent>) {
        _ = self.0.send(event);
    }

    /// Same as [`EventTarget::send`] except takes an
    /// owned instanced of [ServerEvent] rather than
    /// a shared reference
    ///
    /// # Arguments
    /// * event - The server event to send
    #[inline]
    pub fn send_owned(&self, event: ServerEvent) {
        self.send(Arc::new(event));
    }
}

use crate::{
    game::GameRef,
    games::{Games, InitializedMessage},
    msg::{ClientMessage, ResponseMessage, ServerEvent, ServerResponse},
    session_store::{SessionStore, SessionStoreMessage},
    types::{Answer, GameToken, HostAction, RemoveReason, ServerError},
};
use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use futures_util::future::BoxFuture;
use log::{debug, error};
use serde::Serialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
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
    /// The socket the session is connected to
    socket: Option<WebSocket>,

    /// Receiver for receiving server events
    rx: mpsc::UnboundedReceiver<Arc<ServerEvent>>,

    /// Receiver for messages from the session store (Resumptions)
    store_rx: mpsc::UnboundedReceiver<SessionStoreMessage>,
    /// Access to the session store to revoke access after destruction
    session_store: Arc<SessionStore>,

    /// Sender for server events
    tx: EventTarget,
}

// Time intervals to check heartbeats
const HB_INTERVAL: Duration = Duration::from_secs(5);
// Timeout for handling loss of connection
const TIMEOUT: Duration = Duration::from_secs(15);
// Allow sessions to continue existing for 3 minutes after the
// socket connection was lost to allow resuming the session
const SESSION_RESUME_TIMEOUT: Duration = Duration::from_secs(60 * 3);

async fn socket_recv_or_pending(
    session_socket: Option<&mut WebSocket>,
) -> Option<Result<Message, axum::Error>> {
    match session_socket {
        Some(socket) => socket.recv().await,
        _ => std::future::pending().await,
    }
}

#[derive(Debug, Error)]
#[error("impossible state, ping received without a socket")]
struct PingOnDeadSocket;

impl Session {
    /// Handler for starting a new session from the provided websocket
    ///
    /// # Arguments
    /// * socket - The websocket to use for the session
    pub async fn start(socket: WebSocket, session_store: Arc<SessionStore>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let (store_tx, store_rx) = mpsc::unbounded_channel();
        let id = Uuid::new_v4();
        debug!("Starting new session {}", id);
        let hb = Instant::now();
        let mut this = Self {
            id,
            game: None,
            hb,
            socket: Some(socket),
            rx,
            tx: EventTarget(tx),
            store_rx,
            session_store,
        };

        // Register the message sender with the session store
        let token = this.session_store.add_session(id, store_tx);

        // Notify the session of its resumption token
        _ = this.send(&ServerEvent::ResumptionToken { token }).await;

        this.process().await;
    }

    /// Handles processing all events for the session
    async fn process(mut self) {
        // Heartbeat interval ticking
        let mut hb_interval = interval(HB_INTERVAL);
        hb_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        // Timeout for resumption
        let mut resumption_timeout: BoxFuture<'static, ()> = Box::pin(std::future::pending());

        loop {
            select! {
                // Server events
                event = self.rx.recv() => {
                    let event = match event {
                        Some(event) => event,
                        None => break,
                    };

                    if self.handle_event(event).await.is_err() {
                        // Failed to send the response
                        break;
                    }
                }

                // Handle socket messages if the socket is active
                req = socket_recv_or_pending(self.socket.as_mut()) => {
                    let msg: Message = match req {
                        Some(Ok(value)) => value,
                        // Error while reading body (Skip the message)
                        Some(Err(_)) => continue,
                        // Socket has become closed
                        None => {
                            self.socket = None;
                            resumption_timeout = Box::pin(tokio::time::sleep(SESSION_RESUME_TIMEOUT));
                            continue;
                        },
                    };

                    match self.handle_message(msg).await {
                        Err(_) => break,
                        Ok(false)  => {
                            // In debug close triggers resumption logic to allow testing
                            // session resumption
                            #[cfg(debug_assertions)]
                            continue;

                            // In real world situations a close is treated as the end
                            // of the connection
                            #[cfg(not(debug_assertions))]
                            break;
                        },
                        Ok(true )=> {}
                    }
                }

                // If we are disconnected run the resumption timeout future
                _ = &mut resumption_timeout, if self.socket.is_none()  => {
                    debug!("Session connection lost and exceeded resumption window, closing: {}", self.id);
                    break;
                }

                // If we are disconnected wait for messages from the session store to handle a reconnect
                store_msg = self.store_rx.recv(), if self.socket.is_none() => {
                    let socket = match store_msg {
                        Some(SessionStoreMessage::Reconnect { socket }) => socket,
                        // For future variants:
                        // Some(_) => continue,
                        // Session store has closed our channel no possible way to resume
                        None => break,
                    };

                    // We are reconnected
                    self.socket = Some(socket);
                    self.hb = Instant::now();

                    self.resume().await;
                }

                // Heartbeat, only if we are connected
                _ = hb_interval.tick(), if self.socket.is_some() => {
                    if !self.heartbeat().await {
                        // Socket heartbeat has failed, consider the socket dead
                        self.socket = None;
                        resumption_timeout = Box::pin(tokio::time::sleep(SESSION_RESUME_TIMEOUT));
                        continue;
                    }
                }
            };
        }
        self.cleanup().await;
    }

    /// Resume all information about the session sending all the
    /// current details to the player
    async fn resume(&mut self) {
        if let Some(game) = self.game.as_ref() {
            let game = game.read().await;
            game.resume_player(self.id);
        }
    }

    /// Heartbeat returns false if connection is failed
    async fn heartbeat(&mut self) -> bool {
        let socket = match &mut self.socket {
            Some(socket) => socket,
            _ => return false,
        };

        let elapsed = self.hb.elapsed();
        if elapsed >= TIMEOUT {
            // Connection lost timeout
            false
        } else {
            socket.send(Message::Ping(Bytes::new())).await.is_ok()
        }
    }

    /// Handles cleaning up the session after processing has
    /// terminated.
    ///
    /// Removes the player from its game if its present
    async fn cleanup(&mut self) {
        debug!("Session stopped: {}", self.id);

        self.session_store.remove_session(self.id);

        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            let mut lock = game.write().await;

            // Inform game to remove self
            let _ = lock.remove_player(self.id, self.id, RemoveReason::LostConnection);
        }
    }

    /// Handles server events received by this session, processes the
    /// events then sends them to the client
    ///
    /// # Arguments
    /// * event - The event to handle
    async fn handle_event(&mut self, event: Arc<ServerEvent>) -> Result<(), axum::Error> {
        let value = event.as_ref();

        // Ensure we drop our reference to the game when kicked
        if let ServerEvent::Kicked { id, .. } = value
            && self.id.eq(id)
        {
            self.game = None;
        }

        self.send(value).await
    }

    /// Handles processing websocket messages, updating heartbeat, and forwarding
    /// along parsed messages to handle_request
    ///
    /// # Arguments
    /// * msg - The websocket message
    async fn handle_message(&mut self, msg: Message) -> Result<bool, axum::Error> {
        // Update heartbeat
        self.hb = Instant::now();

        // Handle different message types
        let text = match msg {
            Message::Text(value) => value,
            Message::Ping(ping) => {
                let socket = match &mut self.socket {
                    Some(socket) => socket,

                    // This technically should be an impossible state
                    _ => return Err(axum::Error::new(PingOnDeadSocket)),
                };

                // If sending pong failed break
                if socket.send(Message::Pong(ping)).await.is_err() {
                    return Ok(false);
                }
                return Ok(true);
            }
            Message::Close(_) => return Ok(false),
            _ => return Ok(true),
        };

        // Decode the received client message
        let req = match serde_json::from_str::<ClientMessage>(&text) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {}", err);

                self.send(&ServerResponse(ResponseMessage::Error {
                    error: ServerError::MalformedMessage,
                }))
                .await?;

                return Ok(true);
            }
        };

        self.handle_request(req).await?;

        Ok(true)
    }

    /// Handles processing client messages and sending the response
    /// for the message
    ///
    /// # Arguments
    /// * msg - The client message being processed
    async fn handle_request(&mut self, msg: ClientMessage) -> Result<(), axum::Error> {
        let result: Result<ResponseMessage, ServerError> = match msg {
            ClientMessage::Initialize { uuid } => self.initialize(uuid).await,
            ClientMessage::Connect { token } => self.connect(token).await,
            ClientMessage::Join { name } => self.join(name).await,
            ClientMessage::HostAction { action } => self.host_action(action).await,
            ClientMessage::Answer { answer } => self.answer(answer).await,
            ClientMessage::Kick { id } => self.kick(id).await,
            ClientMessage::Ready => self.ready().await,
        };

        let msg = match result {
            Ok(value) => value,
            Err(error) => ResponseMessage::Error { error },
        };

        let res: ServerResponse = ServerResponse(msg);
        self.send(&res).await
    }

    /// Converts the provided message to JSON writing it as a text frame
    /// to the websocket
    ///
    /// # Arguments
    /// * msg - The message to send
    async fn send<S: Serialize>(&mut self, msg: &S) -> Result<(), axum::Error> {
        let value = serde_json::to_string(msg).map_err(|err| axum::Error::new(Box::new(err)))?;
        let message = Message::Text(value.into());
        match &mut self.socket {
            Some(socket) => socket.send(message).await,
            None => Ok(()),
        }
    }

    /// Handler for initialize messages to attempt to initialize a new game.
    /// On success the game reference on this session will be updated.
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    async fn initialize(&mut self, uuid: Uuid) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let msg: InitializedMessage = Games::initialize(uuid, self.id, self.tx.clone()).await?;
        self.game = Some(msg.game);

        Ok(ResponseMessage::Joined {
            id: self.id,
            config: msg.config,
            token: msg.token,
        })
    }

    /// Handler for connect messages to attempt to connect to a game.
    /// On success the game reference on this session will be updated.
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    async fn connect(&mut self, token: String) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let token: GameToken = token.parse()?;

        let game = Games::get_game(&token)
            .await
            .ok_or(ServerError::InvalidToken)?;

        self.game = Some(game);
        Ok(ResponseMessage::Ok)
    }

    /// Disconnects the session from their current game if they
    /// are already in one
    async fn disconnect(&mut self) {
        // If already in a game inform the game that we've left
        if let Some(game) = self.game.take() {
            let mut lock = game.write().await;
            let _ = lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
        }
    }

    /// Handler for join messages to attempt to join the game reference
    /// by the session game field
    ///
    /// # Arguments
    /// * name - The name to attempt to join with
    async fn join(&mut self, name: String) -> Result<ResponseMessage, ServerError> {
        let msg = {
            let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
            let mut game = game.write().await;

            game.join(self.id, self.tx.clone(), name)
        }?;

        Ok(ResponseMessage::Joined {
            id: self.id,
            token: msg.token,
            config: msg.config,
        })
    }

    /// Handler for host action messages
    ///
    /// # Arguments
    /// * action - The host action to execute
    async fn host_action(&mut self, action: HostAction) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.host_action(self.id, action)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for answer messages
    ///
    /// # Arguments
    /// * answer - The player answer
    async fn answer(&mut self, answer: Answer) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.answer(self.id, answer)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for kick messages
    ///
    /// # Arguments
    /// * target_id - The ID of the player to kick
    async fn kick(&mut self, target_id: SessionId) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.remove_player(self.id, target_id, RemoveReason::RemovedByHost)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for ready messages
    async fn ready(&mut self) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.ready(self.id);
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

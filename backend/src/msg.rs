//! Contains the definitions of the Client and Server packets

use crate::{
    game::{GameState, config::GameConfig},
    session::SessionId,
    types::{
        Answer, GameToken, HostAction, ImStr, Question, RemoveReason, Score, ScoreCollection,
        ServerError,
    },
};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use std::sync::Arc;
use uuid::Uuid;

/// Wrapper around the response message type to include
/// "ret": 1, which is used to indicate this is a response
pub struct ServerResponse(pub ResponseMessage);

impl Serialize for ServerResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("ret", &1)?;

        self.0
            .serialize(serde::__private228::ser::FlatMapSerializer(&mut map))?;
        map.end()
    }
}

/// Message to initialize the desired game as a host
#[derive(Deserialize)]
pub struct InitializeMessage {
    /// The UUID of the prepared game to initialize
    pub uuid: Uuid,
}

/// Message to associate the session with the provided game
#[derive(Deserialize)]
pub struct ConnectMessage {
    /// The game token to try and connect to (e.g. W2133)
    pub token: String,
}

/// Message to attempt to join the game using the provided name
#[derive(Deserialize)]
pub struct JoinMessage {
    /// The name to attempt to access with
    pub name: String,
}

/// Message indicating the client is ready to play
///
/// (This is done internally by clients once everything has been loaded)#[derive(Deserialize)]
#[derive(Deserialize)]
pub struct ReadyMessage;

/// Message for actions from the host session
#[derive(Deserialize)]
pub struct HostActionMessage {
    /// Action the host wants to perform
    pub action: HostAction,
}

/// Message to answer the question
#[derive(Deserialize)]
pub struct AnswerMessage {
    /// The answer provided by the user
    pub answer: Answer,
}

/// Message for the host to kick a player from the game
#[derive(Deserialize)]
pub struct KickMessage {
    /// The ID of the player to kick
    pub id: SessionId,
}

/// Messages received from the client
#[derive(Deserialize)]
#[serde(tag = "ty")]
pub enum ClientMessage {
    Initialize(InitializeMessage),
    Connect(ConnectMessage),
    Join(JoinMessage),
    Ready(ReadyMessage),
    HostAction(HostActionMessage),
    Answer(AnswerMessage),
    Kick(KickMessage),
}

#[derive(Serialize)]
#[serde(tag = "ty")]
pub enum ResponseMessage {
    /// Message indicating a complete successful connection
    Joined {
        /// The session ID
        id: SessionId,
        /// The uniquely generated game token (e.g A3DLM)
        token: GameToken,
        /// Copy of the game configuration to send back
        config: Arc<GameConfig>,
    },
    /// Ok message response
    Ok,
    /// Server error
    Error { error: ServerError },
}

/// Messages sent by the server
#[derive(Serialize)]
#[serde(tag = "ty")]
pub enum ServerEvent {
    /// Message providing information about another player in
    /// the game
    PlayerData { id: SessionId, name: ImStr },
    /// Message indicating the current state of the game
    GameState { state: GameState },
    /// Message for telling clients the current countdown timer
    Timer { value: u32 },
    /// Question data for the next question
    Question { question: Arc<Question> },
    /// Updates the player scores with the new scores
    Scores { scores: ScoreCollection },
    /// Message telling the player the score that they obtained
    Score { score: Score },
    /// Player has been kicked from the game
    Kicked {
        /// The ID of the player that was kicked
        id: SessionId,
        /// The reason the player was kicked
        reason: RemoveReason,
    },
    /// Provides the token that the session can use to resume itself
    /// if the socket is lost
    ResumptionToken {
        /// The token to resume with
        token: String,
    },
    /// Resumed access to a game
    ResumedGame {
        /// The resumed session ID
        id: SessionId,
        /// Whether we are the game host
        host: bool,
        /// The resumed player name if not resuming as a host
        name: Option<ImStr>,
        /// The resumed game token
        token: GameToken,
        /// The resumed game token
        config: Arc<GameConfig>,
    },
}

/// Trait implemented by messages that are serializable by the server
pub trait ServerMessage: Serialize {}

impl ServerMessage for ServerResponse {}

impl ServerMessage for ServerEvent {}

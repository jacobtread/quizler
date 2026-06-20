use super::answer::PlayerAnswers;
use crate::{
    msg::ServerEvent,
    session::{EventTarget, SessionId},
    types::ImStr,
};

/// Represents a session for the host player
pub struct HostSession {
    /// The ID of the referenced session
    pub id: SessionId,
    /// The addr to the session
    pub addr: EventTarget,
    /// The player ready state
    pub ready: bool,
}

impl HostSession {
    pub fn new(id: SessionId, addr: EventTarget) -> Self {
        Self {
            id,
            addr,
            ready: false,
        }
    }
}

/// Represents a session and associated data
/// for a player within a quiz
pub struct PlayerSession {
    /// The ID of the referenced session
    pub id: SessionId,
    /// The addr to the session
    pub addr: EventTarget,
    /// The player ready state
    pub ready: bool,

    /// The player name
    pub name: ImStr,
    /// The players answers and the score they got for them
    pub answers: PlayerAnswers,
    /// The player total score
    pub score: u32,
}

impl PlayerSession {
    pub fn new(id: SessionId, addr: EventTarget, name: ImStr, answers: PlayerAnswers) -> Self {
        Self {
            id,
            addr,
            ready: false,

            name,
            answers,
            score: 0,
        }
    }

    /// Creates an introduction event to introduce this player session
    /// to other players
    pub fn introduction_event(&self) -> ServerEvent {
        ServerEvent::PlayerData {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

pub enum GameSession<'a> {
    Host(&'a HostSession),
    Player(&'a PlayerSession),
}

impl<'a> GameSession<'a> {
    pub fn addr(&self) -> &EventTarget {
        match self {
            GameSession::Host(host_session) => &host_session.addr,
            GameSession::Player(player_session) => &player_session.addr,
        }
    }

    pub fn player(&self) -> Option<&'a PlayerSession> {
        match self {
            GameSession::Host(_) => None,
            GameSession::Player(player_session) => Some(player_session),
        }
    }
}

pub enum GameSessionMut<'a> {
    Host(&'a mut HostSession),
    Player(&'a mut PlayerSession),
}

impl<'a> GameSessionMut<'a> {
    pub fn mark_ready(&mut self) {
        match self {
            GameSessionMut::Host(host_session) => {
                host_session.ready = true;
            }
            GameSessionMut::Player(player_session) => {
                player_session.ready = true;
            }
        }
    }
}

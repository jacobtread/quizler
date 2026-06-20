use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use axum::extract::ws::WebSocket;
use base64ct::{Base64UrlUnpadded, Encoding};
use parking_lot::Mutex;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{session::SessionId, signing::SigningKey};

pub type SessionToken = String;

/// Global instance of the session store
static SESSION_STORE: LazyLock<SessionStore> = LazyLock::new(|| {
    let signing_key = SigningKey::generate();
    SessionStore::new(signing_key)
});

pub struct SessionStore {
    /// Signing key for signing resumption tokens
    key: SigningKey,
    /// Mapping from session ID to a channel for sending messages to the session
    sessions: Arc<Mutex<HashMap<SessionId, mpsc::UnboundedSender<SessionStoreMessage>>>>,
}

impl SessionStore {
    pub fn new(key: SigningKey) -> Self {
        Self {
            key,
            sessions: Default::default(),
        }
    }

    pub fn global() -> &'static SessionStore {
        &SESSION_STORE
    }

    fn generate_session_token(&self, session_id: SessionId) -> SessionToken {
        let data: &[u8; 16] = session_id.as_bytes();

        // Encode the message
        let msg = Base64UrlUnpadded::encode_string(data);

        // Create a signature from the raw message bytes
        let sig = self.key.sign(data);
        let sig = Base64UrlUnpadded::encode_string(sig.as_ref());

        // Join the message and signature to create the token
        [msg, sig].join(".")
    }

    /// Verifies an association token
    pub fn verify_session_token(&self, token: &str) -> Result<SessionId, VerifyError> {
        // Split the token parts
        let (msg_raw, sig_raw) = match token.split_once('.') {
            Some(value) => value,
            None => return Err(VerifyError::Invalid),
        };

        // Decode the 16 byte token message
        let mut msg = [0u8; 16];
        Base64UrlUnpadded::decode(msg_raw, &mut msg).map_err(|_| VerifyError::Invalid)?;

        // Decode 32byte signature (SHA256)
        let mut sig = [0u8; 32];
        Base64UrlUnpadded::decode(sig_raw, &mut sig).map_err(|_| VerifyError::Invalid)?;

        // Verify the signature
        if !self.key.verify(&msg, &sig) {
            return Err(VerifyError::Invalid);
        }
        let uuid = *Uuid::from_bytes_ref(&msg);
        Ok(uuid)
    }

    pub fn get_session_tx(
        &self,
        session_id: SessionId,
    ) -> Option<mpsc::UnboundedSender<SessionStoreMessage>> {
        self.sessions.lock().get(&session_id).cloned()
    }

    /// Add the session sender into the store for the provided `session_id`
    pub fn add_session(
        &self,
        session_id: SessionId,
        tx: mpsc::UnboundedSender<SessionStoreMessage>,
    ) -> SessionToken {
        let token = self.generate_session_token(session_id);
        self.sessions.lock().insert(session_id, tx);
        token
    }

    // Remove a session sender from the session store
    pub fn remove_session(&self, session_id: SessionId) {
        self.sessions.lock().remove(&session_id);
    }
}

/// Messages that the session store can send to a session
pub enum SessionStoreMessage {
    /// Message for a socket reconnection using a specific sessions
    Reconnect {
        /// The socket the session has been resumed from
        socket: WebSocket,
    },
}

/// Errors that can occur while verifying a token
#[derive(Debug, Error)]
pub enum VerifyError {
    /// The token is invalid
    #[error("token is invalid")]
    Invalid,
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use crate::{session_store::SessionStore, signing::SigningKey};

    /// Tests that tokens can be created and verified correctly
    #[test]
    fn test_token() {
        let key = SigningKey::generate();
        let sessions = SessionStore::new(key);

        let player_id = Uuid::new_v4();
        let token = sessions.generate_session_token(player_id);
        let claim = sessions.verify_session_token(&token).unwrap();

        assert_eq!(player_id, claim)
    }
}

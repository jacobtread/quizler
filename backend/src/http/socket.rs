use crate::{
    session::Session,
    session_store::{SessionStore, SessionStoreMessage},
};
use axum::{
    Extension,
    extract::{Query, WebSocketUpgrade},
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;

#[derive(Deserialize)]
pub struct SocketQuery {
    /// Optional resumption token to attempt to resume an
    /// existing session
    resume: Option<String>,
}

/// # GET /api/quiz/socket
///
/// Endpoint for creating a new websocket session
pub async fn quiz_socket(
    Extension(session_store): Extension<Arc<SessionStore>>,
    Query(query): Query<SocketQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(async move |socket| {
        let socket = if let Some(resume) = query.resume
            && let Ok(session_id) = session_store.verify_session_token(&resume)
            && let Some(session_tx) = session_store.get_session_tx(session_id)
        {
            match session_tx.send(SessionStoreMessage::Reconnect { socket }) {
                // Session was resumed we can return early
                Ok(_) => return,

                // Session could not be reached, it must be closed return to starting fresh
                Err(SendError(SessionStoreMessage::Reconnect { socket })) => socket,
            }
        } else {
            socket
        };

        Session::start(socket, session_store).await;
    })
}

use crate::{
    game::GameConfig,
    games::Games,
    types::{ImStr, Image, NameFiltering, Question},
};
use axum::{
    Json,
    extract::{Multipart, multipart::MultipartError},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use uuid::Uuid;

/// Intermediate structure for GameConfigs parsed from
/// quiz upload form data
#[derive(Deserialize)]
pub struct GameConfigUpload {
    /// The quiz name
    name: ImStr,
    /// The quiz description
    text: ImStr,
    /// The max number of quiz players
    max_players: usize,
    /// The quiz name filter
    filtering: NameFiltering,
    /// The quiz questions
    questions: Box<[Arc<Question>]>,
}

/// Errors that can occur when creating a quiz
#[derive(Debug, Error)]
pub enum CreateError {
    /// Quiz was missing its config
    #[error("Missing config data")]
    MissingConfig,
    /// Quiz config was invalid
    #[error(transparent)]
    InvalidConfig(serde_json::Error),
    /// Quiz failed server validation
    #[error("Validation failure incorrect values")]
    ValidationFailed,
    /// Uploaded image had an invalid ID
    #[error(transparent)]
    InvalidImageUuid(uuid::Error),
    /// Image was missing its mime type
    #[error("Missing image mime type for {0}")]
    MissingImageType(Uuid),
    /// Multipart read error
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    /// Content was too large
    #[error("Uploaded content was too large")]
    TooLarge,
}

#[derive(Serialize)]
pub struct QuizCreated {
    uuid: Uuid,
}

/// # POST /api/quiz
///
/// Endpoint for uploading and creating a new Quiz.
pub async fn create_quiz(mut payload: Multipart) -> Result<Response, CreateError> {
    // Configuration data
    let mut config: Option<GameConfigUpload> = None;
    // Map of stored uploaded images
    let mut images = HashMap::new();

    while let Some(mut field) = payload.next_field().await? {
        // Skip un-named fields
        if field.name().is_none() {
            continue;
        }

        /// Cap the upload max size to 15mb
        const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 1024 * 15;

        // Read the field content until the max buffer size
        let mut buffer = BytesMut::new();

        while let Some(chunk) = field.try_next().await? {
            buffer.extend_from_slice(&chunk);

            if buffer.len() >= MAX_BUFFER_SIZE_BYTES {
                return Err(CreateError::TooLarge);
            }
        }

        // Name was already checked at start, reading should not have changed this
        let name = field.name().expect("Field was missing its name");

        // Handle the config
        if name == "config" {
            let value: GameConfigUpload =
                serde_json::from_slice(&buffer).map_err(CreateError::InvalidConfig)?;
            config = Some(value);
            continue;
        }

        let uuid: Uuid = name.parse().map_err(CreateError::InvalidImageUuid)?;
        let mime = field
            .content_type()
            .ok_or(CreateError::MissingImageType(uuid))?;

        debug!(
            "Received uploaded file (UUID: {}, Mime: {}, Size: {})",
            uuid,
            mime,
            buffer.len()
        );

        images.insert(
            uuid,
            Image {
                mime: mime.into(),
                data: buffer.freeze(),
            },
        );
    }

    // Create the full configuration
    let config = config.ok_or(CreateError::MissingConfig)?;

    let config = GameConfig {
        name: config.name,
        text: config.text,
        max_players: config.max_players,
        filtering: config.filtering,
        questions: config.questions,
        images,
    };

    // Validate the config is acceptable
    if !config.validate() {
        return Err(CreateError::ValidationFailed);
    }

    let uuid = Games::prepare(config).await;

    debug!("Created new prepared game {}", uuid);

    Ok((StatusCode::CREATED, Json(QuizCreated { uuid })).into_response())
}

impl IntoResponse for CreateError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

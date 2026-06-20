use crate::{games::Games, types::GameToken};
use axum::{
    body::Body,
    extract::Path,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("The target game could not be found")]
    UnknownGame,
    #[error("The target image could not be found")]
    UnknownImage,
    #[error("Image mime type was invalid")]
    InvalidImageMime,
}

/// # GET /api/quiz/{token}/{uuid}
///
/// Endpoint for getting the contents of an image from
/// a quiz
pub async fn quiz_image(
    Path((token, uuid)): Path<(GameToken, Uuid)>,
) -> Result<Response, ImageError> {
    let game = Games::get_game(&token).ok_or(ImageError::UnknownGame)?;

    let image = game
        .read()
        .await
        .get_image(uuid)
        .ok_or(ImageError::UnknownImage)?;

    let mut res = Body::from(image.data).into_response();
    let content_type =
        HeaderValue::from_str(&image.mime).map_err(|_| ImageError::InvalidImageMime)?;
    res.headers_mut().insert(CONTENT_TYPE, content_type);

    Ok(res)
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::UnknownGame | Self::UnknownImage => StatusCode::BAD_REQUEST,
            Self::InvalidImageMime => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}

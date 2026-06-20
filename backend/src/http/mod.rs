use axum::{
    Router,
    routing::{get, post},
};

mod create;
mod image;
mod public;
mod socket;

/// Configuration function for configuring
/// all the routes
pub fn router() -> Router {
    Router::new()
        .route("/api/quiz", post(create::create_quiz))
        .route("/api/quiz/{token}/{image}", get(image::quiz_image))
        .route("/api/quiz/socket", get(socket::quiz_socket))
        .fallback_service(public::Assets)
}

use axum::{
    body::Body,
    http::{HeaderValue, Request, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use embeddy::Embedded;
use std::{
    convert::Infallible,
    future::{Ready, ready},
    task::{Context, Poll},
};
use tower::Service;

/// Embedded assets for serving the frontend of the application
#[derive(Embedded, Clone)]
#[folder = "public"]
pub struct Assets;

/// Fallback service implementation for using the assets from within
/// the embedded data
impl<T> Service<Request<T>> for Assets {
    type Response = Response;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let path = req.uri().path();
        // Strip the leading slash in order to match paths correctly
        let path = path.strip_prefix('/').unwrap_or(path);

        let (file, content_type) = Assets::get(path)
            .map(|file| (file, get_content_type(path)))
            // Fallback to the index.html file for all unknown pages
            .unwrap_or_else(|| (Assets::get("index.html").unwrap_or_default(), "text/html"));

        let mut res = Body::from(file).into_response();
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

        ready(Ok(res))
    }
}

/// Obtains the content type to use for the provided path by
/// matching its extension against expected types
///
/// # Arguments
/// * path - The path to get the content type for
fn get_content_type(path: &str) -> &'static str {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| {
            if ext == "js" {
                Some("application/javascript")
            } else if ext == "css" {
                Some("text/css")
            } else if ext == "html" {
                Some("text/html")
            } else {
                None
            }
        })
        // Default to the text/plain type
        .unwrap_or("text/plain")
}

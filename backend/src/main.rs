use crate::{session_store::SessionStore, signing::SigningKey};
use axum::{Extension, extract::DefaultBodyLimit};
use dotenvy::dotenv;
use log::{LevelFilter, error, info};
use std::{net::Ipv4Addr, process::exit, sync::Arc};
use tokio::net::TcpListener;

mod game;
mod http;
mod msg;
mod session;
mod session_store;
mod signing;
mod types;

// Cargo package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_MAX_BODY_SIZE: usize = 50 * 1000 * 1000;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::builder()
        .filter_module("quizler", LevelFilter::Info)
        .parse_default_env()
        .init();

    let signing_key = SigningKey::generate();
    let session_store = Arc::new(SessionStore::new(signing_key));

    let port: u16 = std::env::var("QUIZLER_PORT")
        .map(|value| {
            value
                .parse::<u16>()
                .expect("Provided QUIZLER_PORT was not a valid port")
        })
        .unwrap_or(80);

    let max_body_size_byte: usize = std::env::var("QUIZLER_MAX_BODY_SIZE_BYTES")
        .map(|value| {
            value
                .parse::<usize>()
                .expect("Provided QUIZLER_MAX_BODY_SIZE_BYTES was not a valid unsigned integer")
        })
        .unwrap_or(DEFAULT_MAX_BODY_SIZE); // Default max size of 50mb

    info!("Starting Quizler on port {} (v{})", port, VERSION);

    if max_body_size_byte != DEFAULT_MAX_BODY_SIZE {
        log::debug!("custom max http body size is set = {max_body_size_byte}")
    }

    let router = http::router()
        .layer(DefaultBodyLimit::max(max_body_size_byte))
        .layer(Extension(session_store));

    // Add CORS and tracing layer to the router in debug mode
    #[cfg(debug_assertions)]
    let router = router
        .layer(tower_http::cors::CorsLayer::very_permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();

    if let Err(err) = axum::serve(listener, router).await {
        error!("Server error: {}", err);
        exit(1);
    }
}

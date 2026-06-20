use axum::extract::DefaultBodyLimit;
use config::{Config, DEFAULT_MAX_BODY_SIZE};
use dotenvy::dotenv;
use log::LevelFilter;
use std::process::exit;
use tokio::net::TcpListener;

mod config;
mod game;
mod http;
mod msg;
mod session;
mod session_store;
mod signing;
mod types;

// Cargo package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Load environment variables
    _ = dotenv();

    // Initialize logger
    env_logger::builder()
        .filter_module("quizler", LevelFilter::Info)
        .parse_default_env()
        .init();

    let Config {
        host,
        port,
        max_body_size_byte,
    } = Config::load();

    log::info!("Starting Quizler on {host}:{port} (v{VERSION})");

    if max_body_size_byte != DEFAULT_MAX_BODY_SIZE {
        log::debug!("custom max http body size is set = {max_body_size_byte}")
    }

    let router = http::router().layer(DefaultBodyLimit::max(max_body_size_byte));

    // Add CORS and tracing layer to the router in debug mode
    #[cfg(debug_assertions)]
    let router = router
        .layer(tower_http::cors::CorsLayer::very_permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = TcpListener::bind((host, port)).await.unwrap();

    if let Err(err) = axum::serve(listener, router).await {
        log::error!("Server error: {}", err);
        exit(1);
    }
}

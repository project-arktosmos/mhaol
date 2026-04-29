use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;

use mhaol_yt_dlp::{build_router, DownloadManager, YtDownloadConfig};

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = std::env::var("YTDL_PORT").unwrap_or_else(|_| "3040".to_string());
    let cors_origin =
        std::env::var("YTDL_CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:1530".to_string());

    let config = YtDownloadConfig::from_env();
    log::info!("Output directory: {}", config.output_path);

    if let Err(e) = std::fs::create_dir_all(&config.output_path) {
        log::error!("Failed to create output directory: {}", e);
    }

    let manager = Arc::new(DownloadManager::new(config));

    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:1530")),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .nest("/api", build_router(manager))
        .layer(cors);

    let bind_addr = format!("0.0.0.0:{}", port);
    log::info!("Starting YouTube download server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to bind to {}: {}", bind_addr, e);
            std::process::exit(1);
        });

    println!("YouTube download server listening on {}", bind_addr);

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        log::error!("Server error: {}", e);
    });
}

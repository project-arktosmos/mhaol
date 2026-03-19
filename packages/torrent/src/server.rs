use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use axum::http::{HeaderValue, Method};

use mhaol_torrent::{TorrentConfig, TorrentManager};

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = std::env::var("TORRENT_PORT").unwrap_or_else(|_| "3030".to_string());
    let download_dir = std::env::var("TORRENT_DOWNLOAD_DIR").unwrap_or_else(|_| {
        dirs_fallback().unwrap_or_else(|| "/tmp/torrents".to_string())
    });
    let cors_origin = std::env::var("TORRENT_CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:1530".to_string());

    let manager = Arc::new(TorrentManager::new());

    let config = TorrentConfig {
        download_path: std::path::PathBuf::from(&download_dir),
        http_api_bind_addr: None,
        ..Default::default()
    };

    if let Err(e) = manager.initialize(config).await {
        log::error!("Failed to initialize torrent manager: {}", e);
        std::process::exit(1);
    }

    log::info!("Torrent manager initialized, download dir: {}", download_dir);

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
        .nest("/api", mhaol_torrent::api::router())
        .layer(cors)
        .with_state(manager);

    let bind_addr = format!("0.0.0.0:{}", port);
    log::info!("Starting torrent server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to bind to {}: {}", bind_addr, e);
            std::process::exit(1);
        });

    println!("Torrent server listening on {}", bind_addr);

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        log::error!("Server error: {}", e);
    });
}

fn dirs_fallback() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|home| format!("{}/Downloads/torrents", home))
}

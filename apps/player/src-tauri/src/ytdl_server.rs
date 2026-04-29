use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use mhaol_yt_dlp::{build_router, DownloadManager, YtDownloadConfig};
use tower_http::cors::CorsLayer;

const DEFAULT_PORT: u16 = 9897;

/// Spawn the embedded yt-dlp HTTP server on a background thread so it stays
/// alive for the lifetime of the Tauri app. Routes are mounted under
/// `/api/ytdl/*` to match the node's API surface.
pub fn spawn() {
    let _ = env_logger::try_init();

    std::thread::spawn(|| {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create yt-dlp tokio runtime: {}", e);
                return;
            }
        };

        runtime.block_on(async {
            run_server().await;
        });
    });
}

async fn run_server() {
    let port = std::env::var("YTDL_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let cors_origin = std::env::var("YTDL_CORS_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:9595".to_string());

    let config = YtDownloadConfig::from_env();
    if let Err(e) = std::fs::create_dir_all(&config.output_path) {
        log::error!("Failed to create yt-dlp output directory: {}", e);
    }

    let manager = Arc::new(DownloadManager::new(config));

    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:9595")),
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
        .nest("/api/ytdl", build_router(manager))
        .layer(cors);

    let bind_addr = format!("127.0.0.1:{}", port);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Embedded yt-dlp server failed to bind {}: {}", bind_addr, e);
            return;
        }
    };

    log::info!("Embedded yt-dlp server listening on {}", bind_addr);

    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Embedded yt-dlp server error: {}", e);
    }
}

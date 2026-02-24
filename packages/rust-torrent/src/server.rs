use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use mhaol_torrent::{AddTorrentRequest, TorrentConfig, TorrentManager};

type AppState = Arc<TorrentManager>;

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

    // Auto-initialize with config from env
    let config = TorrentConfig {
        download_path: std::path::PathBuf::from(&download_dir),
        http_api_bind_addr: None, // We handle HTTP ourselves
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
        .route("/api/status", get(get_status))
        .route("/api/config", get(get_config))
        .route("/api/config", put(update_config))
        .route("/api/torrents", post(add_torrent))
        .route("/api/torrents", get(list_torrents))
        .route("/api/torrents/events", get(torrent_events))
        .route("/api/torrents/remove-all", post(remove_all))
        .route("/api/torrents/complete/{info_hash}", post(complete_torrent))
        .route("/api/torrents/{id}/pause", post(pause_torrent))
        .route("/api/torrents/{id}/resume", post(resume_torrent))
        .route("/api/torrents/{id}", delete(remove_torrent))
        .route("/api/debug", get(get_debug))
        .route("/api/storage/clear", post(clear_storage))
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

// ── Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct StatusResponse {
    initialized: bool,
    download_path: String,
    stats: Option<mhaol_torrent::TorrentStats>,
}

#[derive(Serialize)]
struct ConfigResponse {
    download_path: String,
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    download_path: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorResponse { error: msg.into() }))
}

// ── Handlers ────────────────────────────────────────────────────────

async fn get_status(State(mgr): State<AppState>) -> impl IntoResponse {
    let stats = mgr.stats().await.ok();
    Json(StatusResponse {
        initialized: mgr.is_initialized(),
        download_path: mgr.download_path().to_string_lossy().to_string(),
        stats,
    })
}

async fn get_config(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(ConfigResponse {
        download_path: mgr.download_path().to_string_lossy().to_string(),
    })
}

async fn update_config(
    State(_mgr): State<AppState>,
    Json(body): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    // Note: changing download_path at runtime requires re-initialization.
    // For now, we just acknowledge the request.
    if let Some(path) = &body.download_path {
        log::info!("Config update requested: download_path={}", path);
    }
    Json(serde_json::json!({ "ok": true }))
}

async fn add_torrent(
    State(mgr): State<AppState>,
    Json(body): Json<AddTorrentRequest>,
) -> impl IntoResponse {
    match mgr.add(body).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn list_torrents(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.list().await {
        Ok(list) => (StatusCode::OK, Json(serde_json::to_value(list).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn pause_torrent(
    State(mgr): State<AppState>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    match mgr.pause(id).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn resume_torrent(
    State(mgr): State<AppState>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    match mgr.resume(id).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn remove_torrent(
    State(mgr): State<AppState>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    match mgr.remove(id).await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn remove_all(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.remove_all().await {
        Ok(count) => Json(serde_json::json!({ "removed": count })).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn complete_torrent(
    State(mgr): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    let output_path = mgr
        .get_tracking_info(&info_hash)
        .and_then(|t| t.output_path)
        .unwrap_or_default();

    match mgr.complete_download(info_hash, output_path) {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn get_debug(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.debug_info().await {
        Ok(logs) => Json(serde_json::json!({ "logs": logs })).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn clear_storage(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.clear_storage().await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn torrent_events(
    State(mgr): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;

            let torrents = mgr.list().await.unwrap_or_default();
            if let Ok(data) = serde_json::to_string(&torrents) {
                yield Ok(Event::default().event("torrents").data(data));
            }

            if let Ok(stats) = mgr.stats().await {
                if let Ok(data) = serde_json::to_string(&stats) {
                    yield Ok(Event::default().event("stats").data(data));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

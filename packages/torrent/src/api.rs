use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{AddTorrentRequest, TorrentManager, TorrentStats};

type AppState = Arc<TorrentManager>;

/// Build a router with all torrent API routes.
/// Routes are relative (no `/api/` prefix) so the caller can nest them.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        .route("/torrents", post(add_torrent))
        .route("/torrents", get(list_torrents))
        .route("/torrents/events", get(torrent_events))
        .route("/torrents/remove-all", post(remove_all))
        .route("/torrents/complete/{info_hash}", post(complete_torrent))
        .route("/torrents/{id}/pause", post(pause_torrent))
        .route("/torrents/{id}/resume", post(resume_torrent))
        .route("/torrents/{id}", delete(remove_torrent))
        .route("/debug", get(get_debug))
        .route("/storage/clear", post(clear_storage))
}

// ── Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct StatusResponse {
    initialized: bool,
    download_path: String,
    stats: Option<TorrentStats>,
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

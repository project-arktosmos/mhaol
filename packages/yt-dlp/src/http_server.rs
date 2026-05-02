use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    manager::SseEvent, related, search, DownloadManager, QueueDownloadRequest,
    QueuePlaylistRequest, YtDlpStatus,
};

type AppState = Arc<DownloadManager>;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorResponse { error: msg.into() }))
}

#[derive(Deserialize)]
struct UrlQuery {
    url: String,
}

/// Build the yt-dlp HTTP router with the routes the frontend expects.
/// All routes are relative; mount it under whatever prefix the host wants
/// (the standalone server uses `/api`, the player tauri shell uses `/api/ytdl`).
pub fn build_router(manager: Arc<DownloadManager>) -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        .route("/ytdlp/status", get(get_ytdlp_status))
        .route("/info/video", get(get_video_info))
        .route("/info/playlist", get(get_playlist_info))
        .route("/info/stream-urls", get(get_stream_urls))
        .route("/info/stream-urls-browser", get(get_stream_urls_browser))
        .route("/search", get(search::search))
        .route("/related", get(related::related))
        .route("/downloads", get(list_downloads))
        .route("/downloads", post(queue_download))
        .route("/downloads/playlist", post(queue_playlist))
        .route("/downloads/events", get(download_events))
        .route("/downloads/queue", delete(clear_queue))
        .route("/downloads/completed", delete(clear_completed))
        .route("/downloads/{id}", get(get_download))
        .route("/downloads/{id}", delete(cancel_download))
        .with_state(manager)
}

async fn get_status(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_stats())
}

async fn get_config(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_config())
}

async fn update_config(
    State(mgr): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    mgr.update_config(body);
    Json(mgr.get_config())
}

async fn get_ytdlp_status(State(_mgr): State<AppState>) -> impl IntoResponse {
    Json(YtDlpStatus {
        available: true,
        version: Some(format!("native-rust-{}", env!("CARGO_PKG_VERSION"))),
        downloading: false,
    })
}

async fn get_video_info(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.fetch_video_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_stream_urls(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.extract_stream_urls(&query.url).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_stream_urls_browser(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.extract_stream_urls_for_browser(&query.url).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_playlist_info(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.fetch_playlist_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn list_downloads(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_all_progress())
}

async fn queue_download(
    State(mgr): State<AppState>,
    Json(body): Json<QueueDownloadRequest>,
) -> impl IntoResponse {
    let download_id = mgr.queue_download(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadId": download_id })),
    )
}

async fn queue_playlist(
    State(mgr): State<AppState>,
    Json(body): Json<QueuePlaylistRequest>,
) -> impl IntoResponse {
    let download_ids = mgr.queue_playlist(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadIds": download_ids })),
    )
}

async fn get_download(
    State(mgr): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match mgr.get_progress(&id) {
        Some(progress) => {
            (StatusCode::OK, Json(serde_json::to_value(progress).unwrap())).into_response()
        }
        None => error_response(StatusCode::NOT_FOUND, "Download not found").into_response(),
    }
}

async fn cancel_download(
    State(mgr): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if mgr.cancel_download(&id) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        error_response(StatusCode::NOT_FOUND, "Download not found").into_response()
    }
}

async fn clear_queue(State(mgr): State<AppState>) -> impl IntoResponse {
    mgr.clear_queue();
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_completed(State(mgr): State<AppState>) -> impl IntoResponse {
    mgr.clear_completed();
    Json(serde_json::json!({ "ok": true }))
}

async fn download_events(
    State(mgr): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = mgr.subscribe_events();

    let stream = async_stream::stream! {
        yield Ok(Event::default()
            .event("connected")
            .data(serde_json::json!({ "message": "Connected to download events" }).to_string()));

        let stats = mgr.get_stats();
        if let Ok(data) = serde_json::to_string(&stats) {
            yield Ok(Event::default().event("stats").data(data));
        }

        for progress in mgr.get_all_progress() {
            if let Ok(data) = serde_json::to_string(&progress) {
                yield Ok(Event::default().event("progress").data(data));
            }
        }

        loop {
            match rx.recv().await {
                Ok(SseEvent::Progress(progress)) => {
                    if let Ok(data) = serde_json::to_string(&*progress) {
                        yield Ok(Event::default().event("progress").data(data));
                    }
                }
                Ok(SseEvent::Stats(stats)) => {
                    if let Ok(data) = serde_json::to_string(&stats) {
                        yield Ok(Event::default().event("stats").data(data));
                    }
                }
                Ok(SseEvent::Connected) => {
                    yield Ok(Event::default()
                        .event("connected")
                        .data(serde_json::json!({ "message": "Connected" }).to_string()));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("SSE client lagged by {} messages", n);
                    continue;
                }
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

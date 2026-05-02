//! `/api/ytdl/*` — re-exposes the same yt-dlp HTTP surface that
//! `mhaol_yt_dlp::http_server::build_router` provides, but as a
//! `Router<CloudState>` so it composes with the rest of the cloud router.
//!
//! Each handler pulls `state.ytdl_manager` from `CloudState` and delegates to
//! the public `DownloadManager` API. We can't reuse `build_router` directly:
//! the cloud's outer router is `Router<CloudState>`, while `build_router`
//! produces a `Router<()>` (state already applied to `Arc<DownloadManager>`),
//! and `Router::nest_service` of a `Router<()>` does not actually route
//! requests through the inner router — they fall through to the outer
//! fallback (the SPA), so the browser sees `<!DOCTYPE html>` instead of JSON.

#![cfg(not(target_os = "android"))]

use crate::state::CloudState;
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
use mhaol_yt_dlp::{
    manager::SseEvent, search, QueueDownloadRequest, QueuePlaylistRequest, YtDlpStatus,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        .route("/ytdlp/status", get(get_ytdlp_status))
        .route("/info/video", get(get_video_info))
        .route("/info/playlist", get(get_playlist_info))
        .route("/info/stream-urls", get(get_stream_urls))
        .route("/info/stream-urls-browser", get(get_stream_urls_browser))
        .route("/search", get(search_proxy))
        .route("/downloads", get(list_downloads))
        .route("/downloads", post(queue_download))
        .route("/downloads/playlist", post(queue_playlist))
        .route("/downloads/events", get(download_events))
        .route("/downloads/queue", delete(clear_queue))
        .route("/downloads/completed", delete(clear_completed))
        .route("/downloads/{id}", get(get_download))
        .route("/downloads/{id}", delete(cancel_download))
}

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

#[derive(Deserialize)]
struct BrowserStreamQuery {
    url: String,
    /// Optional preferred Innertube client (`web`, `web_embedded`, `tv`,
    /// `android`, `ios`) — used by callers that cached a previous result's
    /// `clientName` to skip the failing-candidate iteration on the next call.
    /// Unknown values are ignored and the regular browser priority list is
    /// walked instead.
    prefer: Option<String>,
}

async fn get_status(State(state): State<CloudState>) -> impl IntoResponse {
    Json(state.ytdl_manager.get_stats())
}

async fn get_config(State(state): State<CloudState>) -> impl IntoResponse {
    Json(state.ytdl_manager.get_config())
}

async fn update_config(
    State(state): State<CloudState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    state.ytdl_manager.update_config(body);
    Json(state.ytdl_manager.get_config())
}

async fn get_ytdlp_status(State(_state): State<CloudState>) -> impl IntoResponse {
    Json(YtDlpStatus {
        available: true,
        version: Some(format!("native-rust-{}", env!("CARGO_PKG_VERSION"))),
        downloading: false,
    })
}

async fn get_video_info(
    State(state): State<CloudState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.fetch_video_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_stream_urls(
    State(state): State<CloudState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.extract_stream_urls(&query.url).await {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response()
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_stream_urls_browser(
    State(state): State<CloudState>,
    Query(query): Query<BrowserStreamQuery>,
) -> impl IntoResponse {
    let prefer = query.prefer.as_deref().map(str::trim).filter(|s| !s.is_empty());
    match state
        .ytdl_manager
        .extract_stream_urls_for_browser_with_preference(&query.url, prefer)
        .await
    {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response()
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_playlist_info(
    State(state): State<CloudState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.fetch_playlist_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn search_proxy(query: Query<search::SearchQuery>) -> impl IntoResponse {
    search::search(query).await
}

async fn list_downloads(State(state): State<CloudState>) -> impl IntoResponse {
    Json(state.ytdl_manager.get_all_progress())
}

async fn queue_download(
    State(state): State<CloudState>,
    Json(body): Json<QueueDownloadRequest>,
) -> impl IntoResponse {
    let download_id = state.ytdl_manager.queue_download(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadId": download_id })),
    )
}

async fn queue_playlist(
    State(state): State<CloudState>,
    Json(body): Json<QueuePlaylistRequest>,
) -> impl IntoResponse {
    let download_ids = state.ytdl_manager.queue_playlist(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadIds": download_ids })),
    )
}

async fn get_download(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.ytdl_manager.get_progress(&id) {
        Some(progress) => {
            (StatusCode::OK, Json(serde_json::to_value(progress).unwrap())).into_response()
        }
        None => error_response(StatusCode::NOT_FOUND, "Download not found").into_response(),
    }
}

async fn cancel_download(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if state.ytdl_manager.cancel_download(&id) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        error_response(StatusCode::NOT_FOUND, "Download not found").into_response()
    }
}

async fn clear_queue(State(state): State<CloudState>) -> impl IntoResponse {
    state.ytdl_manager.clear_queue();
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_completed(State(state): State<CloudState>) -> impl IntoResponse {
    state.ytdl_manager.clear_completed();
    Json(serde_json::json!({ "ok": true }))
}

async fn download_events(
    State(state): State<CloudState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let manager = state.ytdl_manager.clone();
    let mut rx = manager.subscribe_events();

    let stream = async_stream::stream! {
        yield Ok(Event::default()
            .event("connected")
            .data(serde_json::json!({ "message": "Connected to download events" }).to_string()));

        let stats = manager.get_stats();
        if let Ok(data) = serde_json::to_string(&stats) {
            yield Ok(Event::default().event("stats").data(data));
        }

        for progress in manager.get_all_progress() {
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
                    tracing::warn!("SSE client lagged by {} messages", n);
                    continue;
                }
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{delete, get, post},
    Json, Router,
};
use mhaol_yt_dlp::manager::SseEvent;
use serde::Deserialize;
use std::convert::Infallible;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config).put(update_config))
        .route("/downloads", get(list_downloads).post(queue_download))
        .route("/downloads/{id}", get(get_download).delete(delete_download))
        .route("/downloads/completed", delete(clear_completed))
        .route("/downloads/queue", delete(clear_queue))
        .route("/downloads/playlist", post(queue_playlist))
        .route("/downloads/events", get(download_events))
        .route("/info/video", get(video_info))
        .route("/info/playlist", get(playlist_info))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/status", get(get_status))
        .route("/ytdlp/status", get(ytdlp_status))
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ytdl_manager.get_config()).unwrap())
}

async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    state.ytdl_manager.update_config(body);
    Json(serde_json::json!({ "ok": true }))
}

async fn list_downloads(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ytdl_manager.get_all_progress()).unwrap())
}

async fn queue_download(
    State(state): State<AppState>,
    Json(body): Json<mhaol_yt_dlp::QueueDownloadRequest>,
) -> impl IntoResponse {
    let download_id = state.ytdl_manager.queue_download(body.clone());

    state.youtube_downloads.upsert(
        &download_id,
        &body.url,
        &body.video_id,
        &body.title,
        "pending",
        0.0,
        0,
        0,
        None,
        None,
        body.mode
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .as_deref()
            .unwrap_or("\"Audio\""),
        body.quality
            .as_ref()
            .map(|q| serde_json::to_string(q).unwrap_or_default())
            .as_deref()
            .unwrap_or("\"Best\""),
        body.format
            .as_ref()
            .map(|f| serde_json::to_string(f).unwrap_or_default())
            .as_deref()
            .unwrap_or("\"Opus\""),
        None,
        None,
        None,
        None,
    );

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadId": download_id })),
    )
}

async fn get_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.ytdl_manager.get_progress(&id) {
        Some(progress) => Json(serde_json::to_value(progress).unwrap()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Download not found" })),
        )
            .into_response(),
    }
}

async fn delete_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.ytdl_manager.cancel_download(&id);
    state.youtube_downloads.delete(&id);
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_completed(State(state): State<AppState>) -> impl IntoResponse {
    state.ytdl_manager.clear_completed();
    state
        .youtube_downloads
        .delete_by_states(&["completed", "failed", "cancelled"]);
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_queue(State(state): State<AppState>) -> impl IntoResponse {
    state.ytdl_manager.clear_queue();
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Deserialize)]
struct QueuePlaylistBody {
    #[serde(flatten)]
    request: mhaol_yt_dlp::QueuePlaylistRequest,
}

async fn queue_playlist(
    State(state): State<AppState>,
    Json(body): Json<QueuePlaylistBody>,
) -> impl IntoResponse {
    let ids = state.ytdl_manager.queue_playlist(body.request);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadIds": ids })),
    )
}

async fn download_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.ytdl_manager.subscribe_events();
    let youtube_downloads = state.youtube_downloads.clone();

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            match &event {
                SseEvent::Progress(progress) => {
                    let state_str = format!("{:?}", progress.state).to_lowercase();
                    youtube_downloads.upsert(
                        &progress.download_id,
                        &progress.url,
                        &progress.video_id,
                        &progress.title,
                        &state_str,
                        progress.progress,
                        progress.downloaded_bytes as i64,
                        progress.total_bytes as i64,
                        progress.output_path.as_deref(),
                        progress.error.as_deref(),
                        &serde_json::to_string(&progress.mode).unwrap_or_default(),
                        &serde_json::to_string(&progress.quality).unwrap_or_default(),
                        &serde_json::to_string(&progress.format).unwrap_or_default(),
                        progress.video_quality.as_ref().map(|q| serde_json::to_string(q).unwrap_or_default()).as_deref(),
                        progress.video_format.as_ref().map(|f| serde_json::to_string(f).unwrap_or_default()).as_deref(),
                        progress.thumbnail_url.as_deref(),
                        progress.duration_seconds.map(|d| d as i64),
                    );
                    if let Ok(json) = serde_json::to_string(&progress) {
                        yield Ok(Event::default().event("progress").data(json));
                    }
                }
                SseEvent::Stats(stats) => {
                    if let Ok(json) = serde_json::to_string(&stats) {
                        yield Ok(Event::default().event("stats").data(json));
                    }
                }
                SseEvent::Connected => {
                    yield Ok(Event::default().event("connected").data("{}"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Deserialize)]
struct VideoInfoQuery {
    url: String,
}

async fn video_info(
    State(state): State<AppState>,
    Query(query): Query<VideoInfoQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.fetch_video_info(&query.url).await {
        Ok(info) => Json(serde_json::to_value(info).unwrap()).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn playlist_info(
    State(state): State<AppState>,
    Query(query): Query<VideoInfoQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.fetch_playlist_info(&query.url).await {
        Ok(info) => Json(serde_json::to_value(info).unwrap()).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let library_id = state
        .metadata
        .get("youtube.libraryId")
        .map(|r| r.value)
        .unwrap_or_default();
    let settings = serde_json::json!({
        "downloadMode": state.settings.get("ytdl.downloadMode").unwrap_or_else(|| "audio".to_string()),
        "quality": state.settings.get("ytdl.quality").unwrap_or_else(|| "best".to_string()),
        "format": state.settings.get("ytdl.format").unwrap_or_else(|| "opus".to_string()),
        "videoQuality": state.settings.get("ytdl.videoQuality").unwrap_or_else(|| "best".to_string()),
        "videoFormat": state.settings.get("ytdl.videoFormat").unwrap_or_else(|| "mp4".to_string()),
        "poToken": state.settings.get("ytdl.poToken").unwrap_or_default(),
        "visitorData": state.settings.get("ytdl.visitorData").unwrap_or_default(),
        "cookies": state.settings.get("ytdl.cookies").unwrap_or_default(),
        "libraryId": library_id,
    });
    Json(settings)
}

async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(obj) = body.as_object() {
        for (key, value) in obj {
            let str_val = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            match key.as_str() {
                "libraryId" => {
                    state.metadata.set_string("youtube.libraryId", &str_val);
                }
                "poToken" | "visitorData" | "cookies" => {
                    state.settings.set(&format!("ytdl.{}", key), &str_val);
                    let config_update = serde_json::json!({ key: str_val });
                    state.ytdl_manager.update_config(config_update);
                }
                _ => {
                    state.settings.set(&format!("ytdl.{}", key), &str_val);
                }
            }
        }
    }
    Json(serde_json::json!({ "ok": true }))
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ytdl_manager.get_stats()).unwrap())
}

async fn ytdlp_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ytdl_manager.get_stats()).unwrap())
}

use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
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

/// Map yt-dlp download mode to unified download type.
fn youtube_download_type(mode: Option<&mhaol_yt_dlp::DownloadMode>) -> &'static str {
    match mode {
        Some(mhaol_yt_dlp::DownloadMode::Audio) => "youtube-audio",
        _ => "youtube-video",
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config).put(update_config))
        .route("/downloads", get(list_downloads).post(queue_download))
        .route("/downloads/{id}", get(get_download).delete(delete_download))
        .route("/downloads/completed", delete(clear_completed))
        .route("/downloads/queue", delete(clear_queue))
        .route("/downloads/playlist", post(queue_playlist))
        .route("/downloads/events", get(download_events))
        .route("/downloads/{id}/stream/video", get(stream_download_video))
        .route("/info/video", get(video_info))
        .route("/info/playlist", get(playlist_info))
        .route("/info/stream-urls", get(stream_urls))
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

/// Resolve the library cache directories for video and audio output.
fn resolve_output_dirs(state: &AppState) -> (Option<String>, Option<String>) {
    let lib = state.libraries.get(crate::AppState::DEFAULT_LIBRARY_ID);
    match lib {
        Some(lib) => {
            let base = std::path::Path::new(&lib.path);
            let video_dir = base.join("video").join(".cache");
            let audio_dir = base.join("audio").join(".cache");
            std::fs::create_dir_all(&video_dir).ok();
            std::fs::create_dir_all(&audio_dir).ok();
            (
                Some(video_dir.to_string_lossy().to_string()),
                Some(audio_dir.to_string_lossy().to_string()),
            )
        }
        None => (None, None),
    }
}

async fn queue_download(
    State(state): State<AppState>,
    Json(mut body): Json<mhaol_yt_dlp::QueueDownloadRequest>,
) -> impl IntoResponse {
    // Set output dirs from library if not already specified
    if body.video_output_dir.is_none() || body.audio_output_dir.is_none() {
        let (video_dir, audio_dir) = resolve_output_dirs(&state);
        if body.video_output_dir.is_none() {
            body.video_output_dir = video_dir;
        }
        if body.audio_output_dir.is_none() {
            body.audio_output_dir = audio_dir;
        }
    }

    let download_id = state.ytdl_manager.queue_download(body.clone());

    state.downloads.upsert_youtube(
        &download_id,
        youtube_download_type(body.mode.as_ref()),
        &body.title,
        0,
        &body.url,
        &body.video_id,
        "pending",
        0.0,
        None,
        None,
        body.thumbnail_url.as_deref(),
        body.duration_seconds.map(|d| d as i64),
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
    state.downloads.delete(&id);
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_completed(State(state): State<AppState>) -> impl IntoResponse {
    state.ytdl_manager.clear_completed();
    state
        .downloads
        .delete_youtube_by_states(&["completed", "failed", "cancelled"]);
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
    Json(mut body): Json<QueuePlaylistBody>,
) -> impl IntoResponse {
    if body.request.video_output_dir.is_none() || body.request.audio_output_dir.is_none() {
        let (video_dir, audio_dir) = resolve_output_dirs(&state);
        if body.request.video_output_dir.is_none() {
            body.request.video_output_dir = video_dir;
        }
        if body.request.audio_output_dir.is_none() {
            body.request.audio_output_dir = audio_dir;
        }
    }

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
    let downloads = state.downloads.clone();
    let youtube_content = state.youtube_content.clone();

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            match &event {
                SseEvent::Progress(progress) => {
                    let state_str = format!("{:?}", progress.state).to_lowercase();
                    let dl_type = youtube_download_type(Some(&progress.mode));
                    downloads.upsert_youtube(
                        &progress.download_id,
                        dl_type,
                        &progress.title,
                        progress.total_bytes as i64,
                        &progress.url,
                        &progress.video_id,
                        &state_str,
                        progress.progress,
                        progress.output_path.as_deref(),
                        progress.error.as_deref(),
                        progress.thumbnail_url.as_deref(),
                        progress.duration_seconds.map(|d| d as i64),
                    );

                    // On completion, upsert into youtube_content
                    if progress.state == mhaol_yt_dlp::DownloadState::Completed {
                        youtube_content.upsert(
                            &progress.video_id,
                            &progress.title,
                            progress.thumbnail_url.as_deref(),
                            progress.duration_seconds.map(|d| d as i64),
                            progress.channel_name.as_deref(),
                            None,
                            progress.video_output_path.as_deref(),
                            progress.audio_output_path.as_deref(),
                        );
                    }

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

async fn stream_urls(
    State(state): State<AppState>,
    Query(query): Query<VideoInfoQuery>,
) -> impl IntoResponse {
    match state.ytdl_manager.extract_stream_urls(&query.url).await {
        Ok(result) => Json(serde_json::to_value(result).unwrap()).into_response(),
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
    let settings = serde_json::json!({
        "downloadMode": state.settings.get("ytdl.downloadMode").unwrap_or_else(|| "both".to_string()),
        "defaultQuality": state.settings.get("ytdl.quality").unwrap_or_else(|| "best".to_string()),
        "defaultFormat": state.settings.get("ytdl.format").unwrap_or_else(|| "aac".to_string()),
        "defaultVideoQuality": state.settings.get("ytdl.videoQuality").unwrap_or_else(|| "best".to_string()),
        "defaultVideoFormat": state.settings.get("ytdl.videoFormat").unwrap_or_else(|| "mp4".to_string()),
        "poToken": state.settings.get("ytdl.poToken").unwrap_or_default(),
        "visitorData": state.settings.get("ytdl.visitorData").unwrap_or_default(),
        "cookies": state.settings.get("ytdl.cookies").unwrap_or_default(),
        "mediaMode": state.settings.get("ytdl.mediaMode").unwrap_or_else(|| "video".to_string()),
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
            let storage_key = match key.as_str() {
                "defaultQuality" => "quality",
                "defaultFormat" => "format",
                "defaultVideoQuality" => "videoQuality",
                "defaultVideoFormat" => "videoFormat",
                other => other,
            };

            match storage_key {
                "poToken" | "visitorData" | "cookies" => {
                    state.settings.set(&format!("ytdl.{}", storage_key), &str_val);
                    let config_update = serde_json::json!({ storage_key: str_val });
                    state.ytdl_manager.update_config(config_update);
                }
                _ => {
                    state.settings.set(&format!("ytdl.{}", storage_key), &str_val);
                }
            }
        }
    }
    Json(serde_json::json!({ "ok": true }))
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ytdl_manager.get_stats()).unwrap())
}

async fn ytdlp_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "available": true,
        "version": format!("native-rust-{}", env!("CARGO_PKG_VERSION")),
        "downloading": false,
    }))
}

async fn stream_download_video(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let progress = match state.ytdl_manager.get_progress(&id) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let path_str = match progress.video_output_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if !std::path::Path::new(&path_str).exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let range = headers
        .get(axum::http::header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    crate::api::libraries::stream_file(&path_str, range.as_deref()).await
}

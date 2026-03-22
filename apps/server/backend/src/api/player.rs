use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

const DEFAULT_SIGNALING_URL: &str = "https://mhaol-signaling.project-arktosmos.partykit.dev";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/playable", get(list_playable))
        .route("/stream-status", get(stream_status))
        .route("/sessions", post(create_session))
        .route("/sessions/{id}", delete(delete_session))
}

#[derive(Serialize)]
struct PlayableFile {
    id: String,
    name: String,
    path: String,
    source: String,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<f64>,
    #[serde(rename = "streamUrl", skip_serializing_if = "Option::is_none")]
    stream_url: Option<String>,
}

async fn list_playable(State(state): State<AppState>) -> impl IntoResponse {
    let mut files: Vec<PlayableFile> = Vec::new();

    // Torrent downloads (completed + in-progress with enough data to stream)
    let torrent_downloads = state.torrent_downloads.get_all();
    for dl in torrent_downloads {
        let is_complete = dl.state == "seeding" || dl.progress >= 1.0;
        let is_streamable = dl.state == "downloading" && dl.progress >= 0.02;

        if (is_complete || is_streamable) && dl.output_path.is_some() {
            let path = match (&dl.output_path, &dl.name) {
                (Some(p), name) if !name.is_empty() => format!("{}/{}", p, name),
                (Some(p), _) => p.clone(),
                _ => continue,
            };

            let (progress, stream_url) = if is_streamable && !is_complete {
                (
                    Some(dl.progress),
                    Some(format!("/api/torrent/torrents/{}/stream", dl.info_hash)),
                )
            } else {
                (None, None)
            };

            files.push(PlayableFile {
                id: format!("torrent:{}", dl.info_hash),
                name: dl.name,
                path,
                source: "torrent".to_string(),
                media_type: "video".to_string(),
                completed_at: Some(dl.updated_at.clone()),
                progress,
                stream_url,
            });
        }
    }

    // Library items (video and audio only)
    let video_items = state.library_items.get_by_media_type("video");
    let audio_items = state.library_items.get_by_media_type("audio");

    for item in video_items.iter().chain(audio_items.iter()) {
        // Extract filename from path for display name
        let name = std::path::Path::new(&item.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| item.path.clone());

        files.push(PlayableFile {
            id: format!("library:{}", item.id),
            name,
            path: item.path.clone(),
            source: "library".to_string(),
            media_type: item.media_type.clone(),
            completed_at: Some(item.created_at.clone()),
            progress: None,
            stream_url: None,
        });
    }

    // Sort by completed_at descending
    files.sort_by(|a, b| {
        let a_time = a.completed_at.as_deref().unwrap_or("");
        let b_time = b.completed_at.as_deref().unwrap_or("");
        b_time.cmp(a_time)
    });

    Json(files)
}

async fn stream_status(State(_state): State<AppState>) -> impl IntoResponse {
    #[cfg(not(target_os = "android"))]
    {
        if mhaol_p2p_stream::init().is_err() {
            return Json(serde_json::json!({ "available": false }));
        }
        let missing = mhaol_p2p_stream::check_required_elements();
        if missing.is_empty() {
            Json(serde_json::json!({ "available": true }))
        } else {
            Json(serde_json::json!({ "available": false, "missing_elements": missing }))
        }
    }
    #[cfg(target_os = "android")]
    Json(serde_json::json!({ "available": false }))
}

#[derive(Deserialize)]
struct CreateSessionBody {
    file_path: String,
    mode: Option<String>,
    video_codec: Option<String>,
    video_quality: Option<String>,
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> impl IntoResponse {
    let file_path = resolve_media_path(&body.file_path);
    if !std::path::Path::new(&file_path).exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("File not found: {}", file_path) })),
        )
            .into_response();
    }

    let signaling_url = DEFAULT_SIGNALING_URL.to_string();

    if !state.worker_bridge.is_ready() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Streaming worker is not running" })),
        )
            .into_response();
    }

    let session_id = uuid::Uuid::new_v4().to_string();

    match state
        .worker_bridge
        .create_session(
            &session_id,
            &file_path,
            &signaling_url,
            body.mode,
            body.video_codec,
            body.video_quality,
        )
        .await
    {
        Ok(crate::worker_bridge::WorkerEvent::SessionCreated { session_id, room_id }) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "session_id": session_id,
                "room_id": room_id,
                "signaling_url": signaling_url,
            })),
        )
            .into_response(),
        Ok(crate::worker_bridge::WorkerEvent::Error { error, .. }) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Unexpected worker response" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if state.worker_bridge.is_ready() {
        let _ = state.worker_bridge.delete_session(&id).await;
    }
    Json(serde_json::json!({ "ok": true }))
}

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"];

/// Resolve a media path to an actual video file.
/// - If it's a file, return as-is.
/// - If it's a directory, find the largest video file inside.
/// - If it doesn't exist, search the parent directory for the largest video file.
fn resolve_media_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_file() {
        return path.to_string();
    }
    if p.is_dir() {
        return find_largest_video(p).unwrap_or_else(|| path.to_string());
    }
    // Path doesn't exist — try the parent directory
    if let Some(parent) = p.parent() {
        if parent.is_dir() {
            if let Some(found) = find_largest_video(parent) {
                return found;
            }
        }
    }
    path.to_string()
}

fn find_largest_video(dir: &std::path::Path) -> Option<String> {
    let mut best: Option<(u64, String)> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let ft = entry.file_type().ok()?;
        let entry_path = entry.path();
        if ft.is_file() {
            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                if VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    if best.as_ref().is_none_or(|(s, _)| size > *s) {
                        best = Some((size, entry_path.to_string_lossy().to_string()));
                    }
                }
            }
        } else if ft.is_dir() {
            if let Some(found) = find_largest_video(&entry_path) {
                let size = std::fs::metadata(&found).map(|m| m.len()).unwrap_or(0);
                if best.as_ref().is_none_or(|(s, _)| size > *s) {
                    best = Some((size, found));
                }
            }
        }
    }
    best.map(|(_, path)| path)
}

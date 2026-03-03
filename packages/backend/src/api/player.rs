use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

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
}

async fn list_playable(State(state): State<AppState>) -> impl IntoResponse {
    let mut files: Vec<PlayableFile> = Vec::new();

    // YouTube completed downloads
    let yt_downloads = state.youtube_downloads.get_by_state("completed");
    for dl in yt_downloads {
        if let Some(path) = &dl.output_path {
            files.push(PlayableFile {
                id: format!("yt:{}", dl.download_id),
                name: dl.title.clone(),
                path: path.clone(),
                source: "youtube".to_string(),
                media_type: dl.mode.clone(),
                completed_at: Some(dl.updated_at.clone()),
            });
        }
    }

    // Torrent completed/seeding downloads
    let torrent_downloads = state.torrent_downloads.get_all();
    for dl in torrent_downloads {
        if (dl.state == "seeding" || dl.progress >= 1.0) && dl.output_path.is_some() {
            let path = match (&dl.output_path, &dl.name) {
                (Some(p), name) if !name.is_empty() => format!("{}/{}", p, name),
                (Some(p), _) => p.clone(),
                _ => continue,
            };
            files.push(PlayableFile {
                id: format!("torrent:{}", dl.info_hash),
                name: dl.name,
                path,
                source: "torrent".to_string(),
                media_type: "video".to_string(),
                completed_at: Some(dl.updated_at.clone()),
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
    if !std::path::Path::new(&body.file_path).exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("File not found: {}", body.file_path) })),
        )
            .into_response();
    }

    // Prefer local dev signaling server
    let signaling_url = if state.signaling_dev.is_available() {
        state.signaling_dev.dev_url()
    } else {
        state.settings.get("signaling.partyUrl").unwrap_or_default()
    };

    if signaling_url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Signaling server URL is not configured" })),
        )
            .into_response();
    }

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
            &body.file_path,
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

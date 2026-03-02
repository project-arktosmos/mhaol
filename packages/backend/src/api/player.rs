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
    let available = mhaol_p2p_stream::init().is_ok();
    Json(serde_json::json!({ "available": available }))
}

#[derive(Deserialize)]
struct CreateSessionBody {
    file_path: String,
}

async fn create_session(
    State(_state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> impl IntoResponse {
    if !std::path::Path::new(&body.file_path).exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("File not found: {}", body.file_path) })),
        )
            .into_response();
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "sessionId": session_id,
            "filePath": body.file_path,
        })),
    )
        .into_response()
}

async fn delete_session(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
}

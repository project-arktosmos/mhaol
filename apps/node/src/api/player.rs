use crate::worker_bridge::IceServerEntry;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

const DEFAULT_SIGNALING_URL: &str = "https://mhaol-signaling.project-arktosmos.partykit.dev";
const ICE_CACHE_TTL_MS: u128 = 12 * 60 * 60 * 1000;

fn get_signaling_url() -> String {
    std::env::var("SIGNALING_URL").unwrap_or_else(|_| DEFAULT_SIGNALING_URL.to_string())
}

struct IceCache {
    servers: Vec<IceServerEntry>,
    expires_at: u128,
}

static ICE_CACHE: OnceLock<Mutex<Option<IceCache>>> = OnceLock::new();

async fn fetch_ice_servers() -> Option<Vec<IceServerEntry>> {
    let cache_mutex = ICE_CACHE.get_or_init(|| Mutex::new(None));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    {
        let cache = cache_mutex.lock().await;
        if let Some(ref cached) = *cache {
            if now < cached.expires_at {
                return Some(cached.servers.clone());
            }
        }
    }

    // Priority: TURN_CREDENTIAL_URL (self-hosted) > METERED_DOMAIN+KEY (Metered.ca)
    let url = if let Ok(credential_url) = std::env::var("TURN_CREDENTIAL_URL") {
        if credential_url.is_empty() {
            return None;
        }
        credential_url
    } else {
        let domain = std::env::var("METERED_DOMAIN").ok()?;
        let secret_key = std::env::var("METERED_SECRET_KEY").ok()?;
        if domain.is_empty() || secret_key.is_empty() {
            tracing::warn!("[player] No TURN credential source configured (set TURN_CREDENTIAL_URL or METERED_DOMAIN+METERED_SECRET_KEY)");
            return None;
        }
        format!("https://{domain}/api/v1/turn/credentials?apiKey={secret_key}")
    };
    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Vec<IceServerEntry>>().await {
            Ok(servers) => {
                tracing::info!(
                    "[player] Fetched {} ICE servers from Metered",
                    servers.len()
                );
                let mut cache = cache_mutex.lock().await;
                *cache = Some(IceCache {
                    servers: servers.clone(),
                    expires_at: now + ICE_CACHE_TTL_MS,
                });
                Some(servers)
            }
            Err(e) => {
                tracing::warn!("[player] Failed to parse Metered response: {e}");
                None
            }
        },
        Ok(resp) => {
            tracing::warn!("[player] Metered API returned {}", resp.status());
            None
        }
        Err(e) => {
            tracing::warn!("[player] Metered API fetch error: {e}");
            None
        }
    }
}

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

    // Completed torrent downloads only
    let torrent_downloads = state.downloads.get_by_type("torrent");
    for dl in torrent_downloads {
        let is_complete = dl.state == "seeding" || dl.progress >= 1.0;

        if is_complete && dl.output_path.is_some() {
            let path = match (&dl.output_path, &dl.name) {
                (Some(p), name) if !name.is_empty() => format!("{}/{}", p, name),
                (Some(p), _) => p.clone(),
                _ => continue,
            };

            files.push(PlayableFile {
                id: format!("torrent:{}", dl.id),
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
    file_path: Option<String>,
    info_hash: Option<String>,
    stream_url: Option<String>,
    mode: Option<String>,
    video_codec: Option<String>,
    video_quality: Option<String>,
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> impl IntoResponse {
    let signaling_url = get_signaling_url();

    if !state.worker_bridge.is_ready() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Streaming worker is not running" })),
        )
            .into_response();
    }

    // Resolution order:
    // 1. file_path pointing to an actual file → use it directly (specific episode in a season pack).
    // 2. info_hash → fall back to "largest video file in torrent" resolution.
    // 3. file_path pointing to a directory or missing path → error.
    let resolved = match (body.file_path.as_ref(), body.info_hash.as_ref()) {
        (Some(file_path), _) => {
            let r = resolve_media_path(file_path);
            let p = std::path::Path::new(&r);
            if p.is_file() {
                r
            } else if let Some(info_hash) = body.info_hash.as_ref() {
                match resolve_torrent_video(&state, info_hash).await {
                    Ok(path) => path,
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({ "error": e })),
                        )
                            .into_response()
                    }
                }
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("File not found: {}", r) })),
                )
                    .into_response();
            }
        }
        (None, Some(info_hash)) => match resolve_torrent_video(&state, info_hash).await {
            Ok(path) => path,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
                    .into_response()
            }
        },
        (None, None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Either file_path or info_hash is required" })),
            )
                .into_response();
        }
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let ice_servers = fetch_ice_servers().await;

    match state
        .worker_bridge
        .create_session(
            &session_id,
            &resolved,
            body.stream_url,
            &signaling_url,
            body.mode,
            body.video_codec,
            body.video_quality,
            ice_servers,
        )
        .await
    {
        Ok(crate::worker_bridge::WorkerEvent::SessionCreated {
            session_id,
            room_id,
        }) => (
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

pub(crate) const VIDEO_EXTENSIONS: &[&str] =
    &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"];

/// Resolve the video file path for a torrent by its info_hash.
/// Uses the torrent manager's file list to find the largest video file,
/// then combines it with the download output_path from the DB.
async fn resolve_torrent_video(state: &AppState, info_hash: &str) -> Result<String, String> {
    let download = state
        .downloads
        .get(info_hash)
        .ok_or_else(|| format!("No download record for info_hash {}", info_hash))?;

    let output_path = download
        .output_path
        .ok_or_else(|| format!("No output_path for torrent {}", info_hash))?;

    let torrent_id = state
        .torrent_manager
        .list()
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .find(|t| t.info_hash.eq_ignore_ascii_case(info_hash))
        .map(|t| t.id)
        .ok_or_else(|| format!("Torrent {} not found in manager", info_hash))?;

    let files = state
        .torrent_manager
        .list_files(torrent_id)
        .await
        .map_err(|e| e.to_string())?;

    // Find the largest video file in this torrent's file list
    let video_file = files
        .iter()
        .filter(|f| {
            let name_lower = f.name.to_lowercase();
            VIDEO_EXTENSIONS
                .iter()
                .any(|ext| name_lower.ends_with(&format!(".{}", ext)))
        })
        .max_by_key(|f| f.size)
        .ok_or_else(|| format!("No video files found in torrent {}", info_hash))?;

    // The output_path from the DB is the torrent's root directory.
    // The file name from rqbit is relative to that root.
    // But rqbit may extract files directly into the parent directory if there's no
    // subdirectory in the torrent. Check both locations.
    let full_path = std::path::Path::new(&output_path).join(&video_file.name);
    if full_path.exists() {
        return Ok(full_path.to_string_lossy().to_string());
    }

    // rqbit sometimes extracts into the parent when the torrent has a single root folder
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let parent_path = parent.join(&video_file.name);
        if parent_path.exists() {
            return Ok(parent_path.to_string_lossy().to_string());
        }
    }

    Err(format!(
        "Video file {} not found at expected path {}",
        video_file.name,
        full_path.display()
    ))
}

/// Resolve a media path to an actual video file.
/// - If it's a file, return as-is.
/// - If it's a directory, find the largest video file inside (recursive).
/// - If it doesn't exist, check the parent for a matching video file (non-recursive).
pub(crate) fn resolve_media_path(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_file() {
        return path.to_string();
    }
    if p.is_dir() {
        return find_largest_video(p).unwrap_or_else(|| path.to_string());
    }
    // Path doesn't exist — try finding a video file in the parent that belongs
    // to this entry (same stem or contained within a same-named subdirectory).
    if let Some(parent) = p.parent() {
        if parent.is_dir() {
            if let Some(found) = find_largest_video_shallow(parent) {
                return found;
            }
        }
    }
    path.to_string()
}

/// Like find_largest_video but only checks direct file children — never recurses
/// into subdirectories. Used when falling back to the parent directory so we don't
/// pick up unrelated files from sibling torrent directories.
fn find_largest_video_shallow(dir: &std::path::Path) -> Option<String> {
    let mut best: Option<(u64, String)> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let ft = entry.file_type().ok()?;
        if !ft.is_file() {
            continue;
        }
        let entry_path = entry.path();
        if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
            if VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if best.as_ref().is_none_or(|(s, _)| size > *s) {
                    best = Some((size, entry_path.to_string_lossy().to_string()));
                }
            }
        }
    }
    best.map(|(_, path)| path)
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

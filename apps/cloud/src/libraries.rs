use crate::state::CloudState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use surrealdb::sql::Thing;

const TABLE: &str = "library";

/// Catalog kinds a library may declare it contains. Mirrors the catalog
/// addon type ids (`/api/catalog/sources`): `movie` + `tv` from TMDB,
/// `album` from MusicBrainz, `book` from OpenLibrary, `game` from
/// RetroAchievements.
pub const LIBRARY_KINDS: &[&str] = &["movie", "tv", "album", "book", "game"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: Option<Thing>,
    pub path: String,
    #[serde(default)]
    pub kinds: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub last_scanned_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct LibraryDto {
    pub id: String,
    pub path: String,
    pub kinds: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_scanned_at: Option<DateTime<Utc>>,
}

impl From<Library> for LibraryDto {
    fn from(lib: Library) -> Self {
        let id = lib
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            path: lib.path,
            kinds: lib.kinds,
            created_at: lib.created_at,
            updated_at: lib.updated_at,
            last_scanned_at: lib.last_scanned_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryRequest {
    pub path: String,
    #[serde(default)]
    pub kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLibraryRequest {
    pub path: String,
    #[serde(default)]
    pub kinds: Option<Vec<String>>,
}

fn sanitize_kinds(
    raw: Vec<String>,
) -> Result<Vec<String>, (StatusCode, Json<serde_json::Value>)> {
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    for k in raw.into_iter() {
        let trimmed = k.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if !LIBRARY_KINDS.contains(&trimmed.as_str()) {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                format!("invalid library kind: {trimmed}"),
            ));
        }
        if !out.iter().any(|x| x == &trimmed) {
            out.push(trimmed);
        }
    }
    Ok(out)
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
        .route("/{id}/scan", get(scan))
        .route("/{id}/pins", get(pins))
}

fn ensure_dir(path: &std::path::Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(path)
}

fn err_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": message.into() })),
    )
}

fn normalize_path(p: &str) -> String {
    PathBuf::from(p).to_string_lossy().to_string()
}

async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<LibraryDto>>, (StatusCode, Json<serde_json::Value>)> {
    let libs: Vec<Library> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut dtos: Vec<LibraryDto> = libs.into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

async fn get_one(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<LibraryDto>, (StatusCode, Json<serde_json::Value>)> {
    let lib: Option<Library> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    match lib {
        Some(l) => Ok(Json(l.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "library not found")),
    }
}

async fn create(
    State(state): State<CloudState>,
    Json(req): Json<CreateLibraryRequest>,
) -> Result<(StatusCode, Json<LibraryDto>), (StatusCode, Json<serde_json::Value>)> {
    let trimmed = req.path.trim();
    if trimmed.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "path is required"));
    }
    let path = PathBuf::from(trimmed);
    if let Err(e) = ensure_dir(&path) {
        return Err(err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create directory '{}': {e}", path.display()),
        ));
    }
    let normalized = normalize_path(&path.to_string_lossy());
    let kinds = sanitize_kinds(req.kinds)?;

    let existing: Vec<Library> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    if existing.iter().any(|l| l.path == normalized) {
        return Err(err_response(
            StatusCode::CONFLICT,
            format!("library already exists for '{normalized}'"),
        ));
    }

    let now = Utc::now();
    let new_id = uuid::Uuid::new_v4().to_string();
    let record = Library {
        id: None,
        path: normalized,
        kinds,
        created_at: now,
        updated_at: now,
        last_scanned_at: None,
    };

    let created: Option<Library> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let dto: LibraryDto = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "library was not persisted"))?
        .into();
    Ok((StatusCode::CREATED, Json(dto)))
}

async fn update(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateLibraryRequest>,
) -> Result<Json<LibraryDto>, (StatusCode, Json<serde_json::Value>)> {
    let existing: Option<Library> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "library not found"))?;

    let trimmed = req.path.trim();
    if trimmed.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "path cannot be empty"));
    }
    let buf = PathBuf::from(trimmed);
    if let Err(e) = ensure_dir(&buf) {
        return Err(err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create directory '{}': {e}", buf.display()),
        ));
    }
    let normalized = normalize_path(&buf.to_string_lossy());

    let all: Vec<Library> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    if all.iter().any(|l| {
        l.id.as_ref().map(|t| t.id.to_raw()).as_deref() != Some(id.as_str()) && l.path == normalized
    }) {
        return Err(err_response(
            StatusCode::CONFLICT,
            format!("library already exists for '{normalized}'"),
        ));
    }

    current.path = normalized;
    if let Some(kinds) = req.kinds {
        current.kinds = sanitize_kinds(kinds)?;
    }
    current.updated_at = Utc::now();
    current.id = None;

    let updated: Option<Library> = state
        .db
        .update((TABLE, id.as_str()))
        .content(current)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db update failed: {e}")))?;

    let dto: LibraryDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "library not found"))?
        .into();
    Ok(Json(dto))
}

async fn delete(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let existing: Option<Library> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let lib = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "library not found"))?;

    #[cfg(not(target_os = "android"))]
    clear_pins_for_library(&state, &lib.path).await;

    let removed: Option<Library> = state
        .db
        .delete((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db delete failed: {e}")))?;
    match removed {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(err_response(StatusCode::NOT_FOUND, "library not found")),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanEntry {
    pub path: String,
    pub relative_path: String,
    pub size: u64,
    pub mime: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub root: String,
    pub total_files: usize,
    pub total_size: u64,
    pub entries: Vec<ScanEntry>,
}

fn scan_directory(root: PathBuf) -> ScanResponse {
    let mut entries: Vec<ScanEntry> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in walkdir::WalkDir::new(&root).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let abs = entry.path();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let mime = mime_guess::from_path(abs)
            .first()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let relative = abs
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| abs.to_string_lossy().to_string());
        total_size += size;
        entries.push(ScanEntry {
            path: abs.to_string_lossy().to_string(),
            relative_path: relative,
            size,
            mime,
        });
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    ScanResponse {
        root: root.to_string_lossy().to_string(),
        total_files: entries.len(),
        total_size,
        entries,
    }
}

async fn scan(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<ScanResponse>, (StatusCode, Json<serde_json::Value>)> {
    let lib: Option<Library> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let lib = lib.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "library not found"))?;

    let root = PathBuf::from(&lib.path);
    if !root.exists() {
        return Err(err_response(
            StatusCode::NOT_FOUND,
            format!("library directory does not exist: {}", root.display()),
        ));
    }
    if !root.is_dir() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            format!("library path is not a directory: {}", root.display()),
        ));
    }

    let response = tokio::task::spawn_blocking(move || scan_directory(root))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("scan task failed: {e}")))?;

    let mut updated = lib.clone();
    updated.id = None;
    updated.last_scanned_at = Some(Utc::now());
    if let Err(e) = state
        .db
        .update::<Option<Library>>((TABLE, id.as_str()))
        .content(updated)
        .await
    {
        tracing::warn!("failed to record last_scanned_at for library {id}: {e}");
    }

    #[cfg(not(target_os = "android"))]
    {
        let kinds = lib.kinds.clone();
        let lib_root = lib.path.clone();
        crate::library_scan::schedule_pins_and_firkins(
            &state,
            &response.entries,
            kinds,
            lib_root,
        );
    }

    Ok(Json(response))
}

async fn pins(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::ipfs_pins::IpfsPinDto>>, (StatusCode, Json<serde_json::Value>)> {
    let lib: Option<Library> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let lib = lib.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "library not found"))?;

    let all: Vec<crate::ipfs_pins::IpfsPin> = state
        .db
        .select(crate::ipfs_pins::TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;

    let prefix = lib.path;
    let mut filtered: Vec<crate::ipfs_pins::IpfsPinDto> = all
        .into_iter()
        .filter(|p| path_under_prefix(&p.path, &prefix))
        .map(Into::into)
        .collect();
    filtered.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Json(filtered))
}

/// True when `path` equals `prefix` or sits under it as a directory child.
/// Plain `str::starts_with` would match `/foo2/bar` against `/foo` — this
/// requires the next character to be a path separator (or end of string).
fn path_under_prefix(path: &str, prefix: &str) -> bool {
    if !path.starts_with(prefix) {
        return false;
    }
    let rest = &path[prefix.len()..];
    rest.is_empty() || rest.starts_with('/') || rest.starts_with('\\')
}

#[cfg(not(target_os = "android"))]
pub(crate) fn is_pinnable_mime(mime: &str) -> bool {
    mime.starts_with("audio/") || mime.starts_with("video/") || mime.starts_with("image/")
}

#[cfg(not(target_os = "android"))]
pub(crate) async fn wait_for_ipfs_ready(
    ipfs: &std::sync::Arc<mhaol_ipfs::IpfsManager>,
) -> bool {
    use mhaol_ipfs::IpfsState;
    // Cap the wait at ~60s so a permanently-broken node doesn't pin tasks
    // forever. Most boots reach Running within a few seconds.
    for _ in 0..120 {
        match ipfs.state() {
            IpfsState::Running => return true,
            IpfsState::Error => return false,
            IpfsState::Stopped | IpfsState::Starting => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
    matches!(ipfs.state(), IpfsState::Running)
}

#[cfg(not(target_os = "android"))]
async fn clear_pins_for_library(state: &CloudState, lib_path: &str) {
    use surrealdb::sql::Thing;

    let pins: Vec<crate::ipfs_pins::IpfsPin> = match state.db.select(crate::ipfs_pins::TABLE).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("[libraries] failed to load pins for cleanup: {e}");
            return;
        }
    };

    let to_clear: Vec<(Thing, String)> = pins
        .into_iter()
        .filter(|p| path_under_prefix(&p.path, lib_path))
        .filter_map(|p| p.id.map(|id| (id, p.cid)))
        .collect();

    if to_clear.is_empty() {
        return;
    }

    tracing::info!(
        "[libraries] removing {} pin record(s) for library {}",
        to_clear.len(),
        lib_path
    );

    for (thing, cid) in to_clear {
        if let Err(e) = state.ipfs_manager.unpin(&cid).await {
            // Pin may not exist on the node (already gc'd, or never made it
            // there) — log and keep going so the DB record still goes away.
            tracing::warn!("[libraries] failed to unpin {cid} from ipfs: {e}");
        }
        let record_id = thing.id.to_raw();
        if let Err(e) = state
            .db
            .delete::<Option<crate::ipfs_pins::IpfsPin>>((crate::ipfs_pins::TABLE, record_id.as_str()))
            .await
        {
            tracing::warn!("[libraries] failed to delete pin record {record_id}: {e}");
        }
    }
}

impl IntoResponse for LibraryDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

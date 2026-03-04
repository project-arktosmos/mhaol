use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_libraries).post(create_library))
        .route("/{id}", delete(delete_library))
        .route("/{id}/items/{item_id}/category", post(update_item_category).delete(clear_item_category))
        .route(
            "/{id}/items/{item_id}/media-type",
            put(update_item_media_type),
        )
        .route(
            "/{id}/items/{item_id}/tmdb",
            put(link_tmdb).delete(unlink_tmdb),
        )
        .route(
            "/{id}/items/{item_id}/youtube",
            put(link_youtube).delete(unlink_youtube),
        )
        .route(
            "/{id}/items/{item_id}/musicbrainz",
            put(link_musicbrainz).delete(unlink_musicbrainz),
        )
        .route("/{id}/files", get(get_library_files))
        .route("/{id}/scan", get(scan_library).post(scan_library))
        .route("/browse", get(browse_directory))
        .route("/media-types", get(get_media_types))
        .route("/categories", get(get_categories))
}

// --- Response types matching frontend expectations ---

#[derive(Serialize)]
struct MappedLibrary {
    id: String,
    name: String,
    path: String,
    #[serde(rename = "mediaTypes")]
    media_types: Vec<String>,
    #[serde(rename = "dateAdded")]
    date_added: i64,
}

impl MappedLibrary {
    fn from_row(row: crate::db::repo::library::LibraryRow) -> Self {
        let media_types: Vec<String> =
            serde_json::from_str(&row.media_types).unwrap_or_default();
        Self {
            id: row.id,
            name: row.name,
            path: row.path,
            media_types,
            date_added: row.date_added,
        }
    }
}

#[derive(Serialize)]
struct MappedFileLink {
    #[serde(rename = "serviceId")]
    service_id: String,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
    #[serde(rename = "episodeNumber")]
    episode_number: Option<i64>,
}

#[derive(Serialize)]
struct MappedFile {
    id: String,
    name: String,
    path: String,
    extension: String,
    #[serde(rename = "mediaType")]
    media_type: String,
    #[serde(rename = "categoryId")]
    category_id: Option<String>,
    links: HashMap<String, MappedFileLink>,
}

#[derive(Serialize)]
struct LibraryFilesResponse {
    #[serde(rename = "libraryId")]
    library_id: String,
    #[serde(rename = "libraryPath")]
    library_path: String,
    files: Vec<MappedFile>,
}

#[derive(Serialize)]
struct DirectoryEntry {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct BrowseResponse {
    path: String,
    parent: Option<String>,
    directories: Vec<DirectoryEntry>,
}

// --- Helper to map library items with their links ---

fn map_library_files(state: &AppState, library_id: &str) -> Vec<MappedFile> {
    let items = state.library_items.get_by_library(library_id);
    items
        .into_iter()
        .map(|item| {
            let link_rows = state.library_item_links.get_by_item(&item.id);
            let mut links = HashMap::new();
            for link in link_rows {
                links.insert(
                    link.service,
                    MappedFileLink {
                        service_id: link.service_id,
                        season_number: link.season_number,
                        episode_number: link.episode_number,
                    },
                );
            }
            let name = std::path::Path::new(&item.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            MappedFile {
                id: item.id,
                name,
                path: item.path,
                extension: item.extension,
                media_type: item.media_type,
                category_id: item.category_id,
                links,
            }
        })
        .collect()
}

// --- Route handlers ---

async fn list_libraries(State(state): State<AppState>) -> impl IntoResponse {
    let libraries: Vec<MappedLibrary> = state
        .libraries
        .get_all()
        .into_iter()
        .map(MappedLibrary::from_row)
        .collect();
    Json(libraries)
}

#[derive(Deserialize)]
struct CreateLibraryBody {
    name: String,
    path: String,
    #[serde(alias = "media_types", alias = "mediaTypes")]
    media_types: Vec<String>,
}

async fn create_library(
    State(state): State<AppState>,
    Json(body): Json<CreateLibraryBody>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let media_types_json = serde_json::to_string(&body.media_types).unwrap_or_else(|_| "[]".into());
    let date_added = chrono::Utc::now().timestamp_millis();
    state
        .libraries
        .insert(&id, &body.name, &body.path, &media_types_json, date_added);
    (
        StatusCode::CREATED,
        Json(MappedLibrary {
            id,
            name: body.name,
            path: body.path,
            media_types: body.media_types,
            date_added,
        }),
    )
}

async fn delete_library(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.library_items.delete_by_library(&id);
    state.libraries.delete(&id);
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct UpdateCategoryBody {
    #[serde(alias = "category_id", alias = "categoryId")]
    category_id: String,
}

async fn update_item_category(
    State(state): State<AppState>,
    Path((_lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<UpdateCategoryBody>,
) -> impl IntoResponse {
    state
        .library_items
        .update_category(&item_id, &body.category_id);
    StatusCode::OK
}

async fn clear_item_category(
    State(state): State<AppState>,
    Path((_lib_id, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    state.library_items.clear_category(&item_id);
    StatusCode::OK
}

#[derive(Deserialize)]
struct UpdateMediaTypeBody {
    #[serde(alias = "media_type", alias = "mediaType", alias = "mediaTypeId")]
    media_type: String,
}

async fn update_item_media_type(
    State(state): State<AppState>,
    Path((_lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<UpdateMediaTypeBody>,
) -> impl IntoResponse {
    state
        .library_items
        .update_media_type(&item_id, &body.media_type);
    StatusCode::OK
}

/// GET /api/libraries/{id}/files — returns files with links in frontend format
async fn get_library_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let library = match state.libraries.get(&id) {
        Some(lib) => lib,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Library not found" })),
            )
                .into_response();
        }
    };

    let files = map_library_files(&state, &id);
    Json(LibraryFilesResponse {
        library_id: id,
        library_path: library.path,
        files,
    })
    .into_response()
}

/// POST/GET /api/libraries/{id}/scan — scan directory and return updated files
async fn scan_library(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let library = match state.libraries.get(&id) {
        Some(lib) => lib,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Library not found" })),
            )
                .into_response();
        }
    };

    let media_types: Vec<String> =
        serde_json::from_str(&library.media_types).unwrap_or_default();
    let ext_map = build_extension_map(&media_types);

    if !ext_map.is_empty() {
        let mut scanned_files = Vec::new();
        scan_dir(&library.path, &id, &ext_map, &mut scanned_files);
        state.library_items.sync_library(&id, &scanned_files);
    }

    generate_auto_lists(&state, &id);

    // Spawn background task to pre-fetch YouTube oEmbed metadata
    {
        let db = state.db.clone();
        let items = state.library_items.get_by_library(&id);
        let links_repo = state.library_item_links.clone();
        tokio::spawn(async move {
            let mut video_ids: Vec<String> = Vec::new();
            for item in &items {
                if let Some(link) = links_repo.get_by_item_and_service(&item.id, "youtube") {
                    if link.service_id.len() == 11 && !video_ids.contains(&link.service_id) {
                        video_ids.push(link.service_id);
                    }
                }
            }
            if !video_ids.is_empty() {
                tracing::info!(
                    "Pre-fetching YouTube metadata for {} video(s)",
                    video_ids.len()
                );
                for vid in &video_ids {
                    if super::youtube::fetch_and_cache_oembed(&db, vid).await.is_none() {
                        tracing::warn!("Failed to pre-fetch YouTube metadata for {}", vid);
                    }
                }
            }
        });
    }

    let files = map_library_files(&state, &id);
    Json(LibraryFilesResponse {
        library_id: id,
        library_path: library.path,
        files,
    })
    .into_response()
}

#[derive(Deserialize)]
struct BrowseQuery {
    path: Option<String>,
}

/// GET /api/libraries/browse — browse directories in BrowseDirectoryResponse format
async fn browse_directory(Query(query): Query<BrowseQuery>) -> impl IntoResponse {
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(_) => {
            return Json(BrowseResponse {
                path: path.clone(),
                parent: std::path::Path::new(&path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string()),
                directories: Vec::new(),
            });
        }
    };

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    dirs.push(DirectoryEntry {
                        name: name.to_string(),
                        path: entry.path().to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    dirs.sort_by(|a, b| a.name.cmp(&b.name));

    let parent = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string());

    Json(BrowseResponse {
        path,
        parent,
        directories: dirs,
    })
}

async fn get_media_types(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.media_types.get_all())
}

async fn get_categories(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.categories.get_all())
}

// --- Library item link handlers ---

fn validate_item(state: &AppState, lib_id: &str, item_id: &str) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    match state.library_items.get(item_id) {
        Some(item) if item.library_id == lib_id => Ok(()),
        _ => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Library item not found" })))),
    }
}

#[derive(Deserialize)]
struct LinkTmdbBody {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
    #[serde(rename = "episodeNumber")]
    episode_number: Option<i64>,
}

async fn link_tmdb(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<LinkTmdbBody>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    state.library_item_links.upsert(
        &uuid::Uuid::new_v4().to_string(), &item_id, "tmdb",
        &body.tmdb_id.to_string(), body.season_number, body.episode_number,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_tmdb(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    state.library_item_links.delete(&item_id, "tmdb");
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct LinkYoutubeBody {
    #[serde(rename = "youtubeId")]
    youtube_id: String,
}

async fn link_youtube(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<LinkYoutubeBody>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    let yt_id = body.youtube_id.trim();
    if yt_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "youtubeId must be a non-empty string" }))).into_response();
    }
    state.library_item_links.upsert(
        &uuid::Uuid::new_v4().to_string(), &item_id, "youtube", yt_id, None, None,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_youtube(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    state.library_item_links.delete(&item_id, "youtube");
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct LinkMusicbrainzBody {
    #[serde(rename = "musicbrainzId")]
    musicbrainz_id: String,
}

async fn link_musicbrainz(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<LinkMusicbrainzBody>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    let mb_id = body.musicbrainz_id.trim();
    if mb_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "musicbrainzId must be a non-empty string" }))).into_response();
    }
    state.library_item_links.upsert(
        &uuid::Uuid::new_v4().to_string(), &item_id, "musicbrainz", mb_id, None, None,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_musicbrainz(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }
    state.library_item_links.delete(&item_id, "musicbrainz");
    Json(serde_json::json!({ "ok": true })).into_response()
}

// --- Scan helpers (ported from packages/tauri/src-tauri/src/commands/db.rs) ---

fn build_extension_map(media_types: &[String]) -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    for mt in media_types {
        match mt.as_str() {
            "video" => {
                for ext in &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"] {
                    map.insert(*ext, "video");
                }
            }
            "audio" => {
                for ext in &["mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "opus"] {
                    map.insert(*ext, "audio");
                }
            }
            "image" => {
                for ext in &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "tiff"] {
                    map.insert(*ext, "image");
                }
            }
            _ => {}
        }
    }
    map
}

fn scan_dir(
    dir: &str,
    library_id: &str,
    ext_map: &HashMap<&str, &str>,
    files: &mut Vec<crate::db::repo::library_item::InsertLibraryItem>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let path = entry.path();

        if file_type.is_dir() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with('.') {
                scan_dir(&path.to_string_lossy(), library_id, ext_map, files);
            }
        } else if file_type.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if let Some(media_type) = ext_map.get(ext_lower.as_str()) {
                    files.push(crate::db::repo::library_item::InsertLibraryItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        library_id: library_id.to_string(),
                        path: path.to_string_lossy().to_string(),
                        extension: ext_lower,
                        media_type: media_type.to_string(),
                        category_id: None,
                    });
                }
            }
        }
    }
}

/// After syncing library items, auto-generate media lists from directories
/// containing 2+ files of the same media type.
fn generate_auto_lists(state: &AppState, library_id: &str) {
    let items = state.library_items.get_by_library(library_id);

    // Group items by parent directory and media type
    let mut dir_items: HashMap<String, Vec<&crate::db::repo::library_item::LibraryItemRow>> =
        HashMap::new();
    for item in &items {
        if let Some(parent) = std::path::Path::new(&item.path).parent() {
            let dir = parent.to_string_lossy().to_string();
            dir_items.entry(dir).or_default().push(item);
        }
    }

    let mut active_source_paths: HashSet<String> = HashSet::new();

    for (dir_path, dir_files) in &dir_items {
        let mut video_items: Vec<&crate::db::repo::library_item::LibraryItemRow> = Vec::new();
        let mut audio_items: Vec<&crate::db::repo::library_item::LibraryItemRow> = Vec::new();

        for item in dir_files {
            match item.media_type.as_str() {
                "video" => video_items.push(item),
                "audio" => audio_items.push(item),
                _ => {}
            }
        }

        if video_items.len() >= 2 {
            let source_key = format!("{}:video", dir_path);
            active_source_paths.insert(source_key.clone());
            upsert_auto_list(state, library_id, dir_path, "video", &source_key, &mut video_items);
        }

        if audio_items.len() >= 2 {
            let source_key = format!("{}:audio", dir_path);
            active_source_paths.insert(source_key.clone());
            upsert_auto_list(state, library_id, dir_path, "audio", &source_key, &mut audio_items);
        }
    }

    // Cleanup: remove auto lists whose source_path is no longer active
    let existing_auto = state.media_lists.get_auto_by_library(library_id);
    for list in existing_auto {
        if let Some(ref sp) = list.source_path {
            if !active_source_paths.contains(sp) {
                state.media_list_items.delete_by_list(&list.id);
                state.media_lists.delete(&list.id);
            }
        }
    }
}

fn upsert_auto_list(
    state: &AppState,
    library_id: &str,
    dir_path: &str,
    media_type: &str,
    source_path: &str,
    items: &mut [&crate::db::repo::library_item::LibraryItemRow],
) {
    let list_id = match state.media_lists.get_by_source_path(source_path) {
        Some(existing) => existing.id,
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            let title = std::path::Path::new(dir_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            state.media_lists.insert(
                &id,
                library_id,
                &title,
                None,
                None,
                media_type,
                "auto",
                Some(source_path),
            );
            id
        }
    };

    items.sort_by(|a, b| a.path.cmp(&b.path));
    let list_items: Vec<(String, String, i64)> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            (
                uuid::Uuid::new_v4().to_string(),
                item.id.clone(),
                i as i64,
            )
        })
        .collect();
    state.media_list_items.sync_list(&list_id, &list_items);
}

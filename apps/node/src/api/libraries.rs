use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_libraries).post(create_library))
        .route("/{id}", delete(delete_library))
        .route("/{id}/items", post(create_item))
        .route("/{id}/items/{item_id}/category", post(update_item_category).delete(clear_item_category))
        .route(
            "/{id}/items/{item_id}/media-type",
            put(update_item_media_type),
        )
        .route(
            "/{id}/items/{item_id}/tmdb",
            put(link_tmdb).delete(unlink_tmdb),
        )
        .route("/{id}/items/{item_id}/torrent", put(link_torrent))

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
    #[serde(rename = "libraryType")]
    library_type: String,
    #[serde(rename = "dateAdded")]
    date_added: i64,
}

impl MappedLibrary {
    fn from_row(row: crate::db::repo::library::LibraryRow) -> Self {
        let media_types: Vec<String> =
            serde_json::from_str(&row.media_types).unwrap_or_default();
        let library_type = media_types.into_iter().next().unwrap_or_else(|| "movies".to_string());
        Self {
            id: row.id,
            name: row.name,
            path: row.path,
            library_type,
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
    #[serde(alias = "library_type", alias = "libraryType")]
    library_type: String,
}

async fn create_library(
    State(state): State<AppState>,
    Json(body): Json<CreateLibraryBody>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let media_types_json = serde_json::to_string(&[&body.library_type]).unwrap_or_else(|_| "[]".into());
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
            library_type: body.library_type,
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
struct CreateItemBody {
    name: String,
    path: String,
    #[serde(alias = "mediaType")]
    media_type: Option<String>,
    #[serde(alias = "categoryId")]
    category_id: Option<String>,
    #[serde(alias = "tmdbId")]
    tmdb_id: Option<i64>,
}

async fn create_item(
    State(state): State<AppState>,
    Path(library_id): Path<String>,
    Json(body): Json<CreateItemBody>,
) -> impl IntoResponse {
    if state.libraries.get(&library_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Library not found" })),
        )
            .into_response();
    }

    // If an item with this path already exists, return it (and update TMDB link if provided)
    if let Some(existing_id) = state.library_items.exists_by_path(&body.path) {
        if let Some(tmdb_id) = body.tmdb_id {
            state.library_item_links.upsert(
                &uuid::Uuid::new_v4().to_string(),
                &existing_id,
                "tmdb",
                &tmdb_id.to_string(),
                None,
                None,
            );
        }
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": existing_id,
                "libraryId": library_id,
                "name": body.name,
                "path": body.path,
            })),
        )
            .into_response();
    }

    let item_id = uuid::Uuid::new_v4().to_string();
    let extension = std::path::Path::new(&body.path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if let Err(e) = state.library_items.insert(&crate::db::repo::library_item::InsertLibraryItem {
        id: item_id.clone(),
        library_id: library_id.clone(),
        path: body.path.clone(),
        extension,
        media_type: body.media_type.unwrap_or_else(|| "video".to_string()),
        category_id: body.category_id,
    }) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    if let Some(tmdb_id) = body.tmdb_id {
        state.library_item_links.upsert(
            &uuid::Uuid::new_v4().to_string(),
            &item_id,
            "tmdb",
            &tmdb_id.to_string(),
            None,
            None,
        );
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": item_id,
            "libraryId": library_id,
            "name": body.name,
            "path": body.path,
        })),
    )
        .into_response()
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
    let library_type = media_types.first().map(|s| s.as_str()).unwrap_or("movies");

    let ext_map = build_extension_map(library_type);
    let mut scanned_files = Vec::new();
    scan_dir(&library.path, &id, &ext_map, &mut scanned_files);
    state.library_items.sync_library(&id, &scanned_files);

    generate_auto_lists(&state, &id, &library.path, library_type);

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
struct LinkTorrentBody {
    #[serde(rename = "infoHash")]
    info_hash: String,
    #[serde(rename = "outputPath")]
    output_path: String,
    mode: String,
}

async fn link_torrent(
    State(state): State<AppState>,
    Path((lib_id, item_id)): Path<(String, String)>,
    Json(body): Json<LinkTorrentBody>,
) -> impl IntoResponse {
    if let Err(e) = validate_item(&state, &lib_id, &item_id) { return e.into_response(); }

    state.library_items.update_path(&item_id, &body.output_path);

    let service = format!("torrent-{}", body.mode);
    state.library_item_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &item_id,
        &service,
        &body.info_hash,
        None,
        None,
    );

    Json(serde_json::json!({ "ok": true })).into_response()
}

// --- Scan helpers ---

fn build_extension_map(library_type: &str) -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    match library_type {
        "movies" | "tv" | "video" => {
            for ext in &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"] {
                map.insert(*ext, "video");
            }
        }
        _ => {}
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

/// Returns video file paths directly inside a directory (non-recursive).
fn video_files_in_dir(dir: &str, ext_map: &HashMap<&str, &str>) -> Vec<String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_file() { continue }
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext_map.contains_key(ext.to_lowercase().as_str()) {
                result.push(path.to_string_lossy().to_string());
            }
        }
    }
    result
}

/// Returns immediate subdirectory paths inside a directory.
fn subdirs_of(dir: &str) -> Vec<(String, String)> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_dir() { continue }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue }
        result.push((name, entry.path().to_string_lossy().to_string()));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// After syncing library items, auto-generate media lists.
fn generate_auto_lists(state: &AppState, library_id: &str, library_path: &str, library_type: &str) {
    let items = state.library_items.get_by_library(library_id);
    let ext_map = build_extension_map(library_type);

    let mut active_source_paths: HashSet<String> = HashSet::new();

    if library_type == "tv" {
        // TV: walk immediate subdirs of library root.
        // Each subdir is either a show (has season subdirs with episodes) or a flat show/season.
        for (show_name, show_path) in subdirs_of(library_path) {
            let season_dirs = subdirs_of(&show_path);
            let has_season_subdirs = season_dirs.iter().any(|(_, season_path)| {
                !video_files_in_dir(season_path, &ext_map).is_empty()
            });

            if has_season_subdirs {
                // Show with season subdirectories — create show-level list, then season children
                let show_source_key = format!("{}:show", show_path);
                active_source_paths.insert(show_source_key.clone());
                let show_list_id = upsert_auto_list_titled(state, library_id, &show_name, "video", &show_source_key, &mut [], None);

                for (season_name, season_path) in &season_dirs {
                    let season_files = video_files_in_dir(season_path, &ext_map);
                    if season_files.is_empty() { continue }

                    let source_key = format!("{}:video", season_path);
                    active_source_paths.insert(source_key.clone());

                    let mut season_items: Vec<&crate::db::repo::library_item::LibraryItemRow> = items
                        .iter()
                        .filter(|i| {
                            std::path::Path::new(&i.path)
                                .parent()
                                .map(|p| p.to_string_lossy() == season_path.as_str())
                                .unwrap_or(false)
                        })
                        .collect();

                    if !season_items.is_empty() {
                        upsert_auto_list_titled(state, library_id, season_name, "video", &source_key, &mut season_items, Some(&show_list_id));
                    }
                }
            } else {
                // Flat show or standalone season — episodes directly inside show_path
                let flat_files = video_files_in_dir(&show_path, &ext_map);
                if flat_files.is_empty() { continue }

                let source_key = format!("{}:show", show_path);
                active_source_paths.insert(source_key.clone());

                let mut show_items: Vec<&crate::db::repo::library_item::LibraryItemRow> = items
                    .iter()
                    .filter(|i| {
                        std::path::Path::new(&i.path)
                            .parent()
                            .map(|p| p.to_string_lossy() == show_path.as_str())
                            .unwrap_or(false)
                    })
                    .collect();

                if !show_items.is_empty() {
                    upsert_auto_list_titled(state, library_id, &show_name, "video", &source_key, &mut show_items, None);
                }
            }
        }
    } else {
        // Movies (and legacy): dir with 2+ video files → auto list named after dir.
        let mut dir_items: HashMap<String, Vec<&crate::db::repo::library_item::LibraryItemRow>> =
            HashMap::new();
        for item in &items {
            if let Some(parent) = std::path::Path::new(&item.path).parent() {
                let dir = parent.to_string_lossy().to_string();
                dir_items.entry(dir).or_default().push(item);
            }
        }

        for (dir_path, dir_files) in &dir_items {
            let mut video_items: Vec<&crate::db::repo::library_item::LibraryItemRow> = dir_files
                .iter()
                .filter(|i| i.media_type == "video")
                .copied()
                .collect();

            if video_items.len() >= 2 {
                let source_key = format!("{}:video", dir_path);
                active_source_paths.insert(source_key.clone());
                let title = std::path::Path::new(dir_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled")
                    .to_string();
                upsert_auto_list_titled(state, library_id, &title, "video", &source_key, &mut video_items, None);
            }
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

fn upsert_auto_list_titled(
    state: &AppState,
    library_id: &str,
    title: &str,
    media_type: &str,
    source_path: &str,
    items: &mut [&crate::db::repo::library_item::LibraryItemRow],
    parent_list_id: Option<&str>,
) -> String {
    let list_id = match state.media_lists.get_by_source_path(source_path) {
        Some(existing) => existing.id,
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            state.media_lists.insert(
                &id,
                library_id,
                title,
                None,
                None,
                media_type,
                "auto",
                Some(source_path),
                parent_list_id,
            );
            id
        }
    };

    if !items.is_empty() {
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

    list_id
}

pub(crate) async fn stream_file(path_str: &str, range_header: Option<&str>) -> axum::response::Response {
    let path = std::path::Path::new(path_str);

    let file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let file_size = match file.metadata().await {
        Ok(m) => m.len(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("opus") => "audio/opus",
        Some("aac") => "audio/aac",
        _ => "application/octet-stream",
    };

    if let Some(range_str) = range_header {
        if let Some(range_val) = range_str.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_val.splitn(2, '-').collect();
            if parts.len() == 2 {
                let start: u64 = parts[0].parse().unwrap_or(0);
                let end: u64 = parts[1]
                    .parse()
                    .unwrap_or_else(|_| file_size.saturating_sub(1))
                    .min(file_size.saturating_sub(1));

                if start >= file_size || start > end {
                    return Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                        .body(Body::empty())
                        .unwrap();
                }

                let length = end - start + 1;
                let mut file = file;

                if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                let limited = file.take(length);
                let stream = ReaderStream::new(limited);

                return Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, file_size),
                    )
                    .header(header::CONTENT_LENGTH, length.to_string())
                    .header(header::ACCEPT_RANGES, "bytes")
                    .body(Body::from_stream(stream))
                    .unwrap();
            }
        }
    }

    // No Range header — stream entire file
    let stream = ReaderStream::new(file);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from_stream(stream))
        .unwrap()
}

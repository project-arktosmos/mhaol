use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::collections::HashMap;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_libraries).post(create_library))
        .route("/{id}", delete(delete_library))
        .route("/{id}/items/{item_id}/category", post(update_item_category))
        .route(
            "/{id}/items/{item_id}/media-type",
            post(update_item_media_type),
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

async fn list_libraries(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.libraries.get_all())
}

#[derive(Deserialize)]
struct CreateLibraryBody {
    name: String,
    path: String,
    media_types: Vec<String>,
}

async fn create_library(
    State(state): State<AppState>,
    Json(body): Json<CreateLibraryBody>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let media_types = serde_json::to_string(&body.media_types).unwrap_or_else(|_| "[]".into());
    let date_added = chrono::Utc::now().timestamp_millis();
    state
        .libraries
        .insert(&id, &body.name, &body.path, &media_types, date_added);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id })),
    )
}

async fn delete_library(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.libraries.delete(&id);
    StatusCode::NO_CONTENT
}

#[derive(Deserialize)]
struct UpdateCategoryBody {
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

#[derive(Deserialize)]
struct UpdateMediaTypeBody {
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

async fn get_library_files(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(state.library_items.get_by_library(&id))
}

/// Scan a library directory for media files and sync with the database.
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
            );
        }
    };

    let media_types: Vec<String> =
        serde_json::from_str(&library.media_types).unwrap_or_default();
    let ext_map = build_extension_map(&media_types);

    if ext_map.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!({ "count": 0 })));
    }

    let mut files = Vec::new();
    scan_dir(&library.path, &id, &ext_map, &mut files);

    let count = files.len();
    state.library_items.sync_library(&id, &files);

    (StatusCode::OK, Json(serde_json::json!({ "count": count })))
}

#[derive(Deserialize)]
struct BrowseQuery {
    path: Option<String>,
}

async fn browse_directory(Query(query): Query<BrowseQuery>) -> impl IntoResponse {
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let entries = match std::fs::read_dir(&path) {
        Ok(e) => e,
        Err(_) => return Json(Vec::<String>::new()),
    };

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    dirs.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    dirs.sort();
    Json(dirs)
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

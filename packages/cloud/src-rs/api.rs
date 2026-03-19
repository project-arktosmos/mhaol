use crate::manager::CloudManager;
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Build the cloud API router. Mount this under `/api/cloud` in the host app.
pub fn router() -> Router<Arc<CloudManager>> {
    Router::new()
        .route("/libraries", get(list_libraries).post(create_library))
        .route("/libraries/browse", get(browse_directory))
        .route("/libraries/{id}", delete(delete_library))
        .route("/libraries/{id}/scan", post(scan_library))
        .route("/libraries/{id}/items", get(get_library_items))
        .route("/items/{id}", get(get_item))
        .route("/items/{id}/attributes", put(set_attributes))
        .route("/items/{id}/attributes/{key}", delete(delete_attribute))
        .route("/attributes/keys", get(get_distinct_keys))
        .route("/attributes/values/{key}", get(get_distinct_values))
        .route("/search", get(search_items))
}

// --- Response types ---

#[derive(Serialize)]
struct MappedCloudLibrary {
    id: String,
    name: String,
    path: String,
    kind: String,
    #[serde(rename = "scanStatus")]
    scan_status: String,
    #[serde(rename = "scanError")]
    scan_error: Option<String>,
    #[serde(rename = "itemCount")]
    item_count: i64,
}

impl From<CloudLibraryRow> for MappedCloudLibrary {
    fn from(lib: CloudLibraryRow) -> Self {
        Self {
            id: lib.id,
            name: lib.name,
            path: lib.path,
            kind: lib.kind,
            scan_status: lib.scan_status,
            scan_error: lib.scan_error,
            item_count: lib.item_count,
        }
    }
}

#[derive(Serialize)]
struct MappedAttribute {
    key: String,
    value: String,
    #[serde(rename = "typeId")]
    type_id: String,
    source: String,
    confidence: Option<f64>,
}

#[derive(Serialize)]
struct MappedItemLink {
    service: String,
    #[serde(rename = "serviceId")]
    service_id: String,
    extra: Option<String>,
}

#[derive(Serialize)]
struct MappedCloudItem {
    id: String,
    #[serde(rename = "libraryId")]
    library_id: String,
    path: String,
    filename: String,
    extension: String,
    #[serde(rename = "sizeBytes")]
    size_bytes: Option<i64>,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    attributes: Vec<MappedAttribute>,
    links: Vec<MappedItemLink>,
}

fn map_item(mgr: &CloudManager, item: &CloudItemRow) -> MappedCloudItem {
    let attrs = mgr.get_item_attributes(&item.id);
    let links = mgr.get_item_links(&item.id);
    MappedCloudItem {
        id: item.id.clone(),
        library_id: item.library_id.clone(),
        path: item.path.clone(),
        filename: item.filename.clone(),
        extension: item.extension.clone(),
        size_bytes: item.size_bytes,
        mime_type: item.mime_type.clone(),
        attributes: attrs
            .into_iter()
            .map(|a| MappedAttribute {
                key: a.key,
                value: a.value,
                type_id: a.attribute_type_id,
                source: a.source,
                confidence: a.confidence,
            })
            .collect(),
        links: links
            .into_iter()
            .map(|l| MappedItemLink {
                service: l.service,
                service_id: l.service_id,
                extra: l.extra,
            })
            .collect(),
    }
}

fn map_items(mgr: &CloudManager, items: &[CloudItemRow]) -> Vec<MappedCloudItem> {
    items.iter().map(|i| map_item(mgr, i)).collect()
}

// --- Handlers ---

async fn list_libraries(State(mgr): State<Arc<CloudManager>>) -> impl IntoResponse {
    let libraries: Vec<MappedCloudLibrary> = mgr
        .list_libraries()
        .into_iter()
        .map(MappedCloudLibrary::from)
        .collect();
    Json(libraries)
}

#[derive(Deserialize)]
struct CreateLibraryBody {
    name: String,
    path: String,
    #[serde(default = "default_kind")]
    kind: String,
}

fn default_kind() -> String {
    "filesystem".to_string()
}

async fn create_library(
    State(mgr): State<Arc<CloudManager>>,
    Json(body): Json<CreateLibraryBody>,
) -> impl IntoResponse {
    let lib = mgr.create_library(&body.name, &body.path, &body.kind);
    (StatusCode::CREATED, Json(MappedCloudLibrary::from(lib)))
}

async fn delete_library(
    State(mgr): State<Arc<CloudManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    mgr.delete_library(&id);
    StatusCode::NO_CONTENT
}

async fn scan_library(
    State(mgr): State<Arc<CloudManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match mgr.scan_library(&id) {
        Some(result) => {
            let mapped = map_items(&mgr, &result.items);
            Json(serde_json::json!({
                "libraryId": result.library_id,
                "libraryPath": result.library_path,
                "itemCount": result.item_count,
                "items": mapped,
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Library not found" })),
        )
            .into_response(),
    }
}

async fn get_library_items(
    State(mgr): State<Arc<CloudManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let items = mgr.get_library_items(&id);
    Json(map_items(&mgr, &items))
}

async fn get_item(
    State(mgr): State<Arc<CloudManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match mgr.get_item(&id) {
        Some(item) => Json(map_item(&mgr, &item)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Item not found" })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SetAttributeBody {
    key: String,
    value: String,
    #[serde(rename = "typeId", default = "default_string")]
    type_id: String,
    #[serde(default = "default_user")]
    source: String,
    confidence: Option<f64>,
}

fn default_string() -> String {
    "string".to_string()
}
fn default_user() -> String {
    "user".to_string()
}

async fn set_attributes(
    State(mgr): State<Arc<CloudManager>>,
    Path(id): Path<String>,
    Json(body): Json<Vec<SetAttributeBody>>,
) -> impl IntoResponse {
    for attr in body {
        mgr.set_attribute(&id, &attr.key, &attr.value, &attr.type_id, &attr.source, attr.confidence);
    }
    StatusCode::OK
}

async fn delete_attribute(
    State(mgr): State<Arc<CloudManager>>,
    Path((id, key)): Path<(String, String)>,
) -> impl IntoResponse {
    mgr.delete_attribute(&id, &key);
    StatusCode::NO_CONTENT
}

async fn get_distinct_keys(State(mgr): State<Arc<CloudManager>>) -> impl IntoResponse {
    Json(mgr.distinct_keys())
}

async fn get_distinct_values(
    State(mgr): State<Arc<CloudManager>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    Json(mgr.distinct_values(&key))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    key: Option<String>,
    value: Option<String>,
}

async fn search_items(
    State(mgr): State<Arc<CloudManager>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    if let (Some(key), Some(value)) = (&query.key, &query.value) {
        let items = mgr.search_by_attribute(key, value);
        return Json(map_items(&mgr, &items)).into_response();
    }

    if let Some(q) = &query.q {
        let items = mgr.search_by_filename(q);
        return Json(map_items(&mgr, &items)).into_response();
    }

    Json(Vec::<MappedCloudItem>::new()).into_response()
}

// --- Browse ---

#[derive(Deserialize)]
struct BrowseQuery {
    path: Option<String>,
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
            })
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

    Json(BrowseResponse {
        parent: std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string()),
        path,
        directories: dirs,
    })
}

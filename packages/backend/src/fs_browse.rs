use crate::state::CloudState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

#[derive(Debug, Deserialize)]
pub struct BrowseQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirEntryDto {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BrowseResponse {
    pub path: String,
    pub parent: Option<String>,
    pub home: String,
    pub separator: String,
    pub roots: Vec<DirEntryDto>,
    pub entries: Vec<DirEntryDto>,
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/browse", get(browse))
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(target_os = "windows")]
fn list_roots() -> Vec<DirEntryDto> {
    let mut out = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let p = PathBuf::from(&drive);
        if p.exists() {
            out.push(DirEntryDto {
                name: format!("{}:", letter as char),
                path: drive,
            });
        }
    }
    out
}

#[cfg(not(target_os = "windows"))]
fn list_roots() -> Vec<DirEntryDto> {
    vec![DirEntryDto {
        name: "/".to_string(),
        path: "/".to_string(),
    }]
}

fn parent_of(path: &Path) -> Option<String> {
    path.parent().map(|p| p.to_string_lossy().to_string())
}

async fn browse(
    State(_state): State<CloudState>,
    Query(q): Query<BrowseQuery>,
) -> Result<Json<BrowseResponse>, (StatusCode, Json<serde_json::Value>)> {
    let target = match q.path.as_ref().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => home_dir(),
    };

    if !target.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("path does not exist: {}", target.display()) })),
        ));
    }
    if !target.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("not a directory: {}", target.display()) })),
        ));
    }

    let mut entries: Vec<DirEntryDto> = Vec::new();
    let read = std::fs::read_dir(&target).map_err(|e| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": format!("cannot read directory: {e}") })),
        )
    })?;

    for entry in read.flatten() {
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        entries.push(DirEntryDto {
            name,
            path: entry.path().to_string_lossy().to_string(),
        });
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(BrowseResponse {
        path: target.to_string_lossy().to_string(),
        parent: parent_of(&target),
        home: home_dir().to_string_lossy().to_string(),
        separator: MAIN_SEPARATOR.to_string(),
        roots: list_roots(),
        entries,
    }))
}

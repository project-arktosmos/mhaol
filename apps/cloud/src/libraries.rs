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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: Option<Thing>,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LibraryDto {
    pub id: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            created_at: lib.created_at,
            updated_at: lib.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLibraryRequest {
    pub path: String,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
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
        created_at: now,
        updated_at: now,
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

impl IntoResponse for LibraryDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

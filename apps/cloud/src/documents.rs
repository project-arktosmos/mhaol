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
use surrealdb::sql::Thing;

const TABLE: &str = "document";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<Thing>,
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DocumentDto {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Document> for DocumentDto {
    fn from(doc: Document) -> Self {
        let id = doc
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            name: doc.name,
            author: doc.author,
            description: doc.description,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub author: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
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

async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<DocumentDto>>, (StatusCode, Json<serde_json::Value>)> {
    let docs: Vec<Document> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut dtos: Vec<DocumentDto> = docs.into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

async fn get_one(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentDto>, (StatusCode, Json<serde_json::Value>)> {
    let doc: Option<Document> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    match doc {
        Some(d) => Ok(Json(d.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "document not found")),
    }
}

async fn create(
    State(state): State<CloudState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<DocumentDto>), (StatusCode, Json<serde_json::Value>)> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "name is required"));
    }
    let author = req.author.trim();
    if author.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "author is required"));
    }
    let description = req
        .description
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    let now = Utc::now();
    let new_id = uuid::Uuid::new_v4().to_string();
    let record = Document {
        id: None,
        name: name.to_string(),
        author: author.to_string(),
        description,
        created_at: now,
        updated_at: now,
    };

    let created: Option<Document> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let dto: DocumentDto = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "document was not persisted"))?
        .into();
    Ok((StatusCode::CREATED, Json(dto)))
}

async fn update(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentDto>, (StatusCode, Json<serde_json::Value>)> {
    let existing: Option<Document> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "document not found"))?;

    if let Some(name) = req.name.as_ref().map(|n| n.trim()) {
        if name.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "name cannot be empty"));
        }
        current.name = name.to_string();
    }

    if let Some(author) = req.author.as_ref().map(|a| a.trim()) {
        if author.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "author cannot be empty"));
        }
        current.author = author.to_string();
    }

    if let Some(description) = req.description.as_ref() {
        current.description = description.trim().to_string();
    }

    current.updated_at = Utc::now();
    current.id = None;

    let updated: Option<Document> = state
        .db
        .update((TABLE, id.as_str()))
        .content(current)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db update failed: {e}")))?;

    let dto: DocumentDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "document not found"))?
        .into();
    Ok(Json(dto))
}

async fn delete(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let removed: Option<Document> = state
        .db
        .delete((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db delete failed: {e}")))?;
    match removed {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(err_response(StatusCode::NOT_FOUND, "document not found")),
    }
}

impl IntoResponse for DocumentDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

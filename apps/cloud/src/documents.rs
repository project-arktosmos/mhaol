use crate::state::CloudState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use cid::Cid;
use multihash::Multihash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::sql::Thing;

const TABLE: &str = "document";

const SHA2_256_CODE: u64 = 0x12;
const RAW_CODEC: u64 = 0x55;

const ALLOWED_TYPES: &[&str] = &[
    "movie",
    "tv season",
    "tv episode",
    "tv show",
    "album",
    "track",
    "image",
    "youtube video",
    "youtube channel",
    "book",
    "game",
];

const ALLOWED_SOURCES: &[&str] = &[
    "tmdb",
    "torrent-search-thepiratebay",
    "torrent-search-spanish",
    "musicbrainz",
    "retroachievements",
    "youtube",
    "lrclib",
    "openlibrary",
    "wyzie-subs",
];

#[derive(Serialize)]
struct DocumentPayloadView<'a> {
    title: &'a str,
    author: &'a str,
    description: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
    source: &'a str,
}

fn compute_document_cid(
    title: &str,
    author: &str,
    description: &str,
    kind: &str,
    source: &str,
) -> String {
    let view = DocumentPayloadView { title, author, description, kind, source };
    let json = serde_json::to_string_pretty(&view)
        .expect("DocumentPayloadView serializes to JSON");
    let digest = Sha256::digest(json.as_bytes());
    let mh = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .expect("sha2-256 digest fits in multihash");
    Cid::new_v1(RAW_CODEC, mh).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<Thing>,
    #[serde(alias = "name")]
    pub title: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DocumentDto {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub source: String,
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
            title: doc.title,
            author: doc.author,
            description: doc.description,
            kind: doc.kind,
            source: doc.source,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub source: Option<String>,
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
    let title = req.title.trim();
    if title.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "title is required"));
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
    let kind = req.kind.trim();
    if kind.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "type is required"));
    }
    if !ALLOWED_TYPES.contains(&kind) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            format!("invalid type: {kind}"),
        ));
    }
    let source = req.source.trim();
    if source.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "source is required"));
    }
    if !ALLOWED_SOURCES.contains(&source) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            format!("invalid source: {source}"),
        ));
    }

    let now = Utc::now();
    let new_id = compute_document_cid(title, author, &description, kind, source);

    let existing: Option<Document> = state
        .db
        .select((TABLE, new_id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    if let Some(existing) = existing {
        return Ok((StatusCode::OK, Json(existing.into())));
    }

    let record = Document {
        id: None,
        title: title.to_string(),
        author: author.to_string(),
        description,
        kind: kind.to_string(),
        source: source.to_string(),
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

    if let Some(title) = req.title.as_ref().map(|t| t.trim()) {
        if title.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "title cannot be empty"));
        }
        current.title = title.to_string();
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

    if let Some(kind) = req.kind.as_ref().map(|k| k.trim()) {
        if kind.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "type cannot be empty"));
        }
        if !ALLOWED_TYPES.contains(&kind) {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                format!("invalid type: {kind}"),
            ));
        }
        current.kind = kind.to_string();
    }

    if let Some(source) = req.source.as_ref().map(|s| s.trim()) {
        if source.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "source cannot be empty"));
        }
        if !ALLOWED_SOURCES.contains(&source) {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                format!("invalid source: {source}"),
            ));
        }
        current.source = source.to_string();
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

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

pub const TABLE: &str = "document";

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
    "musicbrainz",
    "retroachievements",
    "youtube",
    "lrclib",
    "openlibrary",
    "wyzie-subs",
    "local",
];

const ALLOWED_FILE_TYPES: &[&str] = &["ipfs", "torrent magnet", "url"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artist {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "imageUrl", default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageMeta {
    pub url: String,
    #[serde(rename = "mimeType", default)]
    pub mime_type: String,
    #[serde(rename = "fileSize", default)]
    pub file_size: u64,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Serialize)]
struct DocumentPayloadView<'a> {
    title: &'a str,
    description: &'a str,
    artists: &'a [Artist],
    images: &'a [ImageMeta],
    files: &'a [FileEntry],
    year: Option<i32>,
    #[serde(rename = "type")]
    kind: &'a str,
    source: &'a str,
    version: u32,
    version_hashes: &'a [String],
}

#[allow(clippy::too_many_arguments)]
pub fn compute_document_cid(
    title: &str,
    description: &str,
    artists: &[Artist],
    images: &[ImageMeta],
    files: &[FileEntry],
    year: Option<i32>,
    kind: &str,
    source: &str,
    version: u32,
    version_hashes: &[String],
) -> String {
    let view = DocumentPayloadView {
        title,
        description,
        artists,
        images,
        files,
        year,
        kind,
        source,
        version,
        version_hashes,
    };
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
    #[serde(default)]
    pub artists: Vec<Artist>,
    pub description: String,
    #[serde(default)]
    pub images: Vec<ImageMeta>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub version_hashes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentDto {
    pub id: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub description: String,
    pub images: Vec<ImageMeta>,
    pub files: Vec<FileEntry>,
    pub year: Option<i32>,
    #[serde(rename = "type")]
    pub kind: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub version_hashes: Vec<String>,
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
            artists: doc.artists,
            description: doc.description,
            images: doc.images,
            files: doc.files,
            year: doc.year,
            kind: doc.kind,
            source: doc.source,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            version: doc.version,
            version_hashes: doc.version_hashes,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    #[serde(default)]
    pub artists: Vec<Artist>,
    pub description: Option<String>,
    #[serde(default)]
    pub images: Vec<ImageMeta>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(rename = "type")]
    pub kind: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub artists: Option<Vec<Artist>>,
    pub description: Option<String>,
    pub images: Option<Vec<ImageMeta>>,
    pub files: Option<Vec<FileEntry>>,
    pub year: Option<i32>,
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
    let artists: Vec<Artist> = req
        .artists
        .into_iter()
        .filter_map(|a| {
            let name = a.name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            Some(Artist {
                name,
                url: a.url.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                image_url: a
                    .image_url
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            })
        })
        .collect();
    let images: Vec<ImageMeta> = req
        .images
        .into_iter()
        .filter(|img| !img.url.trim().is_empty())
        .map(|img| ImageMeta {
            url: img.url.trim().to_string(),
            mime_type: img.mime_type.trim().to_string(),
            file_size: img.file_size,
            width: img.width,
            height: img.height,
        })
        .collect();
    let mut files: Vec<FileEntry> = Vec::with_capacity(req.files.len());
    for f in req.files.into_iter() {
        let value = f.value.trim().to_string();
        let title = f
            .title
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        if value.is_empty() && title.is_none() {
            continue;
        }
        let kind = f.kind.trim();
        if !ALLOWED_FILE_TYPES.contains(&kind) {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                format!("invalid file type: {kind}"),
            ));
        }
        files.push(FileEntry {
            kind: kind.to_string(),
            value,
            title,
        });
    }

    let year = req.year.filter(|y| (1000..=9999).contains(y));

    let now = Utc::now();
    let version: u32 = 0;
    let version_hashes: Vec<String> = Vec::new();
    let new_id = compute_document_cid(
        title,
        &description,
        &artists,
        &images,
        &files,
        year,
        kind,
        source,
        version,
        &version_hashes,
    );

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
        artists,
        description,
        images,
        files,
        year,
        kind: kind.to_string(),
        source: source.to_string(),
        created_at: now,
        updated_at: now,
        version,
        version_hashes,
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

    if let Some(artists) = req.artists {
        current.artists = artists
            .into_iter()
            .filter_map(|a| {
                let name = a.name.trim().to_string();
                if name.is_empty() {
                    return None;
                }
                Some(Artist {
                    name,
                    url: a.url.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                    image_url: a
                        .image_url
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                })
            })
            .collect();
    }

    if let Some(images) = req.images {
        current.images = images
            .into_iter()
            .filter(|img| !img.url.trim().is_empty())
            .map(|img| ImageMeta {
                url: img.url.trim().to_string(),
                mime_type: img.mime_type.trim().to_string(),
                file_size: img.file_size,
                width: img.width,
                height: img.height,
            })
            .collect();
    }

    if let Some(files) = req.files {
        let mut next: Vec<FileEntry> = Vec::with_capacity(files.len());
        for f in files.into_iter() {
            let value = f.value.trim().to_string();
            let title = f
                .title
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty());
            if value.is_empty() && title.is_none() {
                continue;
            }
            let kind = f.kind.trim();
            if !ALLOWED_FILE_TYPES.contains(&kind) {
                return Err(err_response(
                    StatusCode::BAD_REQUEST,
                    format!("invalid file type: {kind}"),
                ));
            }
            next.push(FileEntry {
                kind: kind.to_string(),
                value,
                title,
            });
        }
        current.files = next;
    }

    if let Some(description) = req.description.as_ref() {
        current.description = description.trim().to_string();
    }

    if let Some(year) = req.year {
        current.year = if (1000..=9999).contains(&year) {
            Some(year)
        } else {
            None
        };
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

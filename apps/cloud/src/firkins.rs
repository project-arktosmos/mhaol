use crate::catalog::is_known_addon;
use crate::state::CloudState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use cid::Cid;
use multihash::Multihash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::sql::Thing;

pub const TABLE: &str = "firkin";

const SHA2_256_CODE: u64 = 0x12;
const RAW_CODEC: u64 = 0x55;

const ALLOWED_FILE_TYPES: &[&str] = &["ipfs", "torrent magnet", "url"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artist {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
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
struct FirkinPayloadView<'a> {
    title: &'a str,
    description: &'a str,
    artists: &'a [Artist],
    images: &'a [ImageMeta],
    files: &'a [FileEntry],
    year: Option<i32>,
    addon: &'a str,
    version: u32,
    version_hashes: &'a [String],
}

pub fn serialize_firkin_payload(
    title: &str,
    description: &str,
    artists: &[Artist],
    images: &[ImageMeta],
    files: &[FileEntry],
    year: Option<i32>,
    addon: &str,
    version: u32,
    version_hashes: &[String],
) -> String {
    let view = FirkinPayloadView {
        title,
        description,
        artists,
        images,
        files,
        year,
        addon,
        version,
        version_hashes,
    };
    serde_json::to_string_pretty(&view).expect("FirkinPayloadView serializes to JSON")
}

pub fn compute_firkin_cid(
    title: &str,
    description: &str,
    artists: &[Artist],
    images: &[ImageMeta],
    files: &[FileEntry],
    year: Option<i32>,
    addon: &str,
    version: u32,
    version_hashes: &[String],
) -> String {
    let json = serialize_firkin_payload(
        title,
        description,
        artists,
        images,
        files,
        year,
        addon,
        version,
        version_hashes,
    );
    let digest = Sha256::digest(json.as_bytes());
    let mh = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .expect("sha2-256 digest fits in multihash");
    Cid::new_v1(RAW_CODEC, mh).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Firkin {
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
    /// Addon id (single source of identity for the firkin's content kind).
    /// Replaces the prior split between `type` and `source`.
    #[serde(default, alias = "source")]
    pub addon: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub version_hashes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FirkinDto {
    pub id: String,
    pub title: String,
    pub artists: Vec<Artist>,
    pub description: String,
    pub images: Vec<ImageMeta>,
    pub files: Vec<FileEntry>,
    pub year: Option<i32>,
    pub addon: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub version_hashes: Vec<String>,
}

impl From<Firkin> for FirkinDto {
    fn from(doc: Firkin) -> Self {
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
            addon: doc.addon,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            version: doc.version,
            version_hashes: doc.version_hashes,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFirkinRequest {
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
    pub addon: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFirkinRequest {
    pub title: Option<String>,
    pub artists: Option<Vec<Artist>>,
    pub description: Option<String>,
    pub images: Option<Vec<ImageMeta>>,
    pub files: Option<Vec<FileEntry>>,
    pub year: Option<i32>,
    pub addon: Option<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
        .route("/{id}/finalize", post(finalize))
        .route("/{id}/roms", get(roms))
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
) -> Result<Json<Vec<FirkinDto>>, (StatusCode, Json<serde_json::Value>)> {
    let docs: Vec<Firkin> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;

    // A firkin is "superseded" if its id appears in another firkin's
    // `version_hashes` (i.e. some newer record has rolled it forward).
    // Hide superseded ones so list consumers (e.g. the /catalog Library
    // section) only ever see the current head of each version chain —
    // even when a previous rollforward attempt left the old record in
    // place after creating the new one.
    let mut superseded: std::collections::HashSet<String> = std::collections::HashSet::new();
    for d in &docs {
        for h in &d.version_hashes {
            superseded.insert(h.clone());
        }
    }

    let mut dtos: Vec<FirkinDto> = docs
        .into_iter()
        .filter(|d| {
            let id = d.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
            !superseded.contains(&id)
        })
        .map(Into::into)
        .collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

async fn get_one(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    let doc: Option<Firkin> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    match doc {
        Some(d) => Ok(Json(d.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "firkin not found")),
    }
}

async fn create(
    State(state): State<CloudState>,
    Json(req): Json<CreateFirkinRequest>,
) -> Result<(StatusCode, Json<FirkinDto>), (StatusCode, Json<serde_json::Value>)> {
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
    let addon = req.addon.trim();
    if addon.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "addon is required"));
    }
    if !is_known_addon(addon) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            format!("invalid addon: {addon}"),
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
    let body_json = serialize_firkin_payload(
        title,
        &description,
        &artists,
        &images,
        &files,
        year,
        addon,
        version,
        &version_hashes,
    );
    let digest = Sha256::digest(body_json.as_bytes());
    let mh = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .expect("sha2-256 digest fits in multihash");
    let new_id = Cid::new_v1(RAW_CODEC, mh).to_string();

    let existing: Option<Firkin> = state
        .db
        .select((TABLE, new_id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    if let Some(existing) = existing {
        return Ok((StatusCode::OK, Json(existing.into())));
    }

    let record = Firkin {
        id: None,
        title: title.to_string(),
        artists,
        description,
        images,
        files,
        year,
        addon: addon.to_string(),
        created_at: now,
        updated_at: now,
        version,
        version_hashes,
    };

    let created: Option<Firkin> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let dto: FirkinDto = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "firkin was not persisted"))?
        .into();

    pin_firkin_body(&state, &new_id, body_json).await;

    Ok((StatusCode::CREATED, Json(dto)))
}

/// Pin the firkin body's JSON bytes to the embedded IPFS node and record an
/// `ipfs_pin` row keyed on the synthetic path `firkin://<id>`. Best-effort:
/// failures are logged but do not surface to the caller, since the IPFS node
/// can still be initializing when the first firkins are created.
#[cfg(not(target_os = "android"))]
pub(crate) async fn pin_firkin_body(state: &CloudState, firkin_id: &str, body_json: String) {
    let bytes = body_json.into_bytes();
    let size = bytes.len() as u64;
    let info = match state
        .ipfs_manager
        .add_bytes(format!("firkin-{firkin_id}.json"), bytes)
        .await
    {
        Ok(info) => info,
        Err(e) => {
            tracing::warn!("[firkins] failed to pin body for {firkin_id}: {e}");
            return;
        }
    };
    if let Err(e) = crate::ipfs_pins::record_pin(
        state,
        info.cid,
        format!("firkin://{firkin_id}"),
        "application/json".to_string(),
        size,
    )
    .await
    {
        tracing::warn!("[firkins] failed to record pin row for {firkin_id}: {e}");
    }
}

#[cfg(target_os = "android")]
pub(crate) async fn pin_firkin_body(_state: &CloudState, _firkin_id: &str, _body_json: String) {}

async fn update(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFirkinRequest>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    let existing: Option<Firkin> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?;

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

    if let Some(addon) = req.addon.as_ref().map(|a| a.trim()) {
        if addon.is_empty() {
            return Err(err_response(StatusCode::BAD_REQUEST, "addon cannot be empty"));
        }
        if !is_known_addon(addon) {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                format!("invalid addon: {addon}"),
            ));
        }
        current.addon = addon.to_string();
    }

    current.updated_at = Utc::now();
    current.id = None;

    let updated: Option<Firkin> = state
        .db
        .update((TABLE, id.as_str()))
        .content(current)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db update failed: {e}")))?;

    let dto: FirkinDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?
        .into();
    Ok(Json(dto))
}

async fn delete(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let removed: Option<Firkin> = state
        .db
        .delete((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db delete failed: {e}")))?;
    match removed {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(err_response(StatusCode::NOT_FOUND, "firkin not found")),
    }
}

impl IntoResponse for FirkinDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[cfg(not(target_os = "android"))]
async fn finalize(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    let latest_id = crate::torrent_completion::finalize_firkin(&state, &id)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("finalize failed: {e}"),
            )
        })?;
    let latest_id = latest_id.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?;
    let doc: Option<Firkin> = state
        .db
        .select((TABLE, latest_id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    match doc {
        Some(d) => Ok(Json(d.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "firkin not found")),
    }
}

#[cfg(target_os = "android")]
async fn finalize(
    State(_state): State<CloudState>,
    Path(_id): Path<String>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    Err(err_response(
        StatusCode::NOT_IMPLEMENTED,
        "finalize is not supported on this platform",
    ))
}

#[cfg(not(target_os = "android"))]
async fn roms(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<crate::rom_extract::RomsResponse>, (StatusCode, Json<serde_json::Value>)> {
    crate::rom_extract::extract_roms_for_firkin(&state, &id)
        .await
        .map(Json)
        .map_err(|e| {
            let msg = e.to_string();
            let status = if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            err_response(status, msg)
        })
}

#[cfg(target_os = "android")]
async fn roms(
    State(_state): State<CloudState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err(err_response(
        StatusCode::NOT_IMPLEMENTED,
        "rom extraction is not supported on this platform",
    ))
}

use crate::artists::{self, ArtistDto, UpsertArtistRequest};
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

/// A YouTube trailer attached to a firkin. Movies carry a single trailer
/// for the film; TV shows carry one per season (with `label` set to the
/// season name, e.g. `"Season 1"`). Resolved client-side via the same
/// double-dip YouTube extraction stack the music-track flow uses, then
/// persisted on the firkin so subsequent visits don't re-search.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trailer {
    #[serde(rename = "youtubeUrl")]
    pub youtube_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn slice_is_empty<T>(s: &[T]) -> bool {
    s.is_empty()
}

#[derive(Serialize)]
struct FirkinPayloadView<'a> {
    title: &'a str,
    description: &'a str,
    /// Artist CIDs only — the artist bodies are content-addressed
    /// `artist` records (see [`crate::artists`]). The firkin's CID
    /// therefore depends on the canonical CIDs of its referenced artists,
    /// not on their mutable presentation fields.
    artists: &'a [String],
    images: &'a [ImageMeta],
    files: &'a [FileEntry],
    year: Option<i32>,
    addon: &'a str,
    /// EVM address of the account that created the firkin. Empty string when
    /// the firkin was synthesised by a server-side flow without user context
    /// (e.g. the library scanner).
    creator: &'a str,
    version: u32,
    version_hashes: &'a [String],
    /// Skipped when empty so existing firkin CIDs (created before
    /// trailers existed) remain stable across deserialise → re-serialise.
    #[serde(skip_serializing_if = "slice_is_empty")]
    trailers: &'a [Trailer],
}

pub fn serialize_firkin_payload(
    title: &str,
    description: &str,
    artist_cids: &[String],
    images: &[ImageMeta],
    files: &[FileEntry],
    year: Option<i32>,
    addon: &str,
    creator: &str,
    version: u32,
    version_hashes: &[String],
    trailers: &[Trailer],
) -> String {
    let view = FirkinPayloadView {
        title,
        description,
        artists: artist_cids,
        images,
        files,
        year,
        addon,
        creator,
        version,
        version_hashes,
        trailers,
    };
    serde_json::to_string_pretty(&view).expect("FirkinPayloadView serializes to JSON")
}

pub fn compute_firkin_cid(
    title: &str,
    description: &str,
    artist_cids: &[String],
    images: &[ImageMeta],
    files: &[FileEntry],
    year: Option<i32>,
    addon: &str,
    creator: &str,
    version: u32,
    version_hashes: &[String],
    trailers: &[Trailer],
) -> String {
    let json = serialize_firkin_payload(
        title,
        description,
        artist_cids,
        images,
        files,
        year,
        addon,
        creator,
        version,
        version_hashes,
        trailers,
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
    /// CIDs of the `artist` records this firkin references. The artist
    /// bodies live in their own table + IPFS pins; the firkin only stores
    /// the content-addressed handles so its own CID stays stable across
    /// presentation-only edits to an artist (e.g. a new image URL).
    #[serde(default)]
    pub artists: Vec<String>,
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
    /// EVM address of the account that created the firkin. Filled from the
    /// browser-resident user identity on user-initiated creates; empty for
    /// server-side auto-creates (library scan).
    #[serde(default)]
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub version_hashes: Vec<String>,
    /// YouTube trailers resolved against this firkin. Movies hold one
    /// entry; TV shows hold one entry per season (with `label` set to
    /// `"Season N"`). Empty by default; resolved client-side after the
    /// firkin is created and persisted via PUT.
    #[serde(default)]
    pub trailers: Vec<Trailer>,
}

#[derive(Debug, Serialize)]
pub struct FirkinDto {
    pub id: String,
    pub title: String,
    /// Artist CIDs as persisted on the firkin record. Drives version
    /// hashing and lets clients re-fetch artist docs by CID.
    #[serde(rename = "artistIds")]
    pub artist_ids: Vec<String>,
    /// Resolved artist bodies, in the same order as `artist_ids`. CIDs
    /// that no longer have a backing record are dropped — the order of
    /// the resolved entries still matches the surviving subset of CIDs.
    pub artists: Vec<ArtistDto>,
    pub description: String,
    pub images: Vec<ImageMeta>,
    pub files: Vec<FileEntry>,
    pub year: Option<i32>,
    pub addon: String,
    /// EVM address of the account that created the firkin. Empty when the
    /// record predates the field or was created by a server-side flow.
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub version_hashes: Vec<String>,
    pub trailers: Vec<Trailer>,
}

impl FirkinDto {
    fn from_doc_with_artists(doc: Firkin, artists: Vec<ArtistDto>) -> Self {
        let id = doc
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            title: doc.title,
            artist_ids: doc.artists,
            artists,
            description: doc.description,
            images: doc.images,
            files: doc.files,
            year: doc.year,
            addon: doc.addon,
            creator: doc.creator,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            version: doc.version,
            version_hashes: doc.version_hashes,
            trailers: doc.trailers,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFirkinRequest {
    pub title: String,
    /// Inline artist objects — the server creates the underlying `artist`
    /// records (or reuses existing ones with the same content-address)
    /// before computing the firkin CID. Callers must always speak in
    /// objects, never in raw CIDs, so the same upstream catalog item maps
    /// to the same firkin CID regardless of which client created it.
    #[serde(default)]
    pub artists: Vec<UpsertArtistRequest>,
    pub description: Option<String>,
    #[serde(default)]
    pub images: Vec<ImageMeta>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub year: Option<i32>,
    pub addon: String,
    /// Optional creator address. Defaults to empty when omitted; the WebUI
    /// fills it from the browser-resident user identity (see the profile
    /// page) so every user-created firkin is attributable.
    #[serde(default)]
    pub creator: Option<String>,
    /// Optional trailers to bake into the firkin at create time. The
    /// virtual-catalog page resolves trailers up-front (movies: one;
    /// TV shows: one per season) so the firkin is born with them already
    /// attached.
    #[serde(default)]
    pub trailers: Vec<Trailer>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFirkinRequest {
    pub title: Option<String>,
    pub artists: Option<Vec<UpsertArtistRequest>>,
    pub description: Option<String>,
    pub images: Option<Vec<ImageMeta>>,
    pub files: Option<Vec<FileEntry>>,
    pub year: Option<i32>,
    pub addon: Option<String>,
    pub trailers: Option<Vec<Trailer>>,
}

/// Request body for `POST /api/firkins/:id/enrich`. Carries the metadata
/// fields we typically pull from a catalog API match (title, year,
/// description, poster + backdrop URLs). Only present fields are applied;
/// images are replaced wholesale with the provided poster / backdrop.
#[derive(Debug, Deserialize)]
pub struct EnrichFirkinRequest {
    pub title: Option<String>,
    pub year: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "posterUrl")]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl")]
    pub backdrop_url: Option<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
        .route("/{id}/finalize", post(finalize))
        .route("/{id}/enrich", post(enrich))
        .route("/{id}/roms", post(roms))
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

/// Materialise inline artist requests into stable artist CIDs. Each
/// request is sanitised, deduped against the `artist` table, and pinned to
/// IPFS via [`artists::upsert`]. The returned `(cids, dtos)` are aligned
/// 1-1 — the dtos are used for the response body, the cids for the firkin's
/// `artists` field (and so its CID).
async fn materialise_artists(
    state: &CloudState,
    requests: Vec<UpsertArtistRequest>,
) -> Result<(Vec<String>, Vec<ArtistDto>), (StatusCode, Json<serde_json::Value>)> {
    let mut cids: Vec<String> = Vec::with_capacity(requests.len());
    let mut dtos: Vec<ArtistDto> = Vec::with_capacity(requests.len());
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for req in requests {
        let Some(body) = artists::sanitize(req) else {
            continue;
        };
        let dto = artists::upsert(state, body).await?;
        if !seen.insert(dto.id.clone()) {
            // Same artist provided twice (e.g. cast list duplicates) —
            // keep first occurrence only so the firkin body stays stable.
            continue;
        }
        cids.push(dto.id.clone());
        dtos.push(dto);
    }
    Ok((cids, dtos))
}

async fn assemble_firkin_dto(
    state: &CloudState,
    doc: Firkin,
) -> Result<FirkinDto, (StatusCode, Json<serde_json::Value>)> {
    let artists = artists::fetch_many(state, &doc.artists)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("artist fetch failed: {e}")))?;
    Ok(FirkinDto::from_doc_with_artists(doc, artists))
}

pub(crate) async fn assemble_firkin_dtos(
    state: &CloudState,
    docs: Vec<Firkin>,
) -> Result<Vec<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    // Resolve all referenced artists in one pass (deduped) so listing is
    // O(unique-artists) DB hits rather than O(firkins * artists).
    let mut unique_ids: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for doc in &docs {
        for cid in &doc.artists {
            if seen.insert(cid.as_str()) {
                unique_ids.push(cid.clone());
            }
        }
    }
    let resolved = artists::fetch_many(state, &unique_ids)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("artist fetch failed: {e}")))?;
    let by_id: std::collections::HashMap<String, ArtistDto> = resolved
        .into_iter()
        .map(|d| (d.id.clone(), d))
        .collect();
    let mut out: Vec<FirkinDto> = Vec::with_capacity(docs.len());
    for doc in docs {
        let artists: Vec<ArtistDto> = doc
            .artists
            .iter()
            .filter_map(|cid| by_id.get(cid).cloned())
            .collect();
        out.push(FirkinDto::from_doc_with_artists(doc, artists));
    }
    Ok(out)
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

    let docs: Vec<Firkin> = docs
        .into_iter()
        .filter(|d| {
            let id = d.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
            !superseded.contains(&id)
        })
        .collect();
    let mut dtos = assemble_firkin_dtos(&state, docs).await?;
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
        Some(d) => Ok(Json(assemble_firkin_dto(&state, d).await?)),
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
    let (artist_cids, artist_dtos) = materialise_artists(&state, req.artists).await?;
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
    let creator = req
        .creator
        .map(|c| c.trim().to_string())
        .unwrap_or_default();
    let trailers: Vec<Trailer> = req
        .trailers
        .into_iter()
        .filter_map(|t| {
            let url = t.youtube_url.trim().to_string();
            if url.is_empty() {
                return None;
            }
            let label = t
                .label
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty());
            Some(Trailer {
                youtube_url: url,
                label,
            })
        })
        .collect();

    let now = Utc::now();
    let version: u32 = 0;
    let version_hashes: Vec<String> = Vec::new();
    let body_json = serialize_firkin_payload(
        title,
        &description,
        &artist_cids,
        &images,
        &files,
        year,
        addon,
        &creator,
        version,
        &version_hashes,
        &trailers,
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
        return Ok((StatusCode::OK, Json(assemble_firkin_dto(&state, existing).await?)));
    }

    let record = Firkin {
        id: None,
        title: title.to_string(),
        artists: artist_cids,
        description,
        images,
        files,
        year,
        addon: addon.to_string(),
        creator,
        created_at: now,
        updated_at: now,
        version,
        version_hashes,
        trailers,
    };

    let created: Option<Firkin> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let created_doc = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "firkin was not persisted"))?;
    let dto = FirkinDto::from_doc_with_artists(created_doc, artist_dtos);

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
        let (cids, _) = materialise_artists(&state, artists).await?;
        current.artists = cids;
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

    if let Some(trailers) = req.trailers {
        current.trailers = trailers
            .into_iter()
            .filter_map(|t| {
                let url = t.youtube_url.trim().to_string();
                if url.is_empty() {
                    return None;
                }
                let label = t
                    .label
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty());
                Some(Trailer {
                    youtube_url: url,
                    label,
                })
            })
            .collect();
    }

    current.updated_at = Utc::now();
    current.id = None;

    let updated_doc = rollforward_firkin(&state, &id, current).await?;
    Ok(Json(assemble_firkin_dto(&state, updated_doc).await?))
}

/// Apply a mutated firkin to the store, rolling forward to a new
/// content-addressed id whenever the mutation changes a body-affecting
/// field. When the new CID matches `old_id` (the caller's mutations
/// didn't actually change any field that participates in the CID),
/// updates the existing record in place so non-CID fields like
/// `updated_at` still get persisted without minting a stale version.
///
/// On rollforward: increments `version`, pushes `old_id` onto
/// `version_hashes`, recomputes the CID over the mutated body, deletes
/// the old record, creates a new one at the new CID, and pins the new
/// body JSON to IPFS via `pin_firkin_body`. Idempotent against a
/// concurrent rollforward landing at the same new id (adopts the
/// existing record). Caller is expected to have set `updated.id = None`
/// and bumped `updated.updated_at` before calling.
pub(crate) async fn rollforward_firkin(
    state: &CloudState,
    old_id: &str,
    mut updated: Firkin,
) -> Result<Firkin, (StatusCode, Json<serde_json::Value>)> {
    let new_version = updated.version.saturating_add(1);
    let mut new_hashes = updated.version_hashes.clone();
    new_hashes.push(old_id.to_string());

    let new_id = compute_firkin_cid(
        &updated.title,
        &updated.description,
        &updated.artists,
        &updated.images,
        &updated.files,
        updated.year,
        &updated.addon,
        &updated.creator,
        new_version,
        &new_hashes,
        &updated.trailers,
    );

    if new_id == old_id {
        // No body change — keep the same id but persist mutable fields
        // like `updated_at`. Don't bump version / version_hashes.
        let res: Option<Firkin> = state
            .db
            .update((TABLE, old_id))
            .content(updated)
            .await
            .map_err(|e| {
                err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("db update failed: {e}"),
                )
            })?;
        return res.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"));
    }

    updated.version = new_version;
    updated.version_hashes = new_hashes.clone();

    let new_body_json = serialize_firkin_payload(
        &updated.title,
        &updated.description,
        &updated.artists,
        &updated.images,
        &updated.files,
        updated.year,
        &updated.addon,
        &updated.creator,
        new_version,
        &new_hashes,
        &updated.trailers,
    );

    let _: Option<Firkin> = state
        .db
        .delete((TABLE, old_id))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db delete failed: {e}"),
            )
        })?;
    let create_result: Result<Option<Firkin>, _> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(updated)
        .await;
    let created_doc = match create_result {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err(err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "firkin was not persisted",
            ))
        }
        Err(e) => {
            // A concurrent rollforward landed first at the same new id.
            // Adopt the existing record rather than failing.
            let msg = e.to_string();
            if msg.contains("already exists") {
                let doc: Option<Firkin> = state
                    .db
                    .select((TABLE, new_id.as_str()))
                    .await
                    .map_err(|e| {
                        err_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("db select failed: {e}"),
                        )
                    })?;
                doc.ok_or_else(|| {
                    err_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "rolled-forward firkin missing",
                    )
                })?
            } else {
                return Err(err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("db create failed: {e}"),
                ));
            }
        }
    };

    pin_firkin_body(state, &new_id, new_body_json).await;

    Ok(created_doc)
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
        Some(d) => Ok(Json(assemble_firkin_dto(&state, d).await?)),
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

/// Apply catalog-derived metadata (title, year, description, poster +
/// backdrop) to a firkin and roll its version forward to a new CID.
/// Mirrors the rollforward pattern from
/// [`crate::torrent_completion::rollforward`]: push the old id onto
/// `version_hashes`, increment `version`, recompute the CID, replace the
/// record at the new id, and pin the new body JSON to IPFS. Idempotent
/// when the supplied metadata produces the same CID.
async fn enrich(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    Json(req): Json<EnrichFirkinRequest>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    let existing: Option<Firkin> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?;

    if let Some(title) = req.title.as_ref().map(|t| t.trim()) {
        if !title.is_empty() {
            current.title = title.to_string();
        }
    }
    if let Some(description) = req.description.as_ref() {
        current.description = description.trim().to_string();
    }
    if let Some(year) = req.year {
        if (1000..=9999).contains(&year) {
            current.year = Some(year);
        }
    }

    let mut new_images: Vec<ImageMeta> = Vec::new();
    for url in [req.poster_url.as_deref(), req.backdrop_url.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        new_images.push(ImageMeta {
            url: url.to_string(),
            mime_type: String::new(),
            file_size: 0,
            width: 0,
            height: 0,
        });
    }
    if !new_images.is_empty() {
        current.images = new_images;
    }

    current.updated_at = Utc::now();
    current.id = None;

    let created_doc = rollforward_firkin(&state, &id, current).await?;
    Ok(Json(assemble_firkin_dto(&state, created_doc).await?))
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, serde::Deserialize, Default)]
struct RomsQuery {
    /// When set, restrict extraction to a single archive identified by its
    /// firkin file `title` (which is the relative path under the torrent
    /// output dir). When unset, scan every archive on the firkin.
    archive: Option<String>,
}

#[cfg(not(target_os = "android"))]
async fn roms(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<RomsQuery>,
) -> Result<Json<crate::rom_extract::RomsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let result = match query.archive {
        Some(rel) => crate::rom_extract::extract_single_archive(&state, &id, &rel).await,
        None => crate::rom_extract::extract_roms_for_firkin(&state, &id).await,
    };
    result.map(Json).map_err(|e| {
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

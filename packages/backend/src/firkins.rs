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

const ALLOWED_FILE_TYPES: &[&str] = &[
    "ipfs",
    "torrent magnet",
    "url",
    "lyrics",
    "youtube preferred client",
];

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
/// season name, e.g. `"Season 1"`). The primary source is TMDB (via the
/// `videos` block of the catalog detail response — see
/// `apps/cloud/src/catalog.rs::parse_tmdb_videos`), filtered to English
/// (`iso_639_1 == "en"`); the YouTube fuzzy search is the fallback. The
/// optional `language` carries the ISO 639-1 code from upstream when
/// known. `language` is `skip_serializing_if = "Option::is_none"` so
/// existing firkin CIDs (which don't have it) stay stable across the
/// deserialise → re-serialise round-trip.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trailer {
    #[serde(rename = "youtubeUrl")]
    pub youtube_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// A user-rating (review) attached to a firkin, sourced from an upstream
/// catalog API. `label` names the source (e.g. `"TMDB"`, `"MusicBrainz"`),
/// `score` is the raw rating value as exposed by that source, and
/// `max_score` is the scale (TMDB returns 0–10, MusicBrainz returns 0–5)
/// so the UI can render `score / max_score` without per-source rules.
/// `vote_count` is the number of ratings the average is computed over,
/// when known; `skip_serializing_if = "Option::is_none"` keeps it out of
/// the canonical body when absent so existing CIDs stay stable.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Review {
    pub label: String,
    pub score: f64,
    #[serde(rename = "maxScore")]
    pub max_score: f64,
    #[serde(rename = "voteCount", default, skip_serializing_if = "Option::is_none")]
    pub vote_count: Option<u32>,
}

fn slice_is_empty<T>(s: &[T]) -> bool {
    s.is_empty()
}

/// Trim and validate a review request from a client. Drops entries with
/// an empty label, a non-finite score / max_score, or a non-positive
/// max_score. The label is trimmed; the score is clamped into
/// `[0, max_score]` so a misbehaving upstream can't poison the firkin
/// body with a value outside its declared scale.
fn sanitize_review(r: Review) -> Option<Review> {
    let label = r.label.trim().to_string();
    if label.is_empty() {
        return None;
    }
    if !r.score.is_finite() || !r.max_score.is_finite() || r.max_score <= 0.0 {
        return None;
    }
    let score = r.score.max(0.0).min(r.max_score);
    Some(Review {
        label,
        score,
        max_score: r.max_score,
        vote_count: r.vote_count,
    })
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
    /// Skipped when empty so existing firkin CIDs (created before
    /// reviews existed) remain stable across deserialise → re-serialise.
    #[serde(skip_serializing_if = "slice_is_empty")]
    reviews: &'a [Review],
}

#[allow(clippy::too_many_arguments)]
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
    reviews: &[Review],
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
        reviews,
    };
    serde_json::to_string_pretty(&view).expect("FirkinPayloadView serializes to JSON")
}

#[allow(clippy::too_many_arguments)]
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
    reviews: &[Review],
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
        reviews,
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
    /// the content-addressed handles so the firkin's own CID stays stable
    /// across presentation-only edits to an artist (e.g. a new image URL).
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
    /// Content-addressed CID of the firkin's body, recomputed on every
    /// update. The SurrealDB record id is a stable UUID; this field
    /// carries the IPFS hash so callers can still resolve the body via
    /// UnixFS without hitting the DB.
    #[serde(default)]
    pub cid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u32,
    /// History of past CIDs, oldest first, one per body change. Append-only.
    #[serde(default)]
    pub version_hashes: Vec<String>,
    /// YouTube trailers resolved against this firkin. Movies hold one
    /// entry; TV shows hold one entry per season (with `label` set to
    /// `"Season N"`). Empty by default; resolved client-side after the
    /// firkin is created and persisted via PUT.
    #[serde(default)]
    pub trailers: Vec<Trailer>,
    /// User ratings sourced from upstream catalog APIs (TMDB
    /// `vote_average` / `vote_count`, MusicBrainz `rating.value` /
    /// `votes-count`). Each entry is one source's snapshot at the time
    /// the firkin was bookmarked or last enriched; see [`Review`].
    #[serde(default)]
    pub reviews: Vec<Review>,
    /// `true` when the user has explicitly bookmarked this firkin from a
    /// catalog detail page; `false` for "browse cache" firkins auto-created
    /// when the user clicks through from the catalog grid into a detail
    /// view. **Not** part of `serialize_firkin_payload` / `compute_firkin_cid`,
    /// so flipping the flag doesn't roll the CID forward and the same
    /// upstream item dedups to one firkin regardless of bookmark state.
    /// Defaults to `true` on deserialise so legacy records (which predate
    /// the flag) keep showing up in the bookmarked list.
    #[serde(default = "default_true")]
    pub bookmarked: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct FirkinDto {
    /// Stable UUID identifier — assigned at create time and never changed.
    pub id: String,
    /// Content-addressed CID of the firkin body, recomputed on every update.
    pub cid: String,
    pub title: String,
    /// Artist CIDs as persisted on the firkin record. Drives the body
    /// hash and lets clients re-fetch artist docs by CID.
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
    pub reviews: Vec<Review>,
    /// Mirrors [`Firkin::bookmarked`]. Drives the catalog detail page's
    /// action bar (Bookmark vs. Play / Find / Delete) and the
    /// `GET /api/firkins` listing filter (which only returns bookmarked
    /// records by default).
    pub bookmarked: bool,
}

impl FirkinDto {
    fn from_doc_with_artists(doc: Firkin, artists: Vec<ArtistDto>) -> Self {
        let id = doc
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        // Records created before the schema split carry the CID *as* the
        // record id and have an empty `cid` field. Fall back to the id so
        // callers always see the content-address.
        let cid = if doc.cid.is_empty() {
            id.clone()
        } else {
            doc.cid
        };
        Self {
            id,
            cid,
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
            reviews: doc.reviews,
            bookmarked: doc.bookmarked,
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
    /// Optional reviews to bake into the firkin at create time. The
    /// virtual-catalog page pulls ratings from `/api/catalog/.../metadata`
    /// up-front so the bookmark already carries the upstream score.
    #[serde(default)]
    pub reviews: Vec<Review>,
    /// Optional bookmark state. The catalog "Bookmark" button sends `true`
    /// (default), the `/catalog/visit` resolver flow sends `false` so the
    /// firkin is created as a non-bookmarked browse cache. Omitting the
    /// field defaults to `true` to preserve the prior bookmark-on-create
    /// contract for legacy callers.
    #[serde(default)]
    pub bookmarked: Option<bool>,
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
    pub reviews: Option<Vec<Review>>,
    /// Flip the bookmark state. The catalog detail page sends `true` on
    /// the Bookmark button. Not part of the firkin body / CID, so the
    /// flip does not roll the version forward.
    pub bookmarked: Option<bool>,
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
    let r = Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
        .route("/{id}/finalize", post(finalize))
        .route("/{id}/enrich", post(enrich))
        .route("/{id}/roms", post(roms));
    #[cfg(not(target_os = "android"))]
    let r = r
        .route("/{id}/resolve-tracks", post(resolve_tracks))
        .route("/{id}/resolution-progress", get(resolution_progress));
    r
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


#[derive(Debug, serde::Deserialize, Default)]
struct ListFirkinsQuery {
    /// `?include=all` returns every firkin, including non-bookmarked
    /// browse-cache rows created by the `/catalog/visit` resolver. The
    /// default response only contains bookmarked records so the catalog
    /// "Library" section, the `/firkins` page, and the recommendations
    /// table aren't cluttered with every item the user has clicked on.
    #[serde(default)]
    include: Option<String>,
}

async fn list(
    State(state): State<CloudState>,
    axum::extract::Query(query): axum::extract::Query<ListFirkinsQuery>,
) -> Result<Json<Vec<FirkinDto>>, (StatusCode, Json<serde_json::Value>)> {
    let docs: Vec<Firkin> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;

    let include_all = query.include.as_deref() == Some("all");
    let docs: Vec<Firkin> = if include_all {
        docs
    } else {
        docs.into_iter().filter(|d| d.bookmarked).collect()
    };
    let docs = collapse_to_chain_heads(docs);
    let mut dtos = assemble_firkin_dtos(&state, docs).await?;
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

/// Reduce a raw `firkin` table dump to one record per logical media
/// item. Two distinct firkins can share the same `(addon, title, year)`
/// — one created via the catalog "Bookmark" button (no files), another
/// from a library scan (files attached, may have been updated several
/// times). Keep the one with the higher `version` (and the more recent
/// `updated_at` if tied) so callers see exactly one row per logical
/// item.
///
/// Updates are now applied in place against a stable UUID record id, so
/// no record ever appears in another record's `version_hashes`. The
/// legacy version-chain dedup is therefore unnecessary.
pub(crate) fn collapse_to_chain_heads(docs: Vec<Firkin>) -> Vec<Firkin> {
    let mut by_key: std::collections::HashMap<(String, String, Option<i32>), Firkin> =
        std::collections::HashMap::new();
    for d in docs {
        let key = (d.addon.clone(), d.title.clone(), d.year);
        match by_key.get(&key) {
            Some(existing)
                if (existing.version, existing.updated_at) >= (d.version, d.updated_at) => {}
            _ => {
                by_key.insert(key, d);
            }
        }
    }
    by_key.into_values().collect()
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
    let (status, dto) = create_firkin_record(&state, req).await?;
    Ok((status, Json(dto)))
}

/// Inner handler for `POST /api/firkins`. Extracted so background tasks
/// (the TV-show firkin builder, future bulk importers) can mint firkins
/// through the same code path the HTTP handler uses without round-tripping
/// through the network — same dedup-by-CID, same artist materialisation,
/// same body pin, same album-resolver auto-spawn.
pub(crate) async fn create_firkin_record(
    state: &CloudState,
    req: CreateFirkinRequest,
) -> Result<(StatusCode, FirkinDto), (StatusCode, Json<serde_json::Value>)> {
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
            let language = t
                .language
                .map(|l| l.trim().to_ascii_lowercase())
                .filter(|l| !l.is_empty());
            Some(Trailer {
                youtube_url: url,
                label,
                language,
            })
        })
        .collect();
    let reviews: Vec<Review> = req
        .reviews
        .into_iter()
        .filter_map(sanitize_review)
        .collect();
    // Defaults to `true` when omitted so legacy callers keep their
    // bookmark-on-create contract; the new `/catalog/visit` resolver flow
    // sends an explicit `false` to mark the firkin as a browse cache.
    let bookmarked = req.bookmarked.unwrap_or(true);

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
        &reviews,
    );
    let body_cid = compute_body_cid(&body_json);

    // Dedup by content-address: if a firkin with the same body already
    // exists, return it instead of minting a duplicate UUID. This
    // preserves the previous "same bookmark twice" behaviour even though
    // the record id is no longer the CID. When the incoming request
    // bookmarks an existing browse-cache firkin, upgrade the stored
    // `bookmarked` flag in place (no version roll — the flag is not part
    // of the body) and kick off album resolution if it wasn't bookmarked
    // before.
    if let Some(mut existing) = find_by_cid(&state, &body_cid).await? {
        let existing_id = existing
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        let upgraded = bookmarked && !existing.bookmarked;
        if upgraded {
            existing.bookmarked = true;
            existing.updated_at = Utc::now();
            existing.id = None;
            let updated_doc: Option<Firkin> = state
                .db
                .update((TABLE, existing_id.as_str()))
                .content(existing)
                .await
                .map_err(|e| {
                    err_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("db update failed: {e}"),
                    )
                })?;
            existing = updated_doc.ok_or_else(|| {
                err_response(StatusCode::INTERNAL_SERVER_ERROR, "firkin update returned no row")
            })?;
        }
        #[cfg(not(target_os = "android"))]
        if existing.addon == "musicbrainz"
            && !existing_id.is_empty()
            && existing.bookmarked
            && upgraded
        {
            spawn_resolve_album_tracks(state.clone(), existing_id.clone());
        }
        #[cfg(target_os = "android")]
        let _ = existing_id;
        return Ok((StatusCode::OK, assemble_firkin_dto(state, existing).await?));
    }

    let new_id = uuid::Uuid::new_v4().to_string();

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
        cid: body_cid.clone(),
        created_at: now,
        updated_at: now,
        version,
        version_hashes,
        trailers,
        reviews,
        bookmarked,
    };
    let record_addon = record.addon.clone();
    let record_bookmarked = record.bookmarked;

    let created: Option<Firkin> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let created_doc = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "firkin was not persisted"))?;
    let dto = FirkinDto::from_doc_with_artists(created_doc, artist_dtos);

    pin_firkin_body(state, &new_id, body_json).await;

    // Auto-trigger album track resolution for fresh musicbrainz firkins,
    // but only when the firkin is bookmarked — un-bookmarked browse-cache
    // firkins (created via the catalog `/catalog/visit` resolver) skip
    // the heavy YouTube + LRCLIB resolution until the user actually
    // bookmarks them. Spawned as a background task so the response
    // returns immediately and the resolution outlives the HTTP request.
    #[cfg(not(target_os = "android"))]
    if record_addon == "musicbrainz" && record_bookmarked {
        spawn_resolve_album_tracks(state.clone(), new_id.clone());
    }
    #[cfg(target_os = "android")]
    let _ = (record_addon, record_bookmarked);

    Ok((StatusCode::CREATED, dto))
}

pub(crate) fn compute_body_cid(body_json: &str) -> String {
    let digest = Sha256::digest(body_json.as_bytes());
    let mh = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .expect("sha2-256 digest fits in multihash");
    Cid::new_v1(RAW_CODEC, mh).to_string()
}

/// Look up a firkin by its content-addressed `cid` field. Returns the
/// first match (uuid records keep `cid` populated; legacy records with
/// the cid stored as the record id are matched by id as well so dedup
/// keeps working during the schema transition).
async fn find_by_cid(
    state: &CloudState,
    cid: &str,
) -> Result<Option<Firkin>, (StatusCode, Json<serde_json::Value>)> {
    let mut resp = state
        .db
        .query("SELECT * FROM firkin WHERE cid = $cid OR id = type::thing('firkin', $cid) LIMIT 1")
        .bind(("cid", cid.to_string()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db query failed: {e}"),
            )
        })?;
    let docs: Vec<Firkin> = resp.take(0).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db take failed: {e}"),
        )
    })?;
    Ok(docs.into_iter().next())
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
                let language = t
                    .language
                    .map(|l| l.trim().to_ascii_lowercase())
                    .filter(|l| !l.is_empty());
                Some(Trailer {
                    youtube_url: url,
                    label,
                    language,
                })
            })
            .collect();
    }

    if let Some(reviews) = req.reviews {
        current.reviews = reviews.into_iter().filter_map(sanitize_review).collect();
    }

    // `bookmarked` is not part of the CID body, so the rollforward below
    // will detect "no body change" and persist the flag without rolling
    // the version. We capture the false→true transition here so we can
    // kick off the album resolver after the rollforward completes.
    let bookmark_promoted = match req.bookmarked {
        Some(next) => {
            let was = current.bookmarked;
            current.bookmarked = next;
            !was && next
        }
        None => false,
    };

    current.updated_at = Utc::now();
    current.id = None;

    let updated_doc = rollforward_firkin(&state, &id, current).await?;

    #[cfg(not(target_os = "android"))]
    if bookmark_promoted && updated_doc.addon == "musicbrainz" {
        spawn_resolve_album_tracks(state.clone(), id.clone());
    }
    #[cfg(target_os = "android")]
    let _ = bookmark_promoted;

    Ok(Json(assemble_firkin_dto(&state, updated_doc).await?))
}

/// Apply a mutated firkin to the store, in place. Recomputes the body
/// CID; when the mutation changes a body-affecting field, increments
/// `version`, appends the prior CID to `version_hashes`, updates the
/// record's `cid` field, and re-pins the new body JSON to IPFS. When the
/// new CID matches the existing one (mutations didn't actually change a
/// CID-participating field), the record is still updated so non-CID
/// fields like `updated_at` get persisted, but `version` /
/// `version_hashes` are left alone. The SurrealDB record id (a stable
/// UUID) is never changed. Caller is expected to have set
/// `updated.id = None` and bumped `updated.updated_at` before calling.
pub(crate) async fn rollforward_firkin(
    state: &CloudState,
    id: &str,
    mut updated: Firkin,
) -> Result<Firkin, (StatusCode, Json<serde_json::Value>)> {
    // Look up the prior record so we can read its cid field for the
    // version_hashes append. Falls back to the id itself for legacy
    // records where the record id was the CID and the cid field is empty.
    let prior: Option<Firkin> = state
        .db
        .select((TABLE, id))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    let prior = prior.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?;
    let prior_cid = if prior.cid.is_empty() {
        id.to_string()
    } else {
        prior.cid.clone()
    };

    // Trial-compute the new body's CID against the prior version /
    // version_hashes so a no-op mutation is detected before we touch the
    // version chain.
    let trial_cid = compute_firkin_cid(
        &updated.title,
        &updated.description,
        &updated.artists,
        &updated.images,
        &updated.files,
        updated.year,
        &updated.addon,
        &updated.creator,
        updated.version,
        &updated.version_hashes,
        &updated.trailers,
        &updated.reviews,
    );

    if trial_cid == prior_cid {
        // No body change — persist mutable fields like `updated_at` but
        // leave version / version_hashes alone.
        updated.cid = prior_cid;
        let res: Option<Firkin> = state
            .db
            .update((TABLE, id))
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

    let new_version = updated.version.saturating_add(1);
    let mut new_hashes = updated.version_hashes.clone();
    new_hashes.push(prior_cid);

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
        &updated.reviews,
    );
    let new_cid = compute_body_cid(&new_body_json);

    updated.version = new_version;
    updated.version_hashes = new_hashes;
    updated.cid = new_cid;

    let updated_doc: Option<Firkin> = state
        .db
        .update((TABLE, id))
        .content(updated)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db update failed: {e}"),
            )
        })?;
    let updated_doc =
        updated_doc.ok_or_else(|| err_response(StatusCode::NOT_FOUND, "firkin not found"))?;

    pin_firkin_body(state, id, new_body_json).await;

    Ok(updated_doc)
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

#[cfg(not(target_os = "android"))]
async fn resolve_tracks(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<FirkinDto>, (StatusCode, Json<serde_json::Value>)> {
    let resolved = resolve_album_tracks(&state, &id)
        .await
        .map_err(|(s, m)| err_response(s, m))?;
    Ok(Json(assemble_firkin_dto(&state, resolved).await?))
}

/// Inner album-resolution loop. Walks the firkin's MusicBrainz tracklist,
/// queries YouTube + LRCLIB for each track that's still missing data,
/// scores hits with the same token-based rankers as the WebUI, packs
/// resolved entries into `files`, and rolls the firkin forward (or
/// returns it unchanged when nothing was missing). Designed to be called
/// from both the explicit HTTP endpoint and a fire-and-forget background
/// task spawned by the create handler — the loop's lifetime is tied to
/// the spawned future, *not* the originating HTTP request, so closing
/// the browser tab does not interrupt the resolution.
///
/// Publishes per-track progress to `state.track_progress` so the WebUI
/// can render YT URL + lyrics status as soon as each track resolves,
/// long before the firkin itself rolls forward.
#[cfg(not(target_os = "android"))]
pub(crate) async fn resolve_album_tracks(
    state: &CloudState,
    id: &str,
) -> Result<Firkin, (StatusCode, String)> {
    use crate::track_progress::{
        AlbumProgress, LyricsProgress, TrackProgressEntry, TrackStatus,
    };
    use crate::track_resolve;

    let existing: Option<Firkin> = state
        .db
        .select((TABLE, id))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut current = existing
        .ok_or((StatusCode::NOT_FOUND, "firkin not found".to_string()))?;

    if current.addon != "musicbrainz" {
        return Err((
            StatusCode::BAD_REQUEST,
            "resolve-tracks only supports musicbrainz firkins".to_string(),
        ));
    }

    let release_group_id = current
        .files
        .iter()
        .filter(|f| f.kind == "url")
        .find_map(|f| extract_mb_release_group_id(&f.value))
        .ok_or((
            StatusCode::BAD_REQUEST,
            "firkin is missing a MusicBrainz release-group url in `files`".to_string(),
        ))?;

    tracing::info!(
        firkin_id = %id,
        release_group_id = %release_group_id,
        "fetching musicbrainz tracklist"
    );
    let tracks = track_resolve::fetch_release_group_tracks(&release_group_id)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    tracing::info!(
        firkin_id = %id,
        track_count = tracks.len(),
        "starting per-track resolution"
    );

    // Seed the live progress entry with one row per MB track. Pre-mark
    // any track that already has a YT URL or lyrics in `files` as
    // `Found` so the WebUI picks up the existing state on first poll
    // (idempotent re-runs don't reset the UI to "queued").
    let now = Utc::now();
    let initial_tracks: Vec<TrackProgressEntry> = tracks
        .iter()
        .map(|t| {
            let title = t.title.trim();
            let existing_yt = current
                .files
                .iter()
                .find(|f| {
                    f.kind == "url"
                        && f.title
                            .as_deref()
                            .map(|s| s.trim().eq_ignore_ascii_case(title))
                            .unwrap_or(false)
                        && is_youtube_url(&f.value)
                })
                .map(|f| f.value.clone());
            let existing_lyrics_value = current
                .files
                .iter()
                .find(|f| {
                    f.kind == "lyrics"
                        && f.title
                            .as_deref()
                            .map(|s| s.trim().eq_ignore_ascii_case(title))
                            .unwrap_or(false)
                })
                .map(|f| f.value.clone());
            let lyrics = existing_lyrics_value.as_deref().and_then(decode_lyrics_value);
            TrackProgressEntry {
                position: t.position,
                title: t.title.clone(),
                length_ms: t.length_ms,
                youtube_status: if existing_yt.is_some() {
                    TrackStatus::Found
                } else {
                    TrackStatus::Pending
                },
                youtube_url: existing_yt,
                lyrics_status: if lyrics.is_some() {
                    TrackStatus::Found
                } else {
                    TrackStatus::Pending
                },
                lyrics,
            }
        })
        .collect();
    state.track_progress.insert(
        id.to_string(),
        AlbumProgress {
            firkin_id: id.to_string(),
            started_at: now,
            updated_at: now,
            completed: false,
            completed_id: None,
            tracks: initial_tracks,
        },
    );

    let artist_dtos = artists::fetch_many(state, &current.artists)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("artist fetch failed: {e}"),
            )
        })?;
    let artist_query = artist_dtos
        .iter()
        .map(|a| a.name.as_str())
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

    let album_title = current.title.clone();

    let mut new_files = current.files.clone();
    let mut changed = false;
    for (idx, track) in tracks.iter().enumerate() {
        let track_title = track.title.trim();
        if track_title.is_empty() {
            continue;
        }

        let already_yt = new_files.iter().any(|f| {
            f.kind == "url"
                && f.title
                    .as_deref()
                    .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                    .unwrap_or(false)
                && is_youtube_url(&f.value)
        });
        let already_lyrics = new_files.iter().any(|f| {
            f.kind == "lyrics"
                && f.title
                    .as_deref()
                    .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                    .unwrap_or(false)
        });
        if already_yt && already_lyrics {
            continue;
        }

        // Mark this track's pending statuses as `Searching` so the
        // WebUI can show the spinner-style "YT…" / "Lyrics…" badges
        // while the actual fetch is in flight.
        state.track_progress.update(id, |p| {
            if let Some(t) = p.tracks.get_mut(idx) {
                if !already_yt {
                    t.youtube_status = TrackStatus::Searching;
                }
                if !already_lyrics {
                    t.lyrics_status = TrackStatus::Searching;
                }
            }
        });

        let (yt_url, lyrics_hit) = track_resolve::resolve_track(
            track_title,
            &artist_query,
            &album_title,
            track.length_ms,
        )
        .await;

        // Snapshot the resolved bits for the progress map *before* we
        // move them into the firkin's `files` (those moves consume the
        // strings).
        let progress_yt = if !already_yt { yt_url.clone() } else { None };
        let progress_lyrics = if !already_lyrics {
            lyrics_hit.as_ref().map(|h| LyricsProgress {
                source: "lrclib".to_string(),
                external_id: h.id.clone(),
                synced_lyrics: h.synced_lyrics.clone(),
                plain_lyrics: h.plain_lyrics.clone(),
                instrumental: h.instrumental,
            })
        } else {
            None
        };

        if !already_yt {
            if let Some(url) = yt_url {
                new_files = track_resolve::upsert_track_file(
                    new_files,
                    FileEntry {
                        kind: "url".to_string(),
                        value: url,
                        title: Some(track_title.to_string()),
                    },
                );
                changed = true;
            }
        }
        if !already_lyrics {
            if let Some(hit) = lyrics_hit {
                let value = track_resolve::encode_lyrics_value(&hit);
                new_files = track_resolve::upsert_track_file(
                    new_files,
                    FileEntry {
                        kind: "lyrics".to_string(),
                        value,
                        title: Some(track_title.to_string()),
                    },
                );
                changed = true;
            }
        }

        // Publish the resolved-or-not statuses for this track to the
        // progress map so the WebUI's poll picks them up immediately.
        state.track_progress.update(id, |p| {
            if let Some(t) = p.tracks.get_mut(idx) {
                if !already_yt {
                    t.youtube_url = progress_yt.clone();
                    t.youtube_status = if progress_yt.is_some() {
                        TrackStatus::Found
                    } else {
                        TrackStatus::Missing
                    };
                }
                if !already_lyrics {
                    if let Some(lp) = &progress_lyrics {
                        t.lyrics = Some(lp.clone());
                        t.lyrics_status = TrackStatus::Found;
                    } else {
                        t.lyrics_status = TrackStatus::Missing;
                    }
                }
            }
        });
    }

    if !changed {
        // Nothing to roll forward, but the album is "complete" from the
        // WebUI's perspective — close the progress entry so the polling
        // loop can stop chasing the same id.
        state.track_progress.update(id, |p| {
            p.completed = true;
            p.completed_id = Some(id.to_string());
        });
        return Ok(current);
    }

    current.files = new_files;
    current.updated_at = Utc::now();
    current.id = None;

    let updated = rollforward_firkin(state, id, current)
        .await
        .map_err(|(s, j)| {
            let msg = j
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("rollforward failed")
                .to_string();
            (s, msg)
        })?;

    let new_id = updated
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .unwrap_or_default();
    state.track_progress.update(id, |p| {
        p.completed = true;
        p.completed_id = Some(new_id.clone());
    });
    Ok(updated)
}

/// Decode the JSON value the album resolver stores under a `lyrics`
/// `FileEntry` into a `LyricsProgress` so the progress endpoint can
/// hand back already-resolved lyrics on the very first poll (avoids a
/// "Pending" → "Found" flicker on idempotent re-runs).
#[cfg(not(target_os = "android"))]
fn decode_lyrics_value(value: &str) -> Option<crate::track_progress::LyricsProgress> {
    use crate::track_progress::LyricsProgress;
    let parsed: serde_json::Value = serde_json::from_str(value).ok()?;
    Some(LyricsProgress {
        source: parsed
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("lrclib")
            .to_string(),
        external_id: parsed
            .get("externalId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        synced_lyrics: parsed
            .get("syncedLyrics")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        plain_lyrics: parsed
            .get("plainLyrics")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        instrumental: parsed
            .get("instrumental")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    })
}

#[cfg(not(target_os = "android"))]
async fn resolution_progress(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<crate::track_progress::AlbumProgress>, (StatusCode, Json<serde_json::Value>)> {
    match state.track_progress.get(&id) {
        Some(p) => Ok(Json(p)),
        None => Err(err_response(StatusCode::NOT_FOUND, "no resolution in progress")),
    }
}

/// Spawn `resolve_album_tracks` as a fire-and-forget background task.
/// Called by the firkin create handler so a freshly-bookmarked
/// MusicBrainz album starts processing immediately and continues even if
/// the user navigates away from the detail page. Errors are logged via
/// `tracing::warn!` rather than surfaced — the detail page's polling
/// effect will pick up the rolled-forward firkin once the task finishes.
#[cfg(not(target_os = "android"))]
pub(crate) fn spawn_resolve_album_tracks(state: CloudState, id: String) {
    tracing::info!(firkin_id = %id, "spawning background album resolution");
    tokio::spawn(async move {
        match resolve_album_tracks(&state, &id).await {
            Ok(updated) => {
                let new_id = updated
                    .id
                    .as_ref()
                    .map(|t| t.id.to_raw())
                    .unwrap_or_default();
                tracing::info!(
                    firkin_id = %id,
                    new_id = %new_id,
                    "background album resolution complete"
                );
            }
            Err((status, msg)) => {
                tracing::warn!(
                    firkin_id = %id,
                    status = %status,
                    error = %msg,
                    "background album resolution failed"
                );
            }
        }
    });
}

fn extract_mb_release_group_id(value: &str) -> Option<String> {
    let url = url::Url::parse(value).ok()?;
    if !url
        .host_str()
        .map(|h| h.eq_ignore_ascii_case("musicbrainz.org"))
        .unwrap_or(false)
    {
        return None;
    }
    let mut segments = url.path_segments()?;
    if segments.next()? != "release-group" {
        return None;
    }
    let id = segments.next()?.to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn is_youtube_url(value: &str) -> bool {
    let host = match url::Url::parse(value).ok().and_then(|u| u.host_str().map(str::to_ascii_lowercase)) {
        Some(h) => h,
        None => return false,
    };
    matches!(
        host.as_str(),
        "www.youtube.com"
            | "youtube.com"
            | "m.youtube.com"
            | "music.youtube.com"
            | "youtu.be"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use surrealdb::sql::{Id, Thing};

    fn doc(id: &str, title: &str, addon: &str, version: u32, version_hashes: &[&str]) -> Firkin {
        let now = Utc::now();
        Firkin {
            id: Some(Thing::from((TABLE, Id::String(id.to_string())))),
            title: title.to_string(),
            artists: Vec::new(),
            description: String::new(),
            images: Vec::new(),
            files: Vec::new(),
            year: None,
            addon: addon.to_string(),
            creator: String::new(),
            cid: String::new(),
            created_at: now,
            updated_at: now,
            version,
            version_hashes: version_hashes.iter().map(|s| s.to_string()).collect(),
            trailers: Vec::new(),
            reviews: Vec::new(),
            bookmarked: true,
        }
    }

    #[test]
    fn collapses_parallel_chains_to_highest_version() {
        // Two distinct chains for the same (addon, title): the bookmark
        // chain (v=0, no hashes) and the library-scan chain (v=2 with its
        // own version_hashes that don't overlap the bookmark id).
        let bookmark = doc("bookmark-cid", "X-Men", "tmdb-movie", 0, &[]);
        let scan_v0 = doc("scan-v0", "X-Men", "tmdb-movie", 0, &[]);
        let scan_v1 = doc("scan-v1", "X-Men", "tmdb-movie", 1, &["scan-v0"]);
        let scan_head =
            doc("scan-v2", "X-Men", "tmdb-movie", 2, &["scan-v0", "scan-v1"]);

        let kept =
            collapse_to_chain_heads(vec![bookmark, scan_v0, scan_v1, scan_head]);

        assert_eq!(kept.len(), 1);
        assert_eq!(
            kept[0].id.as_ref().unwrap().id.to_raw(),
            "scan-v2",
            "should keep the v=2 chain head"
        );
    }

    #[test]
    fn keeps_distinct_addons_separate() {
        let local = doc("local-id", "X-Men", "local-movie", 1, &[]);
        let tmdb = doc("tmdb-id", "X-Men", "tmdb-movie", 0, &[]);
        let kept = collapse_to_chain_heads(vec![local, tmdb]);
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn keeps_distinct_years_separate() {
        let mut a = doc("id-a", "Joker", "tmdb-movie", 0, &[]);
        a.year = Some(2019);
        let mut b = doc("id-b", "Joker", "tmdb-movie", 0, &[]);
        b.year = Some(2024);
        let kept = collapse_to_chain_heads(vec![a, b]);
        assert_eq!(kept.len(), 2);
    }
}

use crate::catalog::is_known_addon;
use crate::firkins::{compute_firkin_cid, FileEntry, ImageMeta};
use crate::state::CloudState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::sql::Thing;

pub const TABLE: &str = "recommendation";
pub const SOURCE_TABLE: &str = "recommendation_source";

/// One row per `(user, recommended firkin)` pair. Tracks how many distinct
/// source-firkin pages have recommended this item to this user, plus the
/// user's own annotations (watched flag and 0-100 score).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Option<Thing>,
    /// Lowercased EVM address of the user the recommendation belongs to.
    pub address: String,
    /// Virtual firkin CID computed via [`compute_firkin_cid`] over a minimal
    /// canonical body (title, description, year, addon, upstream-url file,
    /// poster/backdrop images — no artists, no trailers, version=0). See
    /// [`compute_recommendation_cid`].
    pub firkin_id: String,
    pub addon: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "posterUrl", default, skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl", default, skip_serializing_if = "Option::is_none")]
    pub backdrop_url: Option<String>,
    /// Number of distinct source-firkin pages that have recommended this
    /// item to the user. Each (user, source) pair contributes at most once
    /// — see the marker rows in [`SOURCE_TABLE`].
    pub count: u32,
    pub watched: bool,
    /// 0-100 user-provided score.
    pub score: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One row per `(user, source firkin)` pair. Presence means "we already
/// counted the recommendations from this source for this user, don't
/// double-count if they revisit".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationSource {
    pub id: Option<Thing>,
    pub address: String,
    pub source_firkin_id: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationDto {
    pub id: String,
    pub address: String,
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    pub addon: String,
    pub title: String,
    pub year: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "posterUrl")]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl")]
    pub backdrop_url: Option<String>,
    pub count: u32,
    pub watched: bool,
    pub score: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Recommendation> for RecommendationDto {
    fn from(r: Recommendation) -> Self {
        let id = r
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            address: r.address,
            firkin_id: r.firkin_id,
            addon: r.addon,
            title: r.title,
            year: r.year,
            description: r.description,
            poster_url: r.poster_url,
            backdrop_url: r.backdrop_url,
            count: r.count,
            watched: r.watched,
            score: r.score,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IngestItem {
    pub addon: String,
    /// Upstream-provider id (e.g., TMDB movie id, MusicBrainz release-group id).
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "posterUrl", default)]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl", default)]
    pub backdrop_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub address: String,
    /// CID of the firkin whose detail page is being viewed (the "source"
    /// of the recommendations we're ingesting).
    #[serde(rename = "sourceFirkinId")]
    pub source_firkin_id: String,
    #[serde(default)]
    pub items: Vec<IngestItem>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    /// `false` when this `(user, source)` pair was already processed and
    /// the request short-circuited without touching counts.
    pub processed: bool,
    /// Number of recommendation rows actually touched (created or
    /// incremented). `0` when `processed` is `false`.
    pub ingested: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub address: String,
    #[serde(default)]
    pub watched: Option<bool>,
    #[serde(default)]
    pub score: Option<u8>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list))
        .route("/ingest", post(ingest))
        .route("/{firkin_id}", put(update))
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

fn normalize_address(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    let body = lower.strip_prefix("0x").unwrap_or(&lower);
    if body.len() != 40 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("0x{}", body))
}

fn record_id(prefix: &str, address: &str, firkin_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(b":");
    hasher.update(address.as_bytes());
    hasher.update(b":");
    hasher.update(firkin_id.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Build the upstream URL file the virtual catalog page would attach when
/// bookmarking, so the recommendation CID matches what the firkin would
/// hash to if its body had no artists / trailers resolved at create time.
/// Returns `None` for addons that don't map to a stable upstream URL.
fn upstream_file(addon: &str, upstream_id: &str) -> Option<FileEntry> {
    match addon {
        "musicbrainz" => Some(FileEntry {
            kind: "url".to_string(),
            value: format!("https://musicbrainz.org/release-group/{upstream_id}"),
            title: Some("MusicBrainz Release Group".to_string()),
        }),
        "tmdb-tv" => Some(FileEntry {
            kind: "url".to_string(),
            value: format!("https://www.themoviedb.org/tv/{upstream_id}"),
            title: Some("TMDB TV Show".to_string()),
        }),
        "tmdb-movie" => Some(FileEntry {
            kind: "url".to_string(),
            value: format!("https://www.themoviedb.org/movie/{upstream_id}"),
            title: Some("TMDB Movie".to_string()),
        }),
        _ => None,
    }
}

/// Compute the virtual firkin CID for a recommendation. The body shape
/// mirrors what `/catalog/virtual` would persist on bookmark *before* any
/// artists or trailers are resolved client-side: title + description +
/// images (poster, backdrop) + the single upstream URL file + year +
/// addon + creator="" + version=0 + version_hashes=[] + artists=[] +
/// trailers=[]. Stable across visits for the same upstream item, which is
/// all the recommendation table needs.
pub fn compute_recommendation_cid(item: &IngestItem) -> String {
    let images: Vec<ImageMeta> = [item.poster_url.as_deref(), item.backdrop_url.as_deref()]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .map(|url| ImageMeta {
            url: url.to_string(),
            ..Default::default()
        })
        .collect();
    let mut files: Vec<FileEntry> = Vec::new();
    if let Some(file) = upstream_file(&item.addon, &item.id) {
        files.push(file);
    }
    compute_firkin_cid(
        &item.title,
        item.description.as_deref().unwrap_or(""),
        &[],
        &images,
        &files,
        item.year,
        &item.addon,
        "",
        0,
        &[],
        &[],
    )
}

async fn list(
    State(state): State<CloudState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<RecommendationDto>>, (StatusCode, Json<serde_json::Value>)> {
    let address_filter = match q.address.as_deref() {
        Some(raw) => Some(normalize_address(raw).ok_or_else(|| {
            err_response(StatusCode::BAD_REQUEST, "invalid address")
        })?),
        None => None,
    };
    let rows: Vec<Recommendation> = state.db.select(TABLE).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db select failed: {e}"),
        )
    })?;
    let mut dtos: Vec<RecommendationDto> = rows
        .into_iter()
        .filter(|r| match &address_filter {
            Some(addr) => &r.address == addr,
            None => true,
        })
        .map(Into::into)
        .collect();
    dtos.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    Ok(Json(dtos))
}

async fn ingest(
    State(state): State<CloudState>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let source_firkin_id = req.source_firkin_id.trim().to_string();
    if source_firkin_id.is_empty() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "sourceFirkinId is required",
        ));
    }

    let source_id = record_id("recommendation_source", &address, &source_firkin_id);
    let existing_marker: Option<RecommendationSource> = state
        .db
        .select((SOURCE_TABLE, source_id.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;

    if existing_marker.is_some() {
        return Ok(Json(IngestResponse {
            processed: false,
            ingested: 0,
        }));
    }

    let mut ingested: u32 = 0;
    let now = Utc::now();
    for item in &req.items {
        if item.title.trim().is_empty() || item.id.trim().is_empty() {
            continue;
        }
        if !is_known_addon(&item.addon) {
            continue;
        }
        let firkin_id = compute_recommendation_cid(item);
        let rec_id = record_id("recommendation", &address, &firkin_id);
        let existing: Option<Recommendation> = state
            .db
            .select((TABLE, rec_id.as_str()))
            .await
            .map_err(|e| {
                err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("db select failed: {e}"),
                )
            })?;
        let saved: Option<Recommendation> = match existing {
            Some(mut current) => {
                current.id = None;
                current.count = current.count.saturating_add(1);
                current.updated_at = now;
                // Back-fill presentation fields when missing — earlier
                // ingests may have come from a sparse upstream payload.
                if current.description.is_none() {
                    current.description = item.description.clone();
                }
                if current.poster_url.is_none() {
                    current.poster_url = item.poster_url.clone();
                }
                if current.backdrop_url.is_none() {
                    current.backdrop_url = item.backdrop_url.clone();
                }
                if current.year.is_none() {
                    current.year = item.year;
                }
                state
                    .db
                    .update((TABLE, rec_id.as_str()))
                    .content(current)
                    .await
                    .map_err(|e| {
                        err_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("db update failed: {e}"),
                        )
                    })?
            }
            None => {
                let row = Recommendation {
                    id: None,
                    address: address.clone(),
                    firkin_id,
                    addon: item.addon.clone(),
                    title: item.title.trim().to_string(),
                    year: item.year,
                    description: item.description.clone(),
                    poster_url: item.poster_url.clone(),
                    backdrop_url: item.backdrop_url.clone(),
                    count: 1,
                    watched: false,
                    score: 0,
                    created_at: now,
                    updated_at: now,
                };
                state
                    .db
                    .create((TABLE, rec_id.as_str()))
                    .content(row)
                    .await
                    .map_err(|e| {
                        err_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("db create failed: {e}"),
                        )
                    })?
            }
        };
        if saved.is_some() {
            ingested += 1;
        }
    }

    // Mark the (address, source) pair as processed so re-visits don't
    // double-count. We do this last so a partial DB failure mid-way
    // through the loop above doesn't poison the marker — the next
    // request will retry.
    let marker = RecommendationSource {
        id: None,
        address: address.clone(),
        source_firkin_id,
        processed_at: now,
    };
    let _: Option<RecommendationSource> = state
        .db
        .create((SOURCE_TABLE, source_id.as_str()))
        .content(marker)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db create failed: {e}"),
            )
        })?;

    Ok(Json(IngestResponse {
        processed: true,
        ingested,
    }))
}

async fn update(
    State(state): State<CloudState>,
    Path(firkin_id): Path<String>,
    Json(req): Json<UpdateRequest>,
) -> Result<Json<RecommendationDto>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let firkin_id = firkin_id.trim().to_string();
    if firkin_id.is_empty() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "firkin_id is required",
        ));
    }
    if let Some(score) = req.score {
        if score > 100 {
            return Err(err_response(
                StatusCode::BAD_REQUEST,
                "score must be 0..=100",
            ));
        }
    }

    let rec_id = record_id("recommendation", &address, &firkin_id);
    let existing: Option<Recommendation> = state
        .db
        .select((TABLE, rec_id.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    let mut current = existing.ok_or_else(|| {
        err_response(StatusCode::NOT_FOUND, "recommendation not found")
    })?;
    current.id = None;
    if let Some(watched) = req.watched {
        current.watched = watched;
    }
    if let Some(score) = req.score {
        current.score = score;
    }
    current.updated_at = Utc::now();
    let saved: Option<Recommendation> = state
        .db
        .update((TABLE, rec_id.as_str()))
        .content(current)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db update failed: {e}"),
            )
        })?;
    let dto: RecommendationDto = saved
        .ok_or_else(|| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "recommendation was not persisted",
            )
        })?
        .into();
    Ok(Json(dto))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_id_is_deterministic_per_pair() {
        let a = record_id("recommendation", "0xabc", "bafy1");
        let b = record_id("recommendation", "0xabc", "bafy1");
        assert_eq!(a, b);
        assert_ne!(a, record_id("recommendation", "0xabc", "bafy2"));
        assert_ne!(a, record_id("recommendation_source", "0xabc", "bafy1"));
    }

    #[test]
    fn cid_stable_for_same_payload() {
        let item = IngestItem {
            addon: "tmdb-movie".to_string(),
            id: "123".to_string(),
            title: "Star Wars".to_string(),
            year: Some(1977),
            description: Some("A long time ago…".to_string()),
            poster_url: Some("https://example.test/p.jpg".to_string()),
            backdrop_url: None,
        };
        let a = compute_recommendation_cid(&item);
        let b = compute_recommendation_cid(&item);
        assert_eq!(a, b);
    }

    #[test]
    fn cid_changes_when_upstream_id_changes() {
        let mut item = IngestItem {
            addon: "tmdb-movie".to_string(),
            id: "123".to_string(),
            title: "Star Wars".to_string(),
            year: Some(1977),
            description: None,
            poster_url: None,
            backdrop_url: None,
        };
        let a = compute_recommendation_cid(&item);
        item.id = "456".to_string();
        let b = compute_recommendation_cid(&item);
        assert_ne!(a, b);
    }
}

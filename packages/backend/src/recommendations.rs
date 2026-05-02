use crate::catalog::{is_known_addon, CatalogReview};
use crate::firkins::{compute_firkin_cid, FileEntry, ImageMeta};
use crate::state::CloudState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use surrealdb::sql::Thing;

pub const TABLE: &str = "recommendation";
pub const SOURCE_TABLE: &str = "recommendation_source";
pub const ACTION_TABLE: &str = "recommendation_action";

pub const ACTION_LIKE: &str = "like";
pub const ACTION_DISCARD: &str = "discard";
pub const ACTION_BOOKMARK: &str = "bookmark";
const ALLOWED_ACTIONS: &[&str] = &[ACTION_LIKE, ACTION_DISCARD, ACTION_BOOKMARK];

/// One row per `(user, recommended firkin)` pair. Tracks how many distinct
/// source-firkin pages have recommended this item to this user.
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
    /// Upstream provider id (TMDB movie id, MusicBrainz release-group id,
    /// …) — same value the WebUI passes as `IngestItem::id`. Persisted so
    /// the WebUI can rebuild the upstream URL without re-querying the
    /// catalog API (used by the `/recommendations` page's bookmark button
    /// to mint the same `url` file `/catalog/virtual` would attach).
    #[serde(default)]
    pub upstream_id: String,
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
    /// Upstream rating snapshots captured at ingest time so the
    /// `/recommendations` table can render a Rating column without
    /// re-querying the catalog API. Not included in
    /// [`compute_recommendation_cid`] so the firkin id stays stable as
    /// review counts change upstream.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviews: Vec<CatalogReview>,
    /// User-assigned 1-100 score, set via `POST /api/recommendations/rating`.
    /// `None` until the user has rated this item from the feed page; once
    /// set, used as the secondary sort key in the listing (between count
    /// and the upstream review-rating tiebreaker). Preserved across
    /// re-ingests of the same `(user, recommendation)` pair.
    #[serde(
        rename = "userRating",
        alias = "user_rating",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub user_rating: Option<u8>,
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
    #[serde(rename = "upstreamId")]
    pub upstream_id: String,
    pub title: String,
    pub year: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "posterUrl")]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl")]
    pub backdrop_url: Option<String>,
    pub count: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reviews: Vec<CatalogReview>,
    #[serde(rename = "userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<u8>,
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
            upstream_id: r.upstream_id,
            title: r.title,
            year: r.year,
            description: r.description,
            poster_url: r.poster_url,
            backdrop_url: r.backdrop_url,
            count: r.count,
            reviews: r.reviews,
            user_rating: r.user_rating,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub address: Option<String>,
    /// When `true`, drop any recommendation that the user has already
    /// liked / discarded / bookmarked via the `/action` endpoint. Used by
    /// the `/feed` page so the user never sees the same card twice.
    #[serde(rename = "excludeActioned", default)]
    pub exclude_actioned: bool,
}

/// One row per `(user, recommendation firkin)` pair recording the user's
/// most recent action on that recommendation. Liking, discarding, or
/// bookmarking a recommendation upserts the same row so the
/// `/recommendations?excludeActioned=true` filter can hide it from the
/// feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationAction {
    pub id: Option<Thing>,
    pub address: String,
    pub firkin_id: String,
    /// One of [`ACTION_LIKE`], [`ACTION_DISCARD`], [`ACTION_BOOKMARK`].
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub address: String,
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RatingRequest {
    pub address: String,
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    /// 1-100 inclusive.
    pub rating: u8,
}

#[derive(Debug, Serialize)]
pub struct RatingResponse {
    #[serde(rename = "userRating")]
    pub user_rating: u8,
    pub updated_at: DateTime<Utc>,
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
    /// Upstream rating snapshots forwarded verbatim from the catalog
    /// listing response. Persisted on the recommendation row but ignored
    /// by [`compute_recommendation_cid`] (so review-count drift upstream
    /// doesn't churn ids).
    #[serde(default)]
    pub reviews: Vec<CatalogReview>,
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

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list))
        .route("/ingest", post(ingest))
        .route("/action", post(record_action))
        .route("/rating", post(set_rating))
}

/// Mean of `score / max_score` across all reviews. Returns `0.0` for an
/// empty list so unrated items sink under rated ones in the feed sort.
fn rating_score(reviews: &[CatalogReview]) -> f64 {
    if reviews.is_empty() {
        return 0.0;
    }
    let total: f64 = reviews
        .iter()
        .map(|r| {
            if r.max_score > 0.0 {
                (r.score / r.max_score).clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .sum();
    total / reviews.len() as f64
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
    let actioned_ids: HashSet<String> = if q.exclude_actioned {
        let actions: Vec<RecommendationAction> = state
            .db
            .select(ACTION_TABLE)
            .await
            .map_err(|e| {
                err_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("db select failed: {e}"),
                )
            })?;
        actions
            .into_iter()
            .filter(|a| match &address_filter {
                Some(addr) => &a.address == addr,
                None => true,
            })
            .map(|a| a.firkin_id)
            .collect()
    } else {
        HashSet::new()
    };
    let mut dtos: Vec<RecommendationDto> = rows
        .into_iter()
        .filter(|r| match &address_filter {
            Some(addr) => &r.address == addr,
            None => true,
        })
        .filter(|r| !q.exclude_actioned || !actioned_ids.contains(&r.firkin_id))
        // Any user-given rating (including 0 from Discard) takes the item
        // out of the feed on the next load — the user has expressed an
        // opinion, so the card has done its job. Mid-session the card is
        // kept visible so the user can still bookmark after rating; the
        // filter only kicks in on the next fetch.
        .filter(|r| !q.exclude_actioned || r.user_rating.is_none())
        .map(Into::into)
        .collect();
    dtos.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.user_rating.unwrap_or(0).cmp(&a.user_rating.unwrap_or(0)))
            .then_with(|| {
                rating_score(&b.reviews)
                    .partial_cmp(&rating_score(&a.reviews))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    Ok(Json(dtos))
}

async fn record_action(
    State(state): State<CloudState>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let firkin_id = req.firkin_id.trim().to_string();
    if firkin_id.is_empty() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "firkinId is required",
        ));
    }
    let action = req.action.trim().to_string();
    if !ALLOWED_ACTIONS.contains(&action.as_str()) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "action must be one of: like, discard, bookmark",
        ));
    }

    let action_id = record_id("recommendation_action", &address, &firkin_id);
    let now = Utc::now();
    let existing: Option<RecommendationAction> = state
        .db
        .select((ACTION_TABLE, action_id.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;

    let saved: Option<RecommendationAction> = match existing {
        Some(mut current) => {
            current.id = None;
            current.action = action.clone();
            current.updated_at = now;
            state
                .db
                .update((ACTION_TABLE, action_id.as_str()))
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
            let row = RecommendationAction {
                id: None,
                address: address.clone(),
                firkin_id,
                action: action.clone(),
                created_at: now,
                updated_at: now,
            };
            state
                .db
                .create((ACTION_TABLE, action_id.as_str()))
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
    let row = saved.ok_or_else(|| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "action upsert returned no row",
        )
    })?;
    Ok(Json(ActionResponse {
        action: row.action,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

async fn set_rating(
    State(state): State<CloudState>,
    Json(req): Json<RatingRequest>,
) -> Result<Json<RatingResponse>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let firkin_id = req.firkin_id.trim().to_string();
    if firkin_id.is_empty() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "firkinId is required",
        ));
    }
    if req.rating > 100 {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "rating must be between 0 and 100",
        ));
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
    current.user_rating = Some(req.rating);
    let now = Utc::now();
    current.updated_at = now;
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
    let row = saved.ok_or_else(|| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "rating upsert returned no row",
        )
    })?;
    Ok(Json(RatingResponse {
        user_rating: row.user_rating.unwrap_or(req.rating),
        updated_at: row.updated_at,
    }))
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
                if current.upstream_id.is_empty() {
                    current.upstream_id = item.id.trim().to_string();
                }
                if current.reviews.is_empty() && !item.reviews.is_empty() {
                    current.reviews = item.reviews.clone();
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
                    upstream_id: item.id.trim().to_string(),
                    title: item.title.trim().to_string(),
                    year: item.year,
                    description: item.description.clone(),
                    poster_url: item.poster_url.clone(),
                    backdrop_url: item.backdrop_url.clone(),
                    count: 1,
                    reviews: item.reviews.clone(),
                    user_rating: None,
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
            reviews: Vec::new(),
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
            reviews: Vec::new(),
        };
        let a = compute_recommendation_cid(&item);
        item.id = "456".to_string();
        let b = compute_recommendation_cid(&item);
        assert_ne!(a, b);
    }
}

use crate::state::CloudState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use tokio::task::JoinSet;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const MUSICBRAINZ_BASE: &str = "https://musicbrainz.org/ws/2";
const COVERART_BASE: &str = "https://coverartarchive.org";
const OMDB_BASE: &str = "https://www.omdbapi.com";
const USER_AGENT: &str = "Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)";

/// Every addon known to the cloud. Each addon represents a single content
/// kind — the kind is implicit in the addon id, so callers no longer need a
/// separate `type` parameter. `kind` is the firkin kind label this addon
/// produces (and the `addon` value persisted on a firkin record references
/// this id directly).
pub const ADDONS: &[Addon] = &[
    Addon {
        id: "tmdb-movie",
        label: "TMDB Movies",
        kind: "movie",
        filter_label: "Genre",
        has_filter: true,
        has_popular: true,
        browsable: true,
    },
    Addon {
        id: "tmdb-tv",
        label: "TMDB TV Shows",
        kind: "tv show",
        filter_label: "Genre",
        has_filter: true,
        has_popular: true,
        browsable: true,
    },
    Addon {
        id: "musicbrainz",
        label: "MusicBrainz",
        kind: "album",
        filter_label: "Genre",
        has_filter: true,
        has_popular: true,
        browsable: true,
    },
    Addon {
        id: "youtube-video",
        label: "YouTube",
        kind: "youtube video",
        filter_label: "Type",
        has_filter: true,
        has_popular: false,
        browsable: true,
    },
    // Subtitle / lyric lookups — valid firkin addons but not browsable.
    Addon {
        id: "wyzie-subs-movie",
        label: "Wyzie Subs (Movies)",
        kind: "movie",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "wyzie-subs-tv",
        label: "Wyzie Subs (TV)",
        kind: "tv show",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "lrclib",
        label: "LRCLIB",
        kind: "album",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    // Local addons — used by libraries to declare which media kinds they
    // contain, and as the addon value on firkins created by library scans.
    Addon {
        id: "local-movie",
        label: "Local Movies",
        kind: "movie",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "local-tv",
        label: "Local TV Shows",
        kind: "tv show",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "local-album",
        label: "Local Albums",
        kind: "album",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "local-book",
        label: "Local Books",
        kind: "book",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
    Addon {
        id: "local-game",
        label: "Local Games",
        kind: "game",
        filter_label: "Filter",
        has_filter: false,
        has_popular: false,
        browsable: false,
    },
];

#[derive(Clone)]
pub struct Addon {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub filter_label: &'static str,
    pub has_filter: bool,
    pub has_popular: bool,
    pub browsable: bool,
}

pub fn is_known_addon(id: &str) -> bool {
    ADDONS.iter().any(|a| a.id == id)
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/sources", get(list_sources))
        .route("/{addon}/popular", get(popular))
        .route("/{addon}/search", get(search))
        .route("/{addon}/genres", get(genres))
        .route("/{addon}/{id}/metadata", get(metadata_for_item))
        .route("/{addon}/{id}/related", get(related_for_item))
        .route(
            "/musicbrainz/release-groups/{id}/tracks",
            get(musicbrainz_tracks),
        )
        .route(
            "/musicbrainz/release-groups/{id}/albums-by-artist",
            get(musicbrainz_albums_by_artist_handler),
        )
        .route("/tmdb-tv/{id}/seasons", get(tmdb_tv_seasons))
        .route(
            "/tmdb-tv/{id}/season/{season_number}/episodes",
            get(tmdb_tv_season_episodes),
        )
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CatalogItem {
    pub id: String,
    pub title: String,
    pub year: Option<i32>,
    pub description: Option<String>,
    #[serde(rename = "posterUrl")]
    pub poster_url: Option<String>,
    #[serde(rename = "backdropUrl")]
    pub backdrop_url: Option<String>,
    /// Populated by `/related` (mirrors the `artists` shape returned by
    /// `/metadata`). Empty/skipped on `/popular` and `/search` so those
    /// endpoints remain unchanged on the wire.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artists: Vec<CatalogArtist>,
    /// Upstream rating snapshots when the listing response carries them
    /// inline (TMDB's `/popular` + `/search` + `/recommendations` always
    /// do; MusicBrainz `/release-group?query=…` does when the upstream
    /// data has ratings). Empty when the source has no inline ratings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviews: Vec<CatalogReview>,
    /// Compact "creator" handle for the item — release-group artist
    /// credit for music. Populated by addons that have a clear single-
    /// string handle on the listing endpoint; left absent for movies /
    /// TV shows where there's no equivalent.
    #[serde(rename = "artistName", default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CatalogPage {
    pub items: Vec<CatalogItem>,
    pub page: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize)]
struct CatalogGenre {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct CatalogTrack {
    id: String,
    position: i64,
    title: String,
    #[serde(rename = "lengthMs")]
    length_ms: Option<i64>,
}

/// One TV-show season as returned by `GET /api/catalog/tmdb-tv/{id}/seasons`.
/// Used by the WebUI to enumerate seasons for trailer resolution: each
/// season is searched against YouTube as `"{showTitle} season {n} trailer"`
/// and the best match is kept on the firkin's `trailers` array.
#[derive(Clone, Serialize)]
pub(crate) struct CatalogSeason {
    #[serde(rename = "seasonNumber")]
    pub season_number: i64,
    pub name: String,
    #[serde(rename = "airYear", skip_serializing_if = "Option::is_none")]
    pub air_year: Option<i32>,
    #[serde(rename = "episodeCount", skip_serializing_if = "Option::is_none")]
    pub episode_count: Option<i64>,
    #[serde(rename = "posterUrl", skip_serializing_if = "Option::is_none")]
    pub poster_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
}

/// One episode of a TMDB TV-show season, returned by
/// `GET /api/catalog/tmdb-tv/{id}/season/{season_number}/episodes`.
#[derive(Clone, Serialize)]
pub(crate) struct CatalogEpisode {
    #[serde(rename = "episodeNumber")]
    pub episode_number: i64,
    #[serde(rename = "seasonNumber")]
    pub season_number: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(rename = "airDate", skip_serializing_if = "Option::is_none")]
    pub air_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<i64>,
    #[serde(rename = "stillUrl", skip_serializing_if = "Option::is_none")]
    pub still_url: Option<String>,
    #[serde(rename = "voteAverage", skip_serializing_if = "Option::is_none")]
    pub vote_average: Option<f64>,
}

/// Three-field "person/group attached to a media item" record matching the
/// persisted `artist` doc shape. Each addon's handler maps its upstream
/// cast, crew, authors, developers, channels, etc. into this shape; the
/// frontend hands the array verbatim to `POST /api/firkins`, which
/// upserts each entry into the `artist` table and stores the resulting
/// CIDs on the firkin.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CatalogArtist {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// One trailer attached to a catalog item. Mirrors the persisted
/// `Trailer` shape on a firkin so the frontend can hand the array
/// verbatim to `POST /api/firkins` (or `PUT /api/firkins/:id`). For
/// TMDB this is sourced from the `videos` block of the detail
/// response (`append_to_response=videos`); `language` carries TMDB's
/// `iso_639_1` (lower-case ISO 639-1, e.g. `"en"`) so the frontend
/// can show / filter trailers by spoken language. Non-TMDB addons
/// currently leave `language` unset.
#[derive(Clone, Serialize)]
pub(crate) struct CatalogTrailer {
    #[serde(rename = "youtubeUrl")]
    pub youtube_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// One upstream user-rating snapshot for a catalog item. Mirrors the
/// persisted `Review` shape on a firkin so the frontend can hand the
/// array verbatim to `POST /api/firkins` (or `PUT /api/firkins/:id`).
/// `score` is the raw upstream value, `max_score` is the scale (TMDB
/// reports out of 10, MusicBrainz out of 5), `vote_count` is the number
/// of ratings the average is computed over when known.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CatalogReview {
    pub label: String,
    pub score: f64,
    #[serde(rename = "maxScore", alias = "max_score")]
    pub max_score: f64,
    #[serde(
        rename = "voteCount",
        alias = "vote_count",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vote_count: Option<u32>,
}

/// Combined metadata payload for a single catalog item. Returned by
/// `GET /api/catalog/{addon}/{id}/metadata`. Lets the frontend pull
/// artists, trailers and reviews in one call so each upstream provider
/// can satisfy them in a single request (TMDB's `append_to_response`
/// merges credits and videos into one HTTP call).
#[derive(Serialize)]
struct CatalogMetadata {
    artists: Vec<CatalogArtist>,
    trailers: Vec<CatalogTrailer>,
    reviews: Vec<CatalogReview>,
}

#[derive(Serialize)]
struct CatalogSource {
    id: &'static str,
    label: &'static str,
    kind: &'static str,
    #[serde(rename = "filterLabel")]
    filter_label: &'static str,
    #[serde(rename = "hasFilter")]
    has_filter: bool,
    #[serde(rename = "hasPopular")]
    has_popular: bool,
}

async fn list_sources() -> Json<Vec<CatalogSource>> {
    Json(
        ADDONS
            .iter()
            .filter(|a| a.browsable)
            .map(|a| CatalogSource {
                id: a.id,
                label: a.label,
                kind: a.kind,
                filter_label: a.filter_label,
                has_filter: a.has_filter,
                has_popular: a.has_popular,
            })
            .collect(),
    )
}

#[derive(Debug, Deserialize)]
pub struct PopularQuery {
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    /// Currently only honored by `musicbrainz`; selects which release-group
    /// field to target. Accepts `artist` (default) or `release` / `album`.
    #[serde(default)]
    pub field: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenresQuery {}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn popular(
    State(state): State<CloudState>,
    Path(addon): Path<String>,
    Query(q): Query<PopularQuery>,
) -> Result<Json<CatalogPage>, (StatusCode, Json<serde_json::Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    let filter = q.filter.as_deref().unwrap_or("");
    let cache_key = format!("popular::{addon}::{filter}::{page}");
    let addon_str = addon.clone();
    let filter_owned = q.filter.clone();
    let page_for_fetch = page;
    let mut payload = crate::catalog_cache::get_or_fetch(&state, &cache_key, move || async move {
        match addon_str.as_str() {
            "tmdb-movie" => tmdb_popular(false, filter_owned.as_deref(), page_for_fetch).await,
            "tmdb-tv" => tmdb_popular(true, filter_owned.as_deref(), page_for_fetch).await,
            "musicbrainz" => musicbrainz_popular(filter_owned.as_deref(), page_for_fetch).await,
            "youtube-video" => youtube_popular(filter_owned.as_deref(), page_for_fetch).await,
            "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(empty_page(page_for_fetch)),
            _ => Err(err(
                StatusCode::NOT_FOUND,
                format!("addon \"{addon_str}\" is not supported"),
            )),
        }
    })
    .await?;

    // Drop items the user has already minted a firkin for (bookmarked or
    // browse-cache — both count). The cached payload is the pristine
    // upstream response so the filter is recomputed each request as the
    // firkin store changes. DB read failures are logged and swallowed —
    // an unfiltered popular page is still a usable page.
    match crate::firkins::upstream_ids_for_addon(&state, &addon).await {
        Ok(known) if !known.is_empty() => {
            payload.items.retain(|it| !known.contains(&it.id));
        }
        Ok(_) => {}
        Err(e) => {
            tracing::debug!("[catalog] firkin upstream-id read failed for {addon}: {e}");
        }
    }
    Ok(Json(payload))
}

async fn search(
    State(_state): State<CloudState>,
    Path(addon): Path<String>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<CatalogPage>, (StatusCode, Json<serde_json::Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    let query = q.query.unwrap_or_default();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Json(empty_page(page)));
    }
    match addon.as_str() {
        "tmdb-movie" => tmdb_search(false, trimmed, page).await,
        "tmdb-tv" => tmdb_search(true, trimmed, page).await,
        "musicbrainz" => musicbrainz_search(trimmed, page, q.field.as_deref()).await,
        "youtube-video" => youtube_search(trimmed, page, q.filter.as_deref()).await,
        "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(empty_page(page)),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
}

async fn genres(
    State(state): State<CloudState>,
    Path(addon): Path<String>,
    Query(_q): Query<GenresQuery>,
) -> Result<Json<Vec<CatalogGenre>>, (StatusCode, Json<serde_json::Value>)> {
    let cache_key = format!("genres::{addon}");
    let addon_str = addon.clone();
    let payload = crate::catalog_cache::get_or_fetch(&state, &cache_key, move || async move {
        match addon_str.as_str() {
            "tmdb-movie" => tmdb_genres(false).await,
            "tmdb-tv" => tmdb_genres(true).await,
            "musicbrainz" => Ok(static_music_genres()),
            "youtube-video" => Ok(static_youtube_search_kinds()),
            "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(Vec::new()),
            _ => Err(err(
                StatusCode::NOT_FOUND,
                format!("addon \"{addon_str}\" is not supported"),
            )),
        }
    })
    .await?;
    Ok(Json(payload))
}

fn empty_page(page: i64) -> CatalogPage {
    CatalogPage {
        items: Vec::new(),
        page,
        total_pages: 1,
    }
}

/// `GET /api/catalog/{addon}/{id}/metadata` — fetches the people /
/// groups / studios / channels associated with an upstream catalog item
/// (mapped into the universal `CatalogArtist` shape) AND any trailers
/// the upstream provider exposes for the item, in a single call. For
/// TMDB this collapses to one upstream HTTP request via
/// `append_to_response=credits,videos`. Used by the `/catalog/virtual`
/// page to populate the firkin's artists + trailers on bookmark, and by
/// the `/catalog/[ipfsHash]` page to backfill missing data on first
/// visit. Unknown / unsupported addons return empty arrays (200) so the
/// frontend can call this unconditionally.
async fn metadata_for_item(
    State(_state): State<CloudState>,
    Path((addon, id)): Path<(String, String)>,
) -> Result<Json<CatalogMetadata>, (StatusCode, Json<serde_json::Value>)> {
    if id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "id is required"));
    }
    let (artists, trailers, reviews) = match addon.as_str() {
        "musicbrainz" => {
            let (artists, reviews) = musicbrainz_metadata(&id).await?;
            (artists, Vec::new(), reviews)
        }
        "tmdb-movie" => tmdb_metadata(false, &id).await?,
        "tmdb-tv" => tmdb_metadata(true, &id).await?,
        "youtube-video" => (youtube_video_artists(&id).await?, Vec::new(), Vec::new()),
        _ => (Vec::new(), Vec::new(), Vec::new()),
    };
    Ok(Json(CatalogMetadata {
        artists,
        trailers,
        reviews,
    }))
}

/// `GET /api/catalog/{addon}/{id}/related` — fetches items related to the
/// upstream catalog item identified by `id`. For `tmdb-movie` / `tmdb-tv`
/// this proxies TMDB's `/recommendations` endpoint; for `musicbrainz`
/// it browses other release-groups by the same primary artist.
/// Output is ephemeral — the WebUI displays these as virtual catalog
/// links, never persisting them to SurrealDB or pinning to IPFS.
/// Unknown / unsupported addons return an empty list (200) so the
/// frontend can call this unconditionally.
async fn related_for_item(
    State(_state): State<CloudState>,
    Path((addon, id)): Path<(String, String)>,
) -> Result<Json<Vec<CatalogItem>>, (StatusCode, Json<serde_json::Value>)> {
    if id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "id is required"));
    }
    let items = match addon.as_str() {
        "tmdb-movie" => tmdb_related(false, &id).await?,
        "tmdb-tv" => tmdb_related(true, &id).await?,
        "musicbrainz" => musicbrainz_related(&id).await?,
        _ => Vec::new(),
    };
    Ok(Json(items))
}

// ---------- TMDB ----------

async fn tmdb_popular(
    is_tv: bool,
    genre: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let endpoint = if is_tv {
        if let Some(g) = genre.filter(|s| !s.is_empty()) {
            format!(
                "/discover/tv?api_key={}&page={}&with_genres={}&include_adult=false",
                api_key,
                page,
                urlencoding(g)
            )
        } else {
            format!("/tv/popular?api_key={}&page={}", api_key, page)
        }
    } else if let Some(g) = genre.filter(|s| !s.is_empty()) {
        format!(
            "/discover/movie?api_key={}&page={}&with_genres={}&include_adult=false",
            api_key,
            page,
            urlencoding(g)
        )
    } else {
        format!("/movie/popular?api_key={}&page={}", api_key, page)
    };
    let url = format!("{}{}", TMDB_BASE, endpoint);
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;

    let total_pages = payload
        .get("total_pages")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let mut items: Vec<CatalogItem> = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(tmdb_to_item).collect())
        .unwrap_or_default();
    enrich_items_with_omdb(is_tv, &mut items).await;
    Ok(CatalogPage {
        items,
        page,
        total_pages,
    })
}

/// Process-wide cache of `(kind, tmdb_id) -> merged review list`. Keys
/// look like `"movie:278"` or `"tv:1399"`. Populated by
/// [`enrich_items_with_omdb`]; entries persist for the life of the
/// process so a refresh of `/popular` doesn't re-burn the OMDb / TMDB
/// quota for the same items.
fn tmdb_review_cache() -> &'static RwLock<HashMap<String, Vec<CatalogReview>>> {
    static CACHE: OnceLock<RwLock<HashMap<String, Vec<CatalogReview>>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Replace each item's TMDB-only `reviews` with the full TMDB+OMDb list
/// returned by [`tmdb_metadata`]. Per-item lookups run concurrently via
/// `JoinSet` and hit the in-process cache before going upstream so
/// repeat `/popular` page loads are free. Failures (TMDB outage, no
/// IMDb id, OMDb miss) leave the original TMDB-only review in place.
async fn enrich_items_with_omdb(is_tv: bool, items: &mut [CatalogItem]) {
    if items.is_empty() {
        return;
    }
    let kind = if is_tv { "tv" } else { "movie" };
    // Pass 1 (immutable borrow): partition work into "already cached"
    // and "needs upstream call". The cache lookup is cheap; doing it
    // here avoids spawning tasks for hot items.
    let mut cached_hits: Vec<(usize, Vec<CatalogReview>)> = Vec::new();
    let mut set: JoinSet<(usize, String, Option<Vec<CatalogReview>>)> = JoinSet::new();
    for (idx, item) in items.iter().enumerate() {
        if item.id.is_empty() {
            continue;
        }
        let key = format!("{kind}:{}", item.id);
        if let Some(cached) = tmdb_review_cache()
            .read()
            .ok()
            .and_then(|c| c.get(&key).cloned())
        {
            cached_hits.push((idx, cached));
            continue;
        }
        let id = item.id.clone();
        let key_owned = key;
        set.spawn(async move {
            let reviews = match tmdb_metadata(is_tv, &id).await {
                Ok((_, _, r)) => Some(r),
                Err(_) => None,
            };
            (idx, key_owned, reviews)
        });
    }
    // Pass 2 (mutable borrow): apply cached hits, then drain JoinSet
    // results — populating the cache and writing back.
    for (idx, reviews) in cached_hits {
        if let Some(item) = items.get_mut(idx) {
            if !reviews.is_empty() {
                item.reviews = reviews;
            }
        }
    }
    while let Some(joined) = set.join_next().await {
        let Ok((idx, key, Some(reviews))) = joined else {
            continue;
        };
        if reviews.is_empty() {
            continue;
        }
        if let Ok(mut c) = tmdb_review_cache().write() {
            c.insert(key, reviews.clone());
        }
        if let Some(item) = items.get_mut(idx) {
            item.reviews = reviews;
        }
    }
}

pub(crate) async fn tmdb_search(
    is_tv: bool,
    query: &str,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let endpoint = if is_tv { "/search/tv" } else { "/search/movie" };
    let url = format!(
        "{}{}?api_key={}&page={}&query={}&include_adult=false",
        TMDB_BASE,
        endpoint,
        api_key,
        page,
        urlencoding(query)
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let total_pages = payload
        .get("total_pages")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let items = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(tmdb_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn tmdb_to_item(r: &serde_json::Value) -> CatalogItem {
    let title = r
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("name").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let description = r
        .get("overview")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let year = r
        .get("release_date")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("first_air_date").and_then(|v| v.as_str()))
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
    let id = r
        .get("id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());
    let poster_url = r
        .get("poster_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|p| format!("{}/w500{}", TMDB_IMG_BASE, p));
    let backdrop_url = r
        .get("backdrop_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|p| format!("{}/w1280{}", TMDB_IMG_BASE, p));
    let reviews = parse_tmdb_reviews(r);
    CatalogItem {
        id,
        title,
        year,
        description,
        poster_url,
        backdrop_url,
        artists: Vec::new(),
        reviews,
        artist_name: None,
    }
}

async fn tmdb_genres(
    is_tv: bool,
) -> Result<Vec<CatalogGenre>, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let path = if is_tv {
        "/genre/tv/list"
    } else {
        "/genre/movie/list"
    };
    let url = format!("{}{}?api_key={}", TMDB_BASE, path, api_key);
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let genres = payload
        .get("genres")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    let id = g.get("id")?.as_i64()?.to_string();
                    let name = g.get("name")?.as_str()?.to_string();
                    Some(CatalogGenre { id, name })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(genres)
}

const TMDB_TOP_CAST: usize = 12;
/// Crew jobs we surface (everything else from the credits payload is dropped
/// to keep the artists array short and meaningful on the firkin).
const TMDB_CREW_JOBS: &[&str] = &[
    "Director",
    "Writer",
    "Screenplay",
    "Story",
    "Producer",
    "Executive Producer",
    "Original Music Composer",
    "Director of Photography",
];

/// Fetch credits + videos for a TMDB movie or TV show in a single
/// upstream HTTP request via `append_to_response=credits,videos`. The
/// detail endpoint (`/movie/{id}` or `/tv/{id}`) carries both blocks in
/// one response, so callers don't pay two round-trips when the
/// `/metadata` endpoint is hit. The same response also carries the
/// item's `vote_average` (0–10) and `vote_count`, so we extract a
/// `CatalogReview` from it here without an extra request.
///
/// When the upstream item resolves to an IMDb id (movies have it on the
/// detail response directly; TV shows need `external_ids` appended) and
/// `OMDB_API_KEY` is set, this also makes a best-effort call to OMDb
/// and merges Rotten Tomatoes / Metacritic / IMDb reviews into the
/// returned `CatalogReview` list. OMDb failures are swallowed so a
/// missing key, transient outage, or unmatched id never breaks the TMDB
/// path.
pub(crate) async fn tmdb_metadata(
    is_tv: bool,
    id: &str,
) -> Result<
    (Vec<CatalogArtist>, Vec<CatalogTrailer>, Vec<CatalogReview>),
    (StatusCode, Json<serde_json::Value>),
> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let kind = if is_tv { "tv" } else { "movie" };
    // TV shows don't carry `imdb_id` at the root — append `external_ids`
    // so OMDb enrichment works for them too.
    let appends = if is_tv {
        "credits,videos,external_ids"
    } else {
        "credits,videos"
    };
    let url = format!(
        "{}/{}/{}?api_key={}&append_to_response={}",
        TMDB_BASE,
        kind,
        urlencoding(id),
        api_key,
        appends
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;

    let credits = payload.get("credits").cloned().unwrap_or(serde_json::Value::Null);
    let artists = parse_tmdb_credits(&credits);
    let videos = payload.get("videos").cloned().unwrap_or(serde_json::Value::Null);
    let trailers = parse_tmdb_videos(&videos);
    let mut reviews = parse_tmdb_reviews(&payload);

    if let Some(imdb_id) = extract_imdb_id(&payload) {
        let mut omdb_reviews = fetch_omdb_reviews(&imdb_id).await;
        merge_reviews(&mut reviews, &mut omdb_reviews);
    }

    Ok((artists, trailers, reviews))
}

/// Pull the IMDb id off a TMDB detail response. Movies expose it as
/// `imdb_id` at the root; TV shows expose it under
/// `external_ids.imdb_id` (only present when the response was fetched
/// with `append_to_response=external_ids`). Returns `None` when the
/// field is missing or empty.
fn extract_imdb_id(payload: &serde_json::Value) -> Option<String> {
    let candidate = payload
        .get("imdb_id")
        .and_then(|v| v.as_str())
        .or_else(|| {
            payload
                .get("external_ids")
                .and_then(|v| v.get("imdb_id"))
                .and_then(|v| v.as_str())
        })?;
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Best-effort OMDb lookup keyed by IMDb id. Returns the parsed reviews
/// (Rotten Tomatoes, Metacritic, IMDb) on success, or an empty vec on
/// any failure mode — missing `OMDB_API_KEY`, network error, OMDb
/// `Response: "False"`, etc. — so the TMDB metadata flow stays robust
/// to a degraded enricher.
async fn fetch_omdb_reviews(imdb_id: &str) -> Vec<CatalogReview> {
    let api_key = std::env::var("OMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Vec::new();
    }
    let url = format!(
        "{}/?i={}&apikey={}",
        OMDB_BASE,
        urlencoding(imdb_id),
        urlencoding(&api_key)
    );
    let payload: serde_json::Value = match http_get_json(&url, &[("Accept", "application/json")]).await {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    if payload
        .get("Response")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("True"))
        != Some(true)
    {
        return Vec::new();
    }
    parse_omdb_reviews(&payload)
}

/// Project an OMDb response's `Ratings` array (plus the root-level
/// `imdbVotes` count) into our universal `CatalogReview` shape. Source
/// labels are canonicalised (`"Internet Movie Database"` → `"IMDb"`,
/// etc.) and unparseable values (`"N/A"`, malformed strings) are
/// dropped so the firkin body never carries a bogus score.
fn parse_omdb_reviews(payload: &serde_json::Value) -> Vec<CatalogReview> {
    let mut out: Vec<CatalogReview> = Vec::new();
    let Some(ratings) = payload.get("Ratings").and_then(|v| v.as_array()) else {
        return out;
    };
    for r in ratings {
        let source = r.get("Source").and_then(|v| v.as_str()).unwrap_or("").trim();
        let value = r.get("Value").and_then(|v| v.as_str()).unwrap_or("").trim();
        if source.is_empty() || value.is_empty() {
            continue;
        }
        let Some((score, max_score)) = parse_omdb_rating_value(value) else {
            continue;
        };
        out.push(CatalogReview {
            label: canonicalise_omdb_source(source),
            score,
            max_score,
            vote_count: None,
        });
    }
    if let Some(votes) = payload
        .get("imdbVotes")
        .and_then(|v| v.as_str())
        .and_then(parse_imdb_votes)
    {
        if let Some(imdb) = out.iter_mut().find(|r| r.label == "IMDb") {
            imdb.vote_count = Some(votes);
        }
    }
    out
}

/// Parse an OMDb rating string. Accepts `"92%"`, `"7.8/10"`,
/// `"78/100"`, etc. Returns `None` for unparseable or `"N/A"` values.
fn parse_omdb_rating_value(value: &str) -> Option<(f64, f64)> {
    let v = value.trim();
    if let Some(stripped) = v.strip_suffix('%') {
        let n: f64 = stripped.trim().parse().ok()?;
        if n.is_finite() {
            return Some((n, 100.0));
        }
    }
    if let Some((num, den)) = v.split_once('/') {
        let score: f64 = num.trim().parse().ok()?;
        let max: f64 = den.trim().parse().ok()?;
        if score.is_finite() && max.is_finite() && max > 0.0 {
            return Some((score, max));
        }
    }
    None
}

fn canonicalise_omdb_source(source: &str) -> String {
    let lower = source.to_ascii_lowercase();
    match lower.as_str() {
        "internet movie database" | "imdb" => "IMDb".to_string(),
        "rotten tomatoes" => "Rotten Tomatoes".to_string(),
        "metacritic" => "Metacritic".to_string(),
        _ => source.to_string(),
    }
}

/// `"2,945,123"` → `Some(2945123)`. Returns `None` for empty / `"N/A"` /
/// otherwise non-numeric inputs.
fn parse_imdb_votes(raw: &str) -> Option<u32> {
    let cleaned: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<u64>().ok().map(|n| u32::try_from(n).unwrap_or(u32::MAX))
}

/// Append OMDb-sourced reviews to the TMDB-sourced list, but skip any
/// entry whose label already exists in `existing` so a future TMDB
/// upstream change (e.g. TMDB starts surfacing IMDb directly) can't
/// double up. The TMDB review wins on conflict because it's the
/// upstream the rest of the catalog flow is keyed on.
fn merge_reviews(existing: &mut Vec<CatalogReview>, incoming: &mut Vec<CatalogReview>) {
    incoming.retain(|i| !existing.iter().any(|e| e.label == i.label));
    existing.append(incoming);
}

/// Pull TMDB's `vote_average` and `vote_count` off the item detail
/// response into our universal `CatalogReview` shape. TMDB scores are
/// always out of 10. Returns an empty vec when the upstream response
/// has no votes yet (vote_count == 0) so the firkin doesn't carry a
/// meaningless "0.0 / 10 (0 votes)" entry.
fn parse_tmdb_reviews(payload: &serde_json::Value) -> Vec<CatalogReview> {
    let score = payload.get("vote_average").and_then(|v| v.as_f64());
    let count = payload
        .get("vote_count")
        .and_then(|v| v.as_u64())
        .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
    match (score, count) {
        (Some(s), Some(c)) if c > 0 && s.is_finite() => vec![CatalogReview {
            label: "TMDB".to_string(),
            score: s,
            max_score: 10.0,
            vote_count: Some(c),
        }],
        (Some(s), None) if s.is_finite() && s > 0.0 => vec![CatalogReview {
            label: "TMDB".to_string(),
            score: s,
            max_score: 10.0,
            vote_count: None,
        }],
        _ => Vec::new(),
    }
}

/// Fetch a single page of TMDB's recommendations for the given movie /
/// TV show. The recommendations endpoint returns the same item shape as
/// `/popular`, so we reuse `tmdb_to_item` to map it into our universal
/// `CatalogItem`. Per-item credits are intentionally not fetched here —
/// the related grid only renders title / year / poster.
async fn tmdb_related(
    is_tv: bool,
    id: &str,
) -> Result<Vec<CatalogItem>, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/recommendations?api_key={}&page=1",
        TMDB_BASE,
        kind,
        urlencoding(id),
        api_key
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let items: Vec<CatalogItem> = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(tmdb_to_item).collect())
        .unwrap_or_default();
    Ok(items)
}

fn parse_tmdb_credits(credits: &serde_json::Value) -> Vec<CatalogArtist> {
    let mut out: Vec<CatalogArtist> = Vec::new();
    if let Some(cast) = credits.get("cast").and_then(|v| v.as_array()) {
        for member in cast.iter().take(TMDB_TOP_CAST) {
            let Some(name) = member.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let character = member
                .get("character")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let image_url = member
                .get("profile_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|p| format!("{}/w185{}", TMDB_IMG_BASE, p));
            // Three-field shape: bake the character into the role.
            let role = match character {
                Some(c) => Some(format!("Actor as {c}")),
                None => Some("Actor".to_string()),
            };
            out.push(CatalogArtist {
                name: name.to_string(),
                role,
                image_url,
            });
        }
    }
    if let Some(crew) = credits.get("crew").and_then(|v| v.as_array()) {
        // Dedup by (person_id, job): TMDB sometimes lists the same person
        // multiple times if they had several roles in the same job category.
        let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        for member in crew {
            let Some(job) = member.get("job").and_then(|v| v.as_str()) else {
                continue;
            };
            if !TMDB_CREW_JOBS.contains(&job) {
                continue;
            }
            let Some(name) = member.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let person_id_str = member
                .get("id")
                .map(|v| v.to_string())
                .unwrap_or_else(|| name.to_string());
            if !seen.insert((person_id_str.clone(), job.to_string())) {
                continue;
            }
            let image_url = member
                .get("profile_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|p| format!("{}/w185{}", TMDB_IMG_BASE, p));
            out.push(CatalogArtist {
                name: name.to_string(),
                role: Some(job.to_string()),
                image_url,
            });
        }
    }
    out
}

/// Map TMDB's `videos` block into our universal trailer shape. We keep
/// only YouTube videos whose `type` is `Trailer` (the official trailer
/// is what the firkin's `trailers` array is for; teasers, clips, and
/// behind-the-scenes are filtered out), AND whose `iso_639_1` is `"en"`
/// — non-English entries are dropped here so the WebUI surfaces only
/// English trailers (when none survive the filter the frontend falls
/// back to the YouTube fuzzy search). When the upstream entry is
/// flagged `official`, it sorts ahead of fan/redistributed cuts.
fn parse_tmdb_videos(videos: &serde_json::Value) -> Vec<CatalogTrailer> {
    let Some(arr) = videos.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut scored: Vec<(i32, CatalogTrailer)> = Vec::new();
    for v in arr {
        let site = v.get("site").and_then(|s| s.as_str()).unwrap_or("");
        if !site.eq_ignore_ascii_case("YouTube") {
            continue;
        }
        let kind = v.get("type").and_then(|s| s.as_str()).unwrap_or("");
        if !kind.eq_ignore_ascii_case("Trailer") {
            continue;
        }
        let language = v
            .get("iso_639_1")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase());
        // English-only filter. Foreign-language trailers fall through
        // to the YouTube fuzzy search on the frontend so the user
        // doesn't end up with a Spanish/French dub when an English
        // trailer wasn't on TMDB.
        if language.as_deref() != Some("en") {
            continue;
        }
        let Some(key) = v
            .get("key")
            .and_then(|s| s.as_str())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        let name = v
            .get("name")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let official = v
            .get("official")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
        // Higher score = better match. Official trailers always beat
        // unofficial ones; ties broken by the order TMDB returned them.
        let score = if official { 10 } else { 0 } - (scored.len() as i32);
        scored.push((
            score,
            CatalogTrailer {
                youtube_url: format!("https://www.youtube.com/watch?v={key}"),
                label: name,
                language,
            },
        ));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, t)| t).collect()
}

async fn tmdb_tv_seasons(
    State(_state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<CatalogSeason>>, (StatusCode, Json<serde_json::Value>)> {
    fetch_tmdb_tv_seasons(&id).await.map(Json)
}

/// Fetch a TMDB TV show's season list. Extracted from the HTTP handler so
/// background tasks (`tv_build`) can reuse the same upstream call + JSON
/// parser without round-tripping through the network.
pub(crate) async fn fetch_tmdb_tv_seasons(
    id: &str,
) -> Result<Vec<CatalogSeason>, (StatusCode, Json<serde_json::Value>)> {
    if id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "id is required"));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let url = format!(
        "{}/tv/{}?api_key={}",
        TMDB_BASE,
        urlencoding(id.trim()),
        api_key
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let mut out: Vec<CatalogSeason> = Vec::new();
    if let Some(arr) = payload.get("seasons").and_then(|v| v.as_array()) {
        for s in arr {
            let season_number = s.get("season_number").and_then(|v| v.as_i64()).unwrap_or(0);
            // TMDB exposes a virtual "season 0" for specials; skip it so we
            // don't fan out a search query for behind-the-scenes/specials
            // when the user really wants per-season trailers.
            if season_number <= 0 {
                continue;
            }
            let name = s
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Season {season_number}"));
            let air_year = s
                .get("air_date")
                .and_then(|v| v.as_str())
                .and_then(|d| d.get(0..4))
                .and_then(|y| y.parse::<i32>().ok());
            let episode_count = s.get("episode_count").and_then(|v| v.as_i64());
            let poster_url = s
                .get("poster_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|p| format!("{}/w342{}", TMDB_IMG_BASE, p));
            let overview = s
                .get("overview")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            out.push(CatalogSeason {
                season_number,
                name,
                air_year,
                episode_count,
                poster_url,
                overview,
            });
        }
    }
    out.sort_by_key(|s| s.season_number);
    Ok(out)
}

async fn tmdb_tv_season_episodes(
    State(_state): State<CloudState>,
    Path((id, season_number)): Path<(String, i64)>,
) -> Result<Json<Vec<CatalogEpisode>>, (StatusCode, Json<serde_json::Value>)> {
    fetch_tmdb_tv_season_episodes(&id, season_number)
        .await
        .map(Json)
}

/// Fetch a TMDB TV show season's episode list. Extracted from the HTTP
/// handler so background tasks (`tv_build`) can reuse the same upstream
/// call + JSON parser without round-tripping through the network.
pub(crate) async fn fetch_tmdb_tv_season_episodes(
    id: &str,
    season_number: i64,
) -> Result<Vec<CatalogEpisode>, (StatusCode, Json<serde_json::Value>)> {
    if id.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "id is required"));
    }
    if season_number < 0 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "season_number must be non-negative",
        ));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let url = format!(
        "{}/tv/{}/season/{}?api_key={}",
        TMDB_BASE,
        urlencoding(id.trim()),
        season_number,
        api_key
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;
    let mut out: Vec<CatalogEpisode> = Vec::new();
    if let Some(arr) = payload.get("episodes").and_then(|v| v.as_array()) {
        for e in arr {
            let episode_number = e
                .get("episode_number")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let s_num = e
                .get("season_number")
                .and_then(|v| v.as_i64())
                .unwrap_or(season_number);
            let name = e
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Episode {episode_number}"));
            let overview = e
                .get("overview")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let air_date = e
                .get("air_date")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let runtime = e.get("runtime").and_then(|v| v.as_i64());
            let still_url = e
                .get("still_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|p| format!("{}/w300{}", TMDB_IMG_BASE, p));
            let vote_average = e.get("vote_average").and_then(|v| v.as_f64());
            out.push(CatalogEpisode {
                episode_number,
                season_number: s_num,
                name,
                overview,
                air_date,
                runtime,
                still_url,
                vote_average,
            });
        }
    }
    out.sort_by_key(|e| e.episode_number);
    Ok(out)
}

// ---------- MusicBrainz ----------

const MUSICBRAINZ_GENRES: &[&str] = &[
    "rock",
    "pop",
    "electronic",
    "hip hop",
    "jazz",
    "classical",
    "r&b",
    "metal",
    "folk",
    "soul",
    "punk",
    "blues",
    "country",
    "ambient",
    "indie",
    "alternative",
    "reggae",
    "latin",
];

fn static_music_genres() -> Vec<CatalogGenre> {
    MUSICBRAINZ_GENRES
        .iter()
        .map(|g| CatalogGenre {
            id: (*g).to_string(),
            name: capitalize_words(g),
        })
        .collect()
}

async fn musicbrainz_popular(
    genre: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 20;
    let offset = (page - 1) * limit;
    let tag = genre.filter(|s| !s.is_empty()).unwrap_or("rock");
    let query = format!("tag:\"{}\"", tag);
    let url = format!(
        "{}/release-group?query={}&fmt=json&limit={}&offset={}",
        MUSICBRAINZ_BASE,
        urlencoding(&query),
        limit,
        offset
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let count = payload
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(limit);
    let total_pages = ((count as f64) / (limit as f64)).ceil() as i64;
    let items = payload
        .get("release-groups")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(musicbrainz_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

/// Which release-group field the caller wants to search on. The default is
/// `Artist` because the typical free-text query is an artist name ("keane")
/// and the user wants every release-group by that artist to come back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MbSearchField {
    Artist,
    Release,
}

impl MbSearchField {
    fn from_param(s: Option<&str>) -> Self {
        match s.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("release") | Some("releasegroup") | Some("title") | Some("album") => Self::Release,
            _ => Self::Artist,
        }
    }
    fn lucene_field(self) -> &'static str {
        match self {
            Self::Artist => "artist",
            Self::Release => "releasegroup",
        }
    }
}

/// Normalise a string for token comparison: lowercase, strip parenthesised
/// content, replace any non-alphanumeric run with a single space. Mirrors the
/// frontend's `youtube-match.service.ts` pattern so the double-dip filter
/// behaves the same on both sides.
fn mb_normalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth: i32 = 0;
    for ch in input.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                out.push(' ');
            }
            ')' | ']' | '}' => {
                depth = (depth - 1).max(0);
                out.push(' ');
            }
            _ if depth > 0 => out.push(' '),
            _ => {
                let lower = ch.to_ascii_lowercase();
                if lower.is_ascii_alphanumeric() {
                    out.push(lower);
                } else {
                    out.push(' ');
                }
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn mb_tokens(input: &str) -> Vec<String> {
    mb_normalize(input)
        .split(' ')
        .filter(|w| w.len() > 1)
        .map(|w| w.to_string())
        .collect()
}

/// Double-dip: require ≥50% of the query's tokens to appear (as substrings)
/// in the chosen target field of a release-group result. MusicBrainz's
/// upstream search is permissive (Lucene scoring tolerates partial matches),
/// so we filter out results where the user's term isn't actually present in
/// the field they asked us to search on.
fn mb_field_text(rg: &serde_json::Value, field: MbSearchField) -> String {
    match field {
        MbSearchField::Release => rg
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        MbSearchField::Artist => rg
            .get("artist-credit")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        c.get("name")
                            .and_then(|v| v.as_str())
                            .or_else(|| c.get("artist").and_then(|a| a.get("name")?.as_str()))
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default(),
    }
}

fn mb_passes_double_dip(
    rg: &serde_json::Value,
    field: MbSearchField,
    query_tokens: &[String],
) -> bool {
    if query_tokens.is_empty() {
        return true;
    }
    let target = mb_normalize(&mb_field_text(rg, field));
    let hits = query_tokens
        .iter()
        .filter(|t| target.contains(t.as_str()))
        .count();
    let ratio = hits as f64 / query_tokens.len() as f64;
    ratio >= 0.5
}

pub(crate) async fn musicbrainz_search(
    query: &str,
    page: i64,
    field: Option<&str>,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 20;
    let offset = (page - 1).max(0) * limit;
    let field = MbSearchField::from_param(field);
    // Escape Lucene query special chars in the user's query, then constrain
    // the upstream search to the field the user picked. MusicBrainz's
    // `query=` param accepts Lucene syntax; raw quotes/colons in the user
    // input would break the parse. No `primarytype:` restriction — albums,
    // singles, EPs and broadcasts all come back, the double-dip filter below
    // is what guarantees the search term actually appears in the target field.
    let escaped: String = query
        .chars()
        .flat_map(|c| match c {
            '\\' | '+' | '-' | '!' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '"' | '~' | '*'
            | '?' | ':' | '/' => vec!['\\', c],
            _ => vec![c],
        })
        .collect();
    let lucene = format!("{}:\"{}\"", field.lucene_field(), escaped);
    let url = format!(
        "{}/release-group?query={}&fmt=json&limit={}&offset={}",
        MUSICBRAINZ_BASE,
        urlencoding(&lucene),
        limit,
        offset
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let count = payload
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(limit);
    let total_pages = ((count as f64) / (limit as f64)).ceil() as i64;
    let query_tokens = mb_tokens(query);
    let items = payload
        .get("release-groups")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|rg| mb_passes_double_dip(rg, field, &query_tokens))
                .map(musicbrainz_to_item)
                .collect()
        })
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

async fn musicbrainz_tracks(
    Path(id): Path<String>,
) -> Result<Json<Vec<CatalogTrack>>, (StatusCode, Json<serde_json::Value>)> {
    if id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "release-group id is required"));
    }
    let url = format!(
        "{}/release?release-group={}&inc=recordings&fmt=json&limit=1",
        MUSICBRAINZ_BASE,
        urlencoding(&id)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let mut out: Vec<CatalogTrack> = Vec::new();
    if let Some(release) = payload
        .get("releases")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
    {
        if let Some(media) = release.get("media").and_then(|v| v.as_array()) {
            for medium in media {
                let Some(tracks) = medium.get("tracks").and_then(|v| v.as_array()) else {
                    continue;
                };
                for t in tracks {
                    let track_id = t
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let position = t.get("position").and_then(|v| v.as_i64()).unwrap_or(0);
                    let title = t
                        .get("title")
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            t.get("recording")
                                .and_then(|r| r.get("title"))
                                .and_then(|v| v.as_str())
                        })
                        .unwrap_or("")
                        .to_string();
                    let length_ms = t
                        .get("length")
                        .and_then(|v| v.as_i64())
                        .or_else(|| {
                            t.get("recording")
                                .and_then(|r| r.get("length"))
                                .and_then(|v| v.as_i64())
                        });
                    out.push(CatalogTrack {
                        id: track_id,
                        position,
                        title,
                        length_ms,
                    });
                }
            }
        }
    }
    Ok(Json(out))
}

/// Fetch a release-group's artist credits AND community rating in one
/// upstream call via `inc=artist-credits+ratings`. The MusicBrainz
/// rating block has shape `{ "value": <0..5>, "votes-count": <u32> }`;
/// we only emit a `CatalogReview` when at least one vote exists so
/// fresh / niche release-groups don't get a hollow `0.0 / 5 (0 votes)`
/// entry.
async fn musicbrainz_metadata(
    release_group_id: &str,
) -> Result<(Vec<CatalogArtist>, Vec<CatalogReview>), (StatusCode, Json<serde_json::Value>)> {
    let url = format!(
        "{}/release-group/{}?inc=artist-credits+ratings&fmt=json",
        MUSICBRAINZ_BASE,
        urlencoding(release_group_id)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let mut artists: Vec<CatalogArtist> = Vec::new();
    let credits = payload
        .get("artist-credit")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for credit in credits {
        let artist = credit.get("artist").cloned().unwrap_or_default();
        let name = credit
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| artist.get("name").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        artists.push(CatalogArtist {
            name,
            role: Some("Artist".to_string()),
            image_url: None,
        });
    }
    let reviews = parse_musicbrainz_reviews(&payload);
    Ok((artists, reviews))
}

fn parse_musicbrainz_reviews(payload: &serde_json::Value) -> Vec<CatalogReview> {
    let rating = payload.get("rating");
    let value = rating.and_then(|r| r.get("value")).and_then(|v| v.as_f64());
    let votes = rating
        .and_then(|r| r.get("votes-count"))
        .and_then(|v| v.as_u64())
        .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
    match (value, votes) {
        (Some(v), Some(c)) if c > 0 && v.is_finite() => vec![CatalogReview {
            label: "MusicBrainz".to_string(),
            score: v,
            max_score: 5.0,
            vote_count: Some(c),
        }],
        _ => Vec::new(),
    }
}

/// Albums similar to the given release-group, computed from MusicBrainz's
/// curated genres + user-contributed tags. We pull the source release-
/// group with `inc=artist-credits+tags+genres` once, take its top-weighted
/// terms, and run a Lucene `tag:"…"` similarity search over album/EP
/// release-groups (excluding the source itself). When the source has no
/// genres/tags or the similarity query returns nothing, we fall back to
/// the older same-artist browse so the panel still has content.
async fn musicbrainz_related(
    release_group_id: &str,
) -> Result<Vec<CatalogItem>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!(
        "{}/release-group/{}?inc=artist-credits+tags+genres&fmt=json",
        MUSICBRAINZ_BASE,
        urlencoding(release_group_id)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;

    let top_terms = mb_top_similarity_terms(&payload, 4);
    if !top_terms.is_empty() {
        let term_clause = top_terms
            .iter()
            .map(|t| format!("tag:\"{}\"", lucene_escape_phrase(t)))
            .collect::<Vec<_>>()
            .join(" OR ");
        let query = format!(
            "({term_clause}) AND (primarytype:album OR primarytype:ep) AND NOT rgid:{release_group_id}"
        );
        let url = format!(
            "{}/release-group?query={}&fmt=json&limit=24",
            MUSICBRAINZ_BASE,
            urlencoding(&query)
        );
        let search_payload: serde_json::Value = http_get_json(
            &url,
            &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
        )
        .await?;
        let items: Vec<CatalogItem> = search_payload
            .get("release-groups")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|rg| {
                        let item = musicbrainz_to_item(rg);
                        if item.id == release_group_id {
                            return None;
                        }
                        Some(item)
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !items.is_empty() {
            return Ok(items);
        }
    }

    // Fallback: same primary artist as the source release-group. Reuses
    // the artist-credits already on `payload` so this branch costs at most
    // one extra HTTP call (the `release-group?artist=…` browse).
    musicbrainz_albums_by_artist_from_payload(release_group_id, &payload).await
}

async fn musicbrainz_albums_by_artist_handler(
    State(_state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<CatalogItem>>, (StatusCode, Json<serde_json::Value>)> {
    musicbrainz_albums_by_artist(&id).await.map(Json)
}

/// Other release-groups (album / EP) by the source release-group's
/// primary artist. Resolves the artist via `inc=artist-credits` on the
/// release-group, then browses release-groups by that artist id; the
/// source release-group is filtered out of the result. Used by both
/// the same-artist fallback inside [`musicbrainz_related`] and the
/// dedicated sidebar endpoint exposed at
/// `GET /api/catalog/musicbrainz/release-groups/{id}/albums-by-artist`.
async fn musicbrainz_albums_by_artist(
    release_group_id: &str,
) -> Result<Vec<CatalogItem>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!(
        "{}/release-group/{}?inc=artist-credits&fmt=json",
        MUSICBRAINZ_BASE,
        urlencoding(release_group_id)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    musicbrainz_albums_by_artist_from_payload(release_group_id, &payload).await
}

/// Inner helper for [`musicbrainz_albums_by_artist`] that takes an
/// already-fetched release-group payload (which must include
/// `artist-credit`) and browses by that artist id. Lets callers like
/// [`musicbrainz_related`] reuse a payload they already pulled with
/// `inc=artist-credits+tags+genres` instead of paying for a second
/// release-group lookup.
async fn musicbrainz_albums_by_artist_from_payload(
    release_group_id: &str,
    payload: &serde_json::Value,
) -> Result<Vec<CatalogItem>, (StatusCode, Json<serde_json::Value>)> {
    let artist_id = payload
        .get("artist-credit")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("artist"))
        .and_then(|a| a.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let Some(artist_id) = artist_id else {
        return Ok(Vec::new());
    };
    let url = format!(
        "{}/release-group?artist={}&type=album|ep&fmt=json&limit=24",
        MUSICBRAINZ_BASE,
        urlencoding(&artist_id)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let items: Vec<CatalogItem> = payload
        .get("release-groups")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|rg| {
                    let item = musicbrainz_to_item(rg);
                    if item.id == release_group_id {
                        return None;
                    }
                    Some(item)
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(items)
}

/// Pick up to `n` similarity terms from a release-group payload, drawn
/// from MB's curated `genres` (preferred) and free-form user `tags`.
/// Sorted by `count` (number of users who applied the tag) descending.
/// Names are deduped case-insensitively across the two arrays so the
/// same string surfaced as both a genre and a tag only counts once.
fn mb_top_similarity_terms(payload: &serde_json::Value, n: usize) -> Vec<String> {
    let mut terms: Vec<(String, i64)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for field in ["genres", "tags"] {
        let Some(arr) = payload.get(field).and_then(|v| v.as_array()) else {
            continue;
        };
        for entry in arr {
            let name = entry
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if name.is_empty() {
                continue;
            }
            if !seen.insert(name.to_ascii_lowercase()) {
                continue;
            }
            let count = entry.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
            terms.push((name, count));
        }
    }
    terms.sort_by(|a, b| b.1.cmp(&a.1));
    terms.into_iter().take(n).map(|(name, _)| name).collect()
}

/// Escape characters that have special meaning inside a Lucene
/// double-quoted phrase. MB's `query` parameter is parsed with Lucene,
/// so backslash and double-quote both need to be escaped before being
/// embedded in `tag:"…"`.
fn lucene_escape_phrase(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn musicbrainz_to_item(rg: &serde_json::Value) -> CatalogItem {
    let id = rg
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let title = rg
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let year = rg
        .get("first-release-date")
        .and_then(|v| v.as_str())
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
    let credits = rg
        .get("artist-credit")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    c.get("name")
                        .and_then(|v| v.as_str())
                        .or_else(|| c.get("artist").and_then(|a| a.get("name")?.as_str()))
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let primary_type = rg
        .get("primary-type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut description = String::new();
    if !credits.is_empty() {
        description.push_str(&credits);
    }
    if !primary_type.is_empty() {
        if !description.is_empty() {
            description.push_str(" · ");
        }
        description.push_str(primary_type);
    }
    let poster_url = if id.is_empty() {
        None
    } else {
        Some(format!("{}/release-group/{}/front", COVERART_BASE, id))
    };
    let reviews = parse_musicbrainz_reviews(rg);
    let artist_name = if credits.is_empty() {
        None
    } else {
        Some(credits.clone())
    };
    CatalogItem {
        id,
        title,
        year,
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        poster_url,
        backdrop_url: None,
        artists: Vec::new(),
        reviews,
        artist_name,
    }
}

// ---------- YouTube ----------

/// Two options the YouTube addon's filter row offers — the catalog page's
/// generic genre dropdown is reused as a Videos / Channels selector that
/// drives the Piped `filter=` parameter on `/api/catalog/youtube-video/search`.
/// Order matters: the first entry is the default selection.
const YOUTUBE_SEARCH_KINDS: &[(&str, &str)] = &[("videos", "Videos"), ("channels", "Channels")];

fn static_youtube_search_kinds() -> Vec<CatalogGenre> {
    YOUTUBE_SEARCH_KINDS
        .iter()
        .map(|(id, name)| CatalogGenre {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

async fn youtube_popular(
    _filter: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    youtube_search("trending", page, Some("videos")).await
}

async fn youtube_search(
    query: &str,
    page: i64,
    filter: Option<&str>,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let kind = match filter.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("channels") => "channels",
        _ => "videos",
    };
    let resp = mhaol_yt_dlp::search::search_query(query, None)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("youtube search failed: {e}")))?;
    let mut all: Vec<CatalogItem> = if kind == "channels" {
        resp.channels
            .iter()
            .map(channel_search_to_item)
            .collect()
    } else {
        resp.items.iter().map(video_search_to_item).collect()
    };
    let limit: usize = 24;
    let total_pages = ((all.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let items: Vec<CatalogItem> = if offset >= all.len() {
        Vec::new()
    } else {
        all.drain(offset..(offset + limit).min(all.len())).collect()
    };
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn video_search_to_item(item: &mhaol_yt_dlp::search::SearchItem) -> CatalogItem {
    let mut parts: Vec<String> = Vec::new();
    if !item.uploader_name.is_empty() {
        parts.push(item.uploader_name.clone());
    }
    if !item.views_text.is_empty() {
        parts.push(item.views_text.clone());
    } else if item.views > 0 {
        parts.push(format!("{} views", item.views));
    }
    let description = if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    };
    let poster_url = if item.thumbnail.is_empty() {
        None
    } else {
        Some(item.thumbnail.clone())
    };
    CatalogItem {
        id: item.video_id.clone(),
        title: item.title.clone(),
        year: None,
        description,
        poster_url,
        backdrop_url: None,
        artists: Vec::new(),
        reviews: Vec::new(),
        artist_name: None,
    }
}

fn channel_search_to_item(item: &mhaol_yt_dlp::search::SearchChannelItem) -> CatalogItem {
    let mut parts: Vec<String> = Vec::new();
    if !item.description.is_empty() {
        parts.push(item.description.clone());
    }
    if !item.subscriber_text.is_empty() {
        parts.push(item.subscriber_text.clone());
    }
    let description = if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    };
    let poster_url = if item.thumbnail.is_empty() {
        None
    } else {
        Some(item.thumbnail.clone())
    };
    CatalogItem {
        id: item.channel_id.clone(),
        title: item.name.clone(),
        year: None,
        description,
        poster_url,
        backdrop_url: None,
        artists: Vec::new(),
        reviews: Vec::new(),
        artist_name: None,
    }
}

async fn youtube_video_artists(
    video_id: &str,
) -> Result<Vec<CatalogArtist>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!(
        "https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v={}&format=json",
        urlencoding(video_id),
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let name = payload
        .get("author_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![CatalogArtist {
        name,
        role: Some("Channel".to_string()),
        image_url: None,
    }])
}

// ---------- helpers ----------

async fn http_get_json(
    url: &str,
    headers: &[(&str, &str)],
) -> Result<serde_json::Value, (StatusCode, Json<serde_json::Value>)> {
    let mut req = reqwest::Client::new().get(url);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let res = req
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("upstream request failed: {e}")))?;
    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("upstream returned {}", res.status()),
        ));
    }
    res.json::<serde_json::Value>()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("upstream parse failed: {e}")))
}

fn capitalize_words(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn omdb_payload() -> serde_json::Value {
        serde_json::json!({
            "Title": "The Shawshank Redemption",
            "imdbID": "tt0111161",
            "imdbRating": "9.3",
            "imdbVotes": "2,945,123",
            "Ratings": [
                { "Source": "Internet Movie Database", "Value": "9.3/10" },
                { "Source": "Rotten Tomatoes", "Value": "91%" },
                { "Source": "Metacritic", "Value": "82/100" }
            ],
            "Response": "True"
        })
    }

    #[test]
    fn parse_omdb_rating_value_handles_known_shapes() {
        assert_eq!(parse_omdb_rating_value("92%"), Some((92.0, 100.0)));
        assert_eq!(parse_omdb_rating_value("7.8/10"), Some((7.8, 10.0)));
        assert_eq!(parse_omdb_rating_value("78/100"), Some((78.0, 100.0)));
        assert_eq!(parse_omdb_rating_value("N/A"), None);
        assert_eq!(parse_omdb_rating_value(""), None);
    }

    #[test]
    fn parse_omdb_reviews_canonicalises_labels_and_attaches_imdb_votes() {
        let reviews = parse_omdb_reviews(&omdb_payload());
        let labels: Vec<&str> = reviews.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, vec!["IMDb", "Rotten Tomatoes", "Metacritic"]);
        let imdb = reviews.iter().find(|r| r.label == "IMDb").unwrap();
        assert_eq!(imdb.score, 9.3);
        assert_eq!(imdb.max_score, 10.0);
        assert_eq!(imdb.vote_count, Some(2_945_123));
        let rt = reviews.iter().find(|r| r.label == "Rotten Tomatoes").unwrap();
        assert_eq!(rt.score, 91.0);
        assert_eq!(rt.max_score, 100.0);
        assert_eq!(rt.vote_count, None);
    }

    #[test]
    fn parse_omdb_reviews_drops_unparseable_entries() {
        let payload = serde_json::json!({
            "Ratings": [
                { "Source": "Rotten Tomatoes", "Value": "N/A" },
                { "Source": "Metacritic", "Value": "78/100" }
            ],
            "Response": "True"
        });
        let reviews = parse_omdb_reviews(&payload);
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].label, "Metacritic");
    }

    #[test]
    fn parse_omdb_reviews_returns_empty_when_ratings_missing() {
        let payload = serde_json::json!({ "Response": "True" });
        assert!(parse_omdb_reviews(&payload).is_empty());
    }

    #[test]
    fn extract_imdb_id_supports_movies_and_tv_shapes() {
        let movie = serde_json::json!({ "imdb_id": "tt0111161" });
        assert_eq!(extract_imdb_id(&movie).as_deref(), Some("tt0111161"));

        let tv = serde_json::json!({ "external_ids": { "imdb_id": "tt0903747" } });
        assert_eq!(extract_imdb_id(&tv).as_deref(), Some("tt0903747"));

        let neither = serde_json::json!({ "imdb_id": "" });
        assert_eq!(extract_imdb_id(&neither), None);
    }

    #[test]
    fn merge_reviews_skips_duplicates_by_label() {
        let mut existing = vec![CatalogReview {
            label: "TMDB".into(),
            score: 7.5,
            max_score: 10.0,
            vote_count: Some(100),
        }];
        let mut incoming = vec![
            CatalogReview {
                label: "TMDB".into(), // collide — should be dropped
                score: 9.0,
                max_score: 10.0,
                vote_count: None,
            },
            CatalogReview {
                label: "Rotten Tomatoes".into(),
                score: 91.0,
                max_score: 100.0,
                vote_count: None,
            },
        ];
        merge_reviews(&mut existing, &mut incoming);
        assert_eq!(existing.len(), 2);
        assert_eq!(existing[0].label, "TMDB");
        assert_eq!(existing[0].score, 7.5);
        assert_eq!(existing[1].label, "Rotten Tomatoes");
    }
}

use crate::state::CloudState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const MUSICBRAINZ_BASE: &str = "https://musicbrainz.org/ws/2";
const COVERART_BASE: &str = "https://coverartarchive.org";
const PIPED_BASE: &str = "https://pipedapi.kavin.rocks";
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
        browsable: true,
    },
    Addon {
        id: "tmdb-tv",
        label: "TMDB TV Shows",
        kind: "tv show",
        filter_label: "Genre",
        has_filter: true,
        browsable: true,
    },
    Addon {
        id: "musicbrainz",
        label: "MusicBrainz",
        kind: "album",
        filter_label: "Genre",
        has_filter: true,
        browsable: true,
    },
    Addon {
        id: "youtube-video",
        label: "YouTube Videos",
        kind: "youtube video",
        filter_label: "Region",
        has_filter: true,
        browsable: true,
    },
    Addon {
        id: "youtube-channel",
        label: "YouTube Channels",
        kind: "youtube channel",
        filter_label: "Region",
        has_filter: true,
        browsable: true,
    },
    // Subtitle / lyric lookups — valid firkin addons but not browsable.
    Addon {
        id: "wyzie-subs-movie",
        label: "Wyzie Subs (Movies)",
        kind: "movie",
        filter_label: "Filter",
        has_filter: false,
        browsable: false,
    },
    Addon {
        id: "wyzie-subs-tv",
        label: "Wyzie Subs (TV)",
        kind: "tv show",
        filter_label: "Filter",
        has_filter: false,
        browsable: false,
    },
    Addon {
        id: "lrclib",
        label: "LRCLIB",
        kind: "album",
        filter_label: "Filter",
        has_filter: false,
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
        browsable: false,
    },
    Addon {
        id: "local-tv",
        label: "Local TV Shows",
        kind: "tv show",
        filter_label: "Filter",
        has_filter: false,
        browsable: false,
    },
    Addon {
        id: "local-album",
        label: "Local Albums",
        kind: "album",
        filter_label: "Filter",
        has_filter: false,
        browsable: false,
    },
    Addon {
        id: "local-book",
        label: "Local Books",
        kind: "book",
        filter_label: "Filter",
        has_filter: false,
        browsable: false,
    },
    Addon {
        id: "local-game",
        label: "Local Games",
        kind: "game",
        filter_label: "Filter",
        has_filter: false,
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
        .route("/tmdb-tv/{id}/seasons", get(tmdb_tv_seasons))
}

#[derive(Clone, Serialize)]
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
}

#[derive(Serialize)]
pub(crate) struct CatalogPage {
    pub items: Vec<CatalogItem>,
    pub page: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
}

#[derive(Serialize)]
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
#[derive(Serialize)]
struct CatalogSeason {
    #[serde(rename = "seasonNumber")]
    season_number: i64,
    name: String,
    #[serde(rename = "airYear", skip_serializing_if = "Option::is_none")]
    air_year: Option<i32>,
    #[serde(rename = "episodeCount", skip_serializing_if = "Option::is_none")]
    episode_count: Option<i64>,
}

/// Three-field "person/group attached to a media item" record matching the
/// persisted `artist` doc shape. Each addon's handler maps its upstream
/// cast, crew, authors, developers, channels, etc. into this shape; the
/// frontend hands the array verbatim to `POST /api/firkins`, which
/// upserts each entry into the `artist` table and stores the resulting
/// CIDs on the firkin.
#[derive(Clone, Serialize)]
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
#[derive(Serialize)]
struct CatalogTrailer {
    #[serde(rename = "youtubeUrl")]
    youtube_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

/// Combined metadata payload for a single catalog item. Returned by
/// `GET /api/catalog/{addon}/{id}/metadata`. Lets the frontend pull
/// artists and trailers in one call so each upstream provider can
/// satisfy them in a single request (TMDB's `append_to_response`
/// merges credits and videos into one HTTP call).
#[derive(Serialize)]
struct CatalogMetadata {
    artists: Vec<CatalogArtist>,
    trailers: Vec<CatalogTrailer>,
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
}

#[derive(Debug, Deserialize)]
pub struct GenresQuery {}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn popular(
    State(_state): State<CloudState>,
    Path(addon): Path<String>,
    Query(q): Query<PopularQuery>,
) -> Result<Json<CatalogPage>, (StatusCode, Json<serde_json::Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    match addon.as_str() {
        "tmdb-movie" => tmdb_popular(false, q.filter.as_deref(), page).await,
        "tmdb-tv" => tmdb_popular(true, q.filter.as_deref(), page).await,
        "musicbrainz" => musicbrainz_popular(q.filter.as_deref(), page).await,
        "youtube-video" => youtube_popular(false, q.filter.as_deref(), page).await,
        "youtube-channel" => youtube_popular(true, q.filter.as_deref(), page).await,
        "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(empty_page(page)),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
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
        "musicbrainz" => musicbrainz_search(trimmed, page).await,
        "youtube-video" => youtube_search(false, trimmed, page).await,
        "youtube-channel" => youtube_search(true, trimmed, page).await,
        "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(empty_page(page)),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
}

async fn genres(
    State(_state): State<CloudState>,
    Path(addon): Path<String>,
    Query(_q): Query<GenresQuery>,
) -> Result<Json<Vec<CatalogGenre>>, (StatusCode, Json<serde_json::Value>)> {
    match addon.as_str() {
        "tmdb-movie" => tmdb_genres(false).await,
        "tmdb-tv" => tmdb_genres(true).await,
        "musicbrainz" => Ok(static_music_genres()),
        "youtube-video" | "youtube-channel" => Ok(static_youtube_regions()),
        "lrclib" | "wyzie-subs-movie" | "wyzie-subs-tv" => Ok(Vec::new()),
        _ => Err(err(
            StatusCode::NOT_FOUND,
            format!("addon \"{addon}\" is not supported"),
        )),
    }
    .map(Json)
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
    let (artists, trailers) = match addon.as_str() {
        "musicbrainz" => (musicbrainz_artists(&id).await?, Vec::new()),
        "tmdb-movie" => tmdb_metadata(false, &id).await?,
        "tmdb-tv" => tmdb_metadata(true, &id).await?,
        "youtube-video" => (youtube_video_artists(&id).await?, Vec::new()),
        "youtube-channel" => (youtube_channel_artists(&id).await?, Vec::new()),
        _ => (Vec::new(), Vec::new()),
    };
    Ok(Json(CatalogMetadata { artists, trailers }))
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
    let items = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(tmdb_to_item).collect())
        .unwrap_or_default();
    Ok(CatalogPage {
        items,
        page,
        total_pages,
    })
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
    CatalogItem {
        id,
        title,
        year,
        description,
        poster_url,
        backdrop_url,
        artists: Vec::new(),
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
/// `/metadata` endpoint is hit.
async fn tmdb_metadata(
    is_tv: bool,
    id: &str,
) -> Result<(Vec<CatalogArtist>, Vec<CatalogTrailer>), (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }
    let kind = if is_tv { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}?api_key={}&append_to_response=credits,videos",
        TMDB_BASE,
        kind,
        urlencoding(id),
        api_key
    );
    let payload: serde_json::Value = http_get_json(&url, &[("Accept", "application/json")]).await?;

    let credits = payload.get("credits").cloned().unwrap_or(serde_json::Value::Null);
    let artists = parse_tmdb_credits(&credits);
    let videos = payload.get("videos").cloned().unwrap_or(serde_json::Value::Null);
    let trailers = parse_tmdb_videos(&videos);
    Ok((artists, trailers))
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
            out.push(CatalogSeason {
                season_number,
                name,
                air_year,
                episode_count,
            });
        }
    }
    out.sort_by_key(|s| s.season_number);
    Ok(Json(out))
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

pub(crate) async fn musicbrainz_search(
    query: &str,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let limit: i64 = 20;
    let offset = (page - 1).max(0) * limit;
    // Escape Lucene query special chars in the user's query, then OR across
    // the release-group title and the artist name fields so a query like
    // "keane" matches both albums titled Keane and every release-group whose
    // artist credit is Keane. MusicBrainz's `query=` param accepts Lucene
    // syntax; raw quotes/colons in the user input would break the parse.
    let escaped: String = query
        .chars()
        .flat_map(|c| match c {
            '\\' | '+' | '-' | '!' | '(' | ')' | '{' | '}' | '[' | ']' | '^' | '"' | '~' | '*'
            | '?' | ':' | '/' => vec!['\\', c],
            _ => vec![c],
        })
        .collect();
    let lucene = format!("(releasegroup:\"{q}\" OR artist:\"{q}\")", q = escaped);
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

async fn musicbrainz_artists(
    release_group_id: &str,
) -> Result<Vec<CatalogArtist>, (StatusCode, Json<serde_json::Value>)> {
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

    let mut out: Vec<CatalogArtist> = Vec::new();
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
        out.push(CatalogArtist {
            name,
            role: Some("Artist".to_string()),
            image_url: None,
        });
    }
    Ok(out)
}

/// Other release-groups by the same primary artist as the given
/// release-group. Resolves the artist via `inc=artist-credits` on the
/// release-group, then browses release-groups by that artist id.
/// The current release-group is filtered out of the result.
async fn musicbrainz_related(
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
    }
}

// ---------- YouTube ----------

const YOUTUBE_REGIONS: &[(&str, &str)] = &[
    ("US", "United States"),
    ("GB", "United Kingdom"),
    ("CA", "Canada"),
    ("AU", "Australia"),
    ("DE", "Germany"),
    ("FR", "France"),
    ("ES", "Spain"),
    ("IT", "Italy"),
    ("BR", "Brazil"),
    ("MX", "Mexico"),
    ("JP", "Japan"),
    ("KR", "South Korea"),
    ("IN", "India"),
];

fn static_youtube_regions() -> Vec<CatalogGenre> {
    YOUTUBE_REGIONS
        .iter()
        .map(|(id, name)| CatalogGenre {
            id: (*id).to_string(),
            name: (*name).to_string(),
        })
        .collect()
}

async fn youtube_popular(
    want_channel: bool,
    region: Option<&str>,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let region = region
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .unwrap_or_else(|| "US".to_string());
    let url = format!(
        "{}/trending?region={}",
        PIPED_BASE,
        urlencoding(&region)
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload.as_array().cloned().unwrap_or_default();
    let limit: usize = 24;
    let total_pages = ((arr.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let items: Vec<CatalogItem> = arr
        .iter()
        .skip(offset)
        .take(limit)
        .map(|item| youtube_to_item(item, want_channel))
        .collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

async fn youtube_search(
    want_channel: bool,
    query: &str,
    page: i64,
) -> Result<CatalogPage, (StatusCode, Json<serde_json::Value>)> {
    let filter = if want_channel { "channels" } else { "videos" };
    let url = format!(
        "{}/search?q={}&filter={}",
        PIPED_BASE,
        urlencoding(query),
        filter
    );
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let arr = payload
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let limit: usize = 24;
    let total_pages = ((arr.len() as f64) / (limit as f64)).ceil() as i64;
    let offset = ((page - 1).max(0) as usize) * limit;
    let items: Vec<CatalogItem> = arr
        .iter()
        .skip(offset)
        .take(limit)
        .map(|item| youtube_search_to_item(item, want_channel))
        .collect();
    Ok(CatalogPage {
        items,
        page,
        total_pages: total_pages.max(1),
    })
}

fn youtube_search_to_item(item: &serde_json::Value, want_channel: bool) -> CatalogItem {
    // Piped's `/search` items use slightly different keys than `/trending`:
    // - videos: `url` (e.g. "/watch?v=ID"), `title`, `uploaderName`, `thumbnail`, `views`
    // - channels: `url` (e.g. "/channel/ID"), `name`, `description`, `thumbnail`
    if want_channel {
        let raw_url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let id = raw_url
            .trim_start_matches('/')
            .trim_start_matches("channel/")
            .to_string();
        let title = item
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("uploaderName").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();
        let description = item
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let poster_url = item
            .get("thumbnail")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("uploaderAvatar").and_then(|v| v.as_str()))
            .map(|s| s.to_string());
        CatalogItem {
            id,
            title,
            year: None,
            description,
            poster_url,
            backdrop_url: None,
            artists: Vec::new(),
        }
    } else {
        youtube_to_item(item, false)
    }
}

fn youtube_to_item(item: &serde_json::Value, want_channel: bool) -> CatalogItem {
    let raw_id = if want_channel {
        item.get("uploaderUrl").and_then(|v| v.as_str()).map(|s| {
            s.trim_start_matches('/')
                .trim_start_matches("channel/")
                .to_string()
        })
    } else {
        item.get("url").and_then(|v| v.as_str()).map(|s| {
            s.split_once("v=")
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_else(|| s.trim_start_matches('/').to_string())
        })
    };
    let id = raw_id.unwrap_or_default();
    let title = if want_channel {
        item.get("uploaderName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        item.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let description = if want_channel {
        item.get("uploaderDescription")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        let uploader = item
            .get("uploaderName")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let views = item.get("views").and_then(|v| v.as_i64());
        let mut parts: Vec<String> = Vec::new();
        if !uploader.is_empty() {
            parts.push(uploader.to_string());
        }
        if let Some(v) = views {
            parts.push(format!("{} views", v));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" · "))
        }
    };
    let poster_url = if want_channel {
        item.get("uploaderAvatar")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        item.get("thumbnail")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    CatalogItem {
        id,
        title,
        year: None,
        description,
        poster_url,
        backdrop_url: None,
        artists: Vec::new(),
    }
}

async fn youtube_video_artists(
    video_id: &str,
) -> Result<Vec<CatalogArtist>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!("{}/streams/{}", PIPED_BASE, urlencoding(video_id));
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let name = payload
        .get("uploader")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Ok(Vec::new());
    }
    let image_url = payload
        .get("uploaderAvatar")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(vec![CatalogArtist {
        name,
        role: Some("Channel".to_string()),
        image_url,
    }])
}

async fn youtube_channel_artists(
    channel_id: &str,
) -> Result<Vec<CatalogArtist>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!("{}/channel/{}", PIPED_BASE, urlencoding(channel_id));
    let payload: serde_json::Value = http_get_json(
        &url,
        &[("Accept", "application/json"), ("User-Agent", USER_AGENT)],
    )
    .await?;
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Ok(Vec::new());
    }
    let image_url = payload
        .get("avatarUrl")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(vec![CatalogArtist {
        name,
        role: Some("Channel".to_string()),
        image_url,
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

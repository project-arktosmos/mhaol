use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;

const CACHE_TTL_SECS: u64 = 3600; // 1 hour

const CHANNELS_URL: &str = "https://iptv-org.github.io/api/channels.json";
const STREAMS_URL: &str = "https://iptv-org.github.io/api/streams.json";
const CATEGORIES_URL: &str = "https://iptv-org.github.io/api/categories.json";
const COUNTRIES_URL: &str = "https://iptv-org.github.io/api/countries.json";
const LANGUAGES_URL: &str = "https://iptv-org.github.io/api/languages.json";
const LOGOS_URL: &str = "https://iptv-org.github.io/api/logos.json";

// ---------- cached data types (match iptv-org API) ----------

#[derive(Clone, Deserialize)]
struct ApiChannel {
    id: String,
    name: String,
    country: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    is_nsfw: bool,
    website: Option<String>,
}

#[derive(Clone, Deserialize)]
struct ApiStream {
    channel: Option<String>,
    url: String,
    referrer: Option<String>,
    user_agent: Option<String>,
}

#[derive(Clone, Deserialize)]
struct ApiLogo {
    channel: String,
    url: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Category {
    id: String,
    name: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Country {
    code: String,
    name: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Language {
    code: String,
    name: String,
}

// ---------- merged cache types ----------

#[derive(Clone)]
struct CachedChannel {
    id: String,
    name: String,
    country: String,
    categories: Vec<String>,
    is_nsfw: bool,
    website: Option<String>,
    logo: Option<String>,
}

#[derive(Clone)]
struct CachedStream {
    channel: String,
    url: String,
    referrer: Option<String>,
    user_agent: Option<String>,
}

struct IptvCache {
    channels: Vec<CachedChannel>,
    streams: Vec<CachedStream>,
    categories: Vec<Category>,
    countries: Vec<Country>,
    languages: Vec<Language>,
    fetched_at: Instant,
}

static CACHE: OnceLock<RwLock<Option<IptvCache>>> = OnceLock::new();

fn cache_lock() -> &'static RwLock<Option<IptvCache>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

async fn ensure_cache() -> Result<(), String> {
    {
        let guard = cache_lock().read();
        if let Some(ref c) = *guard {
            if c.fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
                return Ok(());
            }
        }
    }
    refresh_cache().await
}

async fn refresh_cache() -> Result<(), String> {
    let client = reqwest::Client::new();

    let (ch_res, st_res, cat_res, co_res, lang_res, logo_res) = tokio::join!(
        client.get(CHANNELS_URL).send(),
        client.get(STREAMS_URL).send(),
        client.get(CATEGORIES_URL).send(),
        client.get(COUNTRIES_URL).send(),
        client.get(LANGUAGES_URL).send(),
        client.get(LOGOS_URL).send(),
    );

    let api_channels: Vec<ApiChannel> = ch_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let api_streams: Vec<ApiStream> = st_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let categories: Vec<Category> = cat_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let countries: Vec<Country> = co_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let languages: Vec<Language> = lang_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let api_logos: Vec<ApiLogo> = logo_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // Build logo lookup (first logo per channel)
    let mut logo_map = std::collections::HashMap::new();
    for logo in api_logos {
        logo_map.entry(logo.channel).or_insert(logo.url);
    }

    let channels: Vec<CachedChannel> = api_channels
        .into_iter()
        .map(|ch| {
            let logo = logo_map.get(&ch.id).cloned();
            CachedChannel {
                id: ch.id,
                name: ch.name,
                country: ch.country,
                categories: ch.categories,
                is_nsfw: ch.is_nsfw,
                website: ch.website,
                logo,
            }
        })
        .collect();

    // Only keep streams that have a channel assigned
    let streams: Vec<CachedStream> = api_streams
        .into_iter()
        .filter_map(|s| {
            s.channel.map(|ch| CachedStream {
                channel: ch,
                url: s.url,
                referrer: s.referrer,
                user_agent: s.user_agent,
            })
        })
        .collect();

    let mut guard = cache_lock().write();
    *guard = Some(IptvCache {
        channels,
        streams,
        categories,
        countries,
        languages,
        fetched_at: Instant::now(),
    });

    Ok(())
}

// ---------- API response types ----------

#[derive(Serialize)]
struct ChannelResponse {
    id: String,
    name: String,
    country: String,
    categories: Vec<String>,
    logo: Option<String>,
    website: Option<String>,
    #[serde(rename = "isNsfw")]
    is_nsfw: bool,
}

#[derive(Serialize)]
struct StreamResponse {
    channel: String,
    url: String,
    #[serde(rename = "httpReferrer")]
    referrer: Option<String>,
    #[serde(rename = "userAgent")]
    user_agent: Option<String>,
}

#[derive(Serialize)]
struct ChannelDetailResponse {
    channel: ChannelResponse,
    streams: Vec<StreamResponse>,
}

#[derive(Serialize)]
struct ChannelListResponse {
    channels: Vec<ChannelResponse>,
    total: usize,
    page: usize,
    limit: usize,
}

impl From<&CachedChannel> for ChannelResponse {
    fn from(ch: &CachedChannel) -> Self {
        Self {
            id: ch.id.clone(),
            name: ch.name.clone(),
            country: ch.country.clone(),
            categories: ch.categories.clone(),
            logo: ch.logo.clone(),
            website: ch.website.clone(),
            is_nsfw: ch.is_nsfw,
        }
    }
}

impl From<&CachedStream> for StreamResponse {
    fn from(s: &CachedStream) -> Self {
        Self {
            channel: s.channel.clone(),
            url: s.url.clone(),
            referrer: s.referrer.clone(),
            user_agent: s.user_agent.clone(),
        }
    }
}

// ---------- router ----------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channels", get(list_channels))
        .route("/channels/{id}", get(get_channel))
        .route("/categories", get(list_categories))
        .route("/countries", get(list_countries))
        .route("/languages", get(list_languages))
        .route("/stream/{channel_id}", get(proxy_stream))
}

// ---------- handlers ----------

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    category: Option<String>,
    country: Option<String>,
    language: Option<String>,
    page: Option<usize>,
    limit: Option<usize>,
}

async fn list_channels(
    State(_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guard = cache_lock().read();
    let cache = guard.as_ref().unwrap();

    let q_lower = query.q.as_deref().unwrap_or("").to_lowercase();
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).min(200);

    let filtered: Vec<&CachedChannel> = cache
        .channels
        .iter()
        .filter(|ch| {
            if !q_lower.is_empty() && !ch.name.to_lowercase().contains(&q_lower) {
                return false;
            }
            if let Some(ref cat) = query.category {
                if !cat.is_empty()
                    && !ch
                        .categories
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(cat))
                {
                    return false;
                }
            }
            if let Some(ref co) = query.country {
                if !co.is_empty() && !ch.country.eq_ignore_ascii_case(co) {
                    return false;
                }
            }
            true
        })
        .collect();

    let total = filtered.len();
    let start = (page - 1) * limit;
    let channels: Vec<ChannelResponse> = filtered
        .into_iter()
        .skip(start)
        .take(limit)
        .map(ChannelResponse::from)
        .collect();

    Json(ChannelListResponse {
        channels,
        total,
        page,
        limit,
    })
    .into_response()
}

async fn get_channel(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guard = cache_lock().read();
    let cache = guard.as_ref().unwrap();

    let channel = cache.channels.iter().find(|ch| ch.id == id);
    match channel {
        Some(ch) => {
            let streams: Vec<StreamResponse> = cache
                .streams
                .iter()
                .filter(|s| s.channel == id)
                .map(StreamResponse::from)
                .collect();
            Json(ChannelDetailResponse {
                channel: ChannelResponse::from(ch),
                streams,
            })
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Channel not found" })),
        )
            .into_response(),
    }
}

async fn list_categories(State(_state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guard = cache_lock().read();
    let cache = guard.as_ref().unwrap();
    Json(cache.categories.clone()).into_response()
}

async fn list_countries(State(_state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guard = cache_lock().read();
    let cache = guard.as_ref().unwrap();
    Json(cache.countries.clone()).into_response()
}

async fn list_languages(State(_state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guard = cache_lock().read();
    let cache = guard.as_ref().unwrap();
    Json(cache.languages.clone()).into_response()
}

async fn proxy_stream(
    State(_state): State<AppState>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let stream_info = {
        let guard = cache_lock().read();
        let cache = guard.as_ref().unwrap();
        cache
            .streams
            .iter()
            .find(|s| s.channel == channel_id)
            .cloned()
    };

    let stream = match stream_info {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "No stream found for this channel" })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let mut req = client.get(&stream.url);

    if let Some(ref referrer) = stream.referrer {
        req = req.header("Referer", referrer);
    }
    if let Some(ref ua) = stream.user_agent {
        req = req.header("User-Agent", ua);
    } else {
        req = req.header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        );
    }

    match req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();

            let body = Body::from_stream(resp.bytes_stream());

            (
                status,
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        "*".to_string(),
                    ),
                ],
                body,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Failed to connect to stream: {e}") })),
        )
            .into_response(),
    }
}

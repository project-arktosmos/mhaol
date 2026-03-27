use crate::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const CACHE_TTL_SECS: u64 = 3600; // 1 hour

const CHANNELS_URL: &str = "https://iptv-org.github.io/api/channels.json";
const STREAMS_URL: &str = "https://iptv-org.github.io/api/streams.json";
const CATEGORIES_URL: &str = "https://iptv-org.github.io/api/categories.json";
const COUNTRIES_URL: &str = "https://iptv-org.github.io/api/countries.json";
const LANGUAGES_URL: &str = "https://iptv-org.github.io/api/languages.json";
const LOGOS_URL: &str = "https://iptv-org.github.io/api/logos.json";
const GUIDES_URL: &str = "https://iptv-org.github.io/api/guides.json";
const MJH_BASE: &str = "https://raw.githubusercontent.com/matthuisman/i.mjh.nz/master";

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

#[derive(Clone, Deserialize)]
struct ApiGuide {
    channel: Option<String>,
    site: String,
    site_id: String,
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

#[derive(Clone)]
enum GuideSource {
    Mjh {
        xml_path: String,
        xml_channel_id: String,
    },
    Tvtv {
        site_id: String,
    },
}

struct IptvCache {
    channels: Vec<CachedChannel>,
    streams: Vec<CachedStream>,
    categories: Vec<Category>,
    countries: Vec<Country>,
    languages: Vec<Language>,
    guides: std::collections::HashMap<String, GuideSource>,
    fetched_at: Instant,
}

// ---------- EPG cache ----------

#[derive(Clone, Serialize)]
struct EpgProgram {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episode: Option<String>,
    start: String,
    stop: String,
}

struct EpgCacheEntry {
    /// xml_channel_id -> programs (sorted by start)
    programs: std::collections::HashMap<String, Vec<EpgProgram>>,
    fetched_at: Instant,
}

const EPG_CACHE_TTL_SECS: u64 = 3600;

static EPG_CACHE: OnceLock<RwLock<std::collections::HashMap<String, EpgCacheEntry>>> =
    OnceLock::new();

fn epg_cache_lock() -> &'static RwLock<std::collections::HashMap<String, EpgCacheEntry>> {
    EPG_CACHE.get_or_init(|| RwLock::new(std::collections::HashMap::new()))
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

    let (ch_res, st_res, cat_res, co_res, lang_res, logo_res, guides_res) = tokio::join!(
        client.get(CHANNELS_URL).send(),
        client.get(STREAMS_URL).send(),
        client.get(CATEGORIES_URL).send(),
        client.get(COUNTRIES_URL).send(),
        client.get(LANGUAGES_URL).send(),
        client.get(LOGOS_URL).send(),
        client.get(GUIDES_URL).send(),
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

    // Only keep channels that have at least one stream
    let channels_with_streams: std::collections::HashSet<&str> =
        streams.iter().map(|s| s.channel.as_str()).collect();
    let channels: Vec<CachedChannel> = channels
        .into_iter()
        .filter(|ch| channels_with_streams.contains(ch.id.as_str()))
        .collect();

    // Build guide lookup from supported EPG sources
    let api_guides: Vec<ApiGuide> = guides_res
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();
    let mut guides: std::collections::HashMap<String, GuideSource> =
        std::collections::HashMap::new();
    // First pass: tvtv.us (broader coverage)
    for g in &api_guides {
        if let Some(ref ch) = g.channel {
            if g.site == "tvtv.us" {
                guides.insert(
                    ch.clone(),
                    GuideSource::Tvtv {
                        site_id: g.site_id.clone(),
                    },
                );
            }
        }
    }
    // Second pass: i.mjh.nz (overwrites tvtv.us if both exist — XMLTV is richer)
    for g in &api_guides {
        if let Some(ref ch) = g.channel {
            if g.site == "i.mjh.nz" {
                let parts: Vec<&str> = g.site_id.splitn(2, '#').collect();
                if parts.len() == 2 {
                    guides.insert(
                        ch.clone(),
                        GuideSource::Mjh {
                            xml_path: parts[0].to_string(),
                            xml_channel_id: parts[1].to_string(),
                        },
                    );
                }
            }
        }
    }

    let mut guard = cache_lock().write();
    *guard = Some(IptvCache {
        channels,
        streams,
        categories,
        countries,
        languages,
        guides,
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
    #[serde(rename = "hasEpg")]
    has_epg: bool,
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

impl ChannelResponse {
    fn from_cached(
        ch: &CachedChannel,
        guides: &std::collections::HashMap<String, GuideSource>,
    ) -> Self {
        Self {
            id: ch.id.clone(),
            name: ch.name.clone(),
            country: ch.country.clone(),
            categories: ch.categories.clone(),
            logo: ch.logo.clone(),
            website: ch.website.clone(),
            is_nsfw: ch.is_nsfw,
            has_epg: guides.contains_key(&ch.id),
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
        .route("/channel", get(get_channel))
        .route("/categories", get(list_categories))
        .route("/countries", get(list_countries))
        .route("/languages", get(list_languages))
        .route("/stream", get(proxy_stream))
        .route("/proxy", get(proxy_url))
        .route("/epg", get(get_epg))
}

// ---------- handlers ----------

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    category: Option<String>,
    country: Option<String>,
    #[serde(rename = "hasEpg")]
    has_epg: Option<bool>,
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
                if !cat.is_empty() && !ch.categories.iter().any(|c| c.eq_ignore_ascii_case(cat)) {
                    return false;
                }
            }
            if let Some(ref co) = query.country {
                if !co.is_empty() && !ch.country.eq_ignore_ascii_case(co) {
                    return false;
                }
            }
            if query.has_epg == Some(true) && !cache.guides.contains_key(&ch.id) {
                return false;
            }
            true
        })
        .collect();

    let total = filtered.len();
    let start = (page - 1) * limit;
    let guides = &cache.guides;
    let channels: Vec<ChannelResponse> = filtered
        .into_iter()
        .skip(start)
        .take(limit)
        .map(|ch| ChannelResponse::from_cached(ch, guides))
        .collect();

    Json(ChannelListResponse {
        channels,
        total,
        page,
        limit,
    })
    .into_response()
}

#[derive(Deserialize)]
struct ChannelQuery {
    id: String,
}

async fn get_channel(
    State(_state): State<AppState>,
    Query(query): Query<ChannelQuery>,
) -> impl IntoResponse {
    let id = query.id;
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
                channel: ChannelResponse::from_cached(ch, &cache.guides),
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

#[derive(Deserialize)]
struct StreamQuery {
    id: String,
}

async fn proxy_stream(
    State(_state): State<AppState>,
    Query(query): Query<StreamQuery>,
) -> impl IntoResponse {
    let channel_id = query.id;
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
            let final_url = resp.url().to_string();
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let is_manifest = content_type.contains("mpegurl")
                || content_type.contains("x-mpegURL")
                || final_url.ends_with(".m3u8")
                || final_url.ends_with(".m3u");

            let cors_headers = [(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".to_string())];

            if is_manifest {
                // Rewrite URLs in the manifest to route through the proxy
                let base_url = final_url.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
                let text = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        return (
                            StatusCode::BAD_GATEWAY,
                            Json(serde_json::json!({ "error": e.to_string() })),
                        )
                            .into_response()
                    }
                };
                let rewritten = rewrite_manifest_urls(&text, base_url);
                (
                    status,
                    [
                        (header::CONTENT_TYPE, content_type),
                        (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".to_string()),
                    ],
                    rewritten,
                )
                    .into_response()
            } else {
                let body = Body::from_stream(resp.bytes_stream());
                (
                    status,
                    cors_headers,
                    [(header::CONTENT_TYPE, content_type)],
                    body,
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Failed to connect to stream: {e}") })),
        )
            .into_response(),
    }
}

// ---------- proxy URL token map ----------
// Maps short numeric tokens to full URLs to avoid URI length limits.

static PROXY_TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);
static PROXY_TOKEN_MAP: OnceLock<RwLock<std::collections::HashMap<String, String>>> =
    OnceLock::new();

fn proxy_token_map() -> &'static RwLock<std::collections::HashMap<String, String>> {
    PROXY_TOKEN_MAP.get_or_init(|| RwLock::new(std::collections::HashMap::new()))
}

fn store_proxy_url(url: &str) -> String {
    let token = PROXY_TOKEN_COUNTER
        .fetch_add(1, Ordering::Relaxed)
        .to_string();
    proxy_token_map()
        .write()
        .insert(token.clone(), url.to_string());
    token
}

fn resolve_proxy_token(token: &str) -> Option<String> {
    proxy_token_map().read().get(token).cloned()
}

/// Resolve a URL to absolute, store it, and return a short proxy URL with a token.
fn to_proxy_url(url: &str, base_url: &str) -> String {
    let absolute = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("{}/{}", base_url, url)
    };
    let token = store_proxy_url(&absolute);
    format!("/api/iptv/proxy?t={}", token)
}

/// Rewrite all URLs in an HLS manifest to go through the proxy endpoint.
fn rewrite_manifest_urls(manifest: &str, base_url: &str) -> String {
    manifest
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                if trimmed.contains("URI=\"") {
                    rewrite_uri_attribute(line, base_url)
                } else {
                    line.to_string()
                }
            } else {
                to_proxy_url(trimmed, base_url)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn rewrite_uri_attribute(line: &str, base_url: &str) -> String {
    let mut result = line.to_string();
    if let Some(start) = result.find("URI=\"") {
        let uri_start = start + 5;
        if let Some(end) = result[uri_start..].find('"') {
            let uri = &result[uri_start..uri_start + end];
            let proxied = to_proxy_url(uri, base_url);
            result = format!(
                "{}URI=\"{}\"{}",
                &line[..start],
                proxied,
                &line[uri_start + end + 1..]
            );
        }
    }
    result
}

// ---------- generic URL proxy ----------

#[derive(Deserialize)]
struct ProxyQuery {
    t: String,
}

async fn proxy_url(
    State(_state): State<AppState>,
    Query(query): Query<ProxyQuery>,
) -> impl IntoResponse {
    let url = match resolve_proxy_token(&query.t) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Unknown proxy token" })),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let resp = match client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Proxy fetch failed: {e}") })),
            )
                .into_response()
        }
    };

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
    let final_url = resp.url().to_string();
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let is_manifest = content_type.contains("mpegurl")
        || content_type.contains("x-mpegURL")
        || final_url.ends_with(".m3u8")
        || final_url.ends_with(".m3u");

    if is_manifest {
        let base_url = final_url.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        };
        let rewritten = rewrite_manifest_urls(&text, base_url);
        (
            status,
            [
                (header::CONTENT_TYPE, content_type),
                (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".to_string()),
            ],
            rewritten,
        )
            .into_response()
    } else {
        let body = Body::from_stream(resp.bytes_stream());
        (
            status,
            [(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".to_string())],
            [(header::CONTENT_TYPE, content_type)],
            body,
        )
            .into_response()
    }
}

// ---------- EPG ----------

#[derive(Deserialize)]
struct EpgQuery {
    id: String,
}

#[derive(Serialize)]
struct EpgResponse {
    available: bool,
    programs: Vec<EpgProgram>,
}

async fn get_epg(
    State(_state): State<AppState>,
    Query(query): Query<EpgQuery>,
) -> impl IntoResponse {
    if let Err(e) = ensure_cache().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to fetch IPTV data: {e}") })),
        )
            .into_response();
    }

    let guide_source = {
        let guard = cache_lock().read();
        let cache = guard.as_ref().unwrap();
        cache.guides.get(&query.id).cloned()
    };

    let source = match guide_source {
        Some(s) => s,
        None => {
            return Json(EpgResponse {
                available: false,
                programs: vec![],
            })
            .into_response()
        }
    };

    match source {
        GuideSource::Mjh {
            xml_path,
            xml_channel_id,
        } => fetch_mjh_epg(&xml_path, &xml_channel_id)
            .await
            .into_response(),
        GuideSource::Tvtv { site_id } => fetch_tvtv_epg(&site_id).await.into_response(),
    }
}

async fn fetch_mjh_epg(xml_path: &str, xml_channel_id: &str) -> Json<EpgResponse> {
    let cache_key = format!("mjh:{}", xml_path);

    // Check cache
    {
        let epg_guard = epg_cache_lock().read();
        if let Some(entry) = epg_guard.get(&cache_key) {
            if entry.fetched_at.elapsed().as_secs() < EPG_CACHE_TTL_SECS {
                let programs = entry
                    .programs
                    .get(xml_channel_id)
                    .cloned()
                    .unwrap_or_default();
                return Json(EpgResponse {
                    available: true,
                    programs: filter_current_programs(&programs),
                });
            }
        }
    }

    let url = format!("{}/{}.xml", MJH_BASE, xml_path);
    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => {
            return Json(EpgResponse {
                available: false,
                programs: vec![],
            })
        }
    };

    let xml_text = match resp.text().await {
        Ok(t) => t,
        Err(_) => {
            return Json(EpgResponse {
                available: false,
                programs: vec![],
            })
        }
    };

    let all_programs = parse_xmltv(&xml_text);
    let result = all_programs
        .get(xml_channel_id)
        .cloned()
        .unwrap_or_default();

    {
        let mut epg_guard = epg_cache_lock().write();
        epg_guard.insert(
            cache_key,
            EpgCacheEntry {
                programs: all_programs,
                fetched_at: Instant::now(),
            },
        );
    }

    Json(EpgResponse {
        available: true,
        programs: filter_current_programs(&result),
    })
}

const TVTV_LINEUP: &str = "USA-NY71652-X";
const TVTV_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

async fn fetch_tvtv_epg(site_id: &str) -> Json<EpgResponse> {
    let cache_key = format!("tvtv:{}", site_id);

    // Check cache
    {
        let epg_guard = epg_cache_lock().read();
        if let Some(entry) = epg_guard.get(&cache_key) {
            if entry.fetched_at.elapsed().as_secs() < EPG_CACHE_TTL_SECS {
                let programs = entry.programs.get(site_id).cloned().unwrap_or_default();
                return Json(EpgResponse {
                    available: true,
                    programs: filter_current_programs(&programs),
                });
            }
        }
    }

    let now = chrono::Utc::now();
    let start = now.format("%Y-%m-%dT00:00:00.000Z").to_string();
    let end = (now + chrono::Duration::days(1))
        .format("%Y-%m-%dT00:00:00.000Z")
        .to_string();

    let url = format!(
        "https://www.tvtv.us/api/v1/lineup/{}/grid/{}/{}/{}",
        TVTV_LINEUP, start, end, site_id
    );

    let client = reqwest::Client::new();
    let resp = match client.get(&url).header("User-Agent", TVTV_UA).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => {
            return Json(EpgResponse {
                available: false,
                programs: vec![],
            })
        }
    };

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return Json(EpgResponse {
                available: false,
                programs: vec![],
            })
        }
    };

    let programs = parse_tvtv_response(&body);

    let mut program_map = std::collections::HashMap::new();
    program_map.insert(site_id.to_string(), programs.clone());

    {
        let mut epg_guard = epg_cache_lock().write();
        epg_guard.insert(
            cache_key,
            EpgCacheEntry {
                programs: program_map,
                fetched_at: Instant::now(),
            },
        );
    }

    Json(EpgResponse {
        available: true,
        programs: filter_current_programs(&programs),
    })
}

fn parse_tvtv_response(body: &serde_json::Value) -> Vec<EpgProgram> {
    let mut programs = Vec::new();

    // Response is [[{title, subtitle, startTime, duration, ...}]]
    let outer = match body.as_array() {
        Some(a) => a,
        None => return programs,
    };

    for inner in outer {
        let items = match inner.as_array() {
            Some(a) => a,
            None => continue,
        };
        for item in items {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if title.is_empty() {
                continue;
            }

            let subtitle = item
                .get("subtitle")
                .and_then(|v| v.as_str())
                .map(String::from);
            let start_time = item.get("startTime").and_then(|v| v.as_str()).unwrap_or("");
            let duration_mins = item.get("duration").and_then(|v| v.as_i64()).unwrap_or(60);

            // Convert ISO startTime to XMLTV format for consistency
            let start = iso_to_xmltv(start_time);
            let stop = if let Ok(dt) = parse_iso_datetime(start_time) {
                let end = dt + chrono::Duration::minutes(duration_mins);
                end.format("%Y%m%d%H%M%S +0000").to_string()
            } else {
                String::new()
            };

            if start.is_empty() || stop.is_empty() {
                continue;
            }

            programs.push(EpgProgram {
                title,
                description: subtitle,
                episode: None,
                start,
                stop,
            });
        }
    }

    programs
}

fn parse_iso_datetime(
    iso: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, chrono::ParseError> {
    // Normalize: tvtv.us sends "2026-03-25T00:00Z" (no seconds)
    let normalized = if iso.ends_with('Z') {
        let base = &iso[..iso.len() - 1];
        let with_secs = if base.len() == 16 {
            format!("{}:00", base)
        } else {
            base.to_string()
        };
        format!("{}+00:00", with_secs)
    } else {
        iso.to_string()
    };
    chrono::DateTime::parse_from_rfc3339(&normalized)
}

fn iso_to_xmltv(iso: &str) -> String {
    match parse_iso_datetime(iso) {
        Ok(dt) => dt.format("%Y%m%d%H%M%S +0000").to_string(),
        Err(_) => String::new(),
    }
}

/// Filter programs to show current + next few upcoming
fn filter_current_programs(programs: &[EpgProgram]) -> Vec<EpgProgram> {
    let now = chrono::Utc::now();
    let now_str = now.format("%Y%m%d%H%M%S +0000").to_string();

    // Find programs that are current or upcoming
    let mut result = Vec::new();
    for p in programs {
        if p.stop > now_str {
            result.push(p.clone());
            if result.len() >= 10 {
                break;
            }
        }
    }
    result
}

/// Parse XMLTV format into a map of channel_id -> programs
fn parse_xmltv(xml: &str) -> std::collections::HashMap<String, Vec<EpgProgram>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut programs: std::collections::HashMap<String, Vec<EpgProgram>> =
        std::collections::HashMap::new();
    let mut reader = Reader::from_str(xml);

    let mut in_programme = false;
    let mut current_channel = String::new();
    let mut current_start = String::new();
    let mut current_stop = String::new();
    let mut current_title = String::new();
    let mut current_desc: Option<String> = None;
    let mut current_episode: Option<String> = None;
    let mut current_tag = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "programme" {
                    in_programme = true;
                    current_title.clear();
                    current_desc = None;
                    current_episode = None;
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"channel" => {
                                current_channel = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"start" => {
                                current_start = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"stop" => {
                                current_stop = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            _ => {}
                        }
                    }
                } else if in_programme {
                    current_tag = name.clone();
                    if name == "episode-num" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"system" && &*attr.value == b"onscreen" {
                                current_tag = "episode-onscreen".to_string();
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_programme {
                    let text = e.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "title" => current_title = text,
                        "desc" => current_desc = Some(text),
                        "episode-onscreen" => current_episode = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "programme" && in_programme {
                    if !current_title.is_empty() {
                        programs
                            .entry(current_channel.clone())
                            .or_default()
                            .push(EpgProgram {
                                title: current_title.clone(),
                                description: current_desc.take(),
                                episode: current_episode.take(),
                                start: current_start.clone(),
                                stop: current_stop.clone(),
                            });
                    }
                    in_programme = false;
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    programs
}

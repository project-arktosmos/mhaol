use crate::documents::{Artist, FileEntry, ImageMeta};
use crate::state::CloudState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const PIRATEBAY_BASE: &str = "https://apibay.org";
const TPB_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://opentracker.i2p.rocks:6969/announce",
];

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/tmdb", post(search_tmdb))
        .route("/tmdb/episodes", post(tmdb_episodes))
        .route("/torrents", post(search_torrents))
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub description: String,
    pub artists: Vec<Artist>,
    pub images: Vec<ImageMeta>,
    pub files: Vec<FileEntry>,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    pub raw: serde_json::Value,
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn search_tmdb(
    State(_state): State<CloudState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResultItem>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    if query.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }

    let is_tv = matches!(req.kind.as_str(), "tv show" | "tv season" | "tv episode");
    let endpoint = if is_tv { "/search/tv" } else { "/search/movie" };

    let url = format!(
        "{}{}?api_key={}&query={}&include_adult=false",
        TMDB_BASE,
        endpoint,
        api_key,
        urlencoding(query)
    );

    let res = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("tmdb returned {}", res.status()),
        ));
    }

    let payload: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb parse failed: {e}")))?;

    let items = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| build_tmdb_item(r))
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(items))
}

fn build_tmdb_item(r: &serde_json::Value) -> SearchResultItem {
    let title = r
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("name").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let description = r
        .get("overview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let external_id = r.get("id").map(|v| v.to_string());

    let mut images: Vec<ImageMeta> = Vec::new();
    if let Some(poster) = r.get("poster_path").and_then(|v| v.as_str()) {
        if !poster.is_empty() {
            images.push(ImageMeta {
                url: format!("{}/w500{}", TMDB_IMG_BASE, poster),
                mime_type: "image/jpeg".to_string(),
                file_size: 0,
                width: 500,
                height: 750,
            });
        }
    }
    if let Some(backdrop) = r.get("backdrop_path").and_then(|v| v.as_str()) {
        if !backdrop.is_empty() {
            images.push(ImageMeta {
                url: format!("{}/w1280{}", TMDB_IMG_BASE, backdrop),
                mime_type: "image/jpeg".to_string(),
                file_size: 0,
                width: 1280,
                height: 720,
            });
        }
    }

    SearchResultItem {
        title,
        description,
        artists: Vec::new(),
        images,
        files: Vec::new(),
        external_id,
        raw: r.clone(),
    }
}

#[derive(Debug, Deserialize)]
pub struct EpisodesRequest {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct EpisodeView {
    pub title: String,
}

async fn tmdb_episodes(
    State(_state): State<CloudState>,
    Json(req): Json<EpisodesRequest>,
) -> Result<Json<Vec<EpisodeView>>, (StatusCode, Json<serde_json::Value>)> {
    let id = req.id.trim();
    if id.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }

    let client = reqwest::Client::new();

    let detail_url = format!("{}/tv/{}?api_key={}", TMDB_BASE, urlencoding(id), api_key);
    let detail: serde_json::Value = client
        .get(&detail_url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb request failed: {e}")))?
        .error_for_status()
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb returned {e}")))?
        .json()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb parse failed: {e}")))?;

    let seasons = detail
        .get("seasons")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut episodes: Vec<EpisodeView> = Vec::new();
    for season in seasons {
        let n = season
            .get("season_number")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let url = format!(
            "{}/tv/{}/season/{}?api_key={}",
            TMDB_BASE,
            urlencoding(id),
            n,
            api_key
        );
        let payload: serde_json::Value = match client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => match r.json().await {
                Ok(v) => v,
                Err(_) => continue,
            },
            _ => continue,
        };
        if let Some(eps) = payload.get("episodes").and_then(|e| e.as_array()) {
            for ep in eps {
                let name = ep
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let s = ep
                    .get("season_number")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(n);
                let e = ep
                    .get("episode_number")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let title = if name.is_empty() {
                    format!("S{:02}E{:02}", s, e)
                } else {
                    format!("S{:02}E{:02} – {}", s, e, name)
                };
                episodes.push(EpisodeView { title });
            }
        }
    }

    Ok(Json(episodes))
}

#[derive(Debug, Deserialize)]
pub struct TorrentSearchRequest {
    pub query: String,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TorrentResult {
    pub title: String,
    pub description: String,
    #[serde(rename = "magnetLink")]
    pub magnet_link: String,
    #[serde(rename = "infoHash")]
    pub info_hash: String,
    pub raw: serde_json::Value,
}

fn build_magnet_link(info_hash: &str, name: &str) -> String {
    let encoded_name = urlencoding(name);
    let trackers = TPB_TRACKERS
        .iter()
        .map(|t| format!("&tr={}", urlencoding(t)))
        .collect::<String>();
    format!("magnet:?xt=urn:btih:{info_hash}&dn={encoded_name}{trackers}")
}

async fn search_torrents(
    State(_state): State<CloudState>,
    Json(req): Json<TorrentSearchRequest>,
) -> Result<Json<Vec<TorrentResult>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    if query.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let category = req.category.as_deref().unwrap_or("0");

    let url = format!(
        "{}/q.php?q={}&cat={}",
        PIRATEBAY_BASE,
        urlencoding(query),
        urlencoding(category)
    );

    let res = reqwest::Client::new()
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; Mhaol/1.0)",
        )
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("piratebay request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("piratebay returned {}", res.status()),
        ));
    }

    let raw: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("piratebay parse failed: {e}")))?;

    let arr = match raw.as_array() {
        Some(a) => a,
        None => return Ok(Json(Vec::new())),
    };

    // The API returns a single sentinel row when there are no results.
    if arr.len() == 1
        && arr[0].get("id").and_then(|v| v.as_str()) == Some("0")
        && arr[0].get("name").and_then(|v| v.as_str()) == Some("No results returned")
    {
        return Ok(Json(Vec::new()));
    }

    let parsed: Vec<TorrentResult> = arr
        .iter()
        .map(|r| {
            let name = r
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let info_hash = r
                .get("info_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let seeders = r
                .get("seeders")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let leechers = r
                .get("leechers")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let size_bytes = r
                .get("size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let magnet_link = if info_hash.is_empty() {
                String::new()
            } else {
                build_magnet_link(&info_hash, &name)
            };
            TorrentResult {
                title: name,
                description: format!(
                    "{seeders} seeders · {leechers} leechers · {size_bytes} bytes"
                ),
                magnet_link,
                info_hash,
                raw: r.clone(),
            }
        })
        .collect();

    Ok(Json(parsed))
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

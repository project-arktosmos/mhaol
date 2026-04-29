use crate::documents::{Artist, FileEntry, ImageMeta};
use crate::state::CloudState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p";
const PIRATEBAY_BASE: &str = "https://apibay.org";
const LRCLIB_BASE: &str = "https://lrclib.net/api";
const WYZIE_BASE: &str = "https://sub.wyzie.io";
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
        .route("/subs-lyrics", post(search_subs_lyrics))
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
    pub year: Option<i32>,
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
    let year = r
        .get("release_date")
        .and_then(|v| v.as_str())
        .or_else(|| r.get("first_air_date").and_then(|v| v.as_str()))
        .and_then(|d| d.get(0..4))
        .and_then(|s| s.parse::<i32>().ok());
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
        year,
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
    pub seeders: u64,
    pub leechers: u64,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: u64,
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
                seeders,
                leechers,
                size_bytes,
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

// ============================================================================
// Subs / Lyrics search — used by the right-side `SubsLyricsFinder` panel in
// the cloud webui (and mirrors `/api/search/subs-lyrics` on the node).
// LRCLIB for music tracks/albums, Wyzie for movies/TV episodes.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncedLine {
    pub time: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubsLyrics {
    pub kind: String,
    pub source: String,
    #[serde(rename = "externalId", default)]
    pub external_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "trackName", default, skip_serializing_if = "Option::is_none")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName", default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    #[serde(rename = "albumName", default, skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "plainLyrics", default, skip_serializing_if = "Option::is_none")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics", default, skip_serializing_if = "Option::is_none")]
    pub synced_lyrics: Option<Vec<SyncedLine>>,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(rename = "isHearingImpaired", default)]
    pub is_hearing_impaired: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(rename = "sourceExternalId", default, skip_serializing_if = "Option::is_none")]
    pub source_external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubsLyricsRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
    #[serde(rename = "externalIds", default)]
    pub external_ids: Vec<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
}

async fn search_subs_lyrics(
    State(_state): State<CloudState>,
    Json(req): Json<SubsLyricsRequest>,
) -> Result<Json<Vec<SubsLyrics>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    let kind = req.kind.trim();

    if matches!(kind, "album" | "track") {
        if query.is_empty() {
            return Ok(Json(Vec::new()));
        }
        return search_lrclib(query).await.map(Json);
    }

    if matches!(kind, "movie" | "tv show" | "tv season" | "tv episode") {
        if req.external_ids.is_empty() {
            return Ok(Json(Vec::new()));
        }
        let wyzie_kind = if kind == "movie" { "movie" } else { "tv" };
        let langs = req
            .languages
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        return search_wyzie(wyzie_kind, &req.external_ids, &langs)
            .await
            .map(Json);
    }

    Ok(Json(Vec::new()))
}

async fn search_lrclib(
    query: &str,
) -> Result<Vec<SubsLyrics>, (StatusCode, Json<serde_json::Value>)> {
    let url = format!("{}/search?q={}", LRCLIB_BASE, urlencoding(query));
    let res = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mhaol/1.0 (https://github.com/project-arktosmos/mhaol)",
        )
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("lrclib request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("lrclib returned {}", res.status()),
        ));
    }

    let payload: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("lrclib parse failed: {e}")))?;

    let arr = match payload.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let out = arr
        .iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string())
                .unwrap_or_default();
            let track_name = item
                .get("trackName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let artist_name = item
                .get("artistName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let album_name = item
                .get("albumName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let duration = item.get("duration").and_then(|v| v.as_f64());
            let plain_lyrics = item
                .get("plainLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let synced_raw = item
                .get("syncedLyrics")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let synced_lyrics = synced_raw.map(parse_lrc_lines);
            let instrumental = item
                .get("instrumental")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let display = if !artist_name.is_empty() {
                format!("{} — {}", artist_name, track_name)
            } else {
                track_name.clone()
            };
            SubsLyrics {
                kind: "lyrics".to_string(),
                source: "lrclib".to_string(),
                external_id: id,
                language: None,
                track_name: Some(track_name),
                artist_name: Some(artist_name),
                album_name,
                duration,
                format: Some("lrc".to_string()),
                url: None,
                plain_lyrics,
                synced_lyrics,
                instrumental,
                is_hearing_impaired: false,
                display: Some(display),
                source_external_id: None,
            }
        })
        .collect();
    Ok(out)
}

fn parse_lrc_lines(lrc: &str) -> Vec<SyncedLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }
        if let Some(bracket_end) = line.find(']') {
            let time_str = &line[1..bracket_end];
            let text = line[bracket_end + 1..].trim().to_string();
            if let Some(time) = parse_lrc_timestamp(time_str) {
                lines.push(SyncedLine { time, text });
            }
        }
    }
    lines.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lines
}

fn parse_lrc_timestamp(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

async fn search_wyzie(
    kind: &str,
    external_ids: &[String],
    languages: &[String],
) -> Result<Vec<SubsLyrics>, (StatusCode, Json<serde_json::Value>)> {
    let api_key = std::env::var("WYZIE_API_KEY").unwrap_or_default();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("http client failed: {e}")))?;

    let mut tasks = Vec::with_capacity(external_ids.len());
    for ext_id in external_ids {
        let trimmed = ext_id.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut params: Vec<(&str, String)> = vec![("id", trimmed.to_string())];
        if !languages.is_empty() {
            params.push(("language", languages.join(",")));
        }
        if !api_key.is_empty() {
            params.push(("key", api_key.clone()));
        }
        let qs = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding(v)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}/search?{}", WYZIE_BASE, qs);
        let client = client.clone();
        let ext_id_owned = trimmed.to_string();
        let kind_owned = kind.to_string();
        tasks.push(tokio::spawn(async move {
            fetch_wyzie_one(&client, &url, &ext_id_owned, &kind_owned).await
        }));
    }

    let mut out: Vec<SubsLyrics> = Vec::new();
    for task in tasks {
        if let Ok(Ok(items)) = task.await {
            out.extend(items);
        }
    }
    Ok(out)
}

async fn fetch_wyzie_one(
    client: &reqwest::Client,
    url: &str,
    external_id: &str,
    kind: &str,
) -> Result<Vec<SubsLyrics>, ()> {
    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("User-Agent", "Mhaol/1.0")
        .send()
        .await
        .map_err(|_| ())?;
    if !res.status().is_success() {
        return Err(());
    }
    let payload: serde_json::Value = res.json().await.map_err(|_| ())?;
    let arr = match payload.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };
    let out = arr
        .iter()
        .map(|item| {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let language = item
                .get("language")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display = item
                .get("display")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("language").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
            let url_str = item
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let format_str = item
                .get("format")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let is_hi = item
                .get("isHearingImpaired")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            SubsLyrics {
                kind: "subtitle".to_string(),
                source: format!("wyzie:{}", kind),
                external_id: id,
                language,
                track_name: None,
                artist_name: None,
                album_name: None,
                duration: None,
                format: format_str,
                url: url_str,
                plain_lyrics: None,
                synced_lyrics: None,
                instrumental: false,
                is_hearing_impaired: is_hi,
                display,
                source_external_id: Some(external_id.to_string()),
            }
        })
        .collect();
    Ok(out)
}

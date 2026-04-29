use crate::AppState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

const LRCLIB_BASE: &str = "https://lrclib.net/api";
const WYZIE_BASE: &str = "https://sub.wyzie.io";

pub fn router() -> Router<AppState> {
    Router::new().route("/subs-lyrics", post(search_subs_lyrics))
}

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

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn search_subs_lyrics(
    State(_state): State<AppState>,
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
    let url = format!("{}/search?q={}", LRCLIB_BASE, urlencoding::encode(query));
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
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
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

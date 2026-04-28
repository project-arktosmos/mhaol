use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

const WYZIE_BASE: &str = "https://sub.wyzie.io";
const USER_AGENT: &str = "mhaol/1.0.0 (https://github.com/arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", post(search))
        .route("/download", post(download))
        .route("/", get(list))
        .route("/{id}", delete(remove))
}

/// Public route for serving subtitle files. UUID-keyed access; no auth required so
/// `<track>` elements can load subtitle content (they cannot send auth headers).
pub fn public_router() -> Router<AppState> {
    Router::new().route("/file/{id}", get(serve_file))
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct SearchBody {
    #[serde(rename = "type")]
    media_type: String,
    tmdb_id: Option<String>,
    imdb_id: Option<String>,
    season: Option<u32>,
    episode: Option<u32>,
    languages: Option<Vec<String>>,
    hearing_impaired: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WyzieResult {
    id: String,
    url: String,
    #[serde(default)]
    format: String,
    #[serde(default)]
    encoding: String,
    #[serde(default)]
    media: String,
    #[serde(rename = "isHearingImpaired", default)]
    is_hearing_impaired: bool,
    #[serde(default)]
    source: String,
    #[serde(default)]
    language: String,
    #[serde(default)]
    display: String,
    #[serde(rename = "flagUrl", default)]
    flag_url: String,
}

/// Build a Wyzie search URL from a base + body + optional API key. Pure: easy to test.
fn build_search_url(base: &str, body: &SearchBody, api_key: Option<&str>) -> Result<String, &'static str> {
    let id = body
        .tmdb_id
        .as_deref()
        .or(body.imdb_id.as_deref())
        .filter(|s| !s.is_empty())
        .ok_or("Missing tmdbId or imdbId")?;

    let mut params: Vec<(&str, String)> = vec![("id", id.to_string())];
    if body.media_type == "tv" {
        if let Some(s) = body.season {
            params.push(("season", s.to_string()));
        }
        if let Some(e) = body.episode {
            params.push(("episode", e.to_string()));
        }
    }
    if let Some(langs) = body.languages.as_ref() {
        if !langs.is_empty() {
            params.push(("language", langs.join(",")));
        }
    }
    if let Some(hi) = body.hearing_impaired {
        params.push(("hi", if hi { "true" } else { "false" }.to_string()));
    }
    if let Some(k) = api_key.filter(|s| !s.is_empty()) {
        params.push(("key", k.to_string()));
    }

    let qs = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    Ok(format!("{}/search?{}", base, qs))
}

async fn search(State(state): State<AppState>, Json(body): Json<SearchBody>) -> Response {
    let api_key = state.settings.get("wyzie-subs.apiKey").unwrap_or_default();

    let url = match build_search_url(WYZIE_BASE, &body, Some(api_key.as_str())) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match client.get(&url).header("User-Agent", USER_AGENT).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Vec<WyzieResult>>().await {
            Ok(results) => Json(results).into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Parse failed: {}", e) })),
            )
                .into_response(),
        },
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            // Wyzie's error payloads are JSON like {"code":401,"message":"...","details":"..."}.
            // Prefer a clean message + details when parseable; fall back to the raw body.
            let detail = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .map(|v| {
                    let msg = v
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("")
                        .to_string();
                    let det = v
                        .get("details")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !det.is_empty() && !msg.is_empty() {
                        format!("{} ({})", msg, det)
                    } else if !msg.is_empty() {
                        msg
                    } else {
                        body_text.clone()
                    }
                })
                .unwrap_or(body_text);
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": format!("Wyzie returned {}: {}", status, detail),
                    "needsApiKey": status == reqwest::StatusCode::UNAUTHORIZED,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadBody {
    media_key: String,
    url: String,
    language_code: String,
    language_name: String,
    source: String,
    source_id: Option<String>,
    format: Option<String>,
    hearing_impaired: Option<bool>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SubtitleRow {
    id: String,
    media_key: String,
    language_code: String,
    language_name: String,
    source: String,
    source_id: Option<String>,
    format: String,
    hearing_impaired: bool,
    url: String,
    downloaded_at: String,
}

async fn download(State(state): State<AppState>, Json(body): Json<DownloadBody>) -> Response {
    let client = reqwest::Client::new();
    let bytes = match client
        .get(&body.url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => match r.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        },
        Ok(r) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Source returned {}", r.status()) })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let raw_text = match std::str::from_utf8(&bytes) {
        Ok(s) => s.to_string(),
        Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
    };

    let in_format = body
        .format
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| {
            if raw_text.trim_start().starts_with("WEBVTT") {
                "vtt".to_string()
            } else {
                "srt".to_string()
            }
        });
    let vtt = if in_format == "vtt" {
        raw_text
    } else {
        srt_to_vtt(&raw_text)
    };

    let id = uuid::Uuid::new_v4().to_string();
    let dir = state.data_dir.join("subtitles");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("mkdir failed: {}", e) })),
        )
            .into_response();
    }
    let file_path = dir.join(format!("{}.vtt", id));
    if let Err(e) = std::fs::write(&file_path, vtt.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("write failed: {}", e) })),
        )
            .into_response();
    }

    let hearing_impaired = body.hearing_impaired.unwrap_or(false);
    let file_path_str = file_path.to_string_lossy().to_string();

    {
        let conn = state.db.lock();
        if let Err(e) = conn.execute(
            "INSERT INTO subtitles (id, media_key, language_code, language_name, source, source_id, format, file_path, hearing_impaired)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'vtt', ?7, ?8)",
            rusqlite::params![
                id,
                body.media_key,
                body.language_code,
                body.language_name,
                body.source,
                body.source_id,
                file_path_str,
                hearing_impaired as i32,
            ],
        ) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    }

    let row = SubtitleRow {
        id: id.clone(),
        media_key: body.media_key,
        language_code: body.language_code,
        language_name: body.language_name,
        source: body.source,
        source_id: body.source_id,
        format: "vtt".to_string(),
        hearing_impaired,
        url: format!("/api/subtitles/file/{}", id),
        downloaded_at: chrono::Utc::now().to_rfc3339(),
    };
    Json(row).into_response()
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(rename = "mediaKey")]
    media_key: String,
}

async fn list(State(state): State<AppState>, Query(q): Query<ListQuery>) -> Response {
    let conn = state.db.lock();
    let mut stmt = match conn.prepare(
        "SELECT id, media_key, language_code, language_name, source, source_id, format, hearing_impaired, downloaded_at
         FROM subtitles WHERE media_key = ?1 ORDER BY downloaded_at DESC",
    ) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };
    let rows = stmt
        .query_map(rusqlite::params![q.media_key], |row| {
            let id: String = row.get(0)?;
            Ok(SubtitleRow {
                id: id.clone(),
                media_key: row.get(1)?,
                language_code: row.get(2)?,
                language_name: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                format: row.get(6)?,
                hearing_impaired: row.get::<_, i32>(7)? != 0,
                url: format!("/api/subtitles/file/{}", id),
                downloaded_at: row.get(8)?,
            })
        })
        .and_then(|iter| iter.collect::<Result<Vec<_>, _>>());

    match rows {
        Ok(list) => Json(list).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn serve_file(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let file_path: Option<String> = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT file_path FROM subtitles WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok()
    };
    let path = match file_path {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/vtt; charset=utf-8")
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(bytes))
        .unwrap()
}

async fn remove(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let file_path: Option<String> = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT file_path FROM subtitles WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .ok()
    };
    {
        let conn = state.db.lock();
        let _ = conn.execute(
            "DELETE FROM subtitles WHERE id = ?1",
            rusqlite::params![id],
        );
    }
    if let Some(p) = file_path {
        let _ = std::fs::remove_file(&p);
    }
    Json(serde_json::json!({ "ok": true })).into_response()
}

/// Convert SubRip (.srt) text to WebVTT.
fn srt_to_vtt(srt: &str) -> String {
    let mut out = String::with_capacity(srt.len() + 16);
    out.push_str("WEBVTT\n\n");
    for line in srt.lines() {
        let trimmed = line.trim_end_matches('\r');
        // SRT timestamps use comma as decimal separator; VTT uses dot.
        if trimmed.contains("-->") && trimmed.contains(',') {
            out.push_str(&trimmed.replace(',', "."));
        } else {
            out.push_str(trimmed);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_to_vtt_replaces_commas_in_timestamps() {
        let srt = "1\n00:00:01,000 --> 00:00:02,500\nHello\n";
        let vtt = srt_to_vtt(srt);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:01.000 --> 00:00:02.500"));
        assert!(vtt.contains("Hello"));
    }

    #[test]
    fn search_url_movie_minimal() {
        let body = SearchBody {
            media_type: "movie".into(),
            tmdb_id: Some("603".into()),
            ..Default::default()
        };
        let url = build_search_url("https://example.test", &body, None).unwrap();
        assert_eq!(url, "https://example.test/search?id=603");
    }

    #[test]
    fn search_url_tv_with_episode_languages_hi_and_key() {
        let body = SearchBody {
            media_type: "tv".into(),
            tmdb_id: Some("1396".into()),
            season: Some(1),
            episode: Some(2),
            languages: Some(vec!["en".into(), "es".into()]),
            hearing_impaired: Some(true),
            ..Default::default()
        };
        let url = build_search_url("https://example.test", &body, Some("k123")).unwrap();
        assert_eq!(
            url,
            "https://example.test/search?id=1396&season=1&episode=2&language=en%2Ces&hi=true&key=k123"
        );
    }

    #[test]
    fn search_url_tv_ignores_season_episode_when_movie() {
        let body = SearchBody {
            media_type: "movie".into(),
            tmdb_id: Some("603".into()),
            season: Some(1),
            episode: Some(2),
            ..Default::default()
        };
        let url = build_search_url("https://example.test", &body, None).unwrap();
        assert_eq!(url, "https://example.test/search?id=603");
    }

    #[test]
    fn search_url_falls_back_to_imdb_id() {
        let body = SearchBody {
            media_type: "movie".into(),
            imdb_id: Some("tt0468569".into()),
            ..Default::default()
        };
        let url = build_search_url("https://example.test", &body, None).unwrap();
        assert_eq!(url, "https://example.test/search?id=tt0468569");
    }

    #[test]
    fn search_url_empty_api_key_is_omitted() {
        let body = SearchBody {
            media_type: "movie".into(),
            tmdb_id: Some("603".into()),
            ..Default::default()
        };
        let url = build_search_url("https://example.test", &body, Some("")).unwrap();
        assert_eq!(url, "https://example.test/search?id=603");
    }

    #[test]
    fn search_url_requires_an_id() {
        let body = SearchBody {
            media_type: "movie".into(),
            ..Default::default()
        };
        let err = build_search_url("https://example.test", &body, None).unwrap_err();
        assert!(err.contains("Missing"));
    }

    /// Live probe against the real Wyzie API. Catches breakages like the .ru → .io migration
    /// or the introduction of API key requirements. Skipped without `WYZIE_API_KEY` so CI stays
    /// green for contributors who haven't claimed a key. Run with:
    ///   `WYZIE_API_KEY=... cargo test -p mhaol-node wyzie_live -- --include-ignored`
    #[tokio::test]
    #[ignore]
    async fn wyzie_live_search_returns_results() {
        let api_key = std::env::var("WYZIE_API_KEY")
            .expect("WYZIE_API_KEY must be set for live API test");
        let body = SearchBody {
            media_type: "movie".into(),
            tmdb_id: Some("603".into()),
            languages: Some(vec!["en".into()]),
            ..Default::default()
        };
        let url = build_search_url(WYZIE_BASE, &body, Some(&api_key)).unwrap();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .unwrap();
        let resp = client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .expect("request failed");
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        assert!(
            status.is_success(),
            "Wyzie returned {}: {}",
            status,
            body_text
        );
        let parsed: Vec<WyzieResult> = serde_json::from_str(&body_text)
            .unwrap_or_else(|e| panic!("Failed to parse Wyzie response as Vec<WyzieResult>: {e}\nBody: {body_text}"));
        assert!(!parsed.is_empty(), "expected at least one English result for tmdb 603");
    }
}

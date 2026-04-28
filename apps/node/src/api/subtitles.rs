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

const WYZIE_BASE: &str = "https://sub.wyzie.ru";
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

#[derive(Deserialize)]
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

async fn search(State(_state): State<AppState>, Json(body): Json<SearchBody>) -> Response {
    let id = match body
        .tmdb_id
        .as_deref()
        .or(body.imdb_id.as_deref())
        .filter(|s| !s.is_empty())
    {
        Some(v) => v.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing tmdbId or imdbId" })),
            )
                .into_response();
        }
    };

    let mut params: Vec<(&str, String)> = vec![("id", id)];
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

    let qs = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let url = format!("{}/search?{}", WYZIE_BASE, qs);

    let client = reqwest::Client::new();
    match client.get(&url).header("User-Agent", USER_AGENT).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Vec<WyzieResult>>().await {
            Ok(results) => Json(results).into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Parse failed: {}", e) })),
            )
                .into_response(),
        },
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Wyzie returned {}", resp.status()) })),
        )
            .into_response(),
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
}

use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;

const LRCLIB_BASE: &str = "https://lrclib.net/api";
const USER_AGENT: &str = "mhaol/1.0.0 (https://github.com/arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new().route("/{item_id}", get(get_lyrics))
}

#[derive(Serialize)]
struct LyricsResponse {
    id: Option<i64>,
    #[serde(rename = "trackName")]
    track_name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    #[serde(rename = "albumName")]
    album_name: String,
    duration: f64,
    instrumental: bool,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<Vec<SyncedLine>>,
}

#[derive(Serialize)]
struct SyncedLine {
    time: f64,
    text: String,
}

async fn get_lyrics(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> impl IntoResponse {
    // Check lookup cache
    {
        let conn = state.db.lock();
        if let Ok((status, lrclib_id)) = conn.query_row(
            "SELECT status, lrclib_id FROM lrclib_lookups WHERE library_item_id = ?1",
            rusqlite::params![item_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?)),
        ) {
            if status == "found" {
                if let Some(lid) = lrclib_id {
                    if let Ok(lyrics) = get_cached_lyrics(&conn, lid) {
                        return Json(lyrics).into_response();
                    }
                }
            } else {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Lyrics not found (cached)" })),
                )
                    .into_response();
            }
        }
    }

    // Get MusicBrainz link for this item
    let mb_link = state
        .library_item_links
        .get_by_item_and_service(&item_id, "musicbrainz");
    let mb_id = match mb_link {
        Some(link) => link.service_id,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "No MusicBrainz link for this item" })),
            )
                .into_response()
        }
    };

    // Get recording metadata from cache
    let recording_data = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT data FROM musicbrainz_recordings WHERE mbid = ?1",
            rusqlite::params![mb_id],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
    };

    let (track_name, artist_name, album_name, duration_secs) = match &recording_data {
        Some(data) => {
            let track = data["title"].as_str().unwrap_or("").to_string();
            let artist = data["artist-credit"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["artist"]["name"].as_str())
                .unwrap_or("")
                .to_string();
            let album = data["releases"]
                .as_array()
                .and_then(|r| r.first())
                .and_then(|r| r["title"].as_str())
                .unwrap_or("")
                .to_string();
            let length_ms = data["length"].as_f64().unwrap_or(0.0);
            (track, artist, album, (length_ms / 1000.0).round() as i64)
        }
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Recording metadata not found" })),
            )
                .into_response()
        }
    };

    // Query LrcLib
    let url = format!(
        "{}/get?track_name={}&artist_name={}&album_name={}&duration={}",
        LRCLIB_BASE,
        urlencoding::encode(&track_name),
        urlencoding::encode(&artist_name),
        urlencoding::encode(&album_name),
        duration_secs
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let lrclib_id = data["id"].as_i64().unwrap_or(0);
                    let plain = data["plainLyrics"].as_str().map(String::from);
                    let synced_raw = data["syncedLyrics"].as_str().map(String::from);
                    let instrumental = data["instrumental"].as_bool().unwrap_or(false);
                    let duration = data["duration"].as_f64().unwrap_or(0.0);

                    // Cache lyrics
                    {
                        let conn = state.db.lock();
                        let _ = conn.execute(
                            "INSERT INTO lrclib_lyrics (lrclib_id, track_name, artist_name, album_name, duration, instrumental, plain_lyrics, synced_lyrics)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                             ON CONFLICT(lrclib_id) DO UPDATE SET plain_lyrics = ?7, synced_lyrics = ?8, fetched_at = datetime('now')",
                            rusqlite::params![lrclib_id, track_name, artist_name, album_name, duration, instrumental as i32, plain, synced_raw],
                        );
                        let _ = conn.execute(
                            "INSERT INTO lrclib_lookups (library_item_id, lrclib_id, status) VALUES (?1, ?2, 'found')
                             ON CONFLICT(library_item_id) DO UPDATE SET lrclib_id = ?2, status = 'found', looked_up_at = datetime('now')",
                            rusqlite::params![item_id, lrclib_id],
                        );
                    }

                    let synced = synced_raw.map(|s| parse_synced_lyrics(&s));

                    Json(LyricsResponse {
                        id: Some(lrclib_id),
                        track_name,
                        artist_name,
                        album_name,
                        duration,
                        instrumental,
                        plain_lyrics: plain,
                        synced_lyrics: synced,
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => {
            // Cache miss
            let conn = state.db.lock();
            let _ = conn.execute(
                "INSERT INTO lrclib_lookups (library_item_id, status) VALUES (?1, 'not_found')
                 ON CONFLICT(library_item_id) DO UPDATE SET status = 'not_found', looked_up_at = datetime('now')",
                rusqlite::params![item_id],
            );
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Lyrics not found" })),
            )
                .into_response()
        }
    }
}

fn get_cached_lyrics(
    conn: &rusqlite::Connection,
    lrclib_id: i64,
) -> Result<LyricsResponse, rusqlite::Error> {
    conn.query_row(
        "SELECT lrclib_id, track_name, artist_name, album_name, duration, instrumental, plain_lyrics, synced_lyrics
         FROM lrclib_lyrics WHERE lrclib_id = ?1",
        rusqlite::params![lrclib_id],
        |row| {
            let synced_raw: Option<String> = row.get(7)?;
            Ok(LyricsResponse {
                id: Some(row.get(0)?),
                track_name: row.get(1)?,
                artist_name: row.get(2)?,
                album_name: row.get(3)?,
                duration: row.get(4)?,
                instrumental: row.get::<_, i32>(5)? != 0,
                plain_lyrics: row.get(6)?,
                synced_lyrics: synced_raw.map(|s| parse_synced_lyrics(&s)),
            })
        },
    )
}

fn parse_synced_lyrics(lrc: &str) -> Vec<SyncedLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Parse [mm:ss.xx] format
        if let Some(bracket_end) = line.find(']') {
            let time_str = &line[1..bracket_end];
            let text = line[bracket_end + 1..].trim().to_string();
            if let Some(time) = parse_lrc_time(time_str) {
                lines.push(SyncedLine { time, text });
            }
        }
    }
    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    lines
}

fn parse_lrc_time(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

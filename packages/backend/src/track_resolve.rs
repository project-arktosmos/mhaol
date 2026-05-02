use crate::firkins::FileEntry;
use crate::search::{lrclib_search_raw, LrclibHit};
use mhaol_yt_dlp::search::{search_query, SearchItem};
use serde::Serialize;

const MUSICBRAINZ_BASE: &str = "https://musicbrainz.org/ws/2";
const USER_AGENT: &str = "Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)";

const NOISE_WORDS: &[&str] = &[
    "official",
    "video",
    "audio",
    "lyric",
    "lyrics",
    "hd",
    "4k",
    "mv",
    "hq",
    "live",
    "remaster",
    "remastered",
    "edit",
    "version",
];

fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth: i32 = 0;
    for c in s.chars() {
        match c {
            '(' | '[' => {
                depth += 1;
                out.push(' ');
            }
            ')' | ']' => {
                depth = (depth - 1).max(0);
                out.push(' ');
            }
            _ if depth > 0 => out.push(' '),
            _ => {
                let lower = c.to_ascii_lowercase();
                if lower.is_ascii_alphanumeric() {
                    out.push(lower);
                } else {
                    out.push(' ');
                }
            }
        }
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .split(' ')
        .filter(|w| !NOISE_WORDS.contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokens(s: &str) -> Vec<String> {
    normalize(s)
        .split(' ')
        .filter(|w| w.len() > 1)
        .map(|w| w.to_string())
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct MbTrack {
    pub position: i64,
    pub title: String,
    pub length_ms: Option<i64>,
}

pub async fn fetch_release_group_tracks(release_group_id: &str) -> Result<Vec<MbTrack>, String> {
    let url = format!(
        "{}/release?release-group={}&inc=recordings&fmt=json&limit=1",
        MUSICBRAINZ_BASE,
        urlencode(release_group_id)
    );
    let res = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("musicbrainz request failed: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("musicbrainz returned {}", res.status()));
    }
    let payload: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("musicbrainz parse failed: {e}"))?;
    let mut out = Vec::new();
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
                    if !title.is_empty() {
                        out.push(MbTrack {
                            position,
                            title,
                            length_ms,
                        });
                    }
                }
            }
        }
    }
    Ok(out)
}

fn urlencode(s: &str) -> String {
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

/// Mirrors the JS `pickBestYouTubeMatch`: hard ≥50% track-token gate,
/// then score by track-title overlap (×10), artist hits (×6), album hits
/// (×2), and duration delta bonuses.
pub fn pick_best_youtube_match<'a>(
    items: &'a [SearchItem],
    track_title: &str,
    artist: &str,
    album_title: &str,
    track_duration_ms: Option<i64>,
) -> Option<&'a SearchItem> {
    if items.is_empty() {
        return None;
    }
    let track_tokens = tokens(track_title);
    let artist_tokens = tokens(artist);
    let album_tokens = tokens(album_title);
    let target_sec = track_duration_ms.filter(|&d| d > 0).map(|d| (d / 1000) as i64);

    let mut best: Option<(&SearchItem, f64)> = None;
    for item in items {
        let title_norm = normalize(&item.title);
        let uploader_norm = normalize(&item.uploader_name);
        let title_and_uploader = format!("{} {}", title_norm, uploader_norm);

        let track_hits = track_tokens
            .iter()
            .filter(|t| title_norm.contains(t.as_str()))
            .count() as f64;
        let track_ratio = if track_tokens.is_empty() {
            0.0
        } else {
            track_hits / track_tokens.len() as f64
        };
        if track_ratio < 0.5 {
            continue;
        }
        let mut score = track_ratio * 10.0;
        if !artist_tokens.is_empty() {
            let hits = artist_tokens
                .iter()
                .filter(|t| title_and_uploader.contains(t.as_str()))
                .count() as f64;
            score += (hits / artist_tokens.len() as f64) * 6.0;
        }
        if !album_tokens.is_empty() {
            let hits = album_tokens
                .iter()
                .filter(|t| title_norm.contains(t.as_str()))
                .count() as f64;
            score += (hits / album_tokens.len() as f64) * 2.0;
        }
        if let Some(target) = target_sec {
            if item.duration > 0 {
                let delta = (item.duration - target).abs();
                if delta <= 3 {
                    score += 6.0;
                } else if delta <= 10 {
                    score += 3.0;
                } else if delta <= 20 {
                    score += 1.0;
                }
            }
        }
        match best {
            Some((_, bs)) if bs >= score => {}
            _ => best = Some((item, score)),
        }
    }
    best.map(|(it, _)| it)
}

/// Mirrors `pickBestLyricsMatch`: hard ≥50% track-token gate against the
/// hit's `trackName`, then score by track-title overlap (×10), artist hits
/// in `artistName` (×6), album hits in `albumName` (×2), duration delta,
/// plus a small bonus for synced lyrics.
pub fn pick_best_lyrics_match<'a>(
    items: &'a [LrclibHit],
    track_title: &str,
    artist: &str,
    album_title: &str,
    track_duration_ms: Option<i64>,
) -> Option<&'a LrclibHit> {
    if items.is_empty() {
        return None;
    }
    let track_tokens = tokens(track_title);
    let artist_tokens = tokens(artist);
    let album_tokens = tokens(album_title);
    let target_sec = track_duration_ms.filter(|&d| d > 0).map(|d| (d / 1000) as f64);

    let mut best: Option<(&LrclibHit, f64)> = None;
    for item in items {
        let item_track = normalize(&item.track_name);
        let item_artist = normalize(&item.artist_name);
        let item_album = normalize(&item.album_name);

        let track_hits = track_tokens
            .iter()
            .filter(|t| item_track.contains(t.as_str()))
            .count() as f64;
        let track_ratio = if track_tokens.is_empty() {
            0.0
        } else {
            track_hits / track_tokens.len() as f64
        };
        if track_ratio < 0.5 {
            continue;
        }
        let mut score = track_ratio * 10.0;
        if !artist_tokens.is_empty() {
            let hits = artist_tokens
                .iter()
                .filter(|t| item_artist.contains(t.as_str()))
                .count() as f64;
            score += (hits / artist_tokens.len() as f64) * 6.0;
        }
        if !album_tokens.is_empty() {
            let hits = album_tokens
                .iter()
                .filter(|t| item_album.contains(t.as_str()))
                .count() as f64;
            score += (hits / album_tokens.len() as f64) * 2.0;
        }
        if let Some(target) = target_sec {
            if let Some(d) = item.duration {
                if d > 0.0 {
                    let delta = (d - target).abs();
                    if delta <= 3.0 {
                        score += 6.0;
                    } else if delta <= 10.0 {
                        score += 3.0;
                    } else if delta <= 20.0 {
                        score += 1.0;
                    }
                }
            }
        }
        if item.synced_lyrics.is_some() {
            score += 1.0;
        }
        match best {
            Some((_, bs)) if bs >= score => {}
            _ => best = Some((item, score)),
        }
    }
    best.map(|(it, _)| it)
}

/// Resolve YouTube URL + lyrics for a single track. Either or both may be
/// `None` when no match crosses the scoring threshold or the upstream
/// search fails — the caller treats failures as "no match" rather than
/// surfacing an error per track.
pub async fn resolve_track(
    track_title: &str,
    artist: &str,
    album_title: &str,
    duration_ms: Option<i64>,
) -> (Option<String>, Option<LrclibHit>) {
    let title = track_title.trim();
    if title.is_empty() {
        return (None, None);
    }
    let yt_query = [artist, album_title, title]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let lrc_query = [artist, title]
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let yt_fut = search_query(&yt_query, None);
    let lrc_fut = lrclib_search_raw(&lrc_query);
    let (yt_res, lrc_res) = tokio::join!(yt_fut, lrc_fut);

    let yt_url = match yt_res {
        Ok(resp) => pick_best_youtube_match(&resp.items, title, artist, album_title, duration_ms)
            .map(|it| format!("https://www.youtube.com/watch?v={}", it.video_id)),
        Err(e) => {
            tracing::warn!(track = %title, error = %e, "youtube search failed");
            None
        }
    };
    let lyrics = match lrc_res {
        Ok(items) => pick_best_lyrics_match(&items, title, artist, album_title, duration_ms)
            .cloned(),
        Err(e) => {
            tracing::warn!(track = %title, error = %e, "lrclib search failed");
            None
        }
    };

    (yt_url, lyrics)
}

/// Encode an `LrclibHit` into the JSON string used as `FileEntry.value`
/// for `lyrics`-type entries on a firkin. Captures the lrclib row id, the
/// raw synced LRC text (when present), the plain lyrics fallback, and the
/// instrumental flag — everything the WebUI needs to render the lyrics
/// without re-querying LRCLIB.
pub fn encode_lyrics_value(hit: &LrclibHit) -> String {
    serde_json::json!({
        "source": "lrclib",
        "externalId": hit.id,
        "syncedLyrics": hit.synced_lyrics,
        "plainLyrics": hit.plain_lyrics,
        "instrumental": hit.instrumental,
    })
    .to_string()
}

/// Replace any existing entry with the same `(type, title)` key. Returns
/// the updated files vec.
pub fn upsert_track_file(mut files: Vec<FileEntry>, entry: FileEntry) -> Vec<FileEntry> {
    let key_title = entry.title.as_deref().unwrap_or("").trim().to_string();
    if key_title.is_empty() {
        files.push(entry);
        return files;
    }
    let mut found = false;
    for f in files.iter_mut() {
        if f.kind == entry.kind
            && f.title
                .as_deref()
                .map(|t| t.trim().eq_ignore_ascii_case(&key_title))
                .unwrap_or(false)
        {
            f.value = entry.value.clone();
            found = true;
            break;
        }
    }
    if !found {
        files.push(entry);
    }
    files
}

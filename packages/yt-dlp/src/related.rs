//! YouTube "watch next" related-videos lookup via the InnerTube `/next`
//! endpoint — the same endpoint that powers the sidebar on
//! `youtube.com/watch?v=…`. Given a video id, returns the list of related
//! videos (mostly `compactVideoRenderer` entries, with a few promoted /
//! continuation items mixed in that we filter out).
//!
//! Mirrors the shape of [`crate::search`]: a pure helper
//! [`related_query`] callable from other crates, plus the lightweight
//! axum handler [`related`] used by the cloud's `/api/ytdl/related`
//! route.

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

const INNERTUBE_URL: &str = "https://www.youtube.com/youtubei/v1/next";
const CLIENT_VERSION: &str = "2.20260301.01.00";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

#[derive(Deserialize)]
pub struct RelatedQuery {
    /// Either a full watch URL (`https://www.youtube.com/watch?v=…`,
    /// `https://youtu.be/…`, etc.) or a bare 11-char video id.
    pub url: Option<String>,
    /// Bare video id alternative; takes precedence over `url` if both
    /// are supplied.
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RelatedItem {
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub title: String,
    pub url: String,
    pub thumbnail: String,
    pub duration: i64,
    #[serde(rename = "durationText")]
    pub duration_text: String,
    pub views: i64,
    #[serde(rename = "viewsText")]
    pub views_text: String,
    #[serde(rename = "uploadedDate")]
    pub uploaded_date: String,
    #[serde(rename = "uploaderName")]
    pub uploader_name: String,
    #[serde(rename = "uploaderUrl")]
    pub uploader_url: String,
    #[serde(rename = "uploaderVerified")]
    pub uploader_verified: bool,
}

#[derive(Serialize)]
pub struct RelatedResponse {
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub items: Vec<RelatedItem>,
}

pub async fn related(Query(query): Query<RelatedQuery>) -> impl IntoResponse {
    let video_id = match resolve_video_id(&query) {
        Ok(id) => id,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response();
        }
    };

    match related_query(&video_id).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

fn resolve_video_id(query: &RelatedQuery) -> Result<String, String> {
    if let Some(id) = query.video_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return crate::util::extract_video_id(id).map_err(|e| e.to_string());
    }
    if let Some(url) = query.url.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return crate::util::extract_video_id(url).map_err(|e| e.to_string());
    }
    Err("Missing query parameter 'url' or 'videoId'".to_string())
}

/// Pure helper that fetches the InnerTube `/next` payload for a video
/// and extracts the secondary-results "Up next" list. Wrapped by
/// [`related`]; also callable from other crates that need related
/// videos server-side without going through HTTP.
pub async fn related_query(video_id: &str) -> Result<RelatedResponse, String> {
    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": CLIENT_VERSION,
                "hl": "en",
                "gl": "US"
            }
        },
        "videoId": video_id
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(INNERTUBE_URL)
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("X-YouTube-Client-Name", "1")
        .header("X-YouTube-Client-Version", CLIENT_VERSION)
        .header("Origin", "https://www.youtube.com")
        .header("Referer", "https://www.youtube.com/")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("YouTube API error: {}", resp.status()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let items = parse_secondary_results(&data);
    Ok(RelatedResponse {
        video_id: video_id.to_string(),
        items,
    })
}

fn parse_secondary_results(data: &serde_json::Value) -> Vec<RelatedItem> {
    let mut items: Vec<RelatedItem> = Vec::new();

    let results = data
        .pointer(
            "/contents/twoColumnWatchNextResults/secondaryResults/secondaryResults/results",
        )
        .and_then(|v| v.as_array());

    let Some(results) = results else {
        return items;
    };

    for entry in results {
        // The current YouTube web client serves the sidebar as
        // `lockupViewModel` entries — the new view-model-style schema.
        if let Some(lockup) = entry.get("lockupViewModel") {
            if let Some(item) = parse_lockup_view_model(lockup) {
                items.push(item);
            }
            continue;
        }
        // Older clients (or fallback) still return `compactVideoRenderer`.
        if let Some(compact) = entry.get("compactVideoRenderer") {
            if let Some(item) = parse_compact_video_renderer(compact) {
                items.push(item);
            }
            continue;
        }
        // Skip continuationItemRenderer, reelShelfRenderer (Shorts),
        // compactRadioRenderer (mixes), compactPlaylistRenderer, and ad
        // slots — the consumer only cares about a flat list of related
        // videos.
    }

    items
}

fn parse_lockup_view_model(v: &serde_json::Value) -> Option<RelatedItem> {
    // Only surface plain videos. Playlists / podcasts / mixes use
    // different content types and would point at a `list=` URL rather
    // than a watchable video.
    let content_type = v.get("contentType").and_then(|t| t.as_str()).unwrap_or("");
    if content_type != "LOCKUP_CONTENT_TYPE_VIDEO" {
        return None;
    }

    let video_id = v.get("contentId")?.as_str()?.to_string();

    let title = v
        .pointer("/metadata/lockupMetadataViewModel/title/content")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    // Pick the largest thumbnail source available. The sources list is
    // ordered small → large, so `last` wins.
    let thumbnail = v
        .pointer("/contentImage/thumbnailViewModel/image/sources")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    // Duration sits inside an overlay badge; walk the badges and pick
    // the first one that has a `text` field shaped like a duration
    // (`MM:SS` or `H:MM:SS`).
    let duration_text = v
        .pointer("/contentImage/thumbnailViewModel/overlays")
        .and_then(|o| o.as_array())
        .and_then(|overlays| {
            overlays.iter().find_map(|overlay| {
                overlay
                    .pointer("/thumbnailBottomOverlayViewModel/badges")
                    .and_then(|b| b.as_array())
                    .and_then(|badges| {
                        badges.iter().find_map(|b| {
                            b.pointer("/thumbnailBadgeViewModel/text")
                                .and_then(|t| t.as_str())
                                .filter(|s| s.contains(':'))
                                .map(str::to_string)
                        })
                    })
            })
        })
        .unwrap_or_default();
    let duration = parse_duration(&duration_text);

    // Metadata rows: row 0 is the byline (uploader name), row 1 is
    // `[<views>, <uploaded date>]`. The shape is stable across regular
    // sidebar entries.
    let metadata_rows = v
        .pointer("/metadata/lockupMetadataViewModel/metadata/contentMetadataViewModel/metadataRows")
        .and_then(|r| r.as_array());

    let uploader_name = metadata_rows
        .and_then(|rows| rows.first())
        .and_then(|row| row.pointer("/metadataParts/0/text/content"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let (views_text, uploaded_date) = metadata_rows
        .and_then(|rows| rows.get(1))
        .and_then(|row| row.get("metadataParts").and_then(|p| p.as_array()))
        .map(|parts| {
            let v = parts
                .first()
                .and_then(|p| p.pointer("/text/content"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            let d = parts
                .get(1)
                .and_then(|p| p.pointer("/text/content"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            (v, d)
        })
        .unwrap_or_default();
    let views = parse_view_count(&views_text);

    let uploader_url = v
        .pointer(
            "/metadata/lockupMetadataViewModel/image/decoratedAvatarViewModel/rendererContext/commandContext/onTap/innertubeCommand/browseEndpoint/canonicalBaseUrl",
        )
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Some(RelatedItem {
        video_id: video_id.clone(),
        title,
        url: format!("/watch?v={}", video_id),
        thumbnail,
        duration,
        duration_text,
        views,
        views_text,
        uploaded_date,
        uploader_name,
        uploader_url,
        // The lockup view model exposes verified / artist badges on a
        // different path than the legacy renderer; we don't surface
        // them yet — the UI currently doesn't render the badge
        // anyway, so leaving it as `false` is harmless.
        uploader_verified: false,
    })
}

fn parse_compact_video_renderer(v: &serde_json::Value) -> Option<RelatedItem> {
    let video_id = v.get("videoId")?.as_str()?.to_string();

    let title = v
        .pointer("/title/simpleText")
        .or_else(|| v.pointer("/title/runs/0/text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let thumbnail = v
        .pointer("/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let duration_text = v
        .pointer("/lengthText/simpleText")
        .or_else(|| v.pointer("/lengthText/runs/0/text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let duration = parse_duration(&duration_text);

    let views_text = v
        .pointer("/viewCountText/simpleText")
        .or_else(|| v.pointer("/viewCountText/runs/0/text"))
        .or_else(|| v.pointer("/shortViewCountText/simpleText"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let views = parse_view_count(&views_text);

    let uploaded_date = v
        .pointer("/publishedTimeText/simpleText")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_name = v
        .pointer("/longBylineText/runs/0/text")
        .or_else(|| v.pointer("/shortBylineText/runs/0/text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_url = v
        .pointer(
            "/longBylineText/runs/0/navigationEndpoint/browseEndpoint/canonicalBaseUrl",
        )
        .or_else(|| {
            v.pointer(
                "/shortBylineText/runs/0/navigationEndpoint/browseEndpoint/canonicalBaseUrl",
            )
        })
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_verified = v
        .get("ownerBadges")
        .and_then(|b| b.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    Some(RelatedItem {
        video_id: video_id.clone(),
        title,
        url: format!("/watch?v={}", video_id),
        thumbnail,
        duration,
        duration_text,
        views,
        views_text,
        uploaded_date,
        uploader_name,
        uploader_url,
        uploader_verified,
    })
}

fn parse_duration(text: &str) -> i64 {
    let parts: Vec<&str> = text.split(':').collect();
    match parts.len() {
        3 => {
            let h: i64 = parts[0].parse().unwrap_or(0);
            let m: i64 = parts[1].parse().unwrap_or(0);
            let s: i64 = parts[2].parse().unwrap_or(0);
            h * 3600 + m * 60 + s
        }
        2 => {
            let m: i64 = parts[0].parse().unwrap_or(0);
            let s: i64 = parts[1].parse().unwrap_or(0);
            m * 60 + s
        }
        1 => parts[0].parse().unwrap_or(0),
        _ => 0,
    }
}

fn parse_view_count(text: &str) -> i64 {
    let cleaned = text
        .replace(" views", "")
        .replace(" view", "")
        .replace(',', "");
    cleaned.parse().unwrap_or(0)
}

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

const INNERTUBE_URL: &str = "https://www.youtube.com/youtubei/v1/search";
const CLIENT_VERSION: &str = "2.20260301.01.00";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub continuation: Option<String>,
}

#[derive(Serialize)]
pub struct SearchItem {
    #[serde(rename = "videoId")]
    pub video_id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub url: String,
    pub title: String,
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
    #[serde(rename = "uploaderAvatar")]
    pub uploader_avatar: String,
    #[serde(rename = "uploaderVerified")]
    pub uploader_verified: bool,
}

#[derive(Serialize)]
pub struct SearchChannelItem {
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub name: String,
    pub thumbnail: String,
    pub url: String,
    #[serde(rename = "subscriberText")]
    pub subscriber_text: String,
    #[serde(rename = "videoCountText")]
    pub video_count_text: String,
    pub description: String,
    pub verified: bool,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub items: Vec<SearchItem>,
    pub channels: Vec<SearchChannelItem>,
    pub continuation: Option<String>,
}

pub async fn search(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    let q = match &query.q {
        Some(q) if !q.is_empty() => q.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response();
        }
    };

    match search_query(&q, query.continuation.as_deref()).await {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// Pure helper that performs a YouTube search via the InnerTube API.
/// Wrapped by the axum handler `search`; also callable directly from
/// other crates that need to run a YouTube search server-side without
/// going through HTTP (e.g. firkin track resolution).
pub async fn search_query(
    q: &str,
    continuation: Option<&str>,
) -> Result<SearchResponse, String> {
    let mut body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": CLIENT_VERSION,
                "hl": "en",
                "gl": "US"
            }
        }
    });

    if let Some(token) = continuation {
        body["continuation"] = serde_json::Value::String(token.to_string());
    } else {
        body["query"] = serde_json::Value::String(q.to_string());
    }

    let client = reqwest::Client::new();
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
    let (items, channels, continuation) = parse_innertube_response(&data);
    Ok(SearchResponse {
        items,
        channels,
        continuation,
    })
}

fn parse_innertube_response(
    data: &serde_json::Value,
) -> (Vec<SearchItem>, Vec<SearchChannelItem>, Option<String>) {
    let mut items = Vec::new();
    let mut channels = Vec::new();
    let mut continuation = None;

    let sections: Vec<&serde_json::Value> = if let Some(contents) = data.pointer(
        "/contents/twoColumnSearchResultsRenderer/primaryContents/sectionListRenderer/contents",
    ) {
        contents
            .as_array()
            .map(|a| a.iter().collect())
            .unwrap_or_default()
    } else if let Some(commands) = data
        .get("onResponseReceivedCommands")
        .and_then(|c| c.as_array())
    {
        let mut cont_items = Vec::new();
        for cmd in commands {
            if let Some(action) = cmd.get("appendContinuationItemsAction") {
                if let Some(ci) = action.get("continuationItems").and_then(|c| c.as_array()) {
                    cont_items.extend(ci.iter());
                }
            }
        }
        cont_items
    } else {
        Vec::new()
    };

    for section in &sections {
        if let Some(contents) = section.pointer("/itemSectionRenderer/contents") {
            if let Some(arr) = contents.as_array() {
                for item in arr {
                    if let Some(video) = item.get("videoRenderer") {
                        if let Some(search_item) = parse_video_renderer(video) {
                            items.push(search_item);
                        }
                    } else if let Some(channel) = item.get("channelRenderer") {
                        if let Some(channel_item) = parse_channel_renderer(channel) {
                            channels.push(channel_item);
                        }
                    }
                }
            }
        }

        if let Some(token) = section
            .pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")
        {
            continuation = token.as_str().map(String::from);
        }
    }

    (items, channels, continuation)
}

fn parse_video_renderer(v: &serde_json::Value) -> Option<SearchItem> {
    let video_id = v.get("videoId")?.as_str()?.to_string();

    let title = v
        .pointer("/title/runs/0/text")
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
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let duration = parse_duration(&duration_text);

    let views_text = v
        .pointer("/viewCountText/simpleText")
        .or_else(|| v.pointer("/viewCountText/runs/0/text"))
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
        .pointer("/ownerText/runs/0/text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_url = v
        .pointer("/ownerText/runs/0/navigationEndpoint/browseEndpoint/canonicalBaseUrl")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_avatar = v
        .pointer("/channelThumbnailSupportedRenderers/channelThumbnailWithLinkRenderer/thumbnail/thumbnails/0/url")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let uploader_verified = v
        .get("ownerBadges")
        .and_then(|b| b.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    Some(SearchItem {
        video_id: video_id.clone(),
        item_type: "stream".to_string(),
        url: format!("/watch?v={}", video_id),
        title,
        thumbnail,
        duration,
        duration_text,
        views,
        views_text,
        uploaded_date,
        uploader_name,
        uploader_url,
        uploader_avatar,
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

fn parse_channel_renderer(c: &serde_json::Value) -> Option<SearchChannelItem> {
    let channel_id = c.get("channelId")?.as_str()?;

    let name = c
        .pointer("/title/simpleText")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let thumbnail = c
        .pointer("/thumbnail/thumbnails")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())
        .map(|url| {
            if url.starts_with("//") {
                format!("https:{}", url)
            } else {
                url.to_string()
            }
        })
        .unwrap_or_default();

    let url = c
        .pointer("/navigationEndpoint/browseEndpoint/canonicalBaseUrl")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let subscriber_text = c
        .pointer("/subscriberCountText/simpleText")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let video_count_text = c
        .pointer("/videoCountText/runs/0/text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let description = c
        .pointer("/descriptionSnippet/runs")
        .and_then(|r| r.as_array())
        .map(|runs| {
            runs.iter()
                .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let verified = c
        .get("ownerBadges")
        .and_then(|b| b.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    Some(SearchChannelItem {
        item_type: "channel".to_string(),
        channel_id: channel_id.to_string(),
        name,
        thumbnail,
        url,
        subscriber_text,
        video_count_text,
        description,
        verified,
    })
}

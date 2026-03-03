use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

const INNERTUBE_URL: &str = "https://www.youtube.com/youtubei/v1/search";

pub fn router() -> Router<crate::AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    continuation: Option<String>,
}

#[derive(Serialize)]
struct SearchItem {
    #[serde(rename = "type")]
    item_type: String,
    url: String,
    title: String,
    thumbnail: String,
    duration: i64,
    #[serde(rename = "durationText")]
    duration_text: String,
    views: i64,
    #[serde(rename = "viewsText")]
    views_text: String,
    #[serde(rename = "uploadedDate")]
    uploaded_date: String,
    #[serde(rename = "uploaderName")]
    uploader_name: String,
    #[serde(rename = "uploaderUrl")]
    uploader_url: String,
    #[serde(rename = "uploaderAvatar")]
    uploader_avatar: String,
    #[serde(rename = "uploaderVerified")]
    uploader_verified: bool,
}

#[derive(Serialize)]
struct SearchResponse {
    items: Vec<SearchItem>,
    continuation: Option<String>,
}

async fn search(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    let q = match &query.q {
        Some(q) if !q.is_empty() => q.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };

    let mut body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240101.00.00",
                "hl": "en",
                "gl": "US"
            }
        }
    });

    if let Some(token) = &query.continuation {
        body["continuation"] = serde_json::Value::String(token.clone());
    } else {
        body["query"] = serde_json::Value::String(q);
    }

    let client = reqwest::Client::new();
    let resp = match client
        .post(INNERTUBE_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if !resp.status().is_success() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("YouTube API error: {}", resp.status()) })),
        )
            .into_response();
    }

    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let (items, continuation) = parse_innertube_response(&data);

    Json(SearchResponse {
        items,
        continuation,
    })
    .into_response()
}

fn parse_innertube_response(data: &serde_json::Value) -> (Vec<SearchItem>, Option<String>) {
    let mut items = Vec::new();
    let mut continuation = None;

    // Initial search: contents.twoColumnSearchResultsRenderer.primaryContents.sectionListRenderer.contents
    // Continuation: onResponseReceivedCommands[].appendContinuationItemsAction.continuationItems
    let sections: Vec<&serde_json::Value> = if let Some(contents) = data
        .pointer("/contents/twoColumnSearchResultsRenderer/primaryContents/sectionListRenderer/contents")
    {
        contents.as_array().map(|a| a.iter().collect()).unwrap_or_default()
    } else if let Some(commands) = data.get("onResponseReceivedCommands").and_then(|c| c.as_array()) {
        // Continuation response
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
        // Extract video results from itemSectionRenderer
        if let Some(contents) = section.pointer("/itemSectionRenderer/contents") {
            if let Some(arr) = contents.as_array() {
                for item in arr {
                    if let Some(video) = item.get("videoRenderer") {
                        if let Some(search_item) = parse_video_renderer(video) {
                            items.push(search_item);
                        }
                    }
                }
            }
        }

        // Extract continuation token
        if let Some(token) = section
            .pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")
        {
            continuation = token.as_str().map(String::from);
        }
    }

    (items, continuation)
}

fn parse_video_renderer(v: &serde_json::Value) -> Option<SearchItem> {
    let video_id = v.get("videoId")?.as_str()?;
    let title = v
        .pointer("/title/runs/0/text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let thumbnails = v.pointer("/thumbnail/thumbnails");
    let thumbnail = thumbnails
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
    // Parse "H:MM:SS" or "M:SS" or "SS" format
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
    // Parse "3,653,302 views" or "1.2M views" etc.
    let cleaned = text
        .replace(" views", "")
        .replace(" view", "")
        .replace(',', "");
    cleaned.parse().unwrap_or(0)
}

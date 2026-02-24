use anyhow::Result;
use serde_json::Value;

use super::innertube::InnertubeApi;
use crate::types::{PlaylistInfo, PlaylistVideo};

/// Extract playlist metadata and video list from YouTube.
pub async fn extract_playlist(api: &InnertubeApi, playlist_id: &str) -> Result<PlaylistInfo> {
    let browse_data = api.browse_playlist(playlist_id).await?;

    let title = extract_playlist_title(&browse_data).unwrap_or_else(|| "Unknown Playlist".into());
    let author = extract_playlist_author(&browse_data);
    let thumbnail_url = extract_playlist_thumbnail(&browse_data);

    let mut videos = Vec::new();
    let mut continuation_token: Option<String> = None;

    // Extract videos from initial response
    if let Some(contents) = get_playlist_contents(&browse_data) {
        extract_videos_from_contents(contents, &mut videos);
        continuation_token = extract_continuation_token(contents);
    }

    // Handle pagination
    while let Some(token) = continuation_token.take() {
        match api.browse_continuation(&token).await {
            Ok(cont_data) => {
                if let Some(items) = get_continuation_items(&cont_data) {
                    let prev_count = videos.len();
                    extract_videos_from_continuation(items, &mut videos);
                    continuation_token = extract_continuation_from_actions(&cont_data);

                    // Safety: stop if no new videos were added
                    if videos.len() == prev_count {
                        break;
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch playlist continuation: {}", e);
                break;
            }
        }
    }

    // Re-index videos sequentially
    for (i, video) in videos.iter_mut().enumerate() {
        video.index = i;
    }

    Ok(PlaylistInfo {
        playlist_id: playlist_id.to_string(),
        title,
        video_count: videos.len(),
        videos,
        thumbnail_url,
        author,
    })
}

fn extract_playlist_title(data: &Value) -> Option<String> {
    // Try header -> playlistHeaderRenderer -> title -> simpleText
    data.pointer("/header/playlistHeaderRenderer/title/simpleText")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            data.pointer("/metadata/playlistMetadataRenderer/title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

fn extract_playlist_author(data: &Value) -> Option<String> {
    data.pointer("/header/playlistHeaderRenderer/ownerText/runs/0/text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_playlist_thumbnail(data: &Value) -> Option<String> {
    data.pointer("/header/playlistHeaderRenderer/playlistHeaderBanner/heroPlaylistThumbnailRenderer/thumbnail/thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_playlist_contents(data: &Value) -> Option<&Vec<Value>> {
    data.pointer("/contents/twoColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/itemSectionRenderer/contents/0/playlistVideoListRenderer/contents")
        .and_then(|v| v.as_array())
}

fn get_continuation_items(data: &Value) -> Option<&Vec<Value>> {
    data.pointer("/onResponseReceivedActions/0/appendContinuationItemsAction/continuationItems")
        .and_then(|v| v.as_array())
}

fn extract_continuation_token(contents: &[Value]) -> Option<String> {
    contents
        .iter()
        .find_map(|item| {
            item.pointer("/continuationItemRenderer/continuationEndpoint/continuationCommand/token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

fn extract_continuation_from_actions(data: &Value) -> Option<String> {
    get_continuation_items(data)
        .and_then(|items| extract_continuation_token(items))
}

fn extract_videos_from_contents(contents: &[Value], videos: &mut Vec<PlaylistVideo>) {
    for item in contents {
        if let Some(renderer) = item.get("playlistVideoRenderer") {
            if let Some(video) = parse_playlist_video_renderer(renderer, videos.len()) {
                videos.push(video);
            }
        }
    }
}

fn extract_videos_from_continuation(items: &[Value], videos: &mut Vec<PlaylistVideo>) {
    for item in items {
        if let Some(renderer) = item.get("playlistVideoRenderer") {
            if let Some(video) = parse_playlist_video_renderer(renderer, videos.len()) {
                videos.push(video);
            }
        }
    }
}

fn parse_playlist_video_renderer(renderer: &Value, index: usize) -> Option<PlaylistVideo> {
    let video_id = renderer.get("videoId")?.as_str()?.to_string();

    let title = renderer
        .pointer("/title/runs/0/text")
        .or_else(|| renderer.pointer("/title/simpleText"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let duration = renderer
        .get("lengthSeconds")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let thumbnail_url = renderer
        .pointer("/thumbnail/thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(PlaylistVideo {
        video_id,
        title,
        duration,
        thumbnail_url,
        index,
    })
}

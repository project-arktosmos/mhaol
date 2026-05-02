//! YouTube channel RSS feed fetcher.
//!
//! YouTube exposes a public Atom/MediaRSS feed of a channel's most recent
//! videos at `https://www.youtube.com/feeds/videos.xml?channel_id=<UC…>`.
//! No API key is required and the response is small (~15 entries), so this
//! is a cheap way to surface a "latest from this channel" rail next to a
//! single-video detail page. The HTTP fetch is unauthenticated; callers
//! are expected to cache responses to avoid hammering the feed endpoint.
//!
//! The feed format is stable enough that a regex-based extractor works
//! reliably here without pulling in a full XML parser.

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelFeedItem {
    pub video_id: String,
    pub title: String,
    pub link: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelFeed {
    pub channel_id: String,
    pub channel_title: Option<String>,
    pub items: Vec<ChannelFeedItem>,
}

static ENTRY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<entry>(.*?)</entry>").expect("entry regex"));
static VIDEO_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<yt:videoId>([^<]+)</yt:videoId>").expect("videoId regex"));
static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<title>([^<]+)</title>").expect("title regex"));
static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<link[^>]*href="([^"]+)""#).expect("link regex"));
static THUMB_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<media:thumbnail[^>]*url="([^"]+)""#).expect("thumb regex"));
static PUBLISHED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<published>([^<]+)</published>").expect("published regex"));
static DESCRIPTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<media:description>(.*?)</media:description>").expect("description regex"));
// Channel-level title sits *before* the entries, so anchor it to the
// outermost `<title>` by capturing the first match in the part of the feed
// that precedes the first `<entry>`.
static CHANNEL_TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)^.*?<title>([^<]+)</title>").expect("channel title regex"));

/// Fetch the public Atom feed for a YouTube channel and parse the most
/// recent video entries. The channel id must be the canonical `UC…`
/// identifier; usernames / handles are not supported by the feed endpoint.
pub async fn fetch_channel_feed(channel_id: &str) -> Result<ChannelFeed> {
    if channel_id.is_empty() {
        return Err(anyhow!("channel_id is empty"));
    }
    let url = format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel_id
    );
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; mhaol-cloud/1.0)")
        .build()?;
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(anyhow!(
            "channel feed HTTP {} for {}",
            res.status(),
            channel_id
        ));
    }
    let body = res.text().await?;
    Ok(parse_feed(channel_id, &body))
}

fn parse_feed(channel_id: &str, body: &str) -> ChannelFeed {
    // Slice off everything from the first <entry> onwards so the channel
    // title regex can't accidentally match an entry's <title> instead.
    let head = match body.find("<entry>") {
        Some(idx) => &body[..idx],
        None => body,
    };
    let channel_title = CHANNEL_TITLE_RE
        .captures(head)
        .and_then(|c| c.get(1))
        .map(|m| decode_xml_entities(m.as_str().trim()));

    let mut items: Vec<ChannelFeedItem> = Vec::new();
    for entry_match in ENTRY_RE.captures_iter(body) {
        let entry = match entry_match.get(1) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let video_id = match VIDEO_ID_RE.captures(entry).and_then(|c| c.get(1)) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        let title = TITLE_RE
            .captures(entry)
            .and_then(|c| c.get(1))
            .map(|m| decode_xml_entities(m.as_str()))
            .unwrap_or_default();
        let link = LINK_RE
            .captures(entry)
            .and_then(|c| c.get(1))
            .map(|m| decode_xml_entities(m.as_str()))
            .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={video_id}"));
        let thumbnail_url = THUMB_RE
            .captures(entry)
            .and_then(|c| c.get(1))
            .map(|m| decode_xml_entities(m.as_str()));
        let published_at = PUBLISHED_RE
            .captures(entry)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());
        let description = DESCRIPTION_RE
            .captures(entry)
            .and_then(|c| c.get(1))
            .map(|m| decode_xml_entities(m.as_str().trim()))
            .filter(|s| !s.is_empty());
        items.push(ChannelFeedItem {
            video_id,
            title,
            link,
            thumbnail_url,
            published_at,
            description,
        });
    }

    ChannelFeed {
        channel_id: channel_id.to_string(),
        channel_title,
        items,
    }
}

/// Atom/Media RSS uses the standard XML entity set; we only need to
/// reverse the five predefined entities to render the values back as
/// human-readable text.
fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_feed() {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
  <title>Channel Title</title>
  <entry>
    <yt:videoId>abc123XYZ_-</yt:videoId>
    <title>First Video &amp; More</title>
    <link rel="alternate" href="https://www.youtube.com/watch?v=abc123XYZ_-"/>
    <published>2024-01-15T12:00:00+00:00</published>
    <media:group>
      <media:thumbnail url="https://i.ytimg.com/vi/abc123XYZ_-/hqdefault.jpg" width="480" height="360"/>
      <media:description>Hello world</media:description>
    </media:group>
  </entry>
  <entry>
    <yt:videoId>def456</yt:videoId>
    <title>Second Video</title>
    <link rel="alternate" href="https://www.youtube.com/watch?v=def456"/>
    <published>2024-01-10T12:00:00+00:00</published>
  </entry>
</feed>"#;
        let feed = parse_feed("UC_test", body);
        assert_eq!(feed.channel_id, "UC_test");
        assert_eq!(feed.channel_title.as_deref(), Some("Channel Title"));
        assert_eq!(feed.items.len(), 2);
        assert_eq!(feed.items[0].video_id, "abc123XYZ_-");
        assert_eq!(feed.items[0].title, "First Video & More");
        assert_eq!(
            feed.items[0].thumbnail_url.as_deref(),
            Some("https://i.ytimg.com/vi/abc123XYZ_-/hqdefault.jpg")
        );
        assert_eq!(feed.items[0].description.as_deref(), Some("Hello world"));
        assert_eq!(feed.items[1].video_id, "def456");
        assert!(feed.items[1].thumbnail_url.is_none());
    }
}

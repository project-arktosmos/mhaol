//! In-memory cache for YouTube channel-related lookups used by the catalog
//! detail pages.
//!
//! YouTube exposes a public Atom feed at
//! `https://www.youtube.com/feeds/videos.xml?channel_id=<UC…>` that lists a
//! channel's most recent videos. The endpoint is unauthenticated and
//! rate-limited at the network layer, so the catalog routes that surface a
//! "latest from this channel" rail can't safely hit it on every page load.
//!
//! This module holds two cache layers behind a shared lock:
//!
//! - **video id → channel id**, long TTL (24 h). Channel ownership is
//!   effectively stable for any given video, so a long entry life is fine.
//!   Resolving the channel id requires a relatively heavy
//!   `extract_stream_urls` call to YouTube's player API; the cache avoids
//!   doing that on every visit.
//! - **channel id → parsed feed**, short TTL (15 min). The Atom feed only
//!   gets new entries when the channel uploads, so a short-but-not-tiny
//!   TTL is plenty fresh while keeping the request volume to YouTube
//!   bounded even if a popular channel page is visited frequently.
//!
//! The cache is purely process-local — restarting the cloud bin clears it.
//! That's deliberate: the data is a presentation convenience, not a
//! correctness invariant.

#![cfg(not(target_os = "android"))]

use mhaol_yt_dlp::ChannelFeed;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const VIDEO_TO_CHANNEL_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const CHANNEL_FEED_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone)]
struct CachedChannelId {
    channel_id: String,
    inserted_at: Instant,
}

#[derive(Clone)]
struct CachedFeed {
    feed: ChannelFeed,
    inserted_at: Instant,
}

#[derive(Clone, Default)]
pub struct YoutubeChannelCache {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    video_to_channel: HashMap<String, CachedChannelId>,
    channel_feed: HashMap<String, CachedFeed>,
}

impl YoutubeChannelCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the cached channel id for a video. Returns `None` if the
    /// entry is missing or has aged past `VIDEO_TO_CHANNEL_TTL`.
    pub fn get_channel_id(&self, video_id: &str) -> Option<String> {
        let guard = self.inner.read();
        let entry = guard.video_to_channel.get(video_id)?;
        if entry.inserted_at.elapsed() > VIDEO_TO_CHANNEL_TTL {
            return None;
        }
        Some(entry.channel_id.clone())
    }

    pub fn put_channel_id(&self, video_id: &str, channel_id: &str) {
        let mut guard = self.inner.write();
        guard.video_to_channel.insert(
            video_id.to_string(),
            CachedChannelId {
                channel_id: channel_id.to_string(),
                inserted_at: Instant::now(),
            },
        );
    }

    /// Look up the cached parsed feed for a channel. Returns `None` if the
    /// entry is missing or has aged past `CHANNEL_FEED_TTL`.
    pub fn get_feed(&self, channel_id: &str) -> Option<ChannelFeed> {
        let guard = self.inner.read();
        let entry = guard.channel_feed.get(channel_id)?;
        if entry.inserted_at.elapsed() > CHANNEL_FEED_TTL {
            return None;
        }
        Some(entry.feed.clone())
    }

    pub fn put_feed(&self, channel_id: &str, feed: ChannelFeed) {
        let mut guard = self.inner.write();
        guard.channel_feed.insert(
            channel_id.to_string(),
            CachedFeed {
                feed,
                inserted_at: Instant::now(),
            },
        );
    }
}

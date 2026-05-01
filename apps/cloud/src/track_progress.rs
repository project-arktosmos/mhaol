use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Per-track resolution status. Mirrors the WebUI's `EntryStatus` so the
/// progress endpoint's payload can be projected straight onto the
/// existing `TrackEntry` shape without translation.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrackStatus {
    Pending,
    Searching,
    Found,
    Missing,
    Error,
}

/// Inline lyrics body we hand back through the progress endpoint. Same
/// shape as the JSON the server stores in a `lyrics`-typed `FileEntry` —
/// the frontend already knows how to decode this into a `SubsLyricsItem`,
/// so it can drop straight into `TrackEntry.lyrics` and the inline
/// preview works the moment the track resolves (well before the firkin
/// is rolled forward).
#[derive(Clone, Debug, Serialize)]
pub struct LyricsProgress {
    pub source: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
    #[serde(rename = "syncedLyrics", skip_serializing_if = "Option::is_none")]
    pub synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics", skip_serializing_if = "Option::is_none")]
    pub plain_lyrics: Option<String>,
    pub instrumental: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrackProgressEntry {
    pub position: i64,
    pub title: String,
    #[serde(rename = "lengthMs", skip_serializing_if = "Option::is_none")]
    pub length_ms: Option<i64>,
    #[serde(rename = "youtubeStatus")]
    pub youtube_status: TrackStatus,
    #[serde(rename = "youtubeUrl", skip_serializing_if = "Option::is_none")]
    pub youtube_url: Option<String>,
    #[serde(rename = "lyricsStatus")]
    pub lyrics_status: TrackStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<LyricsProgress>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AlbumProgress {
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed: bool,
    /// Set after the firkin has been rolled forward to its content-addressed
    /// final id. The WebUI navigates to this id when it sees `completed`
    /// flip from false to true.
    #[serde(rename = "completedId", skip_serializing_if = "Option::is_none")]
    pub completed_id: Option<String>,
    pub tracks: Vec<TrackProgressEntry>,
}

/// In-memory shared progress state. Keyed by firkin id (the id at the
/// time the resolver was spawned — i.e. the bookmark id, before any
/// rollforward). Lives on `CloudState` and is read from the
/// `/api/firkins/:id/resolution-progress` handler. Entries are kept
/// after completion so the WebUI's polling loop can discover the
/// `completedId` even if its first poll lands a tick after the rollforward.
#[derive(Clone, Default)]
pub struct AlbumProgressMap {
    inner: Arc<RwLock<HashMap<String, AlbumProgress>>>,
}

impl AlbumProgressMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, firkin_id: &str) -> Option<AlbumProgress> {
        self.inner.read().get(firkin_id).cloned()
    }

    pub fn insert(&self, firkin_id: String, progress: AlbumProgress) {
        self.inner.write().insert(firkin_id, progress);
    }

    /// Update an in-flight album's progress through a closure. No-op
    /// when the entry has already been removed (e.g. after cleanup).
    pub fn update<F: FnOnce(&mut AlbumProgress)>(&self, firkin_id: &str, f: F) {
        let mut map = self.inner.write();
        if let Some(p) = map.get_mut(firkin_id) {
            f(p);
            p.updated_at = Utc::now();
        }
    }

    /// Drop a completed entry. Called on a timer / new resolution to
    /// keep the map bounded.
    pub fn remove(&self, firkin_id: &str) {
        self.inner.write().remove(firkin_id);
    }
}

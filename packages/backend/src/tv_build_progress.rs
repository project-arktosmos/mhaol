//! Live progress map for the background TV-show firkin builder. A library
//! row in the WebUI may have several "Match TMDB & build firkin" jobs
//! running concurrently (one per show group); each job's state lives here
//! keyed by `<library_id>::<show>::<year?>` so the libraries page can
//! re-hydrate its in-progress badge after a refresh.
//!
//! Survives the orchestrator: the entry is left in the map after
//! completion so a poll that lands a tick after the task finishes still
//! sees `phase == Completed` and the resulting firkin id. A periodic GC
//! pass drops stale terminal entries (see `gc`).

#![cfg(not(target_os = "android"))]

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TvBuildPhase {
    Searching,
    FetchingSeasons,
    FetchingEpisodes,
    FetchingMetadata,
    WaitingPins,
    CreatingFirkin,
    Completed,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct TvBuildProgress {
    #[serde(rename = "libraryId")]
    pub library_id: String,
    #[serde(rename = "jobKey")]
    pub job_key: String,
    pub show: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    pub phase: TvBuildPhase,
    /// Free-form status line shown in the UI ("Matched <title> (year) —
    /// fetching seasons…"). `phase` is the machine-readable signal;
    /// `message` is the human-readable one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// `current / total` for phases that have a meaningful denominator
    /// (episode fetch loop, pin wait loop). Both `None` for phases that
    /// don't.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    #[serde(rename = "tmdbId", skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<String>,
    #[serde(rename = "tmdbTitle", skip_serializing_if = "Option::is_none")]
    pub tmdb_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "completedFirkinId", skip_serializing_if = "Option::is_none")]
    pub completed_firkin_id: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl TvBuildProgress {
    pub fn is_terminal(&self) -> bool {
        matches!(self.phase, TvBuildPhase::Completed | TvBuildPhase::Error)
    }
}

#[derive(Clone, Default)]
pub struct TvBuildProgressMap {
    inner: Arc<RwLock<HashMap<String, TvBuildProgress>>>,
}

impl TvBuildProgressMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, job_key: &str) -> Option<TvBuildProgress> {
        self.inner.read().get(job_key).cloned()
    }

    pub fn list_for_library(&self, library_id: &str) -> Vec<TvBuildProgress> {
        let mut out: Vec<TvBuildProgress> = self
            .inner
            .read()
            .values()
            .filter(|p| p.library_id == library_id)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.started_at.cmp(&b.started_at));
        out
    }

    pub fn insert(&self, progress: TvBuildProgress) {
        self.inner
            .write()
            .insert(progress.job_key.clone(), progress);
    }

    /// Update an in-flight job's progress through a closure. No-op when the
    /// entry has already been removed (e.g. cleared by GC after a long
    /// terminal stay).
    pub fn update<F: FnOnce(&mut TvBuildProgress)>(&self, job_key: &str, f: F) {
        let mut map = self.inner.write();
        if let Some(p) = map.get_mut(job_key) {
            f(p);
            p.updated_at = Utc::now();
        }
    }

    pub fn remove(&self, job_key: &str) {
        self.inner.write().remove(job_key);
    }

    /// Drop terminal entries older than `ttl`. The active orchestrator
    /// keeps each entry alive by stamping `updated_at` on every transition,
    /// so this only ever culls jobs that have been in `Completed` /
    /// `Error` longer than the TTL.
    pub fn gc(&self, ttl: chrono::Duration) {
        let now = Utc::now();
        self.inner
            .write()
            .retain(|_, p| !p.is_terminal() || now - p.updated_at < ttl);
    }
}

use crate::error::{Error, Result};
use crate::pipeline::{build_hls_pipeline, HlsPipeline, HlsPipelineConfig};
use crate::session::{SessionInfo, SessionRecord, SessionState};
use gstreamer as gst;
use gstreamer::prelude::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Manages a pool of running HLS transcoding sessions.
///
/// Each session reads a single IPFS-pinned file off disk, transcodes it
/// into HLS (H.264 + AAC, MPEG-TS segments) under a per-session directory
/// inside `base_dir`, and exposes the path to the playlist + segments for
/// the cloud HTTP server to serve.
pub struct IpfsStreamManager {
    base_dir: PathBuf,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    sessions: HashMap<String, SessionEntry>,
}

struct SessionEntry {
    record: SessionRecord,
    pipeline: HlsPipeline,
}

/// A handle returned by `start_session`.
#[derive(Debug, Clone)]
pub struct StartedSession {
    pub session_id: String,
    pub playlist_path: PathBuf,
    pub output_dir: PathBuf,
    pub playlist_name: String,
}

impl IpfsStreamManager {
    /// Create a manager that stores per-session HLS output under `base_dir`.
    /// The directory is created on first session start.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            inner: Arc::new(Mutex::new(Inner {
                sessions: HashMap::new(),
            })),
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Start a new HLS session that transcodes `source_path` into
    /// `<base_dir>/<session_id>/`. The CID is recorded on the session for
    /// observability but is otherwise opaque to the transcoder — the actual
    /// bytes come from the local filesystem path (which the caller has
    /// pinned via `mhaol-ipfs`).
    pub fn start_session(&self, cid: String, source_path: PathBuf) -> Result<StartedSession> {
        if !source_path.exists() {
            return Err(Error::SourceNotFound(source_path.display().to_string()));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let output_dir = self.base_dir.join(&session_id);
        std::fs::create_dir_all(&output_dir)?;

        let config = HlsPipelineConfig::new(source_path.clone(), output_dir.clone());
        let playlist_name = config.playlist_name.clone();

        let pipeline = build_hls_pipeline(config)?;
        let playlist_path = pipeline.playlist_path();
        pipeline.play()?;

        let record = SessionRecord {
            session_id: session_id.clone(),
            cid,
            source_path,
            output_dir: output_dir.clone(),
            playlist_name: playlist_name.clone(),
            state: SessionState::Running,
        };

        spawn_bus_watcher(
            pipeline.pipeline().clone(),
            session_id.clone(),
            Arc::clone(&self.inner),
        );

        self.inner.lock().sessions.insert(
            session_id.clone(),
            SessionEntry { record, pipeline },
        );

        Ok(StartedSession {
            session_id,
            playlist_path,
            output_dir,
            playlist_name,
        })
    }

    /// Stop an active session. Removes the session record but leaves the
    /// generated files on disk; the caller can clean up the output dir.
    pub fn stop_session(&self, session_id: &str) -> Result<()> {
        let entry = self
            .inner
            .lock()
            .sessions
            .remove(session_id)
            .ok_or_else(|| Error::SessionNotFound(session_id.to_string()))?;
        entry.pipeline.stop()?;
        Ok(())
    }

    /// Stop every running session. Best-effort.
    pub fn stop_all(&self) {
        let entries: Vec<SessionEntry> =
            self.inner.lock().sessions.drain().map(|(_, v)| v).collect();
        for entry in entries {
            let _ = entry.pipeline.stop();
        }
    }

    /// Snapshot of all live sessions.
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.inner
            .lock()
            .sessions
            .values()
            .map(|e| e.record.snapshot())
            .collect()
    }

    pub fn get_session(&self, session_id: &str) -> Option<SessionInfo> {
        self.inner
            .lock()
            .sessions
            .get(session_id)
            .map(|e| e.record.snapshot())
    }

    /// Query the source media's total duration in seconds by polling the
    /// running pipeline's duration query. Returns `None` if the pipeline is
    /// gone or the duration is still unknown after `timeout`. Useful so the
    /// player can show a finite total even though the rolling HLS playlist
    /// looks live (no `#EXT-X-ENDLIST`) until transcoding finishes.
    pub fn query_source_duration(&self, session_id: &str, timeout: Duration) -> Option<f64> {
        let pipeline = self
            .inner
            .lock()
            .sessions
            .get(session_id)
            .map(|e| e.pipeline.pipeline().clone())?;
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if let Some(d) = pipeline.query_duration::<gst::ClockTime>() {
                let secs = d.nseconds() as f64 / 1_000_000_000.0;
                if secs > 0.0 {
                    return Some(secs);
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }

    /// Wait (blocking the current thread, polling) for the playlist file to
    /// be written and contain at least one segment reference. Returns `true`
    /// if the playlist is ready within `timeout`, `false` otherwise.
    /// hlssink2 only flushes the playlist after the first segment closes,
    /// so callers that want to start serving as soon as playback is feasible
    /// should poll on this before redirecting the player at the playlist.
    pub fn wait_for_playlist(&self, session_id: &str, timeout: Duration) -> bool {
        let path = match self
            .inner
            .lock()
            .sessions
            .get(session_id)
            .map(|e| e.pipeline.playlist_path())
        {
            Some(p) => p,
            None => return false,
        };
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if playlist_has_segment(&path) {
                return true;
            }
            thread::sleep(Duration::from_millis(100));
        }
        false
    }
}

fn playlist_has_segment(path: &Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(s) => s.lines().any(|l| !l.trim().is_empty() && !l.starts_with('#')),
        Err(_) => false,
    }
}

fn spawn_bus_watcher(pipeline: gst::Pipeline, session_id: String, inner: Arc<Mutex<Inner>>) {
    thread::spawn(move || {
        let bus = match pipeline.bus() {
            Some(b) => b,
            None => return,
        };
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(_) => {
                    if let Some(entry) = inner.lock().sessions.get_mut(&session_id) {
                        entry.record.state = SessionState::Finished;
                    }
                    break;
                }
                MessageView::Error(err) => {
                    tracing::error!(
                        "[ipfs-stream] session {session_id} pipeline error: {} ({:?})",
                        err.error(),
                        err.debug()
                    );
                    if let Some(entry) = inner.lock().sessions.get_mut(&session_id) {
                        entry.record.state = SessionState::Error;
                    }
                    break;
                }
                _ => {}
            }
        }
    });
}

impl Drop for IpfsStreamManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

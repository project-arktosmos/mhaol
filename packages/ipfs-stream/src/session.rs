use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Lifecycle state of an HLS session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Pipeline created but not yet started.
    Pending,
    /// Pipeline is encoding and writing segments.
    Running,
    /// Pipeline reached EOS (end of source) — playlist is complete.
    Finished,
    /// Pipeline errored out. The playlist may be partial.
    Error,
}

/// Snapshot of an active HLS session for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
    pub cid: String,
    pub source_path: String,
    pub output_dir: String,
    pub playlist_name: String,
    pub state: SessionState,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionRecord {
    pub session_id: String,
    pub cid: String,
    pub source_path: PathBuf,
    pub output_dir: PathBuf,
    pub playlist_name: String,
    pub state: SessionState,
}

impl SessionRecord {
    pub(crate) fn snapshot(&self) -> SessionInfo {
        SessionInfo {
            session_id: self.session_id.clone(),
            cid: self.cid.clone(),
            source_path: self.source_path.display().to_string(),
            output_dir: self.output_dir.display().to_string(),
            playlist_name: self.playlist_name.clone(),
            state: self.state,
        }
    }
}

//! `/api/player/*` — thin stubs so `playerService.initialize()` succeeds when
//! it runs in the cloud webui. The cloud does not own a local library of
//! playable files (that lives in node), but the right-side aside in the cloud
//! webui still hosts `PlayerVideo`/`SubsLyricsFinder` and uses
//! `playerService.playUrl()` (yt-dlp direct streams) and `playRemote()`
//! (P2P-streamed IPFS pins via `/api/p2p-stream/sessions`). Neither of those
//! paths needs a stream-status flag or a `playable` list, but the service's
//! initializer hits both endpoints unconditionally — these stubs let it
//! settle without the user seeing an error toast.

use crate::state::CloudState;
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct StreamStatus {
    available: bool,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/stream-status", get(stream_status))
        .route("/playable", get(playable))
}

async fn stream_status() -> Json<StreamStatus> {
    Json(StreamStatus { available: false })
}

async fn playable() -> Json<Vec<serde_json::Value>> {
    Json(Vec::new())
}

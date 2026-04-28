//! Optional Axum router that exposes the manager directly. The main node app
//! defines its own routes under `/api/ed2k` against `AppState`; this module
//! is kept for parity with the torrent crate so embedders can mount the
//! manager standalone.

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use crate::Ed2kManager;

type AppState = Arc<Ed2kManager>;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/files", get(list))
}

async fn status(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "initialized": mgr.is_initialized(),
        "downloadPath": mgr.download_path().to_string_lossy(),
        "stats": mgr.stats(),
        "server": mgr.server().map(|s| serde_json::json!({
            "name": s.name,
            "host": s.host,
            "port": s.port,
            "userCount": s.user_count,
            "fileCount": s.file_count,
        })),
    }))
}

async fn list(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_builds() {
        let mgr = Arc::new(Ed2kManager::new());
        let _r: Router = router().with_state(mgr);
    }
}

use axum::{extract::State, routing::get, Json, Router};
use mhaol_node::AppState;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const STARTED_AT: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
});

#[derive(Serialize)]
struct CloudStatus {
    status: &'static str,
    version: &'static str,
    started_at: u64,
    now: u64,
    uptime_seconds: u64,
    host: String,
    port: u16,
    local_ip: Option<String>,
    signaling_address: Option<String>,
    client_address: Option<String>,
    library_count: usize,
    queue_depth: usize,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/status", get(status))
}

async fn status(State(state): State<AppState>) -> Json<CloudStatus> {
    let _ = *STARTED_AT;
    let started_at = *STARTED_AT;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(started_at);
    let uptime_seconds = if now > started_at {
        (now - started_at) / 1000
    } else {
        0
    };

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1540);
    let local_ip = mhaol_node::api::network::get_local_ip();

    let signaling_address = state
        .identity_manager
        .get_address("SIGNALING_WALLET")
        .map(|a| mhaol_identity::to_eip55_checksum(&a));
    let client_address = state
        .identity_manager
        .get_address("CLIENT_WALLET")
        .map(|a| mhaol_identity::to_eip55_checksum(&a));

    let library_count = state.libraries.get_all().len();
    let queue_depth = state.queue.list(None, None).len();

    Json(CloudStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        started_at,
        now,
        uptime_seconds,
        host,
        port,
        local_ip,
        signaling_address,
        client_address,
        library_count,
        queue_depth,
    })
}

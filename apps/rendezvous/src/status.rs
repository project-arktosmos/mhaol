use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::state::RendezvousState;

pub fn router() -> Router<RendezvousState> {
    Router::new().route("/", get(status))
}

pub fn health_router() -> Router<RendezvousState> {
    Router::new().route("/", get(health))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusBody {
    role: &'static str,
    ipfs: mhaol_ipfs::IpfsStats,
    bootstrap_multiaddrs: Vec<String>,
    turn_configured: bool,
}

async fn status(State(state): State<RendezvousState>) -> Json<StatusBody> {
    let ipfs = state.ipfs.stats().await;
    let peer_id = ipfs.peer_id.clone();
    let bootstrap_multiaddrs = ipfs
        .listen_addrs
        .iter()
        .filter_map(|addr| {
            let pid = peer_id.as_deref()?;
            if addr.contains("/p2p/") {
                Some(addr.clone())
            } else {
                Some(format!("{addr}/p2p/{pid}"))
            }
        })
        .collect();
    Json(StatusBody {
        role: "rendezvous",
        ipfs,
        bootstrap_multiaddrs,
        turn_configured: state.turn.is_configured(),
    })
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mhaol-rendezvous",
    }))
}

//! Library surface of `mhaol-rendezvous`.
//!
//! Exposes the building blocks of the rendezvous server (router, room
//! manager, TURN credentials, DHT-backed signaling) so integration tests can
//! exercise the WebSocket relay without booting the embedded IPFS node.

pub mod config;
pub mod health_check;
pub mod rooms;
pub mod setup;
pub mod signaling;
pub mod state;
pub mod status;
pub mod turn;
pub mod ws;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

use crate::state::RendezvousState;

/// Build the full rendezvous Axum router from a prepared state. Used by
/// `mhaol-rendezvous serve` and by integration tests.
pub fn build_router(state: RendezvousState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/status", status::router())
        .nest("/api/health", status::health_router())
        .route("/party/{room_id}", get(ws::ws_handler))
        .route("/party/{room_id}/status", get(ws::room_status))
        .route(
            "/api/v1/turn/credentials",
            get(turn::turn_credentials_handler),
        )
        .nest("/signal", signaling::router())
        .with_state(state)
        .layer(cors)
}

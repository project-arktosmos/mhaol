use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{AddIpfsRequest, IpfsManager};

pub type AppState = Arc<IpfsManager>;

/// Minimal HTTP router for the IPFS package. Mount with
/// `Router::new().nest("/api/ipfs", mhaol_ipfs::api::router())` and pass an
/// `Arc<IpfsManager>` as the router state.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(stats))
        .route("/peers", get(peers))
        .route("/pins", get(list_pins))
        .route("/add", post(add))
        .route("/pin/:cid", post(pin_cid))
        .route("/unpin/:cid", post(unpin_cid))
}

async fn stats(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.stats().await)
}

async fn peers(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.peers().await {
        Ok(p) => Json(p).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response(),
    }
}

async fn list_pins(State(mgr): State<AppState>) -> impl IntoResponse {
    match mgr.list_pins().await {
        Ok(pins) => Json(pins).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct AddBody {
    source: String,
    pin: Option<bool>,
}

async fn add(
    State(mgr): State<AppState>,
    Json(body): Json<AddBody>,
) -> impl IntoResponse {
    let req = AddIpfsRequest {
        source: body.source,
        pin: body.pin,
    };
    match mgr.add(req).await {
        Ok(info) => Json(info).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn pin_cid(
    State(mgr): State<AppState>,
    Path(cid): Path<String>,
) -> impl IntoResponse {
    match mgr.pin(&cid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn unpin_cid(
    State(mgr): State<AppState>,
    Path(cid): Path<String>,
) -> impl IntoResponse {
    match mgr.unpin(&cid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

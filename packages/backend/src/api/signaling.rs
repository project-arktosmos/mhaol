use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, head, post},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/wallet", get(get_wallet).delete(regenerate_wallet))
        .route("/wallet/sign", post(sign_message))
        .route("/deploy", head(deploy_status).get(deploy))
}

/// GET /api/signaling/status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let identity_address = state.identity_manager.get_default_address();
    let party_url = state
        .settings
        .get("signaling.partyUrl")
        .unwrap_or_default();
    let deploy_name = state
        .settings
        .get("signaling.deployName")
        .unwrap_or_default();

    // Check if the deployed URL is reachable
    let deployed_available = if !party_url.is_empty() {
        check_url_available(&party_url).await
    } else {
        false
    };

    Json(serde_json::json!({
        "devAvailable": false,
        "deployedAvailable": deployed_available,
        "devUrl": "",
        "partyUrl": party_url,
        "deployName": deploy_name,
        "identityAddress": identity_address,
    }))
}

/// GET /api/signaling/wallet
async fn get_wallet(State(state): State<AppState>) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let address = state.identity_manager.ensure_identity(name);
    Json(serde_json::json!({
        "name": name,
        "address": address,
    }))
}

/// DELETE /api/signaling/wallet
async fn regenerate_wallet(State(state): State<AppState>) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let address = state.identity_manager.regenerate(name);
    Json(serde_json::json!({
        "name": name,
        "address": address,
    }))
}

#[derive(Deserialize)]
struct SignMessageBody {
    message: String,
}

/// POST /api/signaling/wallet/sign
async fn sign_message(
    State(state): State<AppState>,
    Json(body): Json<SignMessageBody>,
) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let pk = match state.identity_manager.get_private_key(name) {
        Some(pk) => pk,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Wallet not found" })),
            )
                .into_response()
        }
    };

    let signature = crate::identity::passport::eip191_sign(&body.message, &pk);
    Json(serde_json::json!({ "signature": signature })).into_response()
}

/// HEAD /api/signaling/deploy — check if a deploy is in progress
async fn deploy_status() -> impl IntoResponse {
    // No deploy mechanism in Rust backend yet
    StatusCode::NO_CONTENT
}

/// GET /api/signaling/deploy — not implemented (requires PartyKit CLI)
async fn deploy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({ "error": "Deploy not available in Rust backend" })),
    )
}

async fn check_url_available(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    match client {
        Ok(c) => c.head(url).send().await.map(|r| r.status().is_success()).unwrap_or(false),
        Err(_) => false,
    }
}

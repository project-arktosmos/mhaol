use crate::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/auth", get(get_auth))
}

/// GET /api/signaling/status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let dev_available = state.signaling_rooms.is_available();
    let dev_url = if dev_available {
        state.signaling_rooms.dev_url()
    } else {
        String::new()
    };

    Json(serde_json::json!({
        "devAvailable": dev_available,
        "devUrl": dev_url,
    }))
}

#[derive(Deserialize)]
struct AuthQuery {
    room_id: String,
    timestamp: String,
}

/// GET /api/signaling/auth?room_id=X&timestamp=Y
///
/// Signs a signaling auth challenge using the server's default identity.
/// Returns the signed challenge + passport data for connecting to the signaling server.
async fn get_auth(
    State(state): State<AppState>,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    let identities = state.identity_manager.get_all();
    let Some((name, _)) = identities.into_iter().next() else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "No identity configured" })),
        )
            .into_response();
    };

    let Some(private_key) = state.identity_manager.get_private_key(&name) else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to load identity key" })),
        )
            .into_response();
    };

    let Some(passport) = state.identity_manager.get_passport(&name) else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to build passport" })),
        )
            .into_response();
    };

    let address = mhaol_identity::passport::eip191_recover(&passport.raw, &passport.signature)
        .unwrap_or_default();

    let message = format!("partykit-auth:{}:{}", query.room_id, query.timestamp);
    let signature = mhaol_identity::eip191_sign(&message, &private_key);

    Json(serde_json::json!({
        "address": address,
        "signature": signature,
        "passport": {
            "raw": passport.raw,
            "signature": passport.signature,
        },
    }))
    .into_response()
}

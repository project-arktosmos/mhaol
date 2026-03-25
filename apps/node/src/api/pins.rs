use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_pins).post(add_pin).delete(remove_pin))
        .route("/count", get(count_pins))
}

#[derive(Deserialize)]
struct WalletQuery {
    wallet: String,
}

async fn list_pins(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    Json(state.pins.get_by_wallet(&q.wallet))
}

async fn count_pins(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    let count = state.pins.count_by_wallet(&q.wallet);
    Json(serde_json::json!({ "count": count }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPinBody {
    wallet: String,
    service: String,
    service_id: String,
    label: String,
}

async fn add_pin(
    State(state): State<AppState>,
    Json(body): Json<AddPinBody>,
) -> impl IntoResponse {
    state
        .pins
        .insert(&body.wallet, &body.service, &body.service_id, &body.label);
    StatusCode::CREATED
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePinBody {
    wallet: String,
    service: String,
    service_id: String,
}

async fn remove_pin(
    State(state): State<AppState>,
    Json(body): Json<RemovePinBody>,
) -> impl IntoResponse {
    if state
        .pins
        .delete(&body.wallet, &body.service, &body.service_id)
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

use crate::AppState;
use axum::{
    extract::State,
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

async fn list_pins(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.pins.get_all())
}

async fn count_pins(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.pins.count();
    Json(serde_json::json!({ "count": count }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddPinBody {
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
        .insert(&body.service, &body.service_id, &body.label);
    StatusCode::CREATED
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePinBody {
    service: String,
    service_id: String,
}

async fn remove_pin(
    State(state): State<AppState>,
    Json(body): Json<RemovePinBody>,
) -> impl IntoResponse {
    if state.pins.delete(&body.service, &body.service_id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

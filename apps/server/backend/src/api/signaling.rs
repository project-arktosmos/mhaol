use crate::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/status", get(get_status))
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

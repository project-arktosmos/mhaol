use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_addons))
        .route("/settings", put(update_setting))
}

async fn list_addons(State(state): State<AppState>) -> impl IntoResponse {
    let registry = state.module_registry.read();
    let all_status = registry.get_status(&state);
    // Filter to only addon-sourced modules
    let addons: Vec<_> = all_status
        .into_iter()
        .filter(|s| s.source == "addon")
        .collect();
    Json(addons)
}

#[derive(Deserialize)]
struct UpdateSettingBody {
    addon: Option<String>,
    key: Option<String>,
    value: Option<String>,
}

async fn update_setting(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingBody>,
) -> impl IntoResponse {
    let addon = match &body.addon {
        Some(a) if !a.is_empty() => a,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing \"addon\" field" })),
            )
                .into_response()
        }
    };

    let key = match &body.key {
        Some(k) if !k.is_empty() => k,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing \"key\" field" })),
            )
                .into_response()
        }
    };

    let value = match &body.value {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing \"value\" field" })),
            )
                .into_response()
        }
    };

    let registry = state.module_registry.read();
    if registry.update_setting(&state, addon, key, value) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Addon or setting not found" })),
        )
            .into_response()
    }
}

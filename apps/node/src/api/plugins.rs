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
        .route("/", get(get_plugins))
        .route("/settings", put(update_setting))
}

async fn get_plugins(State(state): State<AppState>) -> impl IntoResponse {
    let registry = state.module_registry.read();
    let status = registry.get_status(&state);
    Json(status)
}

#[derive(Deserialize)]
struct UpdateSettingBody {
    plugin: Option<String>,
    key: Option<String>,
    value: Option<String>,
}

async fn update_setting(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingBody>,
) -> impl IntoResponse {
    let plugin = match &body.plugin {
        Some(p) if !p.is_empty() => p,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing \"plugin\" field" })),
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
    if registry.update_setting(&state, plugin, key, value) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Plugin or setting not found" })),
        )
            .into_response()
    }
}

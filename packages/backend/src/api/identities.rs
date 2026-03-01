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
    Router::new().route("/", get(list_identities).post(create_identity))
}

async fn list_identities(State(state): State<AppState>) -> impl IntoResponse {
    let identities = state.identity_manager.get_all();
    let entries: Vec<serde_json::Value> = identities
        .into_iter()
        .map(|(name, address)| {
            let passport = state.identity_manager.get_passport(&name);
            let passport_json = passport.map(|p| {
                serde_json::json!({
                    "raw": p.raw,
                    "hash": p.hash,
                    "signature": p.signature,
                })
                .to_string()
            });
            serde_json::json!({
                "name": name,
                "address": address,
                "passport": passport_json,
            })
        })
        .collect();
    Json(entries)
}

#[derive(Deserialize)]
struct CreateIdentityBody {
    name: Option<String>,
}

async fn create_identity(
    State(state): State<AppState>,
    Json(body): Json<CreateIdentityBody>,
) -> impl IntoResponse {
    let name = match &body.name {
        Some(n) if !n.trim().is_empty() => n
            .trim()
            .to_uppercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing or invalid \"name\" field" })),
            )
                .into_response()
        }
    };

    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Name must contain at least one alphanumeric character" })),
        )
            .into_response();
    }

    if state.identity_manager.get_address(&name).is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("Identity \"{}\" already exists", name) })),
        )
            .into_response();
    }

    let address = state.identity_manager.regenerate(&name);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "name": name, "address": address })),
    )
        .into_response()
}

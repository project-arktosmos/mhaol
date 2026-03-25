use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_profiles).post(add_profile))
        .route("/{wallet}", get(get_profile))
}

async fn list_profiles(State(state): State<AppState>) -> impl IntoResponse {
    let profiles = state.profiles.get_all();
    Json(profiles)
}

#[derive(Deserialize)]
struct AddProfileBody {
    username: String,
    wallet: String,
}

async fn get_profile(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> impl IntoResponse {
    match state.profiles.get_by_wallet(&wallet) {
        Some(profile) => {
            let favorites = state.favorites.get_by_wallet(&wallet);
            Json(serde_json::json!({
                "profile": profile,
                "favorites": favorites
            }))
            .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn add_profile(
    State(state): State<AppState>,
    Json(body): Json<AddProfileBody>,
) -> impl IntoResponse {
    state.profiles.upsert(&body.username, &body.wallet);
    StatusCode::CREATED
}

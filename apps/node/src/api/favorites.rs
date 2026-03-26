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
        .route("/", get(list_favorites).post(add_favorite).delete(remove_favorite))
        .route("/count", get(count_favorites))
}

#[derive(Deserialize)]
struct WalletQuery {
    wallet: String,
}

async fn list_favorites(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    Json(state.favorites.get_by_wallet(&q.wallet))
}

async fn count_favorites(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    let count = state.favorites.count_by_wallet(&q.wallet);
    Json(serde_json::json!({ "count": count }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddFavoriteBody {
    wallet: String,
    service: String,
    service_id: String,
    label: String,
}

async fn add_favorite(
    State(state): State<AppState>,
    Json(body): Json<AddFavoriteBody>,
) -> impl IntoResponse {
    if state
        .favorites
        .insert(&body.wallet, &body.service, &body.service_id, &body.label)
    {
        StatusCode::CREATED
    } else {
        StatusCode::CONFLICT
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveFavoriteBody {
    wallet: String,
    service: String,
    service_id: String,
}

async fn remove_favorite(
    State(state): State<AppState>,
    Json(body): Json<RemoveFavoriteBody>,
) -> impl IntoResponse {
    if state
        .favorites
        .delete(&body.wallet, &body.service, &body.service_id)
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

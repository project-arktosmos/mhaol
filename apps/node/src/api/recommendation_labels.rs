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
        .route("/definitions", get(list_labels))
        .route(
            "/",
            get(get_assignments)
                .put(set_assignment)
                .delete(remove_assignment),
        )
}

async fn list_labels(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.recommendation_labels.list_labels())
}

#[derive(Deserialize)]
struct WalletQuery {
    wallet: String,
}

async fn get_assignments(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    Json(state.recommendation_labels.get_assignments_by_wallet(&q.wallet))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetAssignmentBody {
    wallet: String,
    source: String,
    source_id: String,
    source_type: String,
    label_id: String,
}

async fn set_assignment(
    State(state): State<AppState>,
    Json(body): Json<SetAssignmentBody>,
) -> impl IntoResponse {
    if state.recommendation_labels.upsert(
        &body.wallet,
        &body.source,
        &body.source_id,
        &body.source_type,
        &body.label_id,
    ) {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveAssignmentBody {
    wallet: String,
    source: String,
    source_id: String,
    source_type: String,
}

async fn remove_assignment(
    State(state): State<AppState>,
    Json(body): Json<RemoveAssignmentBody>,
) -> impl IntoResponse {
    if state.recommendation_labels.delete(
        &body.wallet,
        &body.source,
        &body.source_id,
        &body.source_type,
    ) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

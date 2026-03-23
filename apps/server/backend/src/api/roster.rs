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
        .route("/", get(list_contacts).post(add_contact))
        .route("/{address}", axum::routing::delete(remove_contact))
}

async fn list_contacts(State(state): State<AppState>) -> impl IntoResponse {
    let contacts = state.roster_contacts.get_all();
    Json(contacts)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddContactBody {
    name: String,
    address: String,
    passport: Option<String>,
    instance_type: Option<String>,
}

async fn add_contact(
    State(state): State<AppState>,
    Json(body): Json<AddContactBody>,
) -> impl IntoResponse {
    state.roster_contacts.insert(
        &body.address,
        &body.name,
        body.passport.as_deref(),
        body.instance_type.as_deref(),
    );
    StatusCode::CREATED
}

async fn remove_contact(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    if state.roster_contacts.delete(&address) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

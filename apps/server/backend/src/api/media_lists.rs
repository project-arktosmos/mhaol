use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::put,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/{list_id}/tmdb",
            put(link_tmdb).delete(unlink_tmdb),
        )
        .route(
            "/{list_id}/musicbrainz",
            put(link_musicbrainz).delete(unlink_musicbrainz),
        )
}

#[derive(Deserialize)]
struct LinkTmdbBody {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
}

async fn link_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkTmdbBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "tmdb",
        &body.tmdb_id.to_string(),
        body.season_number,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "tmdb");
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct LinkMusicbrainzBody {
    #[serde(rename = "musicbrainzId")]
    musicbrainz_id: String,
}

async fn link_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkMusicbrainzBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    let mb_id = body.musicbrainz_id.trim();
    if mb_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "musicbrainzId must be a non-empty string" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "musicbrainz",
        mb_id,
        None,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "musicbrainz");
    Json(serde_json::json!({ "ok": true })).into_response()
}

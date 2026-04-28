use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::convert::Infallible;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config).put(set_config))
        .route("/debug", get(debug_info))
        .route("/search", get(search))
        .route("/status", get(get_status))
        .route("/server/connect", post(connect_server))
        .route("/files", get(list_files).post(add_file))
        .route("/files/{file_hash}", delete(remove_file))
        .route("/files/{file_hash}/pause", post(pause_file))
        .route("/files/{file_hash}/resume", post(resume_file))
        .route("/files/events", get(file_events))
        .route("/files/remove-all", post(remove_all))
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "downloadPath": state.ed2k_manager.download_path(),
        "initialized": state.ed2k_manager.is_initialized(),
    }))
}

async fn set_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(library_id) = body["library_id"].as_str() {
        if library_id.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing library_id" })),
            )
                .into_response();
        }
        match state.libraries.get(library_id) {
            Some(library) => {
                state
                    .ed2k_manager
                    .set_download_path(std::path::PathBuf::from(&library.path));
                state
                    .metadata
                    .set_string("ed2k.libraryId", library_id);
                return Json(serde_json::json!({
                    "downloadPath": state.ed2k_manager.download_path(),
                }))
                .into_response();
            }
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Library not found" })),
                )
                    .into_response();
            }
        }
    }

    if let Some(path) = body["downloadPath"].as_str() {
        if let Err(e) = std::fs::create_dir_all(path) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create directory: {}", e) })),
            )
                .into_response();
        }
        state
            .ed2k_manager
            .set_download_path(std::path::PathBuf::from(path));
        return Json(serde_json::json!({
            "downloadPath": state.ed2k_manager.download_path(),
        }))
        .into_response();
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Provide library_id or downloadPath" })),
    )
        .into_response()
}

async fn debug_info(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({ "debug": state.ed2k_manager.debug_info() }))
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "initialized": state.ed2k_manager.is_initialized(),
        "downloadPath": state.ed2k_manager.download_path(),
        "stats": state.ed2k_manager.stats(),
        "server": state.ed2k_manager.server().map(|s| serde_json::json!({
            "name": s.name,
            "host": s.host,
            "port": s.port,
            "userCount": s.user_count,
            "fileCount": s.file_count,
            "message": s.message,
            "assignedId": s.assigned_id,
        })),
    }))
}

async fn connect_server(State(state): State<AppState>) -> impl IntoResponse {
    match state.ed2k_manager.connect_any_server().await {
        Ok(server) => Json(serde_json::json!({
            "ok": true,
            "server": {
                "name": server.name,
                "host": server.host,
                "port": server.port,
                "userCount": server.user_count,
                "fileCount": server.file_count,
                "message": server.message,
                "assignedId": server.assigned_id,
            }
        }))
        .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = match query.q.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response();
        }
    };

    match state.ed2k_manager.search(&q).await {
        Ok(results) => Json(serde_json::to_value(results).unwrap()).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e.to_string(), "results": [] })),
        )
            .into_response(),
    }
}

async fn list_files(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(state.ed2k_manager.list()).unwrap())
}

async fn add_file(
    State(state): State<AppState>,
    Json(body): Json<mhaol_ed2k::AddEd2kRequest>,
) -> impl IntoResponse {
    if let Some(ref path) = body.download_path {
        if let Err(e) = std::fs::create_dir_all(path) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create download directory: {}", e) })),
            )
                .into_response();
        }
    }

    match state.ed2k_manager.add(body) {
        Ok(info) => (StatusCode::CREATED, Json(serde_json::to_value(info).unwrap()))
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn remove_file(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> impl IntoResponse {
    match state.ed2k_manager.remove(&file_hash) {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn pause_file(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> impl IntoResponse {
    match state.ed2k_manager.pause(&file_hash) {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn resume_file(
    State(state): State<AppState>,
    Path(file_hash): Path<String>,
) -> impl IntoResponse {
    match state.ed2k_manager.resume(&file_hash) {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn remove_all(State(state): State<AppState>) -> impl IntoResponse {
    let removed = state.ed2k_manager.remove_all();
    Json(serde_json::json!({ "ok": true, "removed": removed }))
}

async fn file_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mgr = state.ed2k_manager.clone();
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            let files = mgr.list();
            if let Ok(json) = serde_json::to_string(&files) {
                yield Ok(Event::default().event("files").data(json));
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

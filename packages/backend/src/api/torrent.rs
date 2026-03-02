use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{delete, get, post},
    Json, Router,
};
use std::convert::Infallible;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config))
        .route("/debug", get(debug_info))
        .route("/status", get(get_status))
        .route("/storage/clear", post(clear_storage))
        .route("/torrents", get(list_torrents).post(add_torrent))
        .route("/torrents/{info_hash}", delete(remove_torrent))
        .route("/torrents/{info_hash}/pause", post(pause_torrent))
        .route("/torrents/{info_hash}/resume", post(resume_torrent))
        .route("/torrents/events", get(torrent_events))
        .route("/torrents/remove-all", post(remove_all))
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "downloadPath": state.torrent_manager.download_path(),
        "initialized": state.torrent_manager.is_initialized(),
    }))
}

async fn debug_info(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.debug_info().await {
        Ok(info) => Json(serde_json::json!({ "debug": info })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.stats().await {
        Ok(stats) => Json(serde_json::to_value(stats).unwrap()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn clear_storage(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.clear_storage().await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_torrents(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.list().await {
        Ok(torrents) => Json(serde_json::to_value(torrents).unwrap()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn add_torrent(
    State(state): State<AppState>,
    Json(body): Json<mhaol_torrent::AddTorrentRequest>,
) -> impl IntoResponse {
    match state.torrent_manager.add(body).await {
        Ok(info) => {
            let state_str = format!("{:?}", info.state).to_lowercase();
            state.torrent_downloads.upsert(
                &info.info_hash,
                &info.name,
                info.size as i64,
                info.progress,
                &state_str,
                info.download_speed as i64,
                info.upload_speed as i64,
                info.peers as i64,
                info.seeds as i64,
                info.added_at,
                info.eta.map(|e| e as i64),
                info.output_path.as_deref(),
                "magnet",
            );
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(info).unwrap()),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn remove_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match state.torrent_manager.list().await {
        Ok(torrents) => {
            if let Some(torrent) = torrents.iter().find(|t| t.info_hash == info_hash) {
                if let Err(e) = state.torrent_manager.remove(torrent.id).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                        .into_response();
                }
            }
            state.torrent_downloads.delete(&info_hash);
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn pause_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match find_torrent_id(&state, &info_hash).await {
        Some(id) => match state.torrent_manager.pause(id).await {
            Ok(()) => {
                if let Ok(torrents) = state.torrent_manager.list().await {
                    if let Some(t) = torrents.iter().find(|t| t.info_hash == info_hash) {
                        let state_str = format!("{:?}", t.state).to_lowercase();
                        state.torrent_downloads.update_state(
                            &info_hash,
                            t.progress,
                            &state_str,
                            t.download_speed as i64,
                            t.upload_speed as i64,
                            t.peers as i64,
                            t.seeds as i64,
                            t.eta.map(|e| e as i64),
                            t.output_path.as_deref(),
                        );
                    }
                }
                Json(serde_json::json!({ "ok": true })).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Torrent not found" })),
        )
            .into_response(),
    }
}

async fn resume_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match find_torrent_id(&state, &info_hash).await {
        Some(id) => match state.torrent_manager.resume(id).await {
            Ok(()) => {
                if let Ok(torrents) = state.torrent_manager.list().await {
                    if let Some(t) = torrents.iter().find(|t| t.info_hash == info_hash) {
                        let state_str = format!("{:?}", t.state).to_lowercase();
                        state.torrent_downloads.update_state(
                            &info_hash,
                            t.progress,
                            &state_str,
                            t.download_speed as i64,
                            t.upload_speed as i64,
                            t.peers as i64,
                            t.seeds as i64,
                            t.eta.map(|e| e as i64),
                            t.output_path.as_deref(),
                        );
                    }
                }
                Json(serde_json::json!({ "ok": true })).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Torrent not found" })),
        )
            .into_response(),
    }
}

async fn remove_all(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.remove_all().await {
        Ok(count) => {
            state.torrent_downloads.delete_all();
            Json(serde_json::json!({ "ok": true, "removed": count })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn torrent_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let torrent_manager = state.torrent_manager.clone();
    let torrent_downloads = state.torrent_downloads.clone();

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Ok(torrents) = torrent_manager.list().await {
                for t in &torrents {
                    let state_str = format!("{:?}", t.state).to_lowercase();
                    torrent_downloads.upsert(
                        &t.info_hash,
                        &t.name,
                        t.size as i64,
                        t.progress,
                        &state_str,
                        t.download_speed as i64,
                        t.upload_speed as i64,
                        t.peers as i64,
                        t.seeds as i64,
                        t.added_at,
                        t.eta.map(|e| e as i64),
                        t.output_path.as_deref(),
                        "magnet",
                    );
                }
                if let Ok(json) = serde_json::to_string(&torrents) {
                    yield Ok(Event::default().event("torrents").data(json));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn find_torrent_id(state: &AppState, info_hash: &str) -> Option<usize> {
    state
        .torrent_manager
        .list()
        .await
        .ok()?
        .iter()
        .find(|t| t.info_hash == info_hash)
        .map(|t| t.id)
}

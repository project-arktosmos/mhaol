use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::convert::Infallible;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).delete(cancel_or_remove_task))
        .route("/subscribe", get(subscribe))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
    #[serde(rename = "taskType")]
    task_type: Option<String>,
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let tasks = state
        .queue
        .list(query.status.as_deref(), query.task_type.as_deref());
    Json(tasks)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskRequest {
    task_type: String,
    payload: serde_json::Value,
}

async fn create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let task = state.queue.enqueue(&body.task_type, body.payload);
    (StatusCode::CREATED, Json(task))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.queue.get(&id) {
        Some(task) => Json(serde_json::to_value(task).unwrap()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn cancel_or_remove_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Try to cancel first (for pending/running tasks)
    if state.queue.cancel(&id) {
        return StatusCode::NO_CONTENT;
    }
    // If can't cancel, try to remove (for terminal states)
    if state.queue.remove(&id) {
        return StatusCode::NO_CONTENT;
    }
    StatusCode::NOT_FOUND
}

async fn subscribe(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.queue.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Ok(json) = serde_json::to_string(&event) {
                        yield Ok(Event::default().data(json));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("[queue-sse] Lagged by {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

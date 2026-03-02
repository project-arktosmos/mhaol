use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_images))
        .route("/tagger-status", get(tagger_status))
        .route("/tag", post(tag_image))
        .route("/tag-batch", post(tag_batch))
        .route("/tags", post(add_tag).delete(remove_tag))
}

/// GET /api/images — list all image library items with their tags
async fn list_images(State(state): State<AppState>) -> impl IntoResponse {
    let items = state.library_items.get_by_media_type("image");
    let images: Vec<serde_json::Value> = items
        .into_iter()
        .map(|item| {
            let tags = state.image_tags.get_by_item(&item.id);
            let tag_values: Vec<serde_json::Value> = tags
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "tag": t.tag,
                        "score": t.score,
                    })
                })
                .collect();
            serde_json::json!({
                "id": item.id,
                "libraryId": item.library_id,
                "name": item.path.rsplit('/').next().unwrap_or(&item.path),
                "path": item.path,
                "extension": item.extension,
                "tags": tag_values,
            })
        })
        .collect();
    Json(serde_json::json!({ "images": images }))
}

/// GET /api/images/tagger-status — image tagger is not available in Rust backend
async fn tagger_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "ready": false,
        "status": "idle",
        "overallProgress": 0,
        "error": serde_json::Value::Null,
    }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TagImageBody {
    #[serde(rename = "libraryItemId")]
    library_item_id: String,
}

/// POST /api/images/tag — tag a single image (not implemented without tagger process)
async fn tag_image(Json(_body): Json<TagImageBody>) -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "Image tagger not available" })),
    )
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TagBatchBody {
    #[serde(rename = "libraryItemIds")]
    library_item_ids: Vec<String>,
}

/// POST /api/images/tag-batch — tag multiple images (not implemented without tagger process)
async fn tag_batch(Json(_body): Json<TagBatchBody>) -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "Image tagger not available" })),
    )
}

#[derive(Deserialize)]
struct ManageTagBody {
    #[serde(rename = "libraryItemId")]
    library_item_id: String,
    tag: String,
}

/// POST /api/images/tags — manually add a tag to an image
async fn add_tag(
    State(state): State<AppState>,
    Json(body): Json<ManageTagBody>,
) -> impl IntoResponse {
    state.image_tags.add_tag(&body.library_item_id, &body.tag, 1.0);
    StatusCode::OK
}

/// DELETE /api/images/tags — remove a tag from an image
async fn remove_tag(
    State(state): State<AppState>,
    Json(body): Json<ManageTagBody>,
) -> impl IntoResponse {
    state.image_tags.delete_tag(&body.library_item_id, &body.tag);
    StatusCode::OK
}

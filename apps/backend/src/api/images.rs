use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    let router = Router::new()
        .route("/", get(list_images))
        .route("/serve", get(serve_image))
        .route("/tags", post(add_tag).delete(remove_tag));

    #[cfg(not(target_os = "android"))]
    let router = router
        .route("/tagger-status", get(tagger_status))
        .route("/tag", post(tag_image))
        .route("/tag-batch", post(tag_batch));

    router
}

#[derive(Deserialize)]
struct ServeQuery {
    path: String,
}

/// GET /api/images/serve?path=... — serve an image file from disk
async fn serve_image(
    State(state): State<AppState>,
    Query(query): Query<ServeQuery>,
) -> impl IntoResponse {
    // Verify the path exists in the library
    if state.library_items.exists_by_path(&query.path).is_none() {
        return (
            StatusCode::FORBIDDEN,
            "Path not found in library",
        )
            .into_response();
    }

    let file_path = std::path::Path::new(&query.path);
    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found on disk").into_response();
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "tiff" | "tif" => "image/tiff",
        "heic" => "image/heic",
        "heif" => "image/heif",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    };

    match tokio::fs::read(&query.path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, mime_type),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
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

#[cfg(not(target_os = "android"))]
/// GET /api/images/tagger-status — current state of the image tagger
async fn tagger_status(State(state): State<AppState>) -> impl IntoResponse {
    let (ready, status, progress, error) = state.image_tagger_manager.get_status();
    Json(serde_json::json!({
        "ready": ready,
        "status": status,
        "overallProgress": progress,
        "error": error,
    }))
}

#[cfg(not(target_os = "android"))]
#[derive(Deserialize)]
struct TagImageBody {
    #[serde(rename = "libraryItemId")]
    library_item_id: String,
}

#[cfg(not(target_os = "android"))]
/// POST /api/images/tag — tag a single image using the SigLIP model
async fn tag_image(
    State(state): State<AppState>,
    Json(body): Json<TagImageBody>,
) -> impl IntoResponse {
    let item = match state.library_items.get(&body.library_item_id) {
        Some(item) => item,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Library item not found" })),
            )
                .into_response()
        }
    };

    let threshold = state
        .settings
        .get("image-tagger.threshold")
        .and_then(|v| v.parse::<f64>().ok());

    match state
        .image_tagger_manager
        .tag_image(&item.path, threshold)
        .await
    {
        Ok(tags) => {
            let tag_pairs: Vec<(&str, f64)> =
                tags.iter().map(|t| (t.tag.as_str(), t.score)).collect();
            state
                .image_tags
                .replace_for_item(&body.library_item_id, &tag_pairs);

            Json(serde_json::json!({
                "libraryItemId": body.library_item_id,
                "tags": tags.iter().map(|t| serde_json::json!({
                    "tag": t.tag,
                    "score": t.score,
                })).collect::<Vec<_>>(),
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(not(target_os = "android"))]
#[derive(Deserialize)]
struct TagBatchBody {
    #[serde(rename = "libraryItemIds")]
    library_item_ids: Vec<String>,
}

#[cfg(not(target_os = "android"))]
/// POST /api/images/tag-batch — tag multiple images sequentially
async fn tag_batch(
    State(state): State<AppState>,
    Json(body): Json<TagBatchBody>,
) -> impl IntoResponse {
    let threshold = state
        .settings
        .get("image-tagger.threshold")
        .and_then(|v| v.parse::<f64>().ok());

    let mut results = serde_json::Map::new();

    for item_id in &body.library_item_ids {
        let item = match state.library_items.get(item_id) {
            Some(item) => item,
            None => {
                results.insert(item_id.clone(), serde_json::json!([]));
                continue;
            }
        };

        match state
            .image_tagger_manager
            .tag_image(&item.path, threshold)
            .await
        {
            Ok(tags) => {
                let tag_pairs: Vec<(&str, f64)> =
                    tags.iter().map(|t| (t.tag.as_str(), t.score)).collect();
                state.image_tags.replace_for_item(item_id, &tag_pairs);

                results.insert(
                    item_id.clone(),
                    serde_json::json!(tags.iter().map(|t| serde_json::json!({
                        "tag": t.tag,
                        "score": t.score,
                    })).collect::<Vec<_>>()),
                );
            }
            Err(e) => {
                tracing::error!("[image-tagger] Failed to tag {}: {}", item_id, e);
                results.insert(item_id.clone(), serde_json::json!([]));
            }
        }
    }

    Json(serde_json::json!({ "results": results }))
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

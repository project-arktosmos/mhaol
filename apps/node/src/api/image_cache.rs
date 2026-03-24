use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::path::Path;

/// Serve an image from local disk cache, falling back to upstream URL.
///
/// - `data_dir` — root data directory (e.g. `state.data_dir`)
/// - `cache_subdir` — subdirectory name (e.g. "tmdb-images", "musicbrainz-covers")
/// - `cache_key` — relative path under the subdir (e.g. "w342/abc.jpg")
/// - `upstream_url` — full URL to fetch from if not cached
/// - `max_age` — Cache-Control max-age in seconds
pub async fn serve_cached_image(
    data_dir: &Path,
    cache_subdir: &str,
    cache_key: &str,
    upstream_url: &str,
    max_age: u32,
) -> axum::response::Response {
    if cache_key.contains("..") {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let image_dir = data_dir.join(cache_subdir);
    let local_path = image_dir.join(cache_key);

    // Serve from disk cache if available
    if local_path.exists() {
        if let Ok(bytes) = tokio::fs::read(&local_path).await {
            let content_type = mime_from_ext(cache_key);
            let cache_control = format!("public, max-age={}", max_age);
            return (
                [
                    (header::CONTENT_TYPE, content_type.to_string()),
                    (header::CACHE_CONTROL, cache_control),
                ],
                bytes,
            )
                .into_response();
        }
    }

    // Download from upstream
    match reqwest::get(upstream_url).await {
        Ok(resp) if resp.status().is_success() => match resp.bytes().await {
            Ok(bytes) => {
                // Save to disk
                if let Some(parent) = local_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let _ = tokio::fs::write(&local_path, &bytes).await;

                let content_type = mime_from_ext(cache_key);
                let cache_control = format!("public, max-age={}", max_age);
                (
                    [
                        (header::CONTENT_TYPE, content_type.to_string()),
                        (header::CACHE_CONTROL, cache_control),
                    ],
                    bytes.to_vec(),
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Image not found" })),
        )
            .into_response(),
    }
}

pub fn mime_from_ext(path: &str) -> &'static str {
    if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/jpeg"
    }
}

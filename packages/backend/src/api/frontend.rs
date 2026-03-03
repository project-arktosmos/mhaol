use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist-static/"]
struct FrontendAssets;

/// Serve an embedded static file, or fall back to index.html for SPA routing.
pub async fn serve_frontend(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first
    if !path.is_empty() {
        if let Some(file) = FrontendAssets::get(path) {
            return serve_file(path, &file).into_response();
        }
    }

    // Fallback to index.html for SPA client-side routing
    match FrontendAssets::get("index.html") {
        Some(file) => Html(
            std::str::from_utf8(file.data.as_ref())
                .unwrap_or("")
                .to_string(),
        )
        .into_response(),
        None => (StatusCode::NOT_FOUND, "frontend not embedded").into_response(),
    }
}

fn serve_file(path: &str, file: &rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let mut builder = Response::builder().header(header::CONTENT_TYPE, mime);

    // Immutable assets get long cache headers (SvelteKit hashes filenames)
    if path.contains("/immutable/") {
        builder = builder.header(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        );
    } else {
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=60");
    }

    builder
        .body(axum::body::Body::from(file.data.to_vec()))
        .unwrap()
        .into_response()
}

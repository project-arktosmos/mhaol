use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/dist-static/"]
struct CloudWebAssets;

pub async fn serve_frontend(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if !path.is_empty() {
        if let Some(file) = CloudWebAssets::get(path) {
            return serve_file(path, &file).into_response();
        }
    }

    match CloudWebAssets::get("index.html") {
        Some(file) => Html(
            std::str::from_utf8(file.data.as_ref())
                .unwrap_or("")
                .to_string(),
        )
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "cloud web frontend is not embedded. Run `pnpm --filter cloud build` and rebuild mhaol-cloud.",
        )
            .into_response(),
    }
}

fn serve_file(path: &str, file: &rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let mut builder = Response::builder().header(header::CONTENT_TYPE, mime);

    if path.contains("/immutable/") {
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
    } else {
        builder = builder.header(header::CACHE_CONTROL, "public, max-age=60");
    }

    builder
        .body(axum::body::Body::from(file.data.to_vec()))
        .unwrap()
        .into_response()
}

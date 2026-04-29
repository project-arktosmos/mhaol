use axum::{
    http::Uri,
    response::{IntoResponse, Response},
};

#[cfg(debug_assertions)]
use axum::response::Redirect;

#[cfg(not(debug_assertions))]
use axum::{
    http::{header, StatusCode},
    response::Html,
};
#[cfg(not(debug_assertions))]
use rust_embed::Embed;

#[cfg(not(debug_assertions))]
#[derive(Embed)]
#[folder = "web/dist-static/"]
struct CloudWebAssets;

pub async fn serve_frontend(uri: Uri) -> Response {
    #[cfg(debug_assertions)]
    {
        let target = std::env::var("CLOUD_DEV_PROXY")
            .unwrap_or_else(|_| "http://localhost:9596".to_string());
        let pq = uri
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or(uri.path());
        return Redirect::temporary(&format!("{target}{pq}")).into_response();
    }

    #[cfg(not(debug_assertions))]
    {
        let path = uri.path().trim_start_matches('/');

        if !path.is_empty() {
            if let Some(file) = CloudWebAssets::get(path) {
                return serve_file(path, &file);
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
}

#[cfg(not(debug_assertions))]
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

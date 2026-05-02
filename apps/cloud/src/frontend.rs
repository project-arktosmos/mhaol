use axum::{
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "web/dist-static/"]
struct CloudWebAssets;

#[derive(Embed)]
#[folder = "../player/dist-static/"]
struct PlayerWebAssets;

const PLAYER_PREFIX: &str = "player";

pub async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(rest) = path.strip_prefix(PLAYER_PREFIX) {
        return serve_player(rest);
    }

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
            "cloud web frontend is not embedded. In dev, hit the Vite dev server on port 9898 instead. For prod, run `pnpm build:cloud`.",
        )
            .into_response(),
    }
}

fn serve_player(rest: &str) -> Response {
    let inner = rest.trim_start_matches('/');

    if !inner.is_empty() {
        if let Some(file) = PlayerWebAssets::get(inner) {
            return serve_file(inner, &file);
        }
    }

    match PlayerWebAssets::get("index.html") {
        Some(file) => Html(
            std::str::from_utf8(file.data.as_ref())
                .unwrap_or("")
                .to_string(),
        )
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "player frontend is not embedded. Run `BASE_PATH=/player pnpm --filter player build` then rebuild the cloud binary.",
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

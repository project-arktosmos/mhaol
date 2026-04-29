use crate::state::CloudState;
use axum::{
    body::Body,
    extract::Query,
    http::{header, HeaderValue, Response, StatusCode},
    routing::get,
    Router,
};
use serde::Deserialize;
use sha3::{Digest, Sha3_256};
use std::path::PathBuf;

pub fn router() -> Router<CloudState> {
    Router::new().route("/", get(serve))
}

#[derive(Deserialize)]
struct Q {
    url: String,
}

fn cache_dir() -> Option<PathBuf> {
    let docs = dirs::document_dir()?;
    let dir = docs.join("mhaol-cloud").join("image-cache");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn filename_for(url_str: &str) -> String {
    let hash = Sha3_256::digest(url_str.as_bytes());
    let hex = hex::encode(hash);
    let ext = ::url::Url::parse(url_str)
        .ok()
        .and_then(|u| {
            std::path::Path::new(u.path())
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase())
        })
        .filter(|s| !s.is_empty() && s.len() <= 8 && s.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "bin".to_string());
    format!("{}.{}", hex, ext)
}

fn build_response(bytes: Vec<u8>, content_type: HeaderValue) -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        )
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn serve(Query(q): Query<Q>) -> Result<Response<Body>, StatusCode> {
    let url = q.url;
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let dir = cache_dir().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let path = dir.join(filename_for(&url));

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let content_type = HeaderValue::from_str(mime.as_ref())
        .unwrap_or(HeaderValue::from_static("application/octet-stream"));

    if let Ok(bytes) = tokio::fs::read(&path).await {
        return build_response(bytes, content_type);
    }

    let resp = reqwest::get(&url)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !resp.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let upstream_ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| HeaderValue::from_str(s).ok());
    let bytes = resp
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .to_vec();
    if let Err(e) = tokio::fs::write(&path, &bytes).await {
        tracing::warn!("image cache write failed for {}: {}", path.display(), e);
    }
    build_response(bytes, upstream_ct.unwrap_or(content_type))
}

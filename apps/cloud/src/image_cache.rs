use crate::state::CloudState;
use axum::{
    extract::Query,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
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

async fn serve(Query(q): Query<Q>) -> Result<impl IntoResponse, StatusCode> {
    let url = q.url;
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let dir = cache_dir().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let path = dir.join(filename_for(&url));

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    if let Ok(bytes) = tokio::fs::read(&path).await {
        return Ok((headers, bytes).into_response());
    }

    let resp = reqwest::get(&url)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !resp.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .to_vec();
    if let Err(e) = tokio::fs::write(&path, &bytes).await {
        tracing::warn!("image cache write failed for {}: {}", path.display(), e);
    }
    Ok((headers, bytes).into_response())
}

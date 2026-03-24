use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::warn;

/// Proxy a streaming request to a librqbit HTTP streaming endpoint.
///
/// Forwards the `Range` header from the client and streams the response back,
/// preserving status codes (200/206) and content headers. This enables the
/// browser's native `<video>` element to seek and buffer from a torrent that
/// is still downloading — librqbit automatically prioritises pieces near the
/// read position with a 32 MB lookahead window.
pub async fn proxy_stream(upstream_url: &str, range_header: Option<&str>) -> Response {
    let client = reqwest::Client::new();
    let mut req = client.get(upstream_url);

    if let Some(range) = range_header {
        req = req.header("Range", range);
    }

    let upstream = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("http-stream upstream error: {}", e);
            return (
                StatusCode::BAD_GATEWAY,
                serde_json::json!({ "error": format!("Upstream error: {}", e) }).to_string(),
            )
                .into_response();
        }
    };

    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::OK);
    let mut builder = Response::builder().status(status);

    for name in &[
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::ACCEPT_RANGES,
    ] {
        if let Some(val) = upstream.headers().get(name) {
            builder = builder.header(name, val);
        }
    }

    let body = Body::from_stream(upstream.bytes_stream());

    builder.body(body).unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    })
}

/// Build the librqbit streaming URL for a given torrent and file.
pub fn build_stream_url(http_api_addr: &str, torrent_id: usize, file_idx: usize) -> String {
    format!(
        "http://{}/torrents/{}/stream/{}",
        http_api_addr, torrent_id, file_idx
    )
}

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

const AUTH_TIMESTAMP_MAX_AGE_MS: u64 = 30_000;
const INTERNAL_HEADER: &str = "x-verified-address";
const AUTH_ADDRESS_HEADER: &str = "x-auth-address";
const AUTH_SIGNATURE_HEADER: &str = "x-auth-signature";
const AUTH_TIMESTAMP_HEADER: &str = "x-auth-timestamp";

/// Verified identity extracted from auth headers, available to handlers via request extensions.
#[derive(Clone, Debug)]
pub struct VerifiedIdentity {
    pub address: String,
}

/// Middleware that strips the internal `x-verified-address` header from external HTTP requests.
/// Must be applied as an outer layer so that only WS/WebRTC RPC handlers can set this header.
pub async fn strip_internal_header(mut req: Request<Body>, next: Next) -> Response {
    req.headers_mut().remove(INTERNAL_HEADER);
    next.run(req).await
}

/// Auth middleware that verifies passport identity.
///
/// Checks (in order):
/// 1. `x-verified-address` header (pre-verified by WS/WebRTC handlers)
/// 2. `x-auth-address` + `x-auth-signature` + `x-auth-timestamp` headers (HTTP per-request auth)
/// 3. Same fields as query params (fallback for EventSource/SSE which can't set headers)
///
/// On success, inserts `VerifiedIdentity` into request extensions.
/// On failure, returns 401 Unauthorized.
pub async fn require_auth(mut req: Request<Body>, next: Next) -> Response {
    // 1. Trust internal pre-verified header (set by WS/WebRTC RPC handlers)
    if let Some(address) = req
        .headers()
        .get(INTERNAL_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        if !address.is_empty() {
            req.extensions_mut().insert(VerifiedIdentity { address });
            return next.run(req).await;
        }
    }

    // 2. Try headers first
    let auth = extract_auth_from_headers(&req);

    // 3. Fallback to query params (for EventSource/SSE)
    let auth = auth.or_else(|| extract_auth_from_query(&req));

    let (address, signature, timestamp_str) = match auth {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, "Missing auth credentials").into_response(),
    };

    // Validate timestamp
    let ts: u64 = match timestamp_str.parse() {
        Ok(t) => t,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid timestamp").into_response(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    if now.abs_diff(ts) > AUTH_TIMESTAMP_MAX_AGE_MS {
        return (StatusCode::UNAUTHORIZED, "Expired timestamp").into_response();
    }

    // Verify EIP-191 signature
    let message = format!("mhaol-auth:{}", timestamp_str);
    let recovered = match mhaol_identity::eip191_recover(&message, &signature) {
        Ok(addr) => addr,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Signature verification failed").into_response()
        }
    };

    if recovered.to_lowercase() != address.to_lowercase() {
        return (StatusCode::UNAUTHORIZED, "Signature mismatch").into_response();
    }

    req.extensions_mut().insert(VerifiedIdentity {
        address: recovered.to_lowercase(),
    });
    next.run(req).await
}

fn extract_auth_from_headers(req: &Request<Body>) -> Option<(String, String, String)> {
    let address = req
        .headers()
        .get(AUTH_ADDRESS_HEADER)?
        .to_str()
        .ok()?
        .to_string();
    let signature = req
        .headers()
        .get(AUTH_SIGNATURE_HEADER)?
        .to_str()
        .ok()?
        .to_string();
    let timestamp = req
        .headers()
        .get(AUTH_TIMESTAMP_HEADER)?
        .to_str()
        .ok()?
        .to_string();
    Some((address, signature, timestamp))
}

fn extract_auth_from_query(req: &Request<Body>) -> Option<(String, String, String)> {
    let query = req.uri().query()?;
    let mut address = None;
    let mut signature = None;
    let mut timestamp = None;

    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        let value = urlencoding::decode(value).ok()?;
        match key {
            k if k == AUTH_ADDRESS_HEADER => address = Some(value.to_string()),
            k if k == AUTH_SIGNATURE_HEADER => signature = Some(value.to_string()),
            k if k == AUTH_TIMESTAMP_HEADER => timestamp = Some(value.to_string()),
            _ => {}
        }
    }

    Some((address?, signature?, timestamp?))
}

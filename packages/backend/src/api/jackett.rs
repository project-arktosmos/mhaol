use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    cat: Option<String>,
    tracker: Option<String>,
}

#[derive(Serialize)]
struct JackettSearchResponse {
    results: Vec<JackettResult>,
    indexers: Vec<JackettIndexer>,
}

#[derive(Serialize)]
struct JackettResult {
    id: String,
    name: String,
    #[serde(rename = "infoHash")]
    info_hash: String,
    seeders: i64,
    leechers: i64,
    size: i64,
    category: String,
    #[serde(rename = "uploadedAt")]
    uploaded_at: i64,
    #[serde(rename = "magnetLink")]
    magnet_link: String,
    tracker: String,
}

#[derive(Serialize, Clone)]
struct JackettIndexer {
    id: String,
    name: String,
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = match &query.q {
        Some(q) if !q.is_empty() => q,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };

    let api_url = state
        .settings
        .get("jackett.apiUrl")
        .unwrap_or_else(|| "http://localhost:9117".to_string());
    let api_key = state
        .settings
        .get("jackett.apiKey")
        .unwrap_or_default();

    if api_key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Jackett API key not configured. Set it in Addons settings." })),
        )
            .into_response();
    }

    let mut url = format!(
        "{}/api/v2.0/indexers/all/results?apikey={}&Query={}",
        api_url.trim_end_matches('/'),
        urlencoding::encode(&api_key),
        urlencoding::encode(q),
    );

    if let Some(cat) = &query.cat {
        if !cat.is_empty() {
            for c in cat.split(',') {
                url.push_str(&format!("&Category[]={}", urlencoding::encode(c.trim())));
            }
        }
    }

    if let Some(tracker) = &query.tracker {
        if !tracker.is_empty() {
            for t in tracker.split(',') {
                url.push_str(&format!("&Tracker[]={}", urlencoding::encode(t.trim())));
            }
        }
    }

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(body) => {
                    let results: Vec<JackettResult> = body["Results"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|r| {
                                    let title = r["Title"].as_str().unwrap_or("").to_string();
                                    let magnet = r["MagnetUri"].as_str().unwrap_or("").to_string();
                                    let info_hash = extract_info_hash(&magnet);

                                    if magnet.is_empty() {
                                        return None;
                                    }

                                    let published = r["PublishDate"].as_str().unwrap_or("");
                                    let uploaded_at = parse_jackett_date(published);

                                    Some(JackettResult {
                                        id: r["Guid"]
                                            .as_str()
                                            .unwrap_or(&uuid::Uuid::new_v4().to_string())
                                            .to_string(),
                                        name: title,
                                        info_hash,
                                        seeders: r["Seeders"].as_i64().unwrap_or(0),
                                        leechers: r["Peers"]
                                            .as_i64()
                                            .unwrap_or(0)
                                            .saturating_sub(r["Seeders"].as_i64().unwrap_or(0)),
                                        size: r["Size"].as_i64().unwrap_or(0),
                                        category: r["CategoryDesc"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                        uploaded_at,
                                        magnet_link: magnet,
                                        tracker: r["Tracker"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let indexers: Vec<JackettIndexer> = body["Indexers"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|i| {
                                    Some(JackettIndexer {
                                        id: i["ID"].as_str()?.to_string(),
                                        name: i["Name"].as_str()?.to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    Json(JackettSearchResponse { results, indexers }).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to parse Jackett response: {}", e) })),
                )
                    .into_response(),
            }
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("Jackett returned {}: {}", status, body) })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Failed to reach Jackett: {}", e) })),
        )
            .into_response(),
    }
}

/// Extract info_hash from a magnet URI.
fn extract_info_hash(magnet: &str) -> String {
    if let Some(xt_start) = magnet.find("xt=urn:btih:") {
        let hash_start = xt_start + 12;
        let rest = &magnet[hash_start..];
        let end = rest.find('&').unwrap_or(rest.len());
        rest[..end].to_lowercase()
    } else {
        String::new()
    }
}

/// Parse Jackett's date format (ISO 8601) to unix timestamp.
fn parse_jackett_date(date_str: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .or_else(|_| chrono::DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}


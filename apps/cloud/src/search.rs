use crate::state::CloudState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";

pub fn router() -> Router<CloudState> {
    Router::new().route("/tmdb", post(search_tmdb))
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

async fn search_tmdb(
    State(_state): State<CloudState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResultItem>>, (StatusCode, Json<serde_json::Value>)> {
    let query = req.query.trim();
    if query.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let api_key = std::env::var("TMDB_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "TMDB_API_KEY env var is not set on the cloud server",
        ));
    }

    let endpoint = if matches!(req.kind.as_str(), "tv show" | "tv season" | "tv episode") {
        "/search/tv"
    } else {
        "/search/movie"
    };

    let url = format!(
        "{}{}?api_key={}&query={}&include_adult=false",
        TMDB_BASE,
        endpoint,
        api_key,
        urlencoding(query)
    );

    let res = reqwest::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            format!("tmdb returned {}", res.status()),
        ));
    }

    let payload: serde_json::Value = res
        .json()
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, format!("tmdb parse failed: {e}")))?;

    let items = payload
        .get("results")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| {
                    let title = r
                        .get("title")
                        .and_then(|v| v.as_str())
                        .or_else(|| r.get("name").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .to_string();
                    let date = r
                        .get("release_date")
                        .and_then(|v| v.as_str())
                        .or_else(|| r.get("first_air_date").and_then(|v| v.as_str()))
                        .unwrap_or("");
                    let year = date.get(0..4).unwrap_or("").to_string();
                    let description = r
                        .get("overview")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let external_id = r
                        .get("id")
                        .map(|v| v.to_string());
                    SearchResultItem {
                        title,
                        author: year,
                        description,
                        external_id,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(items))
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

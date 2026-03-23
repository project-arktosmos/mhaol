use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

const OL_BASE: &str = "https://openlibrary.org";
const COVERS_BASE: &str = "https://covers.openlibrary.org";
const USER_AGENT: &str = "Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_books))
        .route("/works/{key}", get(get_work))
        .route("/authors/{key}", get(get_author))
        .route("/trending/{subject}", get(get_trending))
        .route("/cover/{id}/{size}", get(proxy_cover))
        .route(
            "/fetch-cache",
            get(get_fetch_cache).put(put_fetch_cache).delete(delete_fetch_cache),
        )
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
}

async fn search_books(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let cache_key = format!("search:{}:{}", query.q, page);

    // Check cache (valid for 1 hour for search results)
    if let Some((data, is_stale)) = state.openlibrary_api_cache.get(&cache_key) {
        if !is_stale {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/search.json?q={}&page={}&limit={}&fields=key,title,author_name,author_key,first_publish_year,cover_i,isbn,subject,publisher,language,number_of_pages_median,edition_count,ratings_average,ratings_count",
        OL_BASE,
        urlencoding::encode(&query.q),
        page,
        limit
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    state.openlibrary_api_cache.upsert(&cache_key, &data_str);
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => {
            // Try stale cache
            if let Some((data, _)) = state.openlibrary_api_cache.get(&cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Open Library API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_work(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let cache_key = format!("work:{}", key);

    if let Some((data, is_stale)) = state.openlibrary_api_cache.get(&cache_key) {
        if !is_stale {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!("{}/works/{}.json", OL_BASE, key);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    state.openlibrary_api_cache.upsert(&cache_key, &data_str);
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Work not found" })),
        )
            .into_response(),
        _ => {
            if let Some((data, _)) = state.openlibrary_api_cache.get(&cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Open Library API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_author(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let cache_key = format!("author:{}", key);

    if let Some((data, is_stale)) = state.openlibrary_api_cache.get(&cache_key) {
        if !is_stale {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!("{}/authors/{}.json", OL_BASE, key);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    state.openlibrary_api_cache.upsert(&cache_key, &data_str);
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Author not found" })),
        )
            .into_response(),
        _ => {
            if let Some((data, _)) = state.openlibrary_api_cache.get(&cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Open Library API unavailable" })),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct TrendingQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_trending(
    State(state): State<AppState>,
    Path(subject): Path<String>,
    Query(query): Query<TrendingQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let page = query.page.unwrap_or(1);
    let offset = (page - 1) * limit;
    let cache_key = format!("subject:{}:{}:{}", subject, limit, offset);

    if let Some((data, is_stale)) = state.openlibrary_api_cache.get(&cache_key) {
        if !is_stale {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/subjects/{}.json?limit={}&offset={}",
        OL_BASE, subject, limit, offset
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    state.openlibrary_api_cache.upsert(&cache_key, &data_str);
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => {
            if let Some((data, _)) = state.openlibrary_api_cache.get(&cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Open Library API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn proxy_cover(Path((id, size)): Path<(String, String)>) -> impl IntoResponse {
    let valid_sizes = ["S", "M", "L"];
    let size = if valid_sizes.contains(&size.as_str()) {
        size
    } else {
        "M".to_string()
    };

    let url = format!("{}/b/id/{}-{}.jpg", COVERS_BASE, id, size);
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("image/jpeg")
                .to_string();
            match resp.bytes().await {
                Ok(bytes) => (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, content_type)],
                    bytes,
                )
                    .into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
struct FetchCacheQuery {
    key: String,
}

async fn get_fetch_cache(
    State(state): State<AppState>,
    Query(query): Query<FetchCacheQuery>,
) -> impl IntoResponse {
    match state.book_torrent_fetch_cache.get(&query.key) {
        Some(row) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&row.candidate_json) {
                Json(parsed).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
struct PutFetchCacheBody {
    key: String,
    candidate: serde_json::Value,
}

async fn put_fetch_cache(
    State(state): State<AppState>,
    Json(body): Json<PutFetchCacheBody>,
) -> impl IntoResponse {
    let json = serde_json::to_string(&body.candidate).unwrap_or_default();
    state.book_torrent_fetch_cache.upsert(&body.key, &json);
    StatusCode::OK
}

async fn delete_fetch_cache(
    State(state): State<AppState>,
    Query(query): Query<FetchCacheQuery>,
) -> impl IntoResponse {
    state.book_torrent_fetch_cache.delete(&query.key);
    StatusCode::OK
}

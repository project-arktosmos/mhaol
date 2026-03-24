use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config))
        .route("/search/movies", get(search_movies))
        .route("/search/tv", get(search_tv))
        .route("/movies/{id}", get(get_movie))
        .route("/tv/{id}", get(get_tv))
        .route("/tv/{id}/season/{season}", get(get_tv_season))
        .route("/popular/movies", get(popular_movies))
        .route("/popular/tv", get(popular_tv))
        .route("/genres/movie", get(genres_movie))
        .route("/genres/tv", get(genres_tv))
        .route("/discover/movies", get(discover_movies))
        .route("/discover/tv", get(discover_tv))
        .route("/movies/{id}/recommendations", get(movie_recommendations))
        .route("/tv/{id}/recommendations", get(tv_recommendations))
        .route("/image/{*path}", get(serve_tmdb_image))
        .route("/image-overrides/{media_type}", get(get_all_image_overrides))
        .route("/image-overrides/{media_type}/{id}", get(get_image_overrides))
        .route(
            "/image-overrides/{media_type}/{id}/{role}",
            put(set_image_override).delete(delete_image_override),
        )
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let api_key = state.settings.get("tmdb.apiKey").unwrap_or_default();
    Json(serde_json::json!({ "configured": !api_key.is_empty() }))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    page: Option<u32>,
    year: Option<String>,
}

async fn search_movies(
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

    let page = query.page.unwrap_or(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("query", q), ("page", &page)];
    let year = query.year.clone().unwrap_or_default();
    if !year.is_empty() {
        params.push(("year", &year));
    }
    tmdb_cached_proxy(&state, "/search/movie", &params).await
}

async fn search_tv(
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

    let page = query.page.unwrap_or(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("query", q), ("page", &page)];
    let year = query.year.clone().unwrap_or_default();
    if !year.is_empty() {
        params.push(("first_air_date_year", &year));
    }
    tmdb_cached_proxy(&state, "/search/tv", &params).await
}

/// Build a cache key from the TMDB path and params (excludes api_key).
fn build_cache_key(path: &str, params: &[(&str, &str)]) -> String {
    let mut key = path.to_string();
    if !params.is_empty() {
        key.push('?');
        let pairs: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        key.push_str(&pairs.join("&"));
    }
    key
}

/// Fetch JSON from TMDB with caching. Returns the parsed JSON value on success.
/// Used by both the HTTP proxy endpoints and the smart pairing module.
pub(crate) async fn tmdb_fetch_json(
    state: &AppState,
    path: &str,
    extra_params: &[(&str, &str)],
) -> Result<serde_json::Value, String> {
    let cache_key = build_cache_key(path, extra_params);

    // Check cache
    if let Some((data, is_stale)) = state.tmdb_api_cache.get(&cache_key) {
        if !is_stale {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Ok(parsed);
            }
        }
    }

    let api_key = match state.settings.get("tmdb.apiKey") {
        Some(key) if !key.is_empty() => key,
        _ => return Err("TMDB API key not configured".to_string()),
    };

    let mut url = format!("{}{}?api_key={}", TMDB_BASE, path, api_key);
    for (k, v) in extra_params {
        url.push_str(&format!("&{}={}", k, v));
    }

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    state.tmdb_api_cache.upsert(&cache_key, &data_str);
                    Ok(data)
                }
                Err(e) => Err(e.to_string()),
            }
        }
        _ => {
            // Graceful degradation: return stale cache on error
            if let Some((data, _)) = state.tmdb_api_cache.get(&cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Ok(parsed);
                }
            }
            Err("TMDB API unavailable".to_string())
        }
    }
}

/// Cached proxy: wraps tmdb_fetch_json into an HTTP response.
async fn tmdb_cached_proxy(
    state: &AppState,
    path: &str,
    extra_params: &[(&str, &str)],
) -> axum::response::Response {
    match tmdb_fetch_json(state, path, extra_params).await {
        Ok(data) => Json(data).into_response(),
        Err(e) if e.contains("not configured") => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct PageQuery {
    page: Option<u32>,
}

async fn popular_movies(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).to_string();
    tmdb_cached_proxy(&state,"/movie/popular", &[("page", &page)]).await
}

async fn popular_tv(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).to_string();
    tmdb_cached_proxy(&state,"/tv/popular", &[("page", &page)]).await
}

async fn genres_movie(State(state): State<AppState>) -> impl IntoResponse {
    tmdb_cached_proxy(&state,"/genre/movie/list", &[]).await
}

async fn genres_tv(State(state): State<AppState>) -> impl IntoResponse {
    tmdb_cached_proxy(&state,"/genre/tv/list", &[]).await
}

#[derive(Deserialize)]
struct DiscoverQuery {
    page: Option<u32>,
    with_genres: Option<String>,
    sort_by: Option<String>,
}

async fn discover_movies(
    State(state): State<AppState>,
    Query(query): Query<DiscoverQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).to_string();
    let sort = query.sort_by.unwrap_or_else(|| "popularity.desc".to_string());
    let mut params: Vec<(&str, &str)> = vec![("page", &page), ("sort_by", &sort)];
    let genres = query.with_genres.unwrap_or_default();
    if !genres.is_empty() {
        params.push(("with_genres", &genres));
    }
    tmdb_cached_proxy(&state,"/discover/movie", &params).await
}

async fn discover_tv(
    State(state): State<AppState>,
    Query(query): Query<DiscoverQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).to_string();
    let sort = query.sort_by.unwrap_or_else(|| "popularity.desc".to_string());
    let mut params: Vec<(&str, &str)> = vec![("page", &page), ("sort_by", &sort)];
    let genres = query.with_genres.unwrap_or_default();
    if !genres.is_empty() {
        params.push(("with_genres", &genres));
    }
    tmdb_cached_proxy(&state,"/discover/tv", &params).await
}

async fn movie_recommendations(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PageQuery>,
) -> impl IntoResponse {
    let tmdb_id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid movie ID" })),
            )
                .into_response()
        }
    };
    let page = query.page.unwrap_or(1).to_string();
    tmdb_cached_proxy(
        &state,
        &format!("/movie/{}/recommendations", tmdb_id),
        &[("page", &page)],
    )
    .await
}

async fn tv_recommendations(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<PageQuery>,
) -> impl IntoResponse {
    let tmdb_id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid TV show ID" })),
            )
                .into_response()
        }
    };
    let page = query.page.unwrap_or(1).to_string();
    tmdb_cached_proxy(
        &state,
        &format!("/tv/{}/recommendations", tmdb_id),
        &[("page", &page)],
    )
    .await
}

#[derive(Deserialize)]
struct DetailQuery {
    refresh: Option<String>,
}

async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DetailQuery>,
) -> impl IntoResponse {
    let tmdb_id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid movie ID" })),
            )
                .into_response()
        }
    };

    let refresh = query.refresh.as_deref() == Some("true");

    // Check cache
    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM tmdb_movies WHERE tmdb_id = ?1",
            rusqlite::params![tmdb_id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let api_key = match state.settings.get("tmdb.apiKey") {
        Some(key) if !key.is_empty() => key,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "TMDB API key not configured" })),
            )
                .into_response()
        }
    };

    let url = format!(
        "{}/movie/{}?api_key={}&append_to_response=credits,images&include_image_language=en,null",
        TMDB_BASE, tmdb_id, api_key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    // Cache result
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO tmdb_movies (tmdb_id, data) VALUES (?1, ?2)
                         ON CONFLICT(tmdb_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![tmdb_id, data_str],
                    );
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
            Json(serde_json::json!({ "error": "Movie not found" })),
        )
            .into_response(),
        _ => {
            // Try stale cache on error
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM tmdb_movies WHERE tmdb_id = ?1",
                rusqlite::params![tmdb_id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "TMDB API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_tv_season(
    State(state): State<AppState>,
    Path((id, season)): Path<(String, i64)>,
    Query(query): Query<DetailQuery>,
) -> impl IntoResponse {
    let tmdb_id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid TV show ID" })),
            )
                .into_response()
        }
    };

    let refresh = query.refresh.as_deref() == Some("true");

    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM tmdb_seasons WHERE tmdb_id = ?1 AND season_number = ?2",
            rusqlite::params![tmdb_id, season],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let api_key = match state.settings.get("tmdb.apiKey") {
        Some(key) if !key.is_empty() => key,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "TMDB API key not configured" })),
            )
                .into_response()
        }
    };

    let url = format!(
        "{}/tv/{}/season/{}?api_key={}",
        TMDB_BASE, tmdb_id, season, api_key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO tmdb_seasons (tmdb_id, season_number, data) VALUES (?1, ?2, ?3)
                         ON CONFLICT(tmdb_id, season_number) DO UPDATE SET data = ?3, fetched_at = datetime('now')",
                        rusqlite::params![tmdb_id, season, data_str],
                    );
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
            Json(serde_json::json!({ "error": "Season not found" })),
        )
            .into_response(),
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM tmdb_seasons WHERE tmdb_id = ?1 AND season_number = ?2",
                rusqlite::params![tmdb_id, season],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "TMDB API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_tv(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DetailQuery>,
) -> impl IntoResponse {
    let tmdb_id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid TV show ID" })),
            )
                .into_response()
        }
    };

    let refresh = query.refresh.as_deref() == Some("true");

    // Check cache
    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM tmdb_tv_shows WHERE tmdb_id = ?1",
            rusqlite::params![tmdb_id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let api_key = match state.settings.get("tmdb.apiKey") {
        Some(key) if !key.is_empty() => key,
        _ => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "TMDB API key not configured" })),
            )
                .into_response()
        }
    };

    let url = format!(
        "{}/tv/{}?api_key={}&append_to_response=credits,images&include_image_language=en,null",
        TMDB_BASE, tmdb_id, api_key
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO tmdb_tv_shows (tmdb_id, data) VALUES (?1, ?2)
                         ON CONFLICT(tmdb_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![tmdb_id, data_str],
                    );
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
            Json(serde_json::json!({ "error": "TV show not found" })),
        )
            .into_response(),
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM tmdb_tv_shows WHERE tmdb_id = ?1",
                rusqlite::params![tmdb_id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "TMDB API unavailable" })),
            )
                .into_response()
        }
    }
}

/// GET /api/tmdb/image/{*path} — serve TMDB images from local disk cache.
/// Path format: {size}/{filename} e.g. w342/abc123.jpg
async fn serve_tmdb_image(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let upstream_url = format!("{}/{}", TMDB_IMAGE_BASE, path);
    super::image_cache::serve_cached_image(&state.data_dir, "tmdb-images", &path, &upstream_url, 604800).await
}

// Image override handlers

async fn get_all_image_overrides(
    State(state): State<AppState>,
    Path(media_type): Path<String>,
) -> impl IntoResponse {
    let overrides = state.tmdb_image_overrides.get_all_for_media_type(&media_type);
    Json(overrides)
}

async fn get_image_overrides(
    State(state): State<AppState>,
    Path((media_type, id)): Path<(String, i64)>,
) -> impl IntoResponse {
    let overrides = state.tmdb_image_overrides.get_for_item(id, &media_type);
    Json(overrides)
}

#[derive(Deserialize)]
struct ImageOverrideBody {
    file_path: String,
}

async fn set_image_override(
    State(state): State<AppState>,
    Path((media_type, id, role)): Path<(String, i64, String)>,
    Json(body): Json<ImageOverrideBody>,
) -> impl IntoResponse {
    state
        .tmdb_image_overrides
        .upsert(id, &media_type, &role, &body.file_path);
    StatusCode::NO_CONTENT
}

async fn delete_image_override(
    State(state): State<AppState>,
    Path((media_type, id, role)): Path<(String, i64, String)>,
) -> impl IntoResponse {
    state.tmdb_image_overrides.delete(id, &media_type, &role);
    StatusCode::NO_CONTENT
}


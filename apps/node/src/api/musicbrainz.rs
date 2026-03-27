use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const USER_AGENT: &str = "mhaol/1.0.0 (https://github.com/arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/recording/{id}", get(get_recording))
        .route("/release-group/{id}", get(get_release_group))
        .route("/release/{id}", get(get_release))
        .route("/artist/{id}", get(get_artist))
        .route("/popular", get(get_popular))
        .route("/popular-artists", get(get_popular_artists))
        .route("/search/artists", get(search_artists))
        .route("/search/release-groups", get(search_release_groups))
        .route("/cover/{id}/{size}", get(serve_cover_art))
        .route("/artist-image/{id}/{size}", get(serve_artist_image))
}

#[derive(Deserialize)]
struct MbQuery {
    refresh: Option<String>,
}

#[derive(Deserialize)]
struct PopularQuery {
    genre: Option<String>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<u32>,
}

async fn get_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid recording ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");
    let cache_key = format!("recording:{}", id);

    // Check cache
    if !refresh {
        if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/recording/{}?inc=artist-credits+releases+release-groups&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Recording not found" })),
        )
            .into_response(),
        _ => {
            // Try stale cache
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_release_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid release group ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");
    let cache_key = format!("release-group:{}", id);

    if !refresh {
        if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/release-group/{}?inc=artist-credits+releases&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Release group not found" })),
        )
            .into_response(),
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_release(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid release ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");
    let cache_key = format!("release:{}", id);

    if !refresh {
        if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/release/{}?inc=recordings+artist-credits+media&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Release not found" })),
        )
            .into_response(),
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_artist(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid artist ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");
    let cache_key = format!("artist:{}", id);

    if !refresh {
        if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!("{}/artist/{}?inc=release-groups+tags&fmt=json", MB_BASE, id);

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Artist not found" })),
        )
            .into_response(),
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

const POPULAR_GENRES: &[&str] = &[
    "rock",
    "pop",
    "electronic",
    "hip hop",
    "jazz",
    "classical",
    "r&b",
    "metal",
    "folk",
    "soul",
    "punk",
    "blues",
    "country",
    "ambient",
    "indie",
    "alternative",
];

async fn get_popular(
    State(state): State<AppState>,
    Query(query): Query<PopularQuery>,
) -> impl IntoResponse {
    let genre = query.genre.as_deref().unwrap_or("rock");

    // Validate genre against allowed list
    let genre_lower = genre.to_lowercase();
    if !POPULAR_GENRES.contains(&genre_lower.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid genre", "allowed": POPULAR_GENRES })),
        )
            .into_response();
    }

    let cache_key = format!("popular:{}", genre_lower);

    // Check cache (valid for 24 hours)
    if let Some(data) = state.api_cache.get_fresh("musicbrainz", &cache_key, 24) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
            return Json(parsed).into_response();
        }
    }

    let search_query = format!(
        "tag:{} AND primarytype:album AND status:official",
        genre_lower
    );
    let url = format!(
        "{}/release-group?query={}&fmt=json&limit=30",
        MB_BASE,
        urlencoding::encode(&search_query)
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        _ => {
            // Try stale cache
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_popular_artists(
    State(state): State<AppState>,
    Query(query): Query<PopularQuery>,
) -> impl IntoResponse {
    let genre = query.genre.as_deref().unwrap_or("rock");

    let genre_lower = genre.to_lowercase();
    if !POPULAR_GENRES.contains(&genre_lower.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid genre", "allowed": POPULAR_GENRES })),
        )
            .into_response();
    }

    let cache_key = format!("popular-artists:{}", genre_lower);

    // Check cache (valid for 24 hours)
    if let Some(data) = state.api_cache.get_fresh("musicbrainz", &cache_key, 24) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
            return Json(parsed).into_response();
        }
    }

    let search_query = format!("tag:{}", genre_lower);
    let url = format!(
        "{}/artist?query={}&fmt=json&limit=30",
        MB_BASE,
        urlencoding::encode(&search_query)
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn search_artists(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = match query.q.as_deref() {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };
    let limit = query.limit.unwrap_or(20).min(100);
    let cache_key = format!("search:artist:{}", q.to_lowercase());

    // Check cache (1 hour TTL)
    if let Some(data) = state.api_cache.get_fresh("musicbrainz", &cache_key, 1) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
            return Json(parsed).into_response();
        }
    }

    let url = format!(
        "{}/artist?query={}&fmt=json&limit={}",
        MB_BASE,
        urlencoding::encode(&q),
        limit
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn search_release_groups(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = match query.q.as_deref() {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };
    let limit = query.limit.unwrap_or(20).min(100);
    let cache_key = format!("search:release-group:{}", q.to_lowercase());

    // Check cache (1 hour TTL)
    if let Some(data) = state.api_cache.get_fresh("musicbrainz", &cache_key, 1) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
            return Json(parsed).into_response();
        }
    }

    let url = format!(
        "{}/release-group?query={}&fmt=json&limit={}",
        MB_BASE,
        urlencoding::encode(&q),
        limit
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(data) => {
                let data_str = serde_json::to_string(&data).unwrap_or_default();
                state.api_cache.upsert("musicbrainz", &cache_key, &data_str);
                Json(data).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        _ => {
            if let Some(data) = state.api_cache.get_any("musicbrainz", &cache_key) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

/// GET /api/musicbrainz/cover/{id}/{size} — serve MusicBrainz cover art from disk cache.
async fn serve_cover_art(
    State(state): State<AppState>,
    Path((id, size)): Path<(String, String)>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid release group ID" })),
        )
            .into_response();
    }

    let size = match size.as_str() {
        "250" | "500" => &size,
        _ => "250",
    };

    let cache_key = format!("{}/front-{}", id, size);
    let upstream_url = format!(
        "https://coverartarchive.org/release-group/{}/front-{}",
        id, size
    );
    super::image_cache::serve_cached_image(
        &state.data_dir,
        "musicbrainz-covers",
        &cache_key,
        &upstream_url,
        604800,
    )
    .await
}

/// GET /api/musicbrainz/artist-image/{id}/{size} — serve artist image using their first release group's cover art.
async fn serve_artist_image(
    State(state): State<AppState>,
    Path((id, size)): Path<(String, String)>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid artist ID" })),
        )
            .into_response();
    }

    let size = match size.as_str() {
        "250" | "500" => size.clone(),
        _ => "250".to_string(),
    };

    // Try to get artist data from cache first
    let artist_cache_key = format!("artist:{}", id);
    let artist_data = state
        .api_cache
        .get_any("musicbrainz", &artist_cache_key)
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

    // If not cached, fetch from MusicBrainz
    let artist_data = match artist_data {
        Some(d) => d,
        None => {
            let url = format!("{}/artist/{}?inc=release-groups&fmt=json", MB_BASE, id);
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
                            state
                                .api_cache
                                .upsert("musicbrainz", &artist_cache_key, &data_str);
                            data
                        }
                        Err(_) => {
                            return (
                                StatusCode::NOT_FOUND,
                                Json(serde_json::json!({ "error": "No image available" })),
                            )
                                .into_response();
                        }
                    }
                }
                _ => {
                    return (
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({ "error": "Artist not found" })),
                    )
                        .into_response();
                }
            }
        }
    };

    // Extract first release group ID
    let rg_id = artist_data["release-groups"]
        .as_array()
        .and_then(|rgs| rgs.first())
        .and_then(|rg| rg["id"].as_str());

    let rg_id = match rg_id {
        Some(id) => id.to_string(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "No release groups found for artist" })),
            )
                .into_response();
        }
    };

    let cache_key = format!("{}/front-{}", rg_id, size);
    let upstream_url = format!(
        "https://coverartarchive.org/release-group/{}/front-{}",
        rg_id, size
    );
    super::image_cache::serve_cached_image(
        &state.data_dir,
        "musicbrainz-covers",
        &cache_key,
        &upstream_url,
        604800,
    )
    .await
}

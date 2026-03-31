use crate::db::repo::library_item::InsertLibraryItem;
use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::tmdb::tmdb_fetch_json;

const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";
const PINNED_LIBRARY_PATH: &str = "pinned://";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pair", post(pair_items))
        .route("/pinned", get(get_pinned))
}

// --- Pair endpoint ---

#[derive(Deserialize)]
struct PairRequest {
    items: Vec<PairRequestItem>,
}

#[derive(Deserialize)]
struct PairRequestItem {
    title: String,
    id: String,
    source: String,
}

#[derive(Serialize)]
struct PairResult {
    #[serde(rename = "sourceId")]
    source_id: String,
    #[serde(rename = "sourceTitle")]
    source_title: String,
    source: String,
    matched: bool,
    #[serde(rename = "tmdbId")]
    tmdb_id: Option<i64>,
    #[serde(rename = "tmdbTitle")]
    tmdb_title: Option<String>,
    #[serde(rename = "tmdbType")]
    tmdb_type: Option<String>,
    #[serde(rename = "tmdbYear")]
    tmdb_year: Option<String>,
    #[serde(rename = "tmdbPosterPath")]
    tmdb_poster_path: Option<String>,
    confidence: String,
}

pub(crate) struct TmdbCandidate {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) year: String,
    pub(crate) poster_path: Option<String>,
    pub(crate) media_type: String, // "movie" or "tv"
    pub(crate) popularity: f64,
    pub(crate) vote_count: i64,
}

pub(crate) fn score_match(
    query_title: &str,
    candidate_title: &str,
    popularity: f64,
    vote_count: i64,
) -> f64 {
    let q = query_title.to_lowercase();
    let r = candidate_title.to_lowercase();

    let title_score = if q == r {
        100.0
    } else if r.contains(&q) || q.contains(&r) {
        70.0
    } else {
        let similarity = strsim::normalized_levenshtein(&q, &r);
        similarity * 50.0
    };

    title_score * (1.0 + (popularity + 1.0).log10()) * (1.0 + (vote_count as f64 + 1.0).log10())
}

pub(crate) fn determine_confidence(
    query_title: &str,
    candidate_title: &str,
    vote_count: i64,
) -> String {
    let q = query_title.to_lowercase();
    let r = candidate_title.to_lowercase();
    if q == r && vote_count > 100 {
        "high".to_string()
    } else if q == r || strsim::normalized_levenshtein(&q, &r) >= 0.8 || vote_count > 50 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub(crate) fn extract_year(date_str: &str) -> String {
    date_str.split('-').next().unwrap_or("").to_string()
}

pub(crate) fn parse_movie_candidate(result: &serde_json::Value) -> Option<TmdbCandidate> {
    Some(TmdbCandidate {
        id: result.get("id")?.as_i64()?,
        title: result.get("title")?.as_str()?.to_string(),
        year: extract_year(result.get("release_date")?.as_str().unwrap_or("")),
        poster_path: result
            .get("poster_path")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string()),
        media_type: "movie".to_string(),
        popularity: result
            .get("popularity")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0),
        vote_count: result
            .get("vote_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    })
}

pub(crate) fn parse_tv_candidate(result: &serde_json::Value) -> Option<TmdbCandidate> {
    Some(TmdbCandidate {
        id: result.get("id")?.as_i64()?,
        title: result.get("name")?.as_str()?.to_string(),
        year: extract_year(result.get("first_air_date")?.as_str().unwrap_or("")),
        poster_path: result
            .get("poster_path")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string()),
        media_type: "tv".to_string(),
        popularity: result
            .get("popularity")
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0),
        vote_count: result
            .get("vote_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    })
}

/// Streams results as newline-delimited JSON so the UI updates per item.
async fn pair_items(State(state): State<AppState>, Json(body): Json<PairRequest>) -> Response {
    // Fail fast if TMDB API key is not configured
    let has_key = state
        .settings
        .get("tmdb.apiKey")
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    if !has_key {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "TMDB API key not configured" })),
        )
            .into_response();
    }

    let total = body.items.len();
    info!(
        "[smart-pair] Pairing {} items from {}",
        total,
        body.items.first().map(|i| i.source.as_str()).unwrap_or("?")
    );

    let library_id = ensure_pinned_library(&state);

    let stream = async_stream::stream! {
        for (idx, item) in body.items.iter().enumerate() {
            let query = &item.title;

            let movie_params: Vec<(&str, &str)> = vec![("query", query), ("page", "1")];
            let tv_params: Vec<(&str, &str)> = vec![("query", query), ("page", "1")];

            let (movie_res, tv_res) = tokio::join!(
                tmdb_fetch_json(&state, "/search/movie", &movie_params),
                tmdb_fetch_json(&state, "/search/tv", &tv_params),
            );

            let mut best: Option<TmdbCandidate> = None;
            let mut best_score: f64 = 0.0;

            if let Ok(data) = &movie_res {
                if let Some(results_arr) = data.get("results").and_then(|r| r.as_array()) {
                    for r in results_arr.iter().take(3) {
                        if let Some(candidate) = parse_movie_candidate(r) {
                            let s = score_match(query, &candidate.title, candidate.popularity, candidate.vote_count);
                            if s > best_score {
                                best_score = s;
                                best = Some(candidate);
                            }
                        }
                    }
                }
            }

            if let Ok(data) = &tv_res {
                if let Some(results_arr) = data.get("results").and_then(|r| r.as_array()) {
                    for r in results_arr.iter().take(3) {
                        if let Some(candidate) = parse_tv_candidate(r) {
                            let s = score_match(query, &candidate.title, candidate.popularity, candidate.vote_count);
                            if s > best_score {
                                best_score = s;
                                best = Some(candidate);
                            }
                        }
                    }
                }
            }

            let result = match &best {
                Some(c) => {
                    info!(
                        "[smart-pair] ({}/{}) \"{}\" -> {} {} (id={})",
                        idx + 1, total, query, c.media_type, c.title, c.id
                    );
                    PairResult {
                        source_id: item.id.clone(),
                        source_title: item.title.clone(),
                        source: item.source.clone(),
                        matched: true,
                        tmdb_id: Some(c.id),
                        tmdb_title: Some(c.title.clone()),
                        tmdb_type: Some(c.media_type.clone()),
                        tmdb_year: Some(c.year.clone()),
                        tmdb_poster_path: c.poster_path.clone(),
                        confidence: determine_confidence(query, &c.title, c.vote_count),
                    }
                }
                None => {
                    info!("[smart-pair] ({}/{}) \"{}\" -> no match", idx + 1, total, query);
                    PairResult {
                        source_id: item.id.clone(),
                        source_title: item.title.clone(),
                        source: item.source.clone(),
                        matched: false,
                        tmdb_id: None,
                        tmdb_title: None,
                        tmdb_type: None,
                        tmdb_year: None,
                        tmdb_poster_path: None,
                        confidence: "none".to_string(),
                    }
                }
            };

            // Save matched items to DB immediately
            if result.matched {
                if let Some(tmdb_id) = result.tmdb_id {
                    let category_id = if result.tmdb_type.as_deref() == Some("tv") {
                        "pinned-tv"
                    } else {
                        "pinned-movies"
                    };
                    let path = format!("pinned://{}/{}", item.source, item.id);

                    if let Some(existing_id) = state.library_items.exists_by_path(&path) {
                        state.library_item_links.upsert(
                            &uuid::Uuid::new_v4().to_string(),
                            &existing_id,
                            "tmdb",
                            &tmdb_id.to_string(),
                            None,
                            None,
                        );
                    } else {
                        let item_id = uuid::Uuid::new_v4().to_string();
                        if let Err(e) = state.library_items.insert(&InsertLibraryItem {
                            id: item_id.clone(),
                            library_id: library_id.clone(),
                            path,
                            extension: String::new(),
                            media_type: "video".to_string(),
                            category_id: Some(category_id.to_string()),
                        }) {
                            tracing::warn!("[smart-pair] Failed to insert library item: {}", e);
                            continue;
                        }
                        state.library_item_links.upsert(
                            &uuid::Uuid::new_v4().to_string(),
                            &item_id,
                            "tmdb",
                            &tmdb_id.to_string(),
                            None,
                            None,
                        );
                    }
                }
            }

            let mut line = serde_json::to_string(&result).unwrap_or_default();
            line.push('\n');
            yield Ok::<_, std::convert::Infallible>(line);

            // Rate-limit every 15 items (30 TMDB requests) to stay under 40 req/10s
            if (idx + 1) % 15 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }

        info!("[smart-pair] Done pairing {} items", total);
    };

    Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .body(Body::from_stream(stream))
        .unwrap()
}

fn ensure_pinned_library(state: &AppState) -> String {
    let libs = state.libraries.get_all();
    for lib in &libs {
        if lib.path == PINNED_LIBRARY_PATH {
            return lib.id.clone();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    state
        .libraries
        .insert(&id, "Pinned", PINNED_LIBRARY_PATH, "[\"video\"]", now);
    id
}

// --- Pinned endpoint ---

#[derive(Serialize)]
struct PinnedResponse {
    movies: Vec<serde_json::Value>,
    tv: Vec<serde_json::Value>,
}

async fn get_pinned(State(state): State<AppState>) -> impl IntoResponse {
    let pinned_movie_items = state.library_items.get_by_category("pinned-movies");
    let pinned_tv_items = state.library_items.get_by_category("pinned-tv");

    let mut movies = Vec::new();
    let mut tv = Vec::new();

    for item in &pinned_movie_items {
        if let Some(display) = resolve_pinned_movie(&state, item).await {
            movies.push(display);
        }
    }

    for item in &pinned_tv_items {
        if let Some(display) = resolve_pinned_tv(&state, item).await {
            tv.push(display);
        }
    }

    Json(PinnedResponse { movies, tv })
}

async fn resolve_pinned_movie(
    state: &AppState,
    item: &crate::db::repo::library_item::LibraryItemRow,
) -> Option<serde_json::Value> {
    let links = state.library_item_links.get_by_item(&item.id);
    let tmdb_link = links.iter().find(|l| l.service == "tmdb")?;
    let tmdb_id: i64 = tmdb_link.service_id.parse().ok()?;

    let data = tmdb_fetch_json(
        state,
        &format!("/movie/{}", tmdb_id),
        &[
            ("append_to_response", "credits,images"),
            ("include_image_language", "en,null"),
        ],
    )
    .await
    .ok()?;

    let poster_path = data.get("poster_path").and_then(|p| p.as_str());
    let backdrop_path = data.get("backdrop_path").and_then(|p| p.as_str());
    let release_date = data
        .get("release_date")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let genres = data
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    g.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(serde_json::json!({
        "id": tmdb_id,
        "title": data.get("title").and_then(|t| t.as_str()).unwrap_or(""),
        "originalTitle": data.get("original_title").and_then(|t| t.as_str()).unwrap_or(""),
        "releaseYear": extract_year(release_date),
        "overview": data.get("overview").and_then(|o| o.as_str()).unwrap_or(""),
        "posterUrl": poster_path.map(|p| format!("{}/w342{}", TMDB_IMAGE_BASE, p)),
        "backdropUrl": backdrop_path.map(|p| format!("{}/w780{}", TMDB_IMAGE_BASE, p)),
        "voteAverage": data.get("vote_average").and_then(|v| v.as_f64()).unwrap_or(0.0),
        "voteCount": data.get("vote_count").and_then(|v| v.as_i64()).unwrap_or(0),
        "genres": genres,
    }))
}

async fn resolve_pinned_tv(
    state: &AppState,
    item: &crate::db::repo::library_item::LibraryItemRow,
) -> Option<serde_json::Value> {
    let links = state.library_item_links.get_by_item(&item.id);
    let tmdb_link = links.iter().find(|l| l.service == "tmdb")?;
    let tmdb_id: i64 = tmdb_link.service_id.parse().ok()?;

    let data = tmdb_fetch_json(
        state,
        &format!("/tv/{}", tmdb_id),
        &[
            ("append_to_response", "credits,images"),
            ("include_image_language", "en,null"),
        ],
    )
    .await
    .ok()?;

    let poster_path = data.get("poster_path").and_then(|p| p.as_str());
    let backdrop_path = data.get("backdrop_path").and_then(|p| p.as_str());
    let first_air_date = data
        .get("first_air_date")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let last_air_date = data.get("last_air_date").and_then(|d| d.as_str());
    let genres = data
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    g.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(serde_json::json!({
        "id": tmdb_id,
        "name": data.get("name").and_then(|n| n.as_str()).unwrap_or(""),
        "originalName": data.get("original_name").and_then(|n| n.as_str()).unwrap_or(""),
        "firstAirYear": extract_year(first_air_date),
        "lastAirYear": last_air_date.map(|d| extract_year(d)),
        "overview": data.get("overview").and_then(|o| o.as_str()).unwrap_or(""),
        "posterUrl": poster_path.map(|p| format!("{}/w342{}", TMDB_IMAGE_BASE, p)),
        "backdropUrl": backdrop_path.map(|p| format!("{}/w780{}", TMDB_IMAGE_BASE, p)),
        "voteAverage": data.get("vote_average").and_then(|v| v.as_f64()).unwrap_or(0.0),
        "voteCount": data.get("vote_count").and_then(|v| v.as_i64()).unwrap_or(0),
        "genres": genres,
        "numberOfSeasons": data.get("number_of_seasons").and_then(|n| n.as_i64()),
        "numberOfEpisodes": data.get("number_of_episodes").and_then(|n| n.as_i64()),
    }))
}

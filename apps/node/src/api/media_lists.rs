use crate::api::smart_pair::{determine_confidence, parse_tv_candidate, score_match};
use crate::api::tmdb::tmdb_fetch_json;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/{list_id}/tmdb",
            put(link_tmdb).delete(unlink_tmdb),
        )
        .route(
            "/{list_id}/musicbrainz",
            put(link_musicbrainz).delete(unlink_musicbrainz),
        )
        .route("/auto-match", post(auto_match))
}

#[derive(Deserialize)]
struct LinkTmdbBody {
    #[serde(rename = "tmdbId")]
    tmdb_id: i64,
    #[serde(rename = "seasonNumber")]
    season_number: Option<i64>,
}

async fn link_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkTmdbBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "tmdb",
        &body.tmdb_id.to_string(),
        body.season_number,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_tmdb(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "tmdb");
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct LinkMusicbrainzBody {
    #[serde(rename = "musicbrainzId")]
    musicbrainz_id: String,
}

async fn link_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    Json(body): Json<LinkMusicbrainzBody>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    let mb_id = body.musicbrainz_id.trim();
    if mb_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "musicbrainzId must be a non-empty string" })),
        )
            .into_response();
    }
    state.media_list_links.upsert(
        &uuid::Uuid::new_v4().to_string(),
        &list_id,
        "musicbrainz",
        mb_id,
        None,
    );
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn unlink_musicbrainz(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
) -> impl IntoResponse {
    if state.media_lists.get_by_id(&list_id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "List not found" })),
        )
            .into_response();
    }
    state.media_list_links.delete(&list_id, "musicbrainz");
    Json(serde_json::json!({ "ok": true })).into_response()
}

// --- Auto-match endpoint ---

#[derive(Deserialize)]
struct AutoMatchRequest {
    lists: Vec<AutoMatchItem>,
}

#[derive(Deserialize)]
struct AutoMatchItem {
    #[serde(rename = "listId")]
    list_id: String,
    title: String,
}

#[derive(Serialize)]
struct AutoMatchResult {
    #[serde(rename = "listId")]
    list_id: String,
    matched: bool,
    #[serde(rename = "tmdbId")]
    tmdb_id: Option<i64>,
    #[serde(rename = "tmdbTitle")]
    tmdb_title: Option<String>,
    #[serde(rename = "tmdbYear")]
    tmdb_year: Option<String>,
    #[serde(rename = "tmdbPosterPath")]
    tmdb_poster_path: Option<String>,
    confidence: String,
}

async fn auto_match(
    State(state): State<AppState>,
    Json(body): Json<AutoMatchRequest>,
) -> Response {
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

    let total = body.lists.len();
    info!("[auto-match] Matching {} TV show lists", total);

    let stream = async_stream::stream! {
        for (idx, item) in body.lists.iter().enumerate() {
            let query = &item.title;
            let tv_params: Vec<(&str, &str)> = vec![("query", query), ("page", "1")];

            let tv_res = tmdb_fetch_json(&state, "/search/tv", &tv_params).await;

            let mut best = None;
            let mut best_score: f64 = 0.0;

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
                    let confidence = determine_confidence(query, &c.title, c.vote_count);
                    let should_link = confidence == "high" || confidence == "medium";

                    if should_link {
                        info!(
                            "[auto-match] ({}/{}) \"{}\" -> {} (id={}, confidence={})",
                            idx + 1, total, query, c.title, c.id, confidence
                        );
                        state.media_list_links.upsert(
                            &uuid::Uuid::new_v4().to_string(),
                            &item.list_id,
                            "tmdb",
                            &c.id.to_string(),
                            None,
                        );
                    } else {
                        info!(
                            "[auto-match] ({}/{}) \"{}\" -> {} (id={}, confidence={}, skipped)",
                            idx + 1, total, query, c.title, c.id, confidence
                        );
                    }

                    AutoMatchResult {
                        list_id: item.list_id.clone(),
                        matched: should_link,
                        tmdb_id: Some(c.id),
                        tmdb_title: Some(c.title.clone()),
                        tmdb_year: Some(c.year.clone()),
                        tmdb_poster_path: c.poster_path.clone(),
                        confidence,
                    }
                }
                None => {
                    info!("[auto-match] ({}/{}) \"{}\" -> no match", idx + 1, total, query);
                    AutoMatchResult {
                        list_id: item.list_id.clone(),
                        matched: false,
                        tmdb_id: None,
                        tmdb_title: None,
                        tmdb_year: None,
                        tmdb_poster_path: None,
                        confidence: "none".to_string(),
                    }
                }
            };

            let mut line = serde_json::to_string(&result).unwrap_or_default();
            line.push('\n');
            yield Ok::<_, std::convert::Infallible>(line);

            // Rate-limit every 15 items to stay under TMDB API limits
            if (idx + 1) % 15 == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }

        info!("[auto-match] Done matching {} lists", total);
    };

    Response::builder()
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .body(Body::from_stream(stream))
        .unwrap()
}

use crate::db::repo::catalog::CatalogItemRow;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", get(search_items).post(upsert_item))
        .route("/items/{id}", get(get_item).delete(delete_item))
        .route("/items/{id}/children", get(get_children))
        .route(
            "/fetch-cache/{item_id}",
            get(get_fetch_cache)
                .post(save_fetch_cache)
                .delete(delete_fetch_cache),
        )
        .route("/fetch-cache/hashes", get(list_fetch_cache_hashes))
        .route("/fetch-cache/summaries", get(list_fetch_cache_summaries))
        .route(
            "/fetch-cache-by-source",
            get(get_fetch_cache_by_source)
                .post(save_fetch_cache_by_source)
                .delete(delete_fetch_cache_by_source),
        )
}

// === Catalog Items ===

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    q: Option<String>,
    kind: Option<String>,
    source: Option<String>,
    source_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn search_items(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    // Lookup by source + source_id + kind
    if let (Some(source), Some(source_id), Some(kind)) =
        (&query.source, &query.source_id, &query.kind)
    {
        let item = state.catalog.get_by_source(source, source_id, kind);
        let total = if item.is_some() { 1 } else { 0 };
        return Json(serde_json::json!({
            "items": item.map(|i| vec![to_json(&i)]).unwrap_or_default(),
            "total": total,
        }));
    }

    // Text search
    if let Some(q) = &query.q {
        let items = state
            .catalog
            .search(q, query.kind.as_deref(), limit, offset);
        let total = items.len() as i64;
        return Json(serde_json::json!({
            "items": items.iter().map(to_json).collect::<Vec<_>>(),
            "total": total,
        }));
    }

    // Browse by kind
    if let Some(kind) = &query.kind {
        let items = state.catalog.get_by_kind(kind, limit, offset);
        let total = state.catalog.count_by_kind(kind);
        return Json(serde_json::json!({
            "items": items.iter().map(to_json).collect::<Vec<_>>(),
            "total": total,
        }));
    }

    Json(serde_json::json!({ "items": [], "total": 0 }))
}

async fn get_item(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.catalog.get_by_id(&id) {
        Some(item) => Json(to_json(&item)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_children(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let children = state.catalog.get_children(&id);
    Json(children.iter().map(to_json).collect::<Vec<_>>())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertItemBody {
    id: Option<String>,
    kind: String,
    title: String,
    year: Option<String>,
    overview: Option<String>,
    poster_url: Option<String>,
    backdrop_url: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<i64>,
    parent_id: Option<String>,
    position: Option<i64>,
    source: String,
    source_id: String,
    metadata: Option<serde_json::Value>,
}

async fn upsert_item(
    State(state): State<AppState>,
    Json(body): Json<UpsertItemBody>,
) -> impl IntoResponse {
    let id = body.id.unwrap_or_else(|| format!("{:032x}", rand_id()));
    let sort_title = body.title.to_lowercase();
    let metadata = body
        .metadata
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let row = CatalogItemRow {
        id: id.clone(),
        kind: body.kind,
        title: body.title,
        sort_title,
        year: body.year,
        overview: body.overview,
        poster_url: body.poster_url,
        backdrop_url: body.backdrop_url,
        vote_average: body.vote_average,
        vote_count: body.vote_count,
        parent_id: body.parent_id,
        position: body.position,
        source: body.source,
        source_id: body.source_id,
        metadata,
        created_at: String::new(),
        updated_at: String::new(),
    };

    state.catalog.upsert(&row);

    let saved = state.catalog.get_by_id(&id);
    (StatusCode::CREATED, Json(saved.map(|i| to_json(&i))))
}

async fn delete_item(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    if state.catalog.delete(&id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// === Fetch Cache ===

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchCacheQuery {
    scope: Option<String>,
    scope_key: Option<String>,
}

async fn get_fetch_cache(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Query(query): Query<FetchCacheQuery>,
) -> impl IntoResponse {
    if let Some(scope) = &query.scope {
        let scope_key = query.scope_key.as_deref().unwrap_or("");
        match state.catalog_fetch_cache.get(&item_id, scope, scope_key) {
            Some(row) => Json(serde_json::json!(row)).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    } else {
        let rows = state.catalog_fetch_cache.get_by_item(&item_id);
        Json(rows).into_response()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveFetchCacheBody {
    scope: String,
    scope_key: Option<String>,
    candidate: serde_json::Value,
}

async fn save_fetch_cache(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<SaveFetchCacheBody>,
) -> impl IntoResponse {
    let scope_key = body.scope_key.as_deref().unwrap_or("");
    let candidate_json = body.candidate.to_string();
    state
        .catalog_fetch_cache
        .upsert(&item_id, &body.scope, scope_key, &candidate_json);
    StatusCode::CREATED
}

async fn delete_fetch_cache(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> impl IntoResponse {
    if state.catalog_fetch_cache.delete_by_item(&item_id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn list_fetch_cache_hashes(State(state): State<AppState>) -> impl IntoResponse {
    let hashes = state.catalog_fetch_cache.get_all_info_hashes();
    let result: Vec<serde_json::Value> = hashes
        .into_iter()
        .map(|(item_id, hash)| serde_json::json!({ "catalogItemId": item_id, "infoHash": hash }))
        .collect();
    Json(result)
}

async fn list_fetch_cache_summaries(State(state): State<AppState>) -> impl IntoResponse {
    let summaries = state.catalog_fetch_cache.get_all_summaries();
    let result: Vec<serde_json::Value> = summaries
        .into_iter()
        .map(|(item_id, scope, name)| {
            serde_json::json!({ "catalogItemId": item_id, "scope": scope, "name": name })
        })
        .collect();
    Json(result)
}

// === Fetch Cache by Source (convenience layer) ===

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchCacheBySourceQuery {
    source: String,
    source_id: String,
    kind: String,
    scope: Option<String>,
    scope_key: Option<String>,
}

async fn get_fetch_cache_by_source(
    State(state): State<AppState>,
    Query(query): Query<FetchCacheBySourceQuery>,
) -> impl IntoResponse {
    let item = match state.catalog.get_by_source(&query.source, &query.source_id, &query.kind) {
        Some(item) => item,
        None => {
            // No catalog item yet — no cache exists, return empty/not-found gracefully
            if query.scope.is_some() {
                return StatusCode::NOT_FOUND.into_response();
            } else {
                return Json(serde_json::json!([])).into_response();
            }
        }
    };
    if let Some(scope) = &query.scope {
        let scope_key = query.scope_key.as_deref().unwrap_or("");
        match state.catalog_fetch_cache.get(&item.id, scope, scope_key) {
            Some(row) => Json(serde_json::json!(row)).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    } else {
        let rows = state.catalog_fetch_cache.get_by_item(&item.id);
        Json(rows).into_response()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveFetchCacheBySourceBody {
    source: String,
    source_id: String,
    kind: String,
    scope: String,
    scope_key: Option<String>,
    candidate: serde_json::Value,
}

async fn save_fetch_cache_by_source(
    State(state): State<AppState>,
    Json(body): Json<SaveFetchCacheBySourceBody>,
) -> impl IntoResponse {
    // Find existing catalog item, or auto-create a placeholder
    let item = match state.catalog.get_by_source(&body.source, &body.source_id, &body.kind) {
        Some(item) => item,
        None => {
            let id = format!("{:032x}", rand_id());
            let title = body
                .candidate
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let row = CatalogItemRow {
                id: id.clone(),
                kind: body.kind.clone(),
                title: title.clone(),
                sort_title: title.to_lowercase(),
                year: None,
                overview: None,
                poster_url: None,
                backdrop_url: None,
                vote_average: None,
                vote_count: None,
                parent_id: None,
                position: None,
                source: body.source.clone(),
                source_id: body.source_id.clone(),
                metadata: "{}".to_string(),
                created_at: String::new(),
                updated_at: String::new(),
            };
            state.catalog.upsert(&row);
            match state.catalog.get_by_id(&id) {
                Some(item) => item,
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": "Failed to create catalog item" })),
                    )
                        .into_response()
                }
            }
        }
    };
    let scope_key = body.scope_key.as_deref().unwrap_or("");
    let candidate_json = body.candidate.to_string();
    state
        .catalog_fetch_cache
        .upsert(&item.id, &body.scope, scope_key, &candidate_json);
    StatusCode::CREATED.into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteFetchCacheBySourceQuery {
    source: String,
    source_id: String,
    kind: String,
}

async fn delete_fetch_cache_by_source(
    State(state): State<AppState>,
    Query(query): Query<DeleteFetchCacheBySourceQuery>,
) -> impl IntoResponse {
    let item = match state.catalog.get_by_source(&query.source, &query.source_id, &query.kind) {
        Some(item) => item,
        None => return StatusCode::NOT_FOUND,
    };
    if state.catalog_fetch_cache.delete_by_item(&item.id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// === Helpers ===

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogItemJson {
    id: String,
    kind: String,
    title: String,
    sort_title: String,
    year: Option<String>,
    overview: Option<String>,
    poster_url: Option<String>,
    backdrop_url: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<i64>,
    parent_id: Option<String>,
    position: Option<i64>,
    source: String,
    source_id: String,
    metadata: serde_json::Value,
    created_at: String,
    updated_at: String,
}

fn to_json(row: &CatalogItemRow) -> CatalogItemJson {
    CatalogItemJson {
        id: row.id.clone(),
        kind: row.kind.clone(),
        title: row.title.clone(),
        sort_title: row.sort_title.clone(),
        year: row.year.clone(),
        overview: row.overview.clone(),
        poster_url: row.poster_url.clone(),
        backdrop_url: row.backdrop_url.clone(),
        vote_average: row.vote_average,
        vote_count: row.vote_count,
        parent_id: row.parent_id.clone(),
        position: row.position,
        source: row.source.clone(),
        source_id: row.source_id.clone(),
        metadata: serde_json::from_str(&row.metadata)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

fn rand_id() -> u128 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    );
    (h.finish() as u128) << 64 | {
        let s2 = RandomState::new();
        s2.build_hasher().finish() as u128
    }
}

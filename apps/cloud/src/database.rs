use crate::state::CloudState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb::sql::Thing;

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/tables", get(list_tables).delete(clear_all))
        .route("/tables/{table}", get(list_records).delete(clear_table))
}

fn err_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": message.into() })),
    )
}

#[derive(Serialize)]
struct TablesResponse {
    namespace: &'static str,
    database: &'static str,
    tables: Vec<TableInfo>,
}

#[derive(Serialize)]
struct TableInfo {
    name: String,
    record_count: u64,
}

async fn list_tables(
    State(state): State<CloudState>,
) -> Result<Json<TablesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mut response = state.db.query("INFO FOR DB").await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("INFO FOR DB failed: {e}"),
        )
    })?;

    let info: Option<serde_json::Value> = response.take(0).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("INFO FOR DB parse failed: {e}"),
        )
    })?;

    let mut names: Vec<String> = info
        .as_ref()
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    names.sort();

    let mut tables = Vec::with_capacity(names.len());
    for name in names {
        let record_count = count_records(&state, &name).await;
        tables.push(TableInfo { name, record_count });
    }

    Ok(Json(TablesResponse {
        namespace: crate::db::NAMESPACE,
        database: crate::db::DATABASE,
        tables,
    }))
}

async fn count_records(state: &CloudState, table: &str) -> u64 {
    if !is_valid_table_name(table) {
        return 0;
    }
    let q = format!("SELECT count() AS c FROM {table} GROUP ALL");
    let mut response = match state.db.query(q).await {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let row: Option<serde_json::Value> = response.take(0).unwrap_or(None);
    row.as_ref()
        .and_then(|v| v.get("c"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Serialize)]
struct RecordsResponse {
    table: String,
    limit: u32,
    offset: u32,
    total: u64,
    records: Vec<serde_json::Value>,
}

async fn list_records(
    State(state): State<CloudState>,
    Path(table): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<RecordsResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !is_valid_table_name(&table) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "invalid table name (only alphanumeric and underscore allowed)",
        ));
    }
    let limit = q.limit.unwrap_or(100).min(1000);
    let offset = q.offset.unwrap_or(0);

    let total = count_records(&state, &table).await;

    let query = format!("SELECT * FROM {table} LIMIT {limit} START {offset}");
    let mut response = state.db.query(query).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("select failed: {e}"),
        )
    })?;

    let raw_records: Vec<GenericRecord> = response.take(0).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("select parse failed: {e}"),
        )
    })?;

    let records: Vec<serde_json::Value> = raw_records.into_iter().map(GenericRecord::into_json).collect();

    Ok(Json(RecordsResponse {
        table,
        limit,
        offset,
        total,
        records,
    }))
}

#[derive(Deserialize)]
struct GenericRecord {
    id: Option<Thing>,
    #[serde(flatten)]
    rest: BTreeMap<String, serde_json::Value>,
}

impl GenericRecord {
    fn into_json(self) -> serde_json::Value {
        let mut map = serde_json::Map::with_capacity(self.rest.len() + 1);
        if let Some(thing) = self.id {
            let id_str = format!("{}:{}", thing.tb, thing.id.to_raw());
            map.insert("id".to_string(), serde_json::Value::String(id_str));
        }
        for (k, v) in self.rest {
            map.insert(k, v);
        }
        serde_json::Value::Object(map)
    }
}

#[derive(Serialize)]
struct ClearResponse {
    cleared: Vec<String>,
}

async fn clear_all(
    State(state): State<CloudState>,
) -> Result<Json<ClearResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mut response = state.db.query("INFO FOR DB").await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("INFO FOR DB failed: {e}"),
        )
    })?;

    let info: Option<serde_json::Value> = response.take(0).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("INFO FOR DB parse failed: {e}"),
        )
    })?;

    let names: Vec<String> = info
        .as_ref()
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();

    let mut cleared = Vec::with_capacity(names.len());
    for name in names {
        if !is_valid_table_name(&name) {
            continue;
        }
        let q = format!("REMOVE TABLE {name}");
        state.db.query(q).await.map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("REMOVE TABLE {name} failed: {e}"),
            )
        })?;
        cleared.push(name);
    }

    Ok(Json(ClearResponse { cleared }))
}

async fn clear_table(
    State(state): State<CloudState>,
    Path(table): Path<String>,
) -> Result<Json<ClearResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !is_valid_table_name(&table) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "invalid table name (only alphanumeric and underscore allowed)",
        ));
    }
    let q = format!("REMOVE TABLE {table}");
    state.db.query(q).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("REMOVE TABLE {table} failed: {e}"),
        )
    })?;
    Ok(Json(ClearResponse {
        cleared: vec![table],
    }))
}

fn is_valid_table_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

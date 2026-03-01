use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tables", get(list_tables))
        .route("/tables/{name}", get(get_table))
        .route("/reset", post(reset_database))
}

#[derive(Serialize)]
struct ColumnInfo {
    name: String,
    #[serde(rename = "type")]
    col_type: String,
}

#[derive(Serialize)]
struct TableInfo {
    name: String,
    columns: Vec<ColumnInfo>,
    #[serde(rename = "rowCount")]
    row_count: i64,
}

async fn list_tables(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock();
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .unwrap();
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut result: Vec<TableInfo> = Vec::new();
    for name in table_names {
        let columns = get_table_columns(&conn, &name);
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        result.push(TableInfo {
            name,
            columns,
            row_count: count,
        });
    }

    Json(result)
}

#[derive(Deserialize)]
struct TableQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct TableDetailResponse {
    table: String,
    columns: Vec<ColumnInfo>,
    rows: Vec<serde_json::Value>,
    pagination: Pagination,
}

#[derive(Serialize)]
struct Pagination {
    page: i64,
    limit: i64,
    total: i64,
    #[serde(rename = "totalPages")]
    total_pages: i64,
}

async fn get_table(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<TableQuery>,
) -> impl IntoResponse {
    let conn = state.db.lock();

    // Check table exists
    let exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?1 AND name NOT LIKE 'sqlite_%'")
        .unwrap()
        .exists(params![name])
        .unwrap_or(false);

    if !exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Table \"{}\" not found", name) })),
        )
            .into_response();
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    let columns = get_table_columns(&conn, &name);
    let col_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

    let sql = format!("SELECT * FROM \"{}\" LIMIT ?1 OFFSET ?2", name);
    let mut stmt = conn.prepare(&sql).unwrap();
    let rows: Vec<serde_json::Value> = stmt
        .query_map(params![limit, offset], |row| {
            let mut map = serde_json::Map::new();
            for (i, col_name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                let json_val = match val {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => serde_json::Value::from(n),
                    rusqlite::types::Value::Real(f) => serde_json::json!(f),
                    rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
                    rusqlite::types::Value::Blob(b) => {
                        serde_json::Value::String(format!("<blob:{} bytes>", b.len()))
                    }
                };
                map.insert(col_name.clone(), json_val);
            }
            Ok(serde_json::Value::Object(map))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let total_pages = if limit > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };

    Json(serde_json::json!(TableDetailResponse {
        table: name,
        columns,
        rows,
        pagination: Pagination {
            page,
            limit,
            total,
            total_pages,
        },
    }))
    .into_response()
}

async fn reset_database(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock();

    // Drop all triggers and tables, then reinitialize
    conn.execute_batch("PRAGMA foreign_keys = OFF").unwrap();

    let triggers: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };
    for trigger in &triggers {
        conn.execute_batch(&format!("DROP TRIGGER IF EXISTS \"{}\"", trigger))
            .unwrap();
    }

    let tables: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };
    for table in &tables {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS \"{}\"", table))
            .unwrap();
    }

    conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();

    crate::db::schema::initialize_schema(&conn).unwrap();
    crate::db::schema::initialize_module_schemas(&conn).unwrap();

    // Re-seed default library
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let downloads_path = format!("{}/Downloads", home);
    let library_id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO libraries (id, name, path, media_types, date_added) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![library_id, "Downloads", downloads_path, "[\"video\",\"image\",\"audio\"]", chrono::Utc::now().timestamp_millis()],
    ).unwrap();

    conn.execute(
        "INSERT INTO metadata (key, value, type) VALUES ('youtube.libraryId', ?1, 'string')
         ON CONFLICT(key) DO UPDATE SET value = ?1",
        params![library_id],
    ).unwrap();
    conn.execute(
        "INSERT INTO metadata (key, value, type) VALUES ('torrent.libraryId', ?1, 'string')
         ON CONFLICT(key) DO UPDATE SET value = ?1",
        params![library_id],
    ).unwrap();

    tracing::info!("Database reset complete");

    let new_tables: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    Json(serde_json::json!({ "ok": true, "tables": new_tables }))
}

fn get_table_columns(conn: &rusqlite::Connection, table: &str) -> Vec<ColumnInfo> {
    let sql = format!("PRAGMA table_info('{}')", table);
    let mut stmt = conn.prepare(&sql).unwrap();
    stmt.query_map([], |row| {
        Ok(ColumnInfo {
            name: row.get(1)?,
            col_type: row.get(2)?,
        })
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

use crate::state::CloudState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

pub const TABLE: &str = "ipfs_pin";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsPin {
    pub id: Option<Thing>,
    pub cid: String,
    pub path: String,
    pub mime: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct IpfsPinDto {
    pub id: String,
    pub cid: String,
    pub path: String,
    pub mime: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

impl From<IpfsPin> for IpfsPinDto {
    fn from(p: IpfsPin) -> Self {
        let id = p
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            cid: p.cid,
            path: p.path,
            mime: p.mime,
            size: p.size,
            created_at: p.created_at,
        }
    }
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/pins", get(list))
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

async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<IpfsPinDto>>, (StatusCode, Json<serde_json::Value>)> {
    let pins: Vec<IpfsPin> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut dtos: Vec<IpfsPinDto> = pins.into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

/// Insert a pin record, deduplicated by `(cid, path)`. Returns `true` when a
/// new record was written. Existing records are left untouched.
pub async fn record_pin(
    state: &CloudState,
    cid: String,
    path: String,
    mime: String,
    size: u64,
) -> anyhow::Result<bool> {
    let existing: Vec<IpfsPin> = state.db.select(TABLE).await?;
    if existing.iter().any(|p| p.cid == cid && p.path == path) {
        return Ok(false);
    }
    let new_id = uuid::Uuid::new_v4().to_string();
    let record = IpfsPin {
        id: None,
        cid,
        path,
        mime,
        size,
        created_at: Utc::now(),
    };
    let _: Option<IpfsPin> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await?;
    Ok(true)
}

impl IntoResponse for IpfsPinDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

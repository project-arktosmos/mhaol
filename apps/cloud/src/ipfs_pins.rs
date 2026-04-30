use crate::state::CloudState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tokio::fs;
use tokio_util::io::ReaderStream;

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
    Router::new()
        .route("/pins", get(list))
        .route("/pins/{cid}/file", get(serve_pin_file))
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

/// Stream the on-disk file for a pinned IPFS object. Used by the WASM
/// emulator modal so the browser can fetch ROM bytes directly from the
/// cloud after `extract_roms_for_firkin` has unpacked any archives. The
/// file is whatever path was recorded in `ipfs_pin` at pin time, so the
/// caller is trusting the same source that scan / torrent-completion /
/// rom-extract trusted when they created the pin.
async fn serve_pin_file(
    State(state): State<CloudState>,
    Path(cid): Path<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let cid = cid.trim().to_string();
    if cid.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "cid is required"));
    }
    let pins: Vec<IpfsPin> = state.db.select(TABLE).await.map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}"))
    })?;
    let pin = pins
        .into_iter()
        .find(|p| p.cid == cid)
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, format!("no pin for cid {cid}")))?;
    if pin.path.starts_with("firkin://") || pin.path.starts_with("artist://") {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "pin is a metadata pin, not a file",
        ));
    }
    let path = std::path::PathBuf::from(&pin.path);
    if !path.exists() {
        return Err(err_response(
            StatusCode::NOT_FOUND,
            format!("file no longer on disk: {}", pin.path),
        ));
    }
    let file = fs::File::open(&path)
        .await
        .map_err(|e| err_response(StatusCode::NOT_FOUND, format!("open failed: {e}")))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let content_type = if pin.mime.is_empty() {
        "application/octet-stream".to_string()
    } else {
        pin.mime
    };
    let content_length = pin.size.to_string();
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_LENGTH, content_length),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        body,
    )
        .into_response())
}

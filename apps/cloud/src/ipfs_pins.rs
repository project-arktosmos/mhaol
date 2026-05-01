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
use sha2::{Digest, Sha256};
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
    let mut dtos: Vec<IpfsPinDto> = dedupe_pins(pins).into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

/// Drop duplicate `(cid, path)` rows that may have leaked in before
/// `record_pin` switched to deterministic ids (or from concurrent writers
/// racing the old non-atomic existence check). Keeps the oldest record per
/// `(cid, path)` so the surfaced `created_at` reflects when the file was
/// actually first pinned.
pub(crate) fn dedupe_pins(mut pins: Vec<IpfsPin>) -> Vec<IpfsPin> {
    pins.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let mut seen: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::with_capacity(pins.len());
    pins.retain(|p| seen.insert((p.cid.clone(), p.path.clone())));
    pins
}

/// SurrealDB record id for a `(cid, path)` pair. Using a deterministic id
/// makes `record_pin` idempotent: a re-scan or a concurrent caller that
/// hits the same file lands on the same row instead of producing a
/// duplicate, regardless of how the existence check races.
fn pin_record_id(cid: &str, path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cid.as_bytes());
    hasher.update(b":");
    hasher.update(path.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
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
    let id = pin_record_id(&cid, &path);
    let existing: Option<IpfsPin> = state.db.select((TABLE, id.as_str())).await?;
    if existing.is_some() {
        return Ok(false);
    }
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
        .create((TABLE, id.as_str()))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(cid: &str, path: &str, ts_secs: i64) -> IpfsPin {
        IpfsPin {
            id: None,
            cid: cid.to_string(),
            path: path.to_string(),
            mime: "application/octet-stream".to_string(),
            size: 0,
            created_at: chrono::DateTime::from_timestamp(ts_secs, 0).unwrap(),
        }
    }

    #[test]
    fn dedupe_pins_collapses_same_cid_and_path() {
        let pins = vec![
            pin("Qm1", "/lib/a.mp4", 200),
            pin("Qm1", "/lib/a.mp4", 100),
            pin("Qm1", "/lib/b.mp4", 150),
            pin("Qm2", "/lib/a.mp4", 175),
        ];
        let result = dedupe_pins(pins);
        assert_eq!(result.len(), 3);
        let kept = result
            .iter()
            .find(|p| p.cid == "Qm1" && p.path == "/lib/a.mp4")
            .unwrap();
        assert_eq!(kept.created_at.timestamp(), 100);
    }

    #[test]
    fn pin_record_id_is_deterministic() {
        let a = pin_record_id("Qm1", "/lib/a.mp4");
        let b = pin_record_id("Qm1", "/lib/a.mp4");
        assert_eq!(a, b);
        assert_ne!(a, pin_record_id("Qm1", "/lib/b.mp4"));
        assert_ne!(a, pin_record_id("Qm2", "/lib/a.mp4"));
    }
}

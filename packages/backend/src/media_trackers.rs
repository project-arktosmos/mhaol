use crate::firkins::{self, Firkin};
use crate::state::CloudState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::sql::Thing;

pub const TABLE: &str = "media_tracker";

/// One row per (firkin, track?, user) tuple. Accumulated playback time is
/// added by the right-side player every 10 seconds while a firkin file is
/// playing (plus a 0-delta heartbeat at play-start so the row exists even if
/// the user immediately stops). When the player surfaces a per-track context
/// (music playback driven by `playPlaylistTrack`), `track_id` is set and the
/// row counts time for that specific track within the album. For everything
/// else (movies, single-file streams) `track_id` is `None` and the row counts
/// time at the firkin level — same shape as before per-track tracking landed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTracker {
    pub id: Option<Thing>,
    /// CID of the firkin being tracked — the firkin's own SurrealDB id, which
    /// is its IPFS-style content address.
    pub firkin_id: String,
    /// Optional per-track identifier (e.g. MusicBrainz track id). When `None`
    /// the row aggregates time at the firkin level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<String>,
    /// Display title for the track, captured at heartbeat time so the
    /// consumption UI can render per-track rows without re-resolving the
    /// album. Optional and best-effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_title: Option<String>,
    /// Lowercased EVM address of the user whose playback time this row tracks.
    pub address: String,
    pub total_seconds: f64,
    pub last_played_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MediaTrackerDto {
    pub id: String,
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    #[serde(rename = "trackId", skip_serializing_if = "Option::is_none")]
    pub track_id: Option<String>,
    #[serde(rename = "trackTitle", skip_serializing_if = "Option::is_none")]
    pub track_title: Option<String>,
    pub address: String,
    #[serde(rename = "totalSeconds")]
    pub total_seconds: f64,
    pub last_played_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MediaTracker> for MediaTrackerDto {
    fn from(t: MediaTracker) -> Self {
        let id = t
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            firkin_id: t.firkin_id,
            track_id: t.track_id,
            track_title: t.track_title,
            address: t.address,
            total_seconds: t.total_seconds,
            last_played_at: t.last_played_at,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

impl IntoResponse for MediaTrackerDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    /// Optional per-track identifier. When set the heartbeat is bucketed
    /// against the (firkin, track, user) tuple instead of the firkin row.
    #[serde(default, rename = "trackId")]
    pub track_id: Option<String>,
    /// Optional display title for the track, persisted on first heartbeat
    /// so the consumption UI doesn't have to re-resolve the album to label
    /// rows. Trimmed; empty values are dropped.
    #[serde(default, rename = "trackTitle")]
    pub track_title: Option<String>,
    pub address: String,
    /// Seconds of playback to add to the tracker. `0` is allowed (used as a
    /// play-start signal) and is the only value that creates a fresh row
    /// without contributing to `total_seconds`. Negative values are rejected.
    #[serde(rename = "deltaSeconds")]
    pub delta_seconds: f64,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    #[serde(rename = "firkinId")]
    pub firkin_id: Option<String>,
    #[serde(rename = "trackId")]
    pub track_id: Option<String>,
    pub address: Option<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list))
        .route("/heartbeat", post(heartbeat))
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

/// Normalise an EVM-style address to lowercase 0x-prefixed hex. Mirrors the
/// helper in `users.rs` so the tracker's `address` column joins cleanly with
/// the user table id.
fn normalize_address(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    let body = lower.strip_prefix("0x").unwrap_or(&lower);
    if body.len() != 40 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("0x{}", body))
}

/// Deterministic SurrealDB record id for a `(firkin_id, track_id?, address)`
/// tuple. Same pattern as `ipfs_pins::pin_record_id` so the heartbeat handler
/// is idempotent under races: every concurrent caller for the same tuple
/// lands on the same row. When `track_id` is `None` the digest matches the
/// pre-per-track shape, so existing firkin-level rows keep accumulating.
fn record_id(firkin_id: &str, track_id: Option<&str>, address: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(firkin_id.as_bytes());
    hasher.update(b":");
    hasher.update(address.as_bytes());
    if let Some(tid) = track_id {
        hasher.update(b":");
        hasher.update(tid.as_bytes());
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

async fn list(
    State(state): State<CloudState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<MediaTrackerDto>>, (StatusCode, Json<serde_json::Value>)> {
    let trackers: Vec<MediaTracker> = state.db.select(TABLE).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db select failed: {e}"),
        )
    })?;

    let address_filter = match q.address.as_deref() {
        Some(raw) => Some(normalize_address(raw).ok_or_else(|| {
            err_response(StatusCode::BAD_REQUEST, "invalid address")
        })?),
        None => None,
    };
    let firkin_filter = q.firkin_id.as_deref().map(|s| s.trim().to_string());
    let track_filter = q.track_id.as_deref().map(|s| s.trim().to_string());

    let mut dtos: Vec<MediaTrackerDto> = trackers
        .into_iter()
        .filter(|t| {
            if let Some(fid) = &firkin_filter {
                if !fid.is_empty() && &t.firkin_id != fid {
                    return false;
                }
            }
            if let Some(tid) = &track_filter {
                if !tid.is_empty() && t.track_id.as_deref() != Some(tid.as_str()) {
                    return false;
                }
            }
            if let Some(addr) = &address_filter {
                if &t.address != addr {
                    return false;
                }
            }
            true
        })
        .map(Into::into)
        .collect();
    dtos.sort_by(|a, b| b.last_played_at.cmp(&a.last_played_at));
    Ok(Json(dtos))
}

async fn heartbeat(
    State(state): State<CloudState>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<MediaTrackerDto>, (StatusCode, Json<serde_json::Value>)> {
    let firkin_id = req.firkin_id.trim().to_string();
    if firkin_id.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "firkinId is required"));
    }
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    if !req.delta_seconds.is_finite() || req.delta_seconds < 0.0 {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "deltaSeconds must be a finite non-negative number",
        ));
    }
    let track_id = req
        .track_id
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let track_title = req
        .track_title
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let firkin_exists: Option<Firkin> = state
        .db
        .select((firkins::TABLE, firkin_id.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    if firkin_exists.is_none() {
        return Err(err_response(StatusCode::NOT_FOUND, "firkin not found"));
    }

    let id = record_id(&firkin_id, track_id.as_deref(), &address);
    let now = Utc::now();

    let existing: Option<MediaTracker> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;

    let saved: Option<MediaTracker> = match existing {
        Some(mut current) => {
            current.id = None;
            current.total_seconds += req.delta_seconds;
            current.last_played_at = now;
            current.updated_at = now;
            // Back-fill metadata that wasn't carried on prior heartbeats but
            // is now known. Once a row has a title, later beats without one
            // leave it intact.
            if current.track_id.is_none() {
                current.track_id = track_id.clone();
            }
            if let Some(t) = track_title.clone() {
                current.track_title = Some(t);
            }
            state
                .db
                .update((TABLE, id.as_str()))
                .content(current)
                .await
                .map_err(|e| {
                    err_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("db update failed: {e}"),
                    )
                })?
        }
        None => {
            let record = MediaTracker {
                id: None,
                firkin_id,
                track_id: track_id.clone(),
                track_title: track_title.clone(),
                address,
                total_seconds: req.delta_seconds,
                last_played_at: now,
                created_at: now,
                updated_at: now,
            };
            state
                .db
                .create((TABLE, id.as_str()))
                .content(record)
                .await
                .map_err(|e| {
                    err_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("db create failed: {e}"),
                    )
                })?
        }
    };

    let dto: MediaTrackerDto = saved
        .ok_or_else(|| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "media tracker was not persisted",
            )
        })?
        .into();
    Ok(Json(dto))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_id_is_deterministic_per_tuple() {
        let a = record_id("bafy1", None, "0xabc");
        let b = record_id("bafy1", None, "0xabc");
        assert_eq!(a, b);
        assert_ne!(a, record_id("bafy1", None, "0xdef"));
        assert_ne!(a, record_id("bafy2", None, "0xabc"));
    }

    #[test]
    fn record_id_separates_track_buckets() {
        let album = record_id("bafyAlbum", None, "0xuser");
        let t1 = record_id("bafyAlbum", Some("track-1"), "0xuser");
        let t2 = record_id("bafyAlbum", Some("track-2"), "0xuser");
        assert_ne!(album, t1);
        assert_ne!(album, t2);
        assert_ne!(t1, t2);
        assert_eq!(t1, record_id("bafyAlbum", Some("track-1"), "0xuser"));
    }

    #[test]
    fn normalize_address_lowercases_and_validates() {
        assert_eq!(
            normalize_address("0xAbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCd"),
            Some("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string())
        );
        assert_eq!(
            normalize_address("AbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCdEfAbCd"),
            Some("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string())
        );
        assert_eq!(normalize_address("0x1234"), None);
        assert_eq!(normalize_address("not-hex-not-hex-not-hex-not-hex-not-hex!"), None);
    }
}

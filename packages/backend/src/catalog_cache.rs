use crate::state::CloudState;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, Duration, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::future::Future;
use surrealdb::sql::Thing;

const TABLE: &str = "catalog_cache";
const TTL_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheRow {
    pub id: Option<Thing>,
    pub payload: String,
    pub cached_at: DateTime<Utc>,
}

fn cache_id(key: &str) -> String {
    let digest = Sha256::digest(key.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

async fn read_row(state: &CloudState, id: &str) -> Option<CacheRow> {
    match state.db.select((TABLE, id)).await {
        Ok(opt) => opt,
        Err(e) => {
            tracing::debug!("[catalog_cache] read failed for {id}: {e}");
            None
        }
    }
}

async fn write_row(state: &CloudState, id: &str, row: CacheRow) {
    let existing: Result<Option<CacheRow>, _> = state.db.select((TABLE, id)).await;
    let result: Result<Option<CacheRow>, _> = match existing {
        Ok(Some(_)) => state.db.update((TABLE, id)).content(row).await,
        Ok(None) => state.db.create((TABLE, id)).content(row).await,
        Err(e) => {
            tracing::debug!("[catalog_cache] existing-check failed for {id}: {e}");
            return;
        }
    };
    if let Err(e) = result {
        tracing::debug!("[catalog_cache] write failed for {id}: {e}");
    }
}

/// Look up `key` and return the parsed payload when a row exists and was
/// written less than [`TTL_HOURS`] ago. Returns `None` on miss, expired
/// row, malformed JSON, or DB read error — the caller falls through to
/// the upstream fetch in those cases.
pub async fn get<T>(state: &CloudState, key: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let id = cache_id(key);
    let row = read_row(state, &id).await?;
    let age = Utc::now().signed_duration_since(row.cached_at);
    if age > Duration::hours(TTL_HOURS) {
        return None;
    }
    match serde_json::from_str::<T>(&row.payload) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::debug!("[catalog_cache] parse failed for {key}: {e}");
            None
        }
    }
}

/// Persist `value` against `key`. Upsert by deterministic id so a later
/// fetch for the same key overwrites the prior row in place. Failures are
/// logged at debug level and swallowed.
pub async fn put<T>(state: &CloudState, key: &str, value: &T)
where
    T: Serialize,
{
    let id = cache_id(key);
    let payload = match serde_json::to_string(value) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("[catalog_cache] serialise failed for {key}: {e}");
            return;
        }
    };
    let row = CacheRow {
        id: None,
        payload,
        cached_at: Utc::now(),
    };
    write_row(state, &id, row).await;
}

/// Cache-aware wrapper. Returns the cached payload when fresh, otherwise
/// runs `fetch`, caches its successful result, and returns it. Errors
/// from `fetch` propagate without being cached so transient upstream
/// failures don't poison the 24h window.
pub async fn get_or_fetch<T, F, Fut>(
    state: &CloudState,
    key: &str,
    fetch: F,
) -> Result<T, (StatusCode, Json<serde_json::Value>)>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, (StatusCode, Json<serde_json::Value>)>>,
{
    if let Some(cached) = get::<T>(state, key).await {
        return Ok(cached);
    }
    let value = fetch().await?;
    put(state, key, &value).await;
    Ok(value)
}

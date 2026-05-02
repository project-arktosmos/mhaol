//! SurrealDB-backed `FilestoreIndex` implementation. Persists every leaf
//! entry recorded by `IpfsManager::compute_file_cid` so the IPFS node can
//! reconstruct leaf blocks from on-disk source files across restarts —
//! without that persistence, the embedded IPFS node would lose its view
//! of every library file at boot and bitswap reads would 404 until a
//! re-scan.
//!
//! On the read side we keep an in-process `HashMap<Cid, FilestoreEntry>`
//! cache populated at boot from `ipfs_filestore_entry`. Reads (which
//! happen on every bitswap "have" / "want" round-trip) hit the cache
//! only; writes (which fire once per leaf during a scan) write through
//! to SurrealDB and update the cache.

#![cfg(not(target_os = "android"))]

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cid::Cid;
use mhaol_ipfs_core::{FilestoreEntry, FilestoreIndex};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

const TABLE: &str = "ipfs_filestore_entry";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredEntry {
    pub id: Option<Thing>,
    pub cid: String,
    pub path: String,
    pub offset: u64,
    pub length: u32,
    pub created_at: DateTime<Utc>,
}

impl StoredEntry {
    fn into_filestore(self) -> Option<FilestoreEntry> {
        let cid = Cid::from_str(&self.cid).ok()?;
        Some(FilestoreEntry {
            cid,
            path: PathBuf::from(self.path),
            offset: self.offset,
            length: self.length,
        })
    }
}

/// SurrealDB record id for a single leaf entry. Deterministic on the CID
/// so re-recording an entry overwrites the row rather than stacking
/// duplicates — useful when a file moves and its leaves get re-indexed
/// against the new path.
fn entry_record_id(cid: &str) -> String {
    let digest = Sha256::digest(cid.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// SurrealDB-backed implementation of `FilestoreIndex`. Read-through cache
/// keeps the bitswap path cheap; writes round-trip to disk on every scan
/// step but the volume is bounded (one row per ~256 KiB chunk, and only
/// once per file across re-scans).
#[derive(Debug)]
pub struct SurrealFilestoreIndex {
    db: Surreal<Db>,
    cache: RwLock<HashMap<Cid, FilestoreEntry>>,
}

impl SurrealFilestoreIndex {
    /// Build the index, eagerly loading every persisted entry into the
    /// in-memory cache. Called once at boot — `bitswap` reads must not
    /// touch the disk on the hot path. The cost scales linearly with the
    /// library size: a 200 GB library at 256 KiB chunks is ~800K entries
    /// (~64 MB resident memory), comparable to a single fully-loaded
    /// browser tab.
    pub async fn load(db: Surreal<Db>) -> Result<Self> {
        let stored: Vec<StoredEntry> = db
            .select(TABLE)
            .await
            .map_err(|e| anyhow!("[filestore] db select failed: {e}"))?;
        let mut cache: HashMap<Cid, FilestoreEntry> = HashMap::with_capacity(stored.len());
        for s in stored {
            if let Some(entry) = s.into_filestore() {
                cache.insert(entry.cid, entry);
            }
        }
        tracing::info!(
            "[filestore] loaded {} leaf entries from {}",
            cache.len(),
            TABLE
        );
        Ok(Self {
            db,
            cache: RwLock::new(cache),
        })
    }
}

#[async_trait]
impl FilestoreIndex for SurrealFilestoreIndex {
    async fn lookup(&self, cid: &Cid) -> Option<FilestoreEntry> {
        // Cache-only read path. `record_leaf` keeps the cache in sync, so
        // hitting disk here would just slow bitswap down with no benefit.
        self.cache.read().get(cid).cloned()
    }

    async fn record_leaf(&self, entry: FilestoreEntry) -> Result<()> {
        let id = entry_record_id(&entry.cid.to_string());
        let path_str = entry.path.to_string_lossy().to_string();
        let row = StoredEntry {
            id: None,
            cid: entry.cid.to_string(),
            path: path_str,
            offset: entry.offset,
            length: entry.length,
            created_at: Utc::now(),
        };
        // Upsert: try update first (deterministic id), fall back to
        // create when no row exists. SurrealDB doesn't have a single
        // "upsert" call here.
        let updated: Result<Option<StoredEntry>, _> =
            self.db.update((TABLE, id.as_str())).content(row.clone()).await;
        match updated {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _: Option<StoredEntry> = self
                    .db
                    .create((TABLE, id.as_str()))
                    .content(row)
                    .await
                    .map_err(|e| anyhow!("[filestore] db create failed: {e}"))?;
            }
            Err(e) => return Err(anyhow!("[filestore] db update failed: {e}")),
        }
        self.cache.write().insert(entry.cid, entry);
        Ok(())
    }

    async fn remove(&self, cid: &Cid) -> Result<()> {
        let id = entry_record_id(&cid.to_string());
        let _: Option<StoredEntry> = self
            .db
            .delete((TABLE, id.as_str()))
            .await
            .map_err(|e| anyhow!("[filestore] db delete failed: {e}"))?;
        self.cache.write().remove(cid);
        Ok(())
    }

    async fn list_cids(&self) -> Vec<Cid> {
        self.cache.read().keys().copied().collect()
    }

    async fn total_size(&self) -> u64 {
        self.cache.read().values().map(|e| e.length as u64).sum()
    }
}


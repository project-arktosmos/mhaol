use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFetchCacheRow {
    pub tmdb_id: i64,
    pub media_type: String,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct TorrentFetchCacheRepo {
    db: DbPool,
}

impl TorrentFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, tmdb_id: i64) -> Option<TorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT tmdb_id, media_type, candidate_json, created_at FROM torrent_fetch_cache WHERE tmdb_id = ?1",
            params![tmdb_id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn upsert(&self, tmdb_id: i64, media_type: &str, candidate_json: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO torrent_fetch_cache (tmdb_id, media_type, candidate_json)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(tmdb_id) DO UPDATE SET
                media_type = excluded.media_type,
                candidate_json = excluded.candidate_json,
                created_at = datetime('now')",
            params![tmdb_id, media_type, candidate_json],
        )
        .unwrap();
    }

    pub fn get_all_tmdb_ids(&self) -> Vec<i64> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT tmdb_id FROM torrent_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_all_info_hashes(&self) -> Vec<(i64, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT tmdb_id, candidate_json FROM torrent_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| {
            let tmdb_id: i64 = row.get(0)?;
            let json: String = row.get(1)?;
            Ok((tmdb_id, json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(tmdb_id, json)| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            let hash = v.get("infoHash")?.as_str()?.to_lowercase();
            Some((tmdb_id, hash))
        })
        .collect()
    }

    pub fn get_all_summaries(&self) -> Vec<(i64, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT tmdb_id, candidate_json FROM torrent_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| {
            let tmdb_id: i64 = row.get(0)?;
            let json: String = row.get(1)?;
            Ok((tmdb_id, json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(tmdb_id, json)| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            let name = v.get("name")?.as_str()?.to_string();
            Some((tmdb_id, name))
        })
        .collect()
    }

    pub fn delete(&self, tmdb_id: i64) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM torrent_fetch_cache WHERE tmdb_id = ?1",
            params![tmdb_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<TorrentFetchCacheRow> {
        Ok(TorrentFetchCacheRow {
            tmdb_id: row.get(0)?,
            media_type: row.get(1)?,
            candidate_json: row.get(2)?,
            created_at: row.get(3)?,
        })
    }
}

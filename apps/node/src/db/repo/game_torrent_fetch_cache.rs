use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTorrentFetchCacheRow {
    pub ra_game_id: i64,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct GameTorrentFetchCacheRepo {
    db: DbPool,
}

impl GameTorrentFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, ra_game_id: i64) -> Option<GameTorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT ra_game_id, candidate_json, created_at FROM game_torrent_fetch_cache WHERE ra_game_id = ?1",
            params![ra_game_id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn upsert(&self, ra_game_id: i64, candidate_json: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO game_torrent_fetch_cache (ra_game_id, candidate_json)
             VALUES (?1, ?2)
             ON CONFLICT(ra_game_id) DO UPDATE SET
                candidate_json = excluded.candidate_json,
                created_at = datetime('now')",
            params![ra_game_id, candidate_json],
        )
        .unwrap();
    }

    pub fn delete(&self, ra_game_id: i64) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM game_torrent_fetch_cache WHERE ra_game_id = ?1",
            params![ra_game_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameTorrentFetchCacheRow> {
        Ok(GameTorrentFetchCacheRow {
            ra_game_id: row.get(0)?,
            candidate_json: row.get(1)?,
            created_at: row.get(2)?,
        })
    }
}

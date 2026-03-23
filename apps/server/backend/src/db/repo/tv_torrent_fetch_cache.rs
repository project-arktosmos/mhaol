use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvTorrentFetchCacheRow {
    pub id: String,
    pub tmdb_id: i64,
    pub scope: String,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct TvTorrentFetchCacheRepo {
    db: DbPool,
}

impl TvTorrentFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_for_show(&self, tmdb_id: i64) -> Vec<TvTorrentFetchCacheRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, tmdb_id, scope, season_number, episode_number, candidate_json, created_at
                 FROM tv_torrent_fetch_cache WHERE tmdb_id = ?1
                 ORDER BY scope, season_number, episode_number",
            )
            .unwrap();
        stmt.query_map(params![tmdb_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_complete(&self, tmdb_id: i64) -> Option<TvTorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, tmdb_id, scope, season_number, episode_number, candidate_json, created_at
             FROM tv_torrent_fetch_cache WHERE tmdb_id = ?1 AND scope = 'complete'",
            params![tmdb_id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_season(&self, tmdb_id: i64, season_number: i64) -> Option<TvTorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, tmdb_id, scope, season_number, episode_number, candidate_json, created_at
             FROM tv_torrent_fetch_cache WHERE tmdb_id = ?1 AND scope = 'season' AND season_number = ?2",
            params![tmdb_id, season_number],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_episode(
        &self,
        tmdb_id: i64,
        season_number: i64,
        episode_number: i64,
    ) -> Option<TvTorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, tmdb_id, scope, season_number, episode_number, candidate_json, created_at
             FROM tv_torrent_fetch_cache WHERE tmdb_id = ?1 AND scope = 'episode' AND season_number = ?2 AND episode_number = ?3",
            params![tmdb_id, season_number, episode_number],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn upsert(
        &self,
        tmdb_id: i64,
        scope: &str,
        season_number: Option<i64>,
        episode_number: Option<i64>,
        candidate_json: &str,
    ) {
        let conn = self.db.lock();
        let id = format!("{:032x}", rand_id());
        conn.execute(
            "INSERT INTO tv_torrent_fetch_cache (id, tmdb_id, scope, season_number, episode_number, candidate_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(tmdb_id, scope, COALESCE(season_number, -1), COALESCE(episode_number, -1))
             DO UPDATE SET candidate_json = excluded.candidate_json, created_at = datetime('now')",
            params![id, tmdb_id, scope, season_number, episode_number, candidate_json],
        )
        .unwrap();
    }

    pub fn delete_for_show(&self, tmdb_id: i64) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM tv_torrent_fetch_cache WHERE tmdb_id = ?1",
            params![tmdb_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<TvTorrentFetchCacheRow> {
        Ok(TvTorrentFetchCacheRow {
            id: row.get(0)?,
            tmdb_id: row.get(1)?,
            scope: row.get(2)?,
            season_number: row.get(3)?,
            episode_number: row.get(4)?,
            candidate_json: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}

fn rand_id() -> u128 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64);
    let a = h.finish() as u128;
    let mut h2 = s.build_hasher();
    h2.write_u64(a as u64 ^ 0xdeadbeef);
    let b = h2.finish() as u128;
    (a << 64) | b
}

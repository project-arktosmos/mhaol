use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTorrentFetchCacheRow {
    pub id: String,
    pub musicbrainz_id: String,
    pub scope: String,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MusicTorrentFetchCacheRepo {
    db: DbPool,
}

impl MusicTorrentFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_for_id(&self, musicbrainz_id: &str) -> Vec<MusicTorrentFetchCacheRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, musicbrainz_id, scope, candidate_json, created_at
                 FROM music_torrent_fetch_cache WHERE musicbrainz_id = ?1
                 ORDER BY scope",
            )
            .unwrap();
        stmt.query_map(params![musicbrainz_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn upsert(&self, musicbrainz_id: &str, scope: &str, candidate_json: &str) {
        let conn = self.db.lock();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO music_torrent_fetch_cache (id, musicbrainz_id, scope, candidate_json)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(musicbrainz_id, scope)
             DO UPDATE SET candidate_json = excluded.candidate_json, created_at = datetime('now')",
            params![id, musicbrainz_id, scope, candidate_json],
        )
        .unwrap();
    }

    pub fn get_all_info_hashes(&self) -> Vec<(String, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT musicbrainz_id, candidate_json FROM music_torrent_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| {
            let musicbrainz_id: String = row.get(0)?;
            let json: String = row.get(1)?;
            Ok((musicbrainz_id, json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(musicbrainz_id, json)| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            let hash = v.get("infoHash")?.as_str()?.to_lowercase();
            Some((musicbrainz_id, hash))
        })
        .collect()
    }

    pub fn delete_for_id(&self, musicbrainz_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM music_torrent_fetch_cache WHERE musicbrainz_id = ?1",
            params![musicbrainz_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<MusicTorrentFetchCacheRow> {
        Ok(MusicTorrentFetchCacheRow {
            id: row.get(0)?,
            musicbrainz_id: row.get(1)?,
            scope: row.get(2)?,
            candidate_json: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

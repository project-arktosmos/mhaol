use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookTorrentFetchCacheRow {
    pub openlibrary_key: String,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct BookTorrentFetchCacheRepo {
    db: DbPool,
}

impl BookTorrentFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, openlibrary_key: &str) -> Option<BookTorrentFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT openlibrary_key, candidate_json, created_at FROM book_torrent_fetch_cache WHERE openlibrary_key = ?1",
            params![openlibrary_key],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn upsert(&self, openlibrary_key: &str, candidate_json: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO book_torrent_fetch_cache (openlibrary_key, candidate_json)
             VALUES (?1, ?2)
             ON CONFLICT(openlibrary_key) DO UPDATE SET
                candidate_json = excluded.candidate_json,
                created_at = datetime('now')",
            params![openlibrary_key, candidate_json],
        )
        .unwrap();
    }

    pub fn delete(&self, openlibrary_key: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM book_torrent_fetch_cache WHERE openlibrary_key = ?1",
            params![openlibrary_key],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<BookTorrentFetchCacheRow> {
        Ok(BookTorrentFetchCacheRow {
            openlibrary_key: row.get(0)?,
            candidate_json: row.get(1)?,
            created_at: row.get(2)?,
        })
    }
}

use crate::db::DbPool;
use rusqlite::params;

#[derive(Clone)]
pub struct OpenLibraryApiCacheRepo {
    db: DbPool,
}

impl OpenLibraryApiCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Get cached response. Returns `(data, is_stale)` where stale means older than 24 hours.
    /// Returns `None` if no cache entry exists.
    pub fn get(&self, cache_key: &str) -> Option<(String, bool)> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT data, (julianday('now') - julianday(fetched_at)) * 24 > 24 AS is_stale
             FROM openlibrary_api_cache WHERE cache_key = ?1",
            params![cache_key],
            |row| {
                let data: String = row.get(0)?;
                let is_stale: bool = row.get(1)?;
                Ok((data, is_stale))
            },
        )
        .ok()
    }

    pub fn upsert(&self, cache_key: &str, data: &str) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO openlibrary_api_cache (cache_key, data)
             VALUES (?1, ?2)
             ON CONFLICT(cache_key) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
            params![cache_key, data],
        );
    }
}

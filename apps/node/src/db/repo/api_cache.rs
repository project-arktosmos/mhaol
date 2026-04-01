use crate::db::DbPool;
use rusqlite::params;

#[derive(Clone)]
pub struct ApiCacheRepo {
    db: DbPool,
}

impl ApiCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Get cached data. Returns `(data, is_stale)` where stale = older than `max_age_hours`.
    /// Pass a very large value (e.g. `f64::MAX`) for caches that never expire.
    pub fn get(&self, source: &str, cache_key: &str, max_age_hours: f64) -> Option<(String, bool)> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT data, (julianday('now') - julianday(fetched_at)) * 24 > ?3 AS is_stale
             FROM api_cache WHERE source = ?1 AND cache_key = ?2",
            params![source, cache_key, max_age_hours],
            |row| {
                let data: String = row.get(0)?;
                let is_stale: bool = row.get(1)?;
                Ok((data, is_stale))
            },
        )
        .ok()
    }

    /// Get cached data only if fresh (younger than `max_age_hours`). Returns `None` if stale or missing.
    pub fn get_fresh(
        &self,
        source: &str,
        cache_key: &str,
        max_age_hours: i64,
    ) -> Option<String> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT data FROM api_cache
             WHERE source = ?1 AND cache_key = ?2
             AND fetched_at > datetime('now', '-' || CAST(?3 AS TEXT) || ' hours')",
            params![source, cache_key, max_age_hours],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    /// Get cached data regardless of age. Returns `None` only if missing.
    pub fn get_any(&self, source: &str, cache_key: &str) -> Option<String> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT data FROM api_cache WHERE source = ?1 AND cache_key = ?2",
            params![source, cache_key],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    /// Get cached data and fetched_at timestamp regardless of age.
    pub fn get_any_with_timestamp(
        &self,
        source: &str,
        cache_key: &str,
    ) -> Option<(String, String)> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT data, fetched_at FROM api_cache WHERE source = ?1 AND cache_key = ?2",
            params![source, cache_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .ok()
    }

    /// Get all entries for a source whose cache_key starts with `prefix`.
    pub fn get_all_for_prefix(&self, source: &str, prefix: &str) -> Vec<(String, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT cache_key, data FROM api_cache WHERE source = ?1 AND cache_key LIKE ?2",
            )
            .unwrap();
        stmt.query_map(params![source, format!("{}%", prefix)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn upsert(&self, source: &str, cache_key: &str, data: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO api_cache (source, cache_key, data)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(source, cache_key) DO UPDATE SET
                data = excluded.data,
                fetched_at = datetime('now')",
            params![source, cache_key, data],
        )
        .unwrap();
    }

    pub fn delete(&self, source: &str, cache_key: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM api_cache WHERE source = ?1 AND cache_key = ?2",
            params![source, cache_key],
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_upsert_and_get_any() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "movie:550", r#"{"title":"Fight Club"}"#);
        let data = repo.get_any("tmdb", "movie:550").unwrap();
        assert!(data.contains("Fight Club"));
    }

    #[test]
    fn test_get_with_staleness() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "search:foo", r#"{"results":[]}"#);

        // Should not be stale with 24h window
        let (data, is_stale) = repo.get("tmdb", "search:foo", 24.0).unwrap();
        assert!(!is_stale);
        assert!(data.contains("results"));

        // Should be stale with 0h window
        let (_, is_stale) = repo.get("tmdb", "search:foo", 0.0).unwrap();
        assert!(is_stale);
    }

    #[test]
    fn test_get_fresh() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("musicbrainz", "popular:rock", r#"{"items":[]}"#);

        // Fresh with 24h window
        assert!(repo.get_fresh("musicbrainz", "popular:rock", 24).is_some());

        // Not fresh with 0h window
        assert!(repo.get_fresh("musicbrainz", "popular:rock", 0).is_none());
    }

    #[test]
    fn test_get_with_timestamp() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "movie:1", r#"{"id":1}"#);
        let (data, ts) = repo.get_any_with_timestamp("tmdb", "movie:1").unwrap();
        assert!(data.contains("\"id\":1"));
        assert!(!ts.is_empty());
    }

    #[test]
    fn test_get_all_for_prefix() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("retroachievements", "game-list:1", r#"{"games":[]}"#);
        repo.upsert("retroachievements", "game-list:2", r#"{"games":[]}"#);
        repo.upsert("retroachievements", "game-details:5", r#"{"id":5}"#);

        let results = repo.get_all_for_prefix("retroachievements", "game-list:");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_delete() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "movie:1", r#"{"id":1}"#);
        assert!(repo.get_any("tmdb", "movie:1").is_some());

        repo.delete("tmdb", "movie:1");
        assert!(repo.get_any("tmdb", "movie:1").is_none());
    }

    #[test]
    fn test_upsert_replaces() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "movie:1", r#"{"title":"Old"}"#);
        repo.upsert("tmdb", "movie:1", r#"{"title":"New"}"#);

        let data = repo.get_any("tmdb", "movie:1").unwrap();
        assert!(data.contains("New"));
        assert!(!data.contains("Old"));
    }

    #[test]
    fn test_different_sources_same_key() {
        let db = open_test_database();
        let repo = ApiCacheRepo::new(db);

        repo.upsert("tmdb", "key1", r#"{"source":"tmdb"}"#);
        repo.upsert("musicbrainz", "key1", r#"{"source":"mb"}"#);

        let tmdb = repo.get_any("tmdb", "key1").unwrap();
        assert!(tmdb.contains("tmdb"));
        let mb = repo.get_any("musicbrainz", "key1").unwrap();
        assert!(mb.contains("mb"));
    }
}

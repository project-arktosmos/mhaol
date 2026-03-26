use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::sync::Arc;

pub type DbPool = Arc<Mutex<Connection>>;

pub const TASK_PREFIX: &str = "recommendations:";
pub const TASK_FETCH: &str = "recommendations:fetch";

pub const RECOMMENDATIONS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS tmdb_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_tmdb_id INTEGER NOT NULL,
    source_media_type TEXT NOT NULL CHECK (source_media_type IN ('movie', 'tv')),
    recommended_tmdb_id INTEGER NOT NULL,
    recommended_media_type TEXT NOT NULL CHECK (recommended_media_type IN ('movie', 'tv')),
    title TEXT,
    genres TEXT,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_tmdb_id, source_media_type, recommended_tmdb_id)
);
CREATE INDEX IF NOT EXISTS idx_tmdb_recs_source
    ON tmdb_recommendations(source_tmdb_id, source_media_type);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationRow {
    pub id: i64,
    pub source_tmdb_id: i64,
    pub source_media_type: String,
    pub recommended_tmdb_id: i64,
    pub recommended_media_type: String,
    pub title: Option<String>,
    pub genres: Option<String>,
    pub data: serde_json::Value,
    pub fetched_at: String,
}

#[derive(Clone)]
pub struct RecommendationsRepo {
    db: DbPool,
}

impl RecommendationsRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn upsert(
        &self,
        source_tmdb_id: i64,
        source_media_type: &str,
        rec_tmdb_id: i64,
        rec_media_type: &str,
        title: Option<&str>,
        genres: Option<&str>,
        data: &str,
    ) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO tmdb_recommendations (source_tmdb_id, source_media_type, recommended_tmdb_id, recommended_media_type, title, genres, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(source_tmdb_id, source_media_type, recommended_tmdb_id)
             DO UPDATE SET title = ?5, genres = ?6, data = ?7, fetched_at = datetime('now')",
            params![source_tmdb_id, source_media_type, rec_tmdb_id, rec_media_type, title, genres, data],
        );
    }

    pub fn get_for_source(&self, source_tmdb_id: i64, media_type: &str) -> Vec<RecommendationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, source_tmdb_id, source_media_type, recommended_tmdb_id, recommended_media_type, title, genres, data, fetched_at
                 FROM tmdb_recommendations
                 WHERE source_tmdb_id = ?1 AND source_media_type = ?2
                 ORDER BY id ASC",
            )
            .unwrap();
        stmt.query_map(params![source_tmdb_id, media_type], |row| {
            let data_str: String = row.get(7)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok(RecommendationRow {
                id: row.get(0)?,
                source_tmdb_id: row.get(1)?,
                source_media_type: row.get(2)?,
                recommended_tmdb_id: row.get(3)?,
                recommended_media_type: row.get(4)?,
                title: row.get(5)?,
                genres: row.get(6)?,
                data,
                fetched_at: row.get(8)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn delete_for_source(&self, source_tmdb_id: i64, media_type: &str) -> usize {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM tmdb_recommendations WHERE source_tmdb_id = ?1 AND source_media_type = ?2",
            params![source_tmdb_id, media_type],
        )
        .unwrap_or(0)
    }

    pub fn top_recommended_movies_with_data(
        &self,
        limit: usize,
    ) -> Vec<(i64, String, Option<String>, i64, serde_json::Value)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt, data
                 FROM tmdb_recommendations
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?1",
            )
            .unwrap();
        stmt.query_map(params![limit as i64], |row| {
            let data_str: String = row.get(4)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, data))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn top_recommended_movies(&self, limit: usize) -> Vec<(i64, String, Option<String>, i64)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt
                 FROM tmdb_recommendations
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?1",
            )
            .unwrap();
        stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn top_genres(&self, limit: usize) -> Vec<(String, i64)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT genres FROM tmdb_recommendations WHERE genres IS NOT NULL AND genres != ''")
            .unwrap();
        let rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let mut counts = std::collections::HashMap::<String, i64>::new();
        for genres_str in &rows {
            for genre in genres_str.split(',') {
                let g = genre.trim();
                if !g.is_empty() {
                    *counts.entry(g.to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut sorted: Vec<(String, i64)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        sorted
    }

    pub fn top_recommended_by_source_type(
        &self,
        source_media_type: &str,
        limit: usize,
    ) -> Vec<(i64, String, Option<String>, i64)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt
                 FROM tmdb_recommendations
                 WHERE source_media_type = ?1
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?2",
            )
            .unwrap();
        stmt.query_map(params![source_media_type, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn list_sources(&self) -> Vec<(i64, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT DISTINCT source_tmdb_id, source_media_type FROM tmdb_recommendations ORDER BY source_tmdb_id")
            .unwrap();
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> RecommendationsRepo {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(RECOMMENDATIONS_SCHEMA_SQL).unwrap();
        let db = Arc::new(Mutex::new(conn));
        RecommendationsRepo::new(db)
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), Some("Crime, Drama"), r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", Some("Forrest Gump"), Some("Drama, Comedy"), r#"{"id":13}"#);

        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].recommended_tmdb_id, 680);
        assert_eq!(recs[0].title.as_deref(), Some("Pulp Fiction"));
        assert_eq!(recs[1].recommended_tmdb_id, 13);
    }

    #[test]
    fn test_upsert_dedup() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Old Title"), None, r#"{"id":680}"#);
        repo.upsert(550, "movie", 680, "movie", Some("New Title"), Some("Action"), r#"{"id":680,"updated":true}"#);

        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].title.as_deref(), Some("New Title"));
    }

    #[test]
    fn test_different_media_types() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Movie Rec"), None, r#"{"id":680}"#);
        repo.upsert(100, "tv", 200, "tv", Some("TV Rec"), None, r#"{"id":200}"#);

        assert_eq!(repo.get_for_source(550, "movie").len(), 1);
        assert_eq!(repo.get_for_source(100, "tv").len(), 1);
        assert_eq!(repo.get_for_source(550, "tv").len(), 0);
    }

    #[test]
    fn test_delete_for_source() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("A"), None, r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", Some("B"), None, r#"{"id":13}"#);
        repo.upsert(100, "tv", 200, "tv", Some("C"), None, r#"{"id":200}"#);

        let deleted = repo.delete_for_source(550, "movie");
        assert_eq!(deleted, 2);
        assert_eq!(repo.get_for_source(550, "movie").len(), 0);
        assert_eq!(repo.get_for_source(100, "tv").len(), 1);
    }

    #[test]
    fn test_list_sources() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", None, None, r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", None, None, r#"{"id":13}"#);
        repo.upsert(100, "tv", 200, "tv", None, None, r#"{"id":200}"#);

        let sources = repo.list_sources();
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&(100, "tv".to_string())));
        assert!(sources.contains(&(550, "movie".to_string())));
    }
}

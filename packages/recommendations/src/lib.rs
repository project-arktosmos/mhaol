pub mod music;

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
    level INTEGER NOT NULL DEFAULT 1,
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
    pub level: i64,
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
        level: i64,
        data: &str,
    ) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO tmdb_recommendations (source_tmdb_id, source_media_type, recommended_tmdb_id, recommended_media_type, title, genres, level, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(source_tmdb_id, source_media_type, recommended_tmdb_id)
             DO UPDATE SET title = ?5, genres = ?6, level = MIN(level, ?7), data = ?8, fetched_at = datetime('now')",
            params![source_tmdb_id, source_media_type, rec_tmdb_id, rec_media_type, title, genres, level, data],
        );
    }

    pub fn get_for_source(&self, source_tmdb_id: i64, media_type: &str) -> Vec<RecommendationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, source_tmdb_id, source_media_type, recommended_tmdb_id, recommended_media_type, title, genres, level, data, fetched_at
                 FROM tmdb_recommendations
                 WHERE source_tmdb_id = ?1 AND source_media_type = ?2
                 ORDER BY id ASC",
            )
            .unwrap();
        stmt.query_map(params![source_tmdb_id, media_type], |row| {
            let data_str: String = row.get(8)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok(RecommendationRow {
                id: row.get(0)?,
                source_tmdb_id: row.get(1)?,
                source_media_type: row.get(2)?,
                recommended_tmdb_id: row.get(3)?,
                recommended_media_type: row.get(4)?,
                title: row.get(5)?,
                genres: row.get(6)?,
                level: row.get(7)?,
                data,
                fetched_at: row.get(9)?,
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
    ) -> Vec<(i64, String, Option<String>, i64, i64, serde_json::Value)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt, MIN(level) as min_level, data
                 FROM tmdb_recommendations
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?1",
            )
            .unwrap();
        stmt.query_map(params![limit as i64], |row| {
            let data_str: String = row.get(5)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, data))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn top_recommended_movies(&self, limit: usize) -> Vec<(i64, String, Option<String>, i64, i64)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt, MIN(level) as min_level
                 FROM tmdb_recommendations
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?1",
            )
            .unwrap();
        stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
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
    ) -> Vec<(i64, String, Option<String>, i64, i64)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_tmdb_id, recommended_media_type, title, COUNT(*) as cnt, MIN(level) as min_level
                 FROM tmdb_recommendations
                 WHERE source_media_type = ?1
                 GROUP BY recommended_tmdb_id, recommended_media_type
                 ORDER BY cnt DESC
                 LIMIT ?2",
            )
            .unwrap();
        stmt.query_map(params![source_media_type, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Top recommended movies with per-level count breakdown.
    /// Returns (tmdb_id, media_type, title, total_count, level_counts).
    pub fn top_recommended_with_level_counts(
        &self,
        source_media_type: Option<&str>,
        limit: usize,
    ) -> Vec<(i64, String, Option<String>, i64, std::collections::HashMap<i64, i64>)> {
        let conn = self.db.lock();
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
            if let Some(mt) = source_media_type {
                (
                    "SELECT recommended_tmdb_id, recommended_media_type, title, level, COUNT(*) as cnt
                     FROM tmdb_recommendations
                     WHERE source_media_type = ?1
                     GROUP BY recommended_tmdb_id, recommended_media_type, level"
                        .to_string(),
                    vec![Box::new(mt.to_string()) as Box<dyn rusqlite::types::ToSql>],
                )
            } else {
                (
                    "SELECT recommended_tmdb_id, recommended_media_type, title, level, COUNT(*) as cnt
                     FROM tmdb_recommendations
                     GROUP BY recommended_tmdb_id, recommended_media_type, level"
                        .to_string(),
                    vec![],
                )
            };

        let mut stmt = conn.prepare(&sql).unwrap();
        let rows: Vec<(i64, String, Option<String>, i64, i64)> = stmt
            .query_map(rusqlite::params_from_iter(&params_vec), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        // Aggregate by (tmdb_id, media_type)
        let mut map: std::collections::HashMap<
            (i64, String),
            (Option<String>, i64, std::collections::HashMap<i64, i64>),
        > = std::collections::HashMap::new();
        for (tmdb_id, media_type, title, level, cnt) in rows {
            let entry = map
                .entry((tmdb_id, media_type))
                .or_insert_with(|| (title.clone(), 0, std::collections::HashMap::new()));
            entry.1 += cnt;
            *entry.2.entry(level).or_insert(0) += cnt;
            if entry.0.is_none() && title.is_some() {
                entry.0 = title;
            }
        }

        let mut result: Vec<_> = map
            .into_iter()
            .map(|((tmdb_id, media_type), (title, total, levels))| {
                (tmdb_id, media_type, title, total, levels)
            })
            .collect();
        result.sort_by(|a, b| b.3.cmp(&a.3));
        result.truncate(limit);
        result
    }

    /// For each recommended_tmdb_id, return its source movies: (recommended_tmdb_id, source_tmdb_id, source_media_type, source_title).
    /// Source title is resolved via self-join (where the source appears as a recommended movie elsewhere).
    pub fn sources_for_recommended(&self, recommended_ids: &[i64]) -> Vec<(i64, i64, String, Option<String>)> {
        if recommended_ids.is_empty() {
            return vec![];
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = recommended_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT r.recommended_tmdb_id, r.source_tmdb_id, r.source_media_type,
                    (SELECT r2.title FROM tmdb_recommendations r2
                     WHERE r2.recommended_tmdb_id = r.source_tmdb_id LIMIT 1) as source_title
             FROM tmdb_recommendations r
             WHERE r.recommended_tmdb_id IN ({})
             GROUP BY r.recommended_tmdb_id, r.source_tmdb_id",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&sql).unwrap();
        let params: Vec<&dyn rusqlite::ToSql> = recommended_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        stmt.query_map(params.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn has_source(&self, tmdb_id: i64, media_type: &str) -> bool {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT 1 FROM tmdb_recommendations WHERE source_tmdb_id = ?1 AND source_media_type = ?2 LIMIT 1",
            )
            .unwrap();
        stmt.exists(params![tmdb_id, media_type]).unwrap_or(false)
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
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), Some("Crime, Drama"), 1, r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", Some("Forrest Gump"), Some("Drama, Comedy"), 1, r#"{"id":13}"#);

        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].recommended_tmdb_id, 680);
        assert_eq!(recs[0].title.as_deref(), Some("Pulp Fiction"));
        assert_eq!(recs[0].level, 1);
        assert_eq!(recs[1].recommended_tmdb_id, 13);
    }

    #[test]
    fn test_upsert_dedup() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Old Title"), None, 1, r#"{"id":680}"#);
        repo.upsert(550, "movie", 680, "movie", Some("New Title"), Some("Action"), 1, r#"{"id":680,"updated":true}"#);

        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].title.as_deref(), Some("New Title"));
    }

    #[test]
    fn test_upsert_keeps_min_level() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), None, 3, r#"{"id":680}"#);
        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs[0].level, 3);

        // Re-upsert at a lower level — should keep the lower one
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), None, 1, r#"{"id":680}"#);
        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs[0].level, 1);

        // Re-upsert at a higher level — should still keep 1
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), None, 5, r#"{"id":680}"#);
        let recs = repo.get_for_source(550, "movie");
        assert_eq!(recs[0].level, 1);
    }

    #[test]
    fn test_different_media_types() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("Movie Rec"), None, 1, r#"{"id":680}"#);
        repo.upsert(100, "tv", 200, "tv", Some("TV Rec"), None, 1, r#"{"id":200}"#);

        assert_eq!(repo.get_for_source(550, "movie").len(), 1);
        assert_eq!(repo.get_for_source(100, "tv").len(), 1);
        assert_eq!(repo.get_for_source(550, "tv").len(), 0);
    }

    #[test]
    fn test_delete_for_source() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", Some("A"), None, 1, r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", Some("B"), None, 1, r#"{"id":13}"#);
        repo.upsert(100, "tv", 200, "tv", Some("C"), None, 1, r#"{"id":200}"#);

        let deleted = repo.delete_for_source(550, "movie");
        assert_eq!(deleted, 2);
        assert_eq!(repo.get_for_source(550, "movie").len(), 0);
        assert_eq!(repo.get_for_source(100, "tv").len(), 1);
    }

    #[test]
    fn test_has_source() {
        let repo = setup();
        assert!(!repo.has_source(550, "movie"));

        repo.upsert(550, "movie", 680, "movie", None, None, 1, r#"{"id":680}"#);
        assert!(repo.has_source(550, "movie"));
        assert!(!repo.has_source(550, "tv"));
        assert!(!repo.has_source(680, "movie"));
    }

    #[test]
    fn test_top_movies_min_level() {
        let repo = setup();
        // Movie 680 recommended at level 1 by source 550
        repo.upsert(550, "movie", 680, "movie", Some("Pulp Fiction"), None, 1, r#"{"id":680}"#);
        // Movie 680 also recommended at level 2 by source 100
        repo.upsert(100, "movie", 680, "movie", Some("Pulp Fiction"), None, 2, r#"{"id":680}"#);
        // Movie 13 only at level 3
        repo.upsert(200, "movie", 13, "movie", Some("Forrest Gump"), None, 3, r#"{"id":13}"#);

        let top = repo.top_recommended_movies(10);
        // 680 has count=2, 13 has count=1
        assert_eq!(top[0].0, 680);
        assert_eq!(top[0].3, 2); // count
        assert_eq!(top[0].4, 1); // min_level
        assert_eq!(top[1].0, 13);
        assert_eq!(top[1].3, 1); // count
        assert_eq!(top[1].4, 3); // min_level
    }

    #[test]
    fn test_list_sources() {
        let repo = setup();
        repo.upsert(550, "movie", 680, "movie", None, None, 1, r#"{"id":680}"#);
        repo.upsert(550, "movie", 13, "movie", None, None, 1, r#"{"id":13}"#);
        repo.upsert(100, "tv", 200, "tv", None, None, 1, r#"{"id":200}"#);

        let sources = repo.list_sources();
        assert_eq!(sources.len(), 2);
        assert!(sources.contains(&(100, "tv".to_string())));
        assert!(sources.contains(&(550, "movie".to_string())));
    }
}

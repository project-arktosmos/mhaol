use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

pub type DbPool = Arc<Mutex<Connection>>;

pub const BOOK_TASK_PREFIX: &str = "book-recommendations:";
pub const BOOK_TASK_FETCH: &str = "book-recommendations:fetch";

pub const BOOK_RECOMMENDATIONS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS book_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_key TEXT NOT NULL,
    recommended_key TEXT NOT NULL,
    title TEXT,
    authors TEXT,
    subjects TEXT,
    score REAL NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_key, recommended_key)
);
CREATE INDEX IF NOT EXISTS idx_book_recs_source
    ON book_recommendations(source_key);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookRecommendationRow {
    pub id: i64,
    pub source_key: String,
    pub recommended_key: String,
    pub title: Option<String>,
    pub authors: Option<String>,
    pub subjects: Option<String>,
    pub score: f64,
    pub level: i64,
    pub data: serde_json::Value,
    pub fetched_at: String,
}

#[derive(Clone)]
pub struct BookRecommendationsRepo {
    db: DbPool,
}

impl BookRecommendationsRepo {
    pub fn new(db: DbPool) -> Self {
        {
            let conn = db.lock();
            conn.execute_batch(BOOK_RECOMMENDATIONS_SCHEMA_SQL).unwrap();
        }
        Self { db }
    }

    pub fn upsert(
        &self,
        source_key: &str,
        rec_key: &str,
        title: Option<&str>,
        authors: Option<&str>,
        subjects: Option<&str>,
        score: f64,
        level: i64,
        data: &str,
    ) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO book_recommendations (source_key, recommended_key, title, authors, subjects, score, level, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(source_key, recommended_key)
             DO UPDATE SET title = ?3, authors = ?4, subjects = ?5, score = MAX(score, ?6), level = MIN(level, ?7), data = ?8, fetched_at = datetime('now')",
            params![source_key, rec_key, title, authors, subjects, score, level, data],
        );
    }

    pub fn get_for_source(&self, source_key: &str) -> Vec<BookRecommendationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, source_key, recommended_key, title, authors, subjects, score, level, data, fetched_at
                 FROM book_recommendations
                 WHERE source_key = ?1
                 ORDER BY score DESC",
            )
            .unwrap();
        stmt.query_map(params![source_key], |row| {
            let data_str: String = row.get(8)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok(BookRecommendationRow {
                id: row.get(0)?,
                source_key: row.get(1)?,
                recommended_key: row.get(2)?,
                title: row.get(3)?,
                authors: row.get(4)?,
                subjects: row.get(5)?,
                score: row.get(6)?,
                level: row.get(7)?,
                data,
                fetched_at: row.get(9)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn delete_for_source(&self, source_key: &str) -> usize {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM book_recommendations WHERE source_key = ?1",
            params![source_key],
        )
        .unwrap_or(0)
    }

    pub fn top_recommended_with_level_counts(
        &self,
        limit: usize,
    ) -> Vec<(String, Option<String>, i64, HashMap<i64, i64>)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_key, title, level, COUNT(*) as cnt
                 FROM book_recommendations
                 GROUP BY recommended_key, level",
            )
            .unwrap();
        let rows: Vec<(String, Option<String>, i64, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let mut map: HashMap<String, (Option<String>, i64, HashMap<i64, i64>)> = HashMap::new();
        for (key, title, level, cnt) in rows {
            let entry = map
                .entry(key)
                .or_insert_with(|| (title.clone(), 0, HashMap::new()));
            entry.1 += cnt;
            *entry.2.entry(level).or_insert(0) += cnt;
            if entry.0.is_none() && title.is_some() {
                entry.0 = title;
            }
        }

        let mut result: Vec<_> = map
            .into_iter()
            .map(|(key, (title, total, levels))| (key, title, total, levels))
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result.truncate(limit);
        result
    }

    pub fn top_recommended_with_data(
        &self,
        limit: usize,
    ) -> Vec<(String, Option<String>, i64, i64, serde_json::Value)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_key, title, COUNT(*) as cnt, MIN(level) as min_level, data
                 FROM book_recommendations
                 GROUP BY recommended_key
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

    pub fn sources_for_recommended(
        &self,
        recommended_keys: &[String],
    ) -> Vec<(String, String, Option<String>)> {
        if recommended_keys.is_empty() {
            return vec![];
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = recommended_keys.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT r.recommended_key, r.source_key,
                    (SELECT r2.title FROM book_recommendations r2
                     WHERE r2.recommended_key = r.source_key LIMIT 1) as source_title
             FROM book_recommendations r
             WHERE r.recommended_key IN ({})
             GROUP BY r.recommended_key, r.source_key",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&sql).unwrap();
        let params: Vec<&dyn rusqlite::ToSql> = recommended_keys
            .iter()
            .map(|k| k as &dyn rusqlite::ToSql)
            .collect();
        stmt.query_map(params.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn has_source(&self, key: &str) -> bool {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT 1 FROM book_recommendations WHERE source_key = ?1 LIMIT 1")
            .unwrap();
        stmt.exists(params![key]).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> BookRecommendationsRepo {
        let conn = Connection::open_in_memory().unwrap();
        let db = Arc::new(Mutex::new(conn));
        BookRecommendationsRepo::new(db)
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = setup();
        repo.upsert(
            "OL1W", "OL2W",
            Some("Dune"), Some("Frank Herbert"), Some("Science Fiction, Space Opera"),
            3.0, 1, r#"{"title":"Dune"}"#,
        );
        repo.upsert(
            "OL1W", "OL3W",
            Some("Foundation"), Some("Isaac Asimov"), Some("Science Fiction"),
            2.0, 1, r#"{"title":"Foundation"}"#,
        );

        let recs = repo.get_for_source("OL1W");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].title.as_deref(), Some("Dune"));
        assert_eq!(recs[0].score, 3.0);
        assert_eq!(recs[0].level, 1);
    }

    #[test]
    fn test_upsert_keeps_max_score() {
        let repo = setup();
        repo.upsert("OL1W", "OL2W", Some("Book"), None, None, 2.0, 1, r#"{}"#);
        repo.upsert("OL1W", "OL2W", Some("Book"), None, None, 5.0, 1, r#"{}"#);
        let recs = repo.get_for_source("OL1W");
        assert_eq!(recs[0].score, 5.0);
    }

    #[test]
    fn test_upsert_keeps_min_level() {
        let repo = setup();
        repo.upsert("OL1W", "OL2W", Some("Book"), None, None, 1.0, 3, r#"{}"#);
        let recs = repo.get_for_source("OL1W");
        assert_eq!(recs[0].level, 3);

        repo.upsert("OL1W", "OL2W", Some("Book"), None, None, 1.0, 1, r#"{}"#);
        let recs = repo.get_for_source("OL1W");
        assert_eq!(recs[0].level, 1);
    }

    #[test]
    fn test_delete_for_source() {
        let repo = setup();
        repo.upsert("OL1W", "OL2W", Some("A"), None, None, 1.0, 1, r#"{}"#);
        repo.upsert("OL1W", "OL3W", Some("B"), None, None, 1.0, 1, r#"{}"#);
        repo.upsert("OL4W", "OL5W", Some("C"), None, None, 1.0, 1, r#"{}"#);

        let deleted = repo.delete_for_source("OL1W");
        assert_eq!(deleted, 2);
        assert_eq!(repo.get_for_source("OL1W").len(), 0);
        assert_eq!(repo.get_for_source("OL4W").len(), 1);
    }

    #[test]
    fn test_has_source() {
        let repo = setup();
        assert!(!repo.has_source("OL1W"));
        repo.upsert("OL1W", "OL2W", None, None, None, 1.0, 1, r#"{}"#);
        assert!(repo.has_source("OL1W"));
        assert!(!repo.has_source("OL2W"));
    }

    #[test]
    fn test_top_recommended() {
        let repo = setup();
        repo.upsert("src1", "rec1", Some("Book A"), None, None, 3.0, 1, r#"{}"#);
        repo.upsert("src2", "rec1", Some("Book A"), None, None, 2.0, 2, r#"{}"#);
        repo.upsert("src1", "rec2", Some("Book B"), None, None, 1.0, 1, r#"{}"#);

        let top = repo.top_recommended_with_level_counts(10);
        assert_eq!(top[0].0, "rec1"); // rec1 recommended by 2 sources
        assert_eq!(top[0].2, 2);
        assert_eq!(top[1].0, "rec2");
        assert_eq!(top[1].2, 1);
    }

    #[test]
    fn test_sources_for_recommended() {
        let repo = setup();
        repo.upsert("src1", "rec1", Some("Book A"), None, None, 3.0, 1, r#"{}"#);
        repo.upsert("src2", "rec1", Some("Book A"), None, None, 2.0, 1, r#"{}"#);

        let sources = repo.sources_for_recommended(&["rec1".to_string()]);
        assert_eq!(sources.len(), 2);
    }
}

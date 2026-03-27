use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

pub type DbPool = Arc<Mutex<Connection>>;

pub const MUSIC_TASK_PREFIX: &str = "music-recommendations:";
pub const MUSIC_TASK_FETCH: &str = "music-recommendations:fetch";

pub const MUSIC_RECOMMENDATIONS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS music_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_mbid TEXT NOT NULL,
    source_type TEXT NOT NULL CHECK (source_type IN ('artist')),
    recommended_mbid TEXT NOT NULL,
    recommended_type TEXT NOT NULL CHECK (recommended_type IN ('artist')),
    name TEXT,
    tags TEXT,
    score REAL,
    level INTEGER NOT NULL DEFAULT 1,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_mbid, source_type, recommended_mbid)
);
CREATE INDEX IF NOT EXISTS idx_music_recs_source
    ON music_recommendations(source_mbid, source_type);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicRecommendationRow {
    pub id: i64,
    pub source_mbid: String,
    pub source_type: String,
    pub recommended_mbid: String,
    pub recommended_type: String,
    pub name: Option<String>,
    pub tags: Option<String>,
    pub score: Option<f64>,
    pub level: i64,
    pub data: serde_json::Value,
    pub fetched_at: String,
}

#[derive(Clone)]
pub struct MusicRecommendationsRepo {
    db: DbPool,
}

impl MusicRecommendationsRepo {
    pub fn new(db: DbPool) -> Self {
        {
            let conn = db.lock();
            conn.execute_batch(MUSIC_RECOMMENDATIONS_SCHEMA_SQL).unwrap();
        }
        Self { db }
    }

    pub fn upsert(
        &self,
        source_mbid: &str,
        source_type: &str,
        rec_mbid: &str,
        rec_type: &str,
        name: Option<&str>,
        tags: Option<&str>,
        score: Option<f64>,
        level: i64,
        data: &str,
    ) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO music_recommendations (source_mbid, source_type, recommended_mbid, recommended_type, name, tags, score, level, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(source_mbid, source_type, recommended_mbid)
             DO UPDATE SET name = ?5, tags = ?6, score = ?7, level = MIN(level, ?8), data = ?9, fetched_at = datetime('now')",
            params![source_mbid, source_type, rec_mbid, rec_type, name, tags, score, level, data],
        );
    }

    pub fn get_for_source(&self, source_mbid: &str, source_type: &str) -> Vec<MusicRecommendationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, source_mbid, source_type, recommended_mbid, recommended_type, name, tags, score, level, data, fetched_at
                 FROM music_recommendations
                 WHERE source_mbid = ?1 AND source_type = ?2
                 ORDER BY score DESC",
            )
            .unwrap();
        stmt.query_map(params![source_mbid, source_type], |row| {
            let data_str: String = row.get(9)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok(MusicRecommendationRow {
                id: row.get(0)?,
                source_mbid: row.get(1)?,
                source_type: row.get(2)?,
                recommended_mbid: row.get(3)?,
                recommended_type: row.get(4)?,
                name: row.get(5)?,
                tags: row.get(6)?,
                score: row.get(7)?,
                level: row.get(8)?,
                data,
                fetched_at: row.get(10)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn delete_for_source(&self, source_mbid: &str, source_type: &str) -> usize {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM music_recommendations WHERE source_mbid = ?1 AND source_type = ?2",
            params![source_mbid, source_type],
        )
        .unwrap_or(0)
    }

    pub fn top_recommended_with_level_counts(
        &self,
        limit: usize,
    ) -> Vec<(String, String, Option<String>, i64, HashMap<i64, i64>)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_mbid, recommended_type, name, level, COUNT(*) as cnt
                 FROM music_recommendations
                 GROUP BY recommended_mbid, recommended_type, level",
            )
            .unwrap();
        let rows: Vec<(String, String, Option<String>, i64, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let mut map: HashMap<(String, String), (Option<String>, i64, HashMap<i64, i64>)> =
            HashMap::new();
        for (mbid, rtype, name, level, cnt) in rows {
            let entry = map
                .entry((mbid, rtype))
                .or_insert_with(|| (name.clone(), 0, HashMap::new()));
            entry.1 += cnt;
            *entry.2.entry(level).or_insert(0) += cnt;
            if entry.0.is_none() && name.is_some() {
                entry.0 = name;
            }
        }

        let mut result: Vec<_> = map
            .into_iter()
            .map(|((mbid, rtype), (name, total, levels))| (mbid, rtype, name, total, levels))
            .collect();
        result.sort_by(|a, b| b.3.cmp(&a.3));
        result.truncate(limit);
        result
    }

    pub fn top_recommended_with_data(
        &self,
        limit: usize,
    ) -> Vec<(String, String, Option<String>, i64, i64, serde_json::Value)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_mbid, recommended_type, name, COUNT(*) as cnt, MIN(level) as min_level, data
                 FROM music_recommendations
                 GROUP BY recommended_mbid, recommended_type
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

    pub fn sources_for_recommended(
        &self,
        recommended_mbids: &[String],
    ) -> Vec<(String, String, String, Option<String>)> {
        if recommended_mbids.is_empty() {
            return vec![];
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = recommended_mbids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT r.recommended_mbid, r.source_mbid, r.source_type,
                    (SELECT r2.name FROM music_recommendations r2
                     WHERE r2.recommended_mbid = r.source_mbid LIMIT 1) as source_name
             FROM music_recommendations r
             WHERE r.recommended_mbid IN ({})
             GROUP BY r.recommended_mbid, r.source_mbid",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&sql).unwrap();
        let params: Vec<&dyn rusqlite::ToSql> = recommended_mbids
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

    pub fn has_source(&self, mbid: &str, source_type: &str) -> bool {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT 1 FROM music_recommendations WHERE source_mbid = ?1 AND source_type = ?2 LIMIT 1",
            )
            .unwrap();
        stmt.exists(params![mbid, source_type]).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> MusicRecommendationsRepo {
        let conn = Connection::open_in_memory().unwrap();
        let db = Arc::new(Mutex::new(conn));
        MusicRecommendationsRepo::new(db)
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = setup();
        repo.upsert(
            "aaaa-1111", "artist", "bbbb-2222", "artist",
            Some("Radiohead"), Some("rock, alternative"), Some(5000.0), 1, r#"{"name":"Radiohead"}"#,
        );
        repo.upsert(
            "aaaa-1111", "artist", "cccc-3333", "artist",
            Some("Portishead"), Some("trip hop"), Some(3000.0), 1, r#"{"name":"Portishead"}"#,
        );

        let recs = repo.get_for_source("aaaa-1111", "artist");
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].name.as_deref(), Some("Radiohead"));
        assert_eq!(recs[0].level, 1);
    }

    #[test]
    fn test_upsert_keeps_min_level() {
        let repo = setup();
        repo.upsert(
            "aaaa-1111", "artist", "bbbb-2222", "artist",
            Some("Artist"), None, Some(100.0), 3, r#"{}"#,
        );
        let recs = repo.get_for_source("aaaa-1111", "artist");
        assert_eq!(recs[0].level, 3);

        repo.upsert(
            "aaaa-1111", "artist", "bbbb-2222", "artist",
            Some("Artist"), None, Some(100.0), 1, r#"{}"#,
        );
        let recs = repo.get_for_source("aaaa-1111", "artist");
        assert_eq!(recs[0].level, 1);
    }

    #[test]
    fn test_delete_for_source() {
        let repo = setup();
        repo.upsert("aaa", "artist", "bbb", "artist", Some("A"), None, None, 1, r#"{}"#);
        repo.upsert("aaa", "artist", "ccc", "artist", Some("B"), None, None, 1, r#"{}"#);
        repo.upsert("ddd", "artist", "eee", "artist", Some("C"), None, None, 1, r#"{}"#);

        let deleted = repo.delete_for_source("aaa", "artist");
        assert_eq!(deleted, 2);
        assert_eq!(repo.get_for_source("aaa", "artist").len(), 0);
        assert_eq!(repo.get_for_source("ddd", "artist").len(), 1);
    }

    #[test]
    fn test_has_source() {
        let repo = setup();
        assert!(!repo.has_source("aaa", "artist"));
        repo.upsert("aaa", "artist", "bbb", "artist", None, None, None, 1, r#"{}"#);
        assert!(repo.has_source("aaa", "artist"));
        assert!(!repo.has_source("bbb", "artist"));
    }

    #[test]
    fn test_top_recommended() {
        let repo = setup();
        repo.upsert("src1", "artist", "rec1", "artist", Some("Artist A"), None, Some(100.0), 1, r#"{}"#);
        repo.upsert("src2", "artist", "rec1", "artist", Some("Artist A"), None, Some(200.0), 2, r#"{}"#);
        repo.upsert("src1", "artist", "rec2", "artist", Some("Artist B"), None, Some(50.0), 1, r#"{}"#);

        let top = repo.top_recommended_with_level_counts(10);
        assert_eq!(top[0].0, "rec1"); // rec1 has 2 recommendations
        assert_eq!(top[0].3, 2);
        assert_eq!(top[1].0, "rec2");
        assert_eq!(top[1].3, 1);
    }
}

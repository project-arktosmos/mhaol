use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

pub type DbPool = Arc<Mutex<Connection>>;

pub const GAME_TASK_PREFIX: &str = "game-recommendations:";
pub const GAME_TASK_FETCH: &str = "game-recommendations:fetch";

pub const GAME_RECOMMENDATIONS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS game_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_game_id INTEGER NOT NULL,
    recommended_game_id INTEGER NOT NULL,
    title TEXT,
    genre TEXT,
    console_id INTEGER,
    console_name TEXT,
    score REAL NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_game_id, recommended_game_id)
);
CREATE INDEX IF NOT EXISTS idx_game_recs_source
    ON game_recommendations(source_game_id);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecommendationRow {
    pub id: i64,
    pub source_game_id: i64,
    pub recommended_game_id: i64,
    pub title: Option<String>,
    pub genre: Option<String>,
    pub console_id: Option<i64>,
    pub console_name: Option<String>,
    pub score: f64,
    pub level: i64,
    pub data: serde_json::Value,
    pub fetched_at: String,
}

#[derive(Clone)]
pub struct GameRecommendationsRepo {
    db: DbPool,
}

impl GameRecommendationsRepo {
    pub fn new(db: DbPool) -> Self {
        {
            let conn = db.lock();
            conn.execute_batch(GAME_RECOMMENDATIONS_SCHEMA_SQL).unwrap();
        }
        Self { db }
    }

    pub fn upsert(
        &self,
        source_game_id: i64,
        rec_game_id: i64,
        title: Option<&str>,
        genre: Option<&str>,
        console_id: Option<i64>,
        console_name: Option<&str>,
        score: f64,
        level: i64,
        data: &str,
    ) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO game_recommendations (source_game_id, recommended_game_id, title, genre, console_id, console_name, score, level, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(source_game_id, recommended_game_id)
             DO UPDATE SET title = ?3, genre = ?4, console_id = ?5, console_name = ?6, score = MAX(score, ?7), level = MIN(level, ?8), data = ?9, fetched_at = datetime('now')",
            params![source_game_id, rec_game_id, title, genre, console_id, console_name, score, level, data],
        );
    }

    pub fn get_for_source(&self, source_game_id: i64) -> Vec<GameRecommendationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, source_game_id, recommended_game_id, title, genre, console_id, console_name, score, level, data, fetched_at
                 FROM game_recommendations
                 WHERE source_game_id = ?1
                 ORDER BY score DESC",
            )
            .unwrap();
        stmt.query_map(params![source_game_id], |row| {
            let data_str: String = row.get(9)?;
            let data = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            Ok(GameRecommendationRow {
                id: row.get(0)?,
                source_game_id: row.get(1)?,
                recommended_game_id: row.get(2)?,
                title: row.get(3)?,
                genre: row.get(4)?,
                console_id: row.get(5)?,
                console_name: row.get(6)?,
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

    pub fn delete_for_source(&self, source_game_id: i64) -> usize {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM game_recommendations WHERE source_game_id = ?1",
            params![source_game_id],
        )
        .unwrap_or(0)
    }

    pub fn top_recommended_with_level_counts(
        &self,
        limit: usize,
    ) -> Vec<(i64, Option<String>, i64, HashMap<i64, i64>)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_game_id, title, level, COUNT(*) as cnt
                 FROM game_recommendations
                 GROUP BY recommended_game_id, level",
            )
            .unwrap();
        let rows: Vec<(i64, Option<String>, i64, i64)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let mut map: HashMap<i64, (Option<String>, i64, HashMap<i64, i64>)> = HashMap::new();
        for (game_id, title, level, cnt) in rows {
            let entry = map
                .entry(game_id)
                .or_insert_with(|| (title.clone(), 0, HashMap::new()));
            entry.1 += cnt;
            *entry.2.entry(level).or_insert(0) += cnt;
            if entry.0.is_none() && title.is_some() {
                entry.0 = title;
            }
        }

        let mut result: Vec<_> = map
            .into_iter()
            .map(|(game_id, (title, total, levels))| (game_id, title, total, levels))
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result.truncate(limit);
        result
    }

    pub fn top_recommended_with_data(
        &self,
        limit: usize,
    ) -> Vec<(i64, Option<String>, i64, i64, serde_json::Value)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT recommended_game_id, title, COUNT(*) as cnt, MIN(level) as min_level, data
                 FROM game_recommendations
                 GROUP BY recommended_game_id
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

    pub fn sources_for_recommended(&self, recommended_ids: &[i64]) -> Vec<(i64, i64, Option<String>)> {
        if recommended_ids.is_empty() {
            return vec![];
        }
        let conn = self.db.lock();
        let placeholders: Vec<String> = recommended_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT r.recommended_game_id, r.source_game_id,
                    (SELECT r2.title FROM game_recommendations r2
                     WHERE r2.recommended_game_id = r.source_game_id LIMIT 1) as source_title
             FROM game_recommendations r
             WHERE r.recommended_game_id IN ({})
             GROUP BY r.recommended_game_id, r.source_game_id",
            placeholders.join(",")
        );
        let mut stmt = conn.prepare(&sql).unwrap();
        let params: Vec<&dyn rusqlite::ToSql> = recommended_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        stmt.query_map(params.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn has_source(&self, game_id: i64) -> bool {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT 1 FROM game_recommendations WHERE source_game_id = ?1 LIMIT 1")
            .unwrap();
        stmt.exists(params![game_id]).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> GameRecommendationsRepo {
        let conn = Connection::open_in_memory().unwrap();
        let db = Arc::new(Mutex::new(conn));
        GameRecommendationsRepo::new(db)
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = setup();
        repo.upsert(100, 200, Some("Zelda"), Some("Action"), Some(3), Some("SNES"), 80.0, 1, r#"{}"#);
        repo.upsert(100, 300, Some("Mario"), Some("Platformer"), Some(3), Some("SNES"), 60.0, 1, r#"{}"#);
        let recs = repo.get_for_source(100);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].title.as_deref(), Some("Zelda"));
        assert_eq!(recs[0].score, 80.0);
    }

    #[test]
    fn test_upsert_keeps_max_score() {
        let repo = setup();
        repo.upsert(100, 200, Some("Game"), None, None, None, 50.0, 1, r#"{}"#);
        repo.upsert(100, 200, Some("Game"), None, None, None, 80.0, 1, r#"{}"#);
        let recs = repo.get_for_source(100);
        assert_eq!(recs[0].score, 80.0);
    }

    #[test]
    fn test_delete_for_source() {
        let repo = setup();
        repo.upsert(100, 200, Some("A"), None, None, None, 50.0, 1, r#"{}"#);
        repo.upsert(100, 300, Some("B"), None, None, None, 50.0, 1, r#"{}"#);
        repo.upsert(400, 500, Some("C"), None, None, None, 50.0, 1, r#"{}"#);
        assert_eq!(repo.delete_for_source(100), 2);
        assert_eq!(repo.get_for_source(100).len(), 0);
        assert_eq!(repo.get_for_source(400).len(), 1);
    }

    #[test]
    fn test_has_source() {
        let repo = setup();
        assert!(!repo.has_source(100));
        repo.upsert(100, 200, None, None, None, None, 50.0, 1, r#"{}"#);
        assert!(repo.has_source(100));
        assert!(!repo.has_source(200));
    }

    #[test]
    fn test_top_recommended() {
        let repo = setup();
        repo.upsert(1, 10, Some("Game A"), None, None, None, 80.0, 1, r#"{}"#);
        repo.upsert(2, 10, Some("Game A"), None, None, None, 70.0, 1, r#"{}"#);
        repo.upsert(1, 20, Some("Game B"), None, None, None, 60.0, 1, r#"{}"#);
        let top = repo.top_recommended_with_level_counts(10);
        assert_eq!(top[0].0, 10); // Game A recommended by 2 sources
        assert_eq!(top[0].2, 2);
        assert_eq!(top[1].0, 20);
        assert_eq!(top[1].2, 1);
    }
}

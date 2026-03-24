use crate::db::DbPool;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TmdbImageOverride {
    pub tmdb_id: i64,
    pub media_type: String,
    pub role: String,
    pub file_path: String,
}

#[derive(Clone)]
pub struct TmdbImageOverrideRepo {
    db: DbPool,
}

impl TmdbImageOverrideRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_for_item(&self, tmdb_id: i64, media_type: &str) -> Vec<TmdbImageOverride> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT tmdb_id, media_type, role, file_path FROM tmdb_image_overrides
                 WHERE tmdb_id = ?1 AND media_type = ?2",
            )
            .unwrap();
        stmt.query_map(params![tmdb_id, media_type], |row| {
            Ok(TmdbImageOverride {
                tmdb_id: row.get(0)?,
                media_type: row.get(1)?,
                role: row.get(2)?,
                file_path: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_all_for_media_type(&self, media_type: &str) -> Vec<TmdbImageOverride> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT tmdb_id, media_type, role, file_path FROM tmdb_image_overrides
                 WHERE media_type = ?1",
            )
            .unwrap();
        stmt.query_map(params![media_type], |row| {
            Ok(TmdbImageOverride {
                tmdb_id: row.get(0)?,
                media_type: row.get(1)?,
                role: row.get(2)?,
                file_path: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn upsert(&self, tmdb_id: i64, media_type: &str, role: &str, file_path: &str) {
        let conn = self.db.lock();
        let _ = conn.execute(
            "INSERT INTO tmdb_image_overrides (tmdb_id, media_type, role, file_path)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(tmdb_id, media_type, role) DO UPDATE SET file_path = ?4, created_at = datetime('now')",
            params![tmdb_id, media_type, role, file_path],
        );
    }

    pub fn delete(&self, tmdb_id: i64, media_type: &str, role: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM tmdb_image_overrides WHERE tmdb_id = ?1 AND media_type = ?2 AND role = ?3",
            params![tmdb_id, media_type, role],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }
}

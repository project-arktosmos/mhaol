use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTypeRow {
    pub id: String,
    pub label: String,
}

#[derive(Clone)]
pub struct MediaTypeRepo {
    db: DbPool,
}

impl MediaTypeRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<MediaTypeRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, label FROM media_types WHERE id = ?1",
            params![id],
            |row| {
                Ok(MediaTypeRow {
                    id: row.get(0)?,
                    label: row.get(1)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<MediaTypeRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, label FROM media_types ORDER BY id ASC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(MediaTypeRow {
                id: row.get(0)?,
                label: row.get(1)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_media_types_seeded() {
        let db = open_test_database();
        let repo = MediaTypeRepo::new(db);

        let all = repo.get_all();
        assert_eq!(all.len(), 3);

        let video = repo.get("video").unwrap();
        assert_eq!(video.label, "Video");
    }
}

use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRow {
    pub id: String,
    pub media_type_id: String,
    pub label: String,
}

#[derive(Clone)]
pub struct CategoryRepo {
    db: DbPool,
}

impl CategoryRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<CategoryRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, media_type_id, label FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(CategoryRow {
                    id: row.get(0)?,
                    media_type_id: row.get(1)?,
                    label: row.get(2)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<CategoryRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, media_type_id, label FROM categories ORDER BY media_type_id ASC, label ASC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                media_type_id: row.get(1)?,
                label: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_by_media_type(&self, media_type_id: &str) -> Vec<CategoryRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, media_type_id, label FROM categories WHERE media_type_id = ?1 ORDER BY label ASC")
            .unwrap();
        stmt.query_map(params![media_type_id], |row| {
            Ok(CategoryRow {
                id: row.get(0)?,
                media_type_id: row.get(1)?,
                label: row.get(2)?,
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
    fn test_categories_seeded() {
        let db = open_test_database();
        let repo = CategoryRepo::new(db);

        let all = repo.get_all();
        assert_eq!(all.len(), 10);

        let video_cats = repo.get_by_media_type("video");
        assert_eq!(video_cats.len(), 4);
    }
}

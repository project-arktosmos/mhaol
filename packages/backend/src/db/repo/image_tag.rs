use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTagRow {
    pub id: String,
    pub library_item_id: String,
    pub tag: String,
    pub score: f64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct ImageTagRepo {
    db: DbPool,
}

impl ImageTagRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_item(&self, library_item_id: &str) -> Vec<ImageTagRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, tag, score, created_at FROM image_tags WHERE library_item_id = ?1 ORDER BY score DESC")
            .unwrap();
        stmt.query_map(params![library_item_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_items(&self, library_item_ids: &[&str]) -> Vec<ImageTagRow> {
        let mut result = Vec::new();
        for id in library_item_ids {
            result.extend(self.get_by_item(id));
        }
        result
    }

    pub fn replace_for_item(&self, library_item_id: &str, tags: &[(&str, f64)]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        tx.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1",
            params![library_item_id],
        )
        .unwrap();
        for (tag, score) in tags {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO image_tags (id, library_item_id, tag, score) VALUES (?1, ?2, ?3, ?4)",
                params![id, library_item_id, tag, score],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    pub fn add_tag(&self, library_item_id: &str, tag: &str, score: f64) {
        let conn = self.db.lock();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO image_tags (id, library_item_id, tag, score) VALUES (?1, ?2, ?3, ?4)",
            params![id, library_item_id, tag, score],
        )
        .unwrap();
    }

    pub fn delete_tag(&self, library_item_id: &str, tag: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1 AND tag = ?2",
            params![library_item_id, tag],
        )
        .unwrap();
    }

    pub fn delete_by_item(&self, library_item_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1",
            params![library_item_id],
        )
        .unwrap();
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<ImageTagRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, tag, score, created_at FROM image_tags WHERE tag = ?1 ORDER BY score DESC")
            .unwrap();
        stmt.query_map(params![tag], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImageTagRow> {
        Ok(ImageTagRow {
            id: row.get(0)?,
            library_item_id: row.get(1)?,
            tag: row.get(2)?,
            score: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

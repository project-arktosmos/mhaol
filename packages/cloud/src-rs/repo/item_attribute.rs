use crate::types::{CloudItemAttributeRow, DbPool};
use rusqlite::params;

#[derive(Clone)]
pub struct CloudItemAttributeRepo {
    db: DbPool,
}

impl CloudItemAttributeRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_item(&self, item_id: &str) -> Vec<CloudItemAttributeRow> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, key, value, attribute_type_id, source, confidence, created_at, updated_at FROM cloud_item_attributes WHERE item_id = ?1 ORDER BY key ASC"
        ).unwrap();
        stmt.query_map(params![item_id], |row| Ok(CloudItemAttributeRow {
            id: row.get(0)?, item_id: row.get(1)?, key: row.get(2)?,
            value: row.get(3)?, attribute_type_id: row.get(4)?, source: row.get(5)?,
            confidence: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn get_by_key_and_value(&self, key: &str, value: &str) -> Vec<CloudItemAttributeRow> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, key, value, attribute_type_id, source, confidence, created_at, updated_at FROM cloud_item_attributes WHERE key = ?1 AND value = ?2 ORDER BY item_id ASC"
        ).unwrap();
        stmt.query_map(params![key, value], |row| Ok(CloudItemAttributeRow {
            id: row.get(0)?, item_id: row.get(1)?, key: row.get(2)?,
            value: row.get(3)?, attribute_type_id: row.get(4)?, source: row.get(5)?,
            confidence: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set(&self, id: &str, item_id: &str, key: &str, value: &str, attribute_type_id: &str, source: &str, confidence: Option<f64>) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO cloud_item_attributes (id, item_id, key, value, attribute_type_id, source, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(item_id, key, source) DO UPDATE SET value = ?4, attribute_type_id = ?5, confidence = ?7",
            params![id, item_id, key, value, attribute_type_id, source, confidence],
        ).unwrap();
    }

    pub fn delete_by_item_and_key(&self, item_id: &str, key: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM cloud_item_attributes WHERE item_id = ?1 AND key = ?2",
            params![item_id, key],
        ).unwrap();
    }

    pub fn distinct_keys(&self) -> Vec<String> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT key FROM cloud_item_attributes ORDER BY key ASC"
        ).unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn distinct_values(&self, key: &str) -> Vec<String> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT value FROM cloud_item_attributes WHERE key = ?1 ORDER BY value ASC"
        ).unwrap();
        stmt.query_map(params![key], |row| row.get::<_, String>(0))
            .unwrap().filter_map(|r| r.ok()).collect()
    }
}

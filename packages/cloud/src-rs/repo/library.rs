use crate::types::{CloudLibraryRow, DbPool};
use rusqlite::params;

#[derive(Clone)]
pub struct CloudLibraryRepo {
    db: DbPool,
}

impl CloudLibraryRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<CloudLibraryRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, path, kind, scan_status, scan_error, item_count, created_at, updated_at FROM cloud_libraries WHERE id = ?1",
            params![id],
            |row| Ok(CloudLibraryRow {
                id: row.get(0)?, name: row.get(1)?, path: row.get(2)?,
                kind: row.get(3)?, scan_status: row.get(4)?, scan_error: row.get(5)?,
                item_count: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
            }),
        ).ok()
    }

    pub fn get_all(&self) -> Vec<CloudLibraryRow> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, kind, scan_status, scan_error, item_count, created_at, updated_at FROM cloud_libraries ORDER BY created_at DESC"
        ).unwrap();
        stmt.query_map([], |row| Ok(CloudLibraryRow {
            id: row.get(0)?, name: row.get(1)?, path: row.get(2)?,
            kind: row.get(3)?, scan_status: row.get(4)?, scan_error: row.get(5)?,
            item_count: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn insert(&self, id: &str, name: &str, path: &str, kind: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO cloud_libraries (id, name, path, kind) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, path, kind],
        ).unwrap();
    }

    pub fn update(&self, id: &str, name: &str, path: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE cloud_libraries SET name = ?2, path = ?3 WHERE id = ?1",
            params![id, name, path],
        ).unwrap();
    }

    pub fn update_scan_status(&self, id: &str, status: &str, error: Option<&str>) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE cloud_libraries SET scan_status = ?2, scan_error = ?3 WHERE id = ?1",
            params![id, status, error],
        ).unwrap();
    }

    pub fn update_item_count(&self, id: &str, count: i64) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE cloud_libraries SET item_count = ?2 WHERE id = ?1",
            params![id, count],
        ).unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM cloud_libraries WHERE id = ?1", params![id]).unwrap();
    }
}

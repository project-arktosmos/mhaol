use crate::types::{CloudItemRow, DbPool, InsertCloudItem};
use rusqlite::params;
use std::collections::HashSet;

#[derive(Clone)]
pub struct CloudItemRepo {
    db: DbPool,
}

impl CloudItemRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<CloudItemRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, library_id, path, filename, extension, size_bytes, mime_type, checksum, created_at, updated_at FROM cloud_items WHERE id = ?1",
            params![id],
            |row| Ok(CloudItemRow {
                id: row.get(0)?, library_id: row.get(1)?, path: row.get(2)?,
                filename: row.get(3)?, extension: row.get(4)?, size_bytes: row.get(5)?,
                mime_type: row.get(6)?, checksum: row.get(7)?,
                created_at: row.get(8)?, updated_at: row.get(9)?,
            }),
        ).ok()
    }

    pub fn get_by_library(&self, library_id: &str) -> Vec<CloudItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT id, library_id, path, filename, extension, size_bytes, mime_type, checksum, created_at, updated_at FROM cloud_items WHERE library_id = ?1 ORDER BY path ASC"
        ).unwrap();
        stmt.query_map(params![library_id], |row| Ok(CloudItemRow {
            id: row.get(0)?, library_id: row.get(1)?, path: row.get(2)?,
            filename: row.get(3)?, extension: row.get(4)?, size_bytes: row.get(5)?,
            mime_type: row.get(6)?, checksum: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn sync_library(&self, library_id: &str, new_files: &[InsertCloudItem]) {
        let conn = self.db.lock();
        let existing = {
            let mut stmt = conn.prepare(
                "SELECT id, path FROM cloud_items WHERE library_id = ?1"
            ).unwrap();
            stmt.query_map(params![library_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }).unwrap().filter_map(|r| r.ok()).collect::<Vec<_>>()
        };

        let scanned_paths: HashSet<&str> = new_files.iter().map(|f| f.path.as_str()).collect();
        let existing_paths: HashSet<&str> = existing.iter().map(|(_, p)| p.as_str()).collect();

        for (id, path) in &existing {
            if !scanned_paths.contains(path.as_str()) {
                conn.execute("DELETE FROM cloud_items WHERE id = ?1", params![id]).unwrap();
            }
        }

        for file in new_files {
            if !existing_paths.contains(file.path.as_str()) {
                conn.execute(
                    "INSERT INTO cloud_items (id, library_id, path, filename, extension, size_bytes, mime_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![file.id, file.library_id, file.path, file.filename, file.extension, file.size_bytes, file.mime_type],
                ).unwrap();
            }
        }
    }

    pub fn delete_by_library(&self, library_id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM cloud_items WHERE library_id = ?1", params![library_id]).unwrap();
    }

    pub fn search(&self, query: &str) -> Vec<CloudItemRow> {
        let conn = self.db.lock();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, library_id, path, filename, extension, size_bytes, mime_type, checksum, created_at, updated_at FROM cloud_items WHERE filename LIKE ?1 ORDER BY filename ASC"
        ).unwrap();
        stmt.query_map(params![pattern], |row| Ok(CloudItemRow {
            id: row.get(0)?, library_id: row.get(1)?, path: row.get(2)?,
            filename: row.get(3)?, extension: row.get(4)?, size_bytes: row.get(5)?,
            mime_type: row.get(6)?, checksum: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }
}

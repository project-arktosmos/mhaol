use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemRow {
    pub id: String,
    pub library_id: String,
    pub path: String,
    pub extension: String,
    pub media_type: String,
    pub category_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct InsertLibraryItem {
    pub id: String,
    pub library_id: String,
    pub path: String,
    pub extension: String,
    pub media_type: String,
    pub category_id: Option<String>,
}

#[derive(Clone)]
pub struct LibraryItemRepo {
    db: DbPool,
}

impl LibraryItemRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<LibraryItemRow> {
        let conn = self.db.lock();
        Self::get_with_conn(&conn, id)
    }

    fn get_with_conn(conn: &rusqlite::Connection, id: &str) -> Option<LibraryItemRow> {
        conn.query_row(
            "SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at FROM library_items WHERE id = ?1",
            params![id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_by_library(&self, library_id: &str) -> Vec<LibraryItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at FROM library_items WHERE library_id = ?1 ORDER BY path ASC")
            .unwrap();
        stmt.query_map(params![library_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_media_type(&self, media_type: &str) -> Vec<LibraryItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at FROM library_items WHERE media_type = ?1 ORDER BY path ASC")
            .unwrap();
        stmt.query_map(params![media_type], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_category(&self, category_id: &str) -> Vec<LibraryItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at FROM library_items WHERE category_id = ?1 ORDER BY path ASC")
            .unwrap();
        stmt.query_map(params![category_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_uncategorized_by_media_type(&self, media_type: &str) -> Vec<LibraryItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at FROM library_items WHERE media_type = ?1 AND category_id IS NULL ORDER BY path ASC")
            .unwrap();
        stmt.query_map(params![media_type], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn insert(&self, item: &InsertLibraryItem) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO library_items (id, library_id, path, extension, media_type, category_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![item.id, item.library_id, item.path, item.extension, item.media_type, item.category_id],
        )
        .unwrap();
    }

    pub fn insert_many(&self, items: &[InsertLibraryItem]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        for item in items {
            tx.execute(
                "INSERT INTO library_items (id, library_id, path, extension, media_type, category_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![item.id, item.library_id, item.path, item.extension, item.media_type, item.category_id],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    /// Sync a library's items: delete items whose paths are no longer present, insert new ones.
    pub fn sync_library(&self, library_id: &str, new_files: &[InsertLibraryItem]) {
        let conn = self.db.lock();

        // Get existing items before starting transaction
        let existing: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, path FROM library_items WHERE library_id = ?1")
                .unwrap();
            stmt.query_map(params![library_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
        };

        let new_paths: HashSet<&str> = new_files.iter().map(|f| f.path.as_str()).collect();
        let existing_paths: HashSet<&str> = existing.iter().map(|(_, p)| p.as_str()).collect();

        let tx = conn.unchecked_transaction().unwrap();

        // Delete items no longer present
        for (id, path) in &existing {
            if !new_paths.contains(path.as_str()) {
                tx.execute("DELETE FROM library_items WHERE id = ?1", params![id])
                    .unwrap();
            }
        }

        // Insert new items
        for item in new_files {
            if !existing_paths.contains(item.path.as_str()) {
                tx.execute(
                    "INSERT INTO library_items (id, library_id, path, extension, media_type, category_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![item.id, item.library_id, item.path, item.extension, item.media_type, item.category_id],
                )
                .unwrap();
            }
        }

        tx.commit().unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM library_items WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn delete_by_library(&self, library_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM library_items WHERE library_id = ?1",
            params![library_id],
        )
        .unwrap();
    }

    pub fn exists_by_path(&self, path: &str) -> Option<String> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id FROM library_items WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn update_category(&self, id: &str, category_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE library_items SET category_id = ?2 WHERE id = ?1",
            params![id, category_id],
        )
        .unwrap();
    }

    pub fn clear_category(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE library_items SET category_id = NULL WHERE id = ?1",
            params![id],
        )
        .unwrap();
    }

    pub fn update_media_type(&self, id: &str, media_type: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE library_items SET media_type = ?2 WHERE id = ?1",
            params![id, media_type],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryItemRow> {
        Ok(LibraryItemRow {
            id: row.get(0)?,
            library_id: row.get(1)?,
            path: row.get(2)?,
            extension: row.get(3)?,
            media_type: row.get(4)?,
            category_id: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::LibraryRepo;

    fn setup() -> (LibraryRepo, LibraryItemRepo) {
        let db = open_test_database();
        let lib_repo = LibraryRepo::new(db.clone());
        let item_repo = LibraryItemRepo::new(db);
        lib_repo.insert("lib1", "Test", "/tmp", "[\"video\"]", 1000);
        (lib_repo, item_repo)
    }

    #[test]
    fn test_library_item_crud() {
        let (_lib_repo, repo) = setup();

        repo.insert(&InsertLibraryItem {
            id: "item1".into(),
            library_id: "lib1".into(),
            path: "/tmp/video.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });

        let item = repo.get("item1").unwrap();
        assert_eq!(item.path, "/tmp/video.mp4");

        repo.update_category("item1", "movies");
        let item = repo.get("item1").unwrap();
        assert_eq!(item.category_id, Some("movies".to_string()));

        repo.clear_category("item1");
        let item = repo.get("item1").unwrap();
        assert!(item.category_id.is_none());
    }

    #[test]
    fn test_sync_library() {
        let (_lib_repo, repo) = setup();

        // Insert initial items
        repo.insert(&InsertLibraryItem {
            id: "a".into(),
            library_id: "lib1".into(),
            path: "/tmp/a.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });
        repo.insert(&InsertLibraryItem {
            id: "b".into(),
            library_id: "lib1".into(),
            path: "/tmp/b.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });

        // Sync: remove b, add c
        let new_files = vec![
            InsertLibraryItem {
                id: "a".into(),
                library_id: "lib1".into(),
                path: "/tmp/a.mp4".into(),
                extension: "mp4".into(),
                media_type: "video".into(),
                category_id: None,
            },
            InsertLibraryItem {
                id: "c".into(),
                library_id: "lib1".into(),
                path: "/tmp/c.mp4".into(),
                extension: "mp4".into(),
                media_type: "video".into(),
                category_id: None,
            },
        ];
        repo.sync_library("lib1", &new_files);

        let items = repo.get_by_library("lib1");
        assert_eq!(items.len(), 2);
        let paths: Vec<&str> = items.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"/tmp/a.mp4"));
        assert!(paths.contains(&"/tmp/c.mp4"));
    }
}

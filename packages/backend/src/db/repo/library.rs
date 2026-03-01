use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_types: String,
    pub date_added: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct LibraryRepo {
    db: DbPool,
}

impl LibraryRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<LibraryRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, path, media_types, date_added, created_at, updated_at FROM libraries WHERE id = ?1",
            params![id],
            |row| {
                Ok(LibraryRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    media_types: row.get(3)?,
                    date_added: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<LibraryRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, path, media_types, date_added, created_at, updated_at FROM libraries ORDER BY date_added DESC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(LibraryRow {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                media_types: row.get(3)?,
                date_added: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn insert(&self, id: &str, name: &str, path: &str, media_types: &str, date_added: i64) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO libraries (id, name, path, media_types, date_added) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, path, media_types, date_added],
        )
        .unwrap();
    }

    pub fn update(&self, id: &str, name: &str, path: &str, media_types: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE libraries SET name = ?2, path = ?3, media_types = ?4 WHERE id = ?1",
            params![id, name, path, media_types],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM libraries WHERE id = ?1", params![id])
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_library_crud() {
        let db = open_test_database();
        let repo = LibraryRepo::new(db);

        repo.insert("lib1", "Downloads", "/tmp/downloads", "[\"video\"]", 1000);
        let lib = repo.get("lib1").unwrap();
        assert_eq!(lib.name, "Downloads");
        assert_eq!(lib.path, "/tmp/downloads");

        repo.update("lib1", "Media", "/tmp/media", "[\"video\",\"audio\"]");
        let lib = repo.get("lib1").unwrap();
        assert_eq!(lib.name, "Media");

        assert_eq!(repo.get_all().len(), 1);

        repo.delete("lib1");
        assert!(repo.get("lib1").is_none());
    }
}

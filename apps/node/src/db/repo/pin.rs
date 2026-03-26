use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinRow {
    pub id: String,
    pub service: String,
    pub service_id: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct PinRepo {
    db: DbPool,
}

impl PinRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Vec<PinRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, service, service_id, label, created_at
                 FROM pins ORDER BY created_at DESC",
            )
            .unwrap();
        stmt.query_map(params![], |row| {
            Ok(PinRow {
                id: row.get(0)?,
                service: row.get(1)?,
                service_id: row.get(2)?,
                label: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn count(&self) -> i64 {
        let conn = self.db.lock();
        conn.query_row("SELECT COUNT(*) FROM pins", params![], |row| row.get(0))
            .unwrap_or(0)
    }

    pub fn insert(&self, service: &str, service_id: &str, label: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT OR IGNORE INTO pins (id, service, service_id, label)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3)",
            params![service, service_id, label],
        )
        .unwrap();
    }

    pub fn delete(&self, service: &str, service_id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM pins WHERE service = ?1 AND service_id = ?2",
            params![service, service_id],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_pin_crud() {
        let db = open_test_database();
        let repo = PinRepo::new(db);

        assert!(repo.get_all().is_empty());
        assert_eq!(repo.count(), 0);

        repo.insert("tmdb", "12345", "Inception");
        repo.insert("tmdb", "67890", "The Matrix");

        let pins = repo.get_all();
        assert_eq!(pins.len(), 2);
        assert_eq!(repo.count(), 2);

        // Duplicate insert is ignored
        repo.insert("tmdb", "12345", "Inception Updated");
        assert_eq!(repo.count(), 2);

        assert!(repo.delete("tmdb", "12345"));
        assert_eq!(repo.count(), 1);

        assert!(!repo.delete("tmdb", "99999"));
    }
}

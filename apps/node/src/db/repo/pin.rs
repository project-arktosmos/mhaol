use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinRow {
    pub id: String,
    pub wallet: String,
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

    pub fn get_by_wallet(&self, wallet: &str) -> Vec<PinRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, wallet, service, service_id, label, created_at
                 FROM pins WHERE wallet = ?1 ORDER BY created_at DESC",
            )
            .unwrap();
        stmt.query_map(params![wallet], |row| {
            Ok(PinRow {
                id: row.get(0)?,
                wallet: row.get(1)?,
                service: row.get(2)?,
                service_id: row.get(3)?,
                label: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn count_by_wallet(&self, wallet: &str) -> i64 {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT COUNT(*) FROM pins WHERE wallet = ?1",
            params![wallet],
            |row| row.get(0),
        )
        .unwrap_or(0)
    }

    pub fn insert(&self, wallet: &str, service: &str, service_id: &str, label: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT OR IGNORE INTO pins (id, wallet, service, service_id, label)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4)",
            params![wallet, service, service_id, label],
        )
        .unwrap();
    }

    pub fn delete(&self, wallet: &str, service: &str, service_id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM pins WHERE wallet = ?1 AND service = ?2 AND service_id = ?3",
            params![wallet, service, service_id],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::ProfileRepo;

    #[test]
    fn test_pin_crud() {
        let db = open_test_database();
        let profiles = ProfileRepo::new(db.clone());
        let repo = PinRepo::new(db);

        profiles.upsert("Alice", "0xabc123");

        assert!(repo.get_by_wallet("0xabc123").is_empty());
        assert_eq!(repo.count_by_wallet("0xabc123"), 0);

        repo.insert("0xabc123", "tmdb", "12345", "Inception");
        repo.insert("0xabc123", "tmdb", "67890", "The Matrix");

        let pins = repo.get_by_wallet("0xabc123");
        assert_eq!(pins.len(), 2);
        assert_eq!(repo.count_by_wallet("0xabc123"), 2);

        // Duplicate insert is ignored
        repo.insert("0xabc123", "tmdb", "12345", "Inception Updated");
        assert_eq!(repo.count_by_wallet("0xabc123"), 2);

        assert!(repo.delete("0xabc123", "tmdb", "12345"));
        assert_eq!(repo.count_by_wallet("0xabc123"), 1);

        assert!(!repo.delete("0xabc123", "tmdb", "99999"));
    }
}

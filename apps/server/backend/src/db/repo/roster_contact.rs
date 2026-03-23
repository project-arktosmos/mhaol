use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterContactRow {
    pub address: String,
    pub name: String,
    pub passport: Option<String>,
    pub instance_type: Option<String>,
    pub added_at: String,
}

#[derive(Clone)]
pub struct RosterContactRepo {
    db: DbPool,
}

impl RosterContactRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Vec<RosterContactRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT address, name, passport, instance_type, added_at FROM roster_contacts ORDER BY added_at DESC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(RosterContactRow {
                address: row.get(0)?,
                name: row.get(1)?,
                passport: row.get(2)?,
                instance_type: row.get(3)?,
                added_at: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn insert(&self, address: &str, name: &str, passport: Option<&str>, instance_type: Option<&str>) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO roster_contacts (address, name, passport, instance_type) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(address) DO UPDATE SET name = ?2, passport = ?3, instance_type = ?4",
            params![address, name, passport, instance_type],
        )
        .unwrap();
    }

    pub fn delete(&self, address: &str) -> bool {
        let conn = self.db.lock();
        conn.execute("DELETE FROM roster_contacts WHERE address = ?1", params![address])
            .map(|n| n > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_roster_contact_crud() {
        let db = open_test_database();
        let repo = RosterContactRepo::new(db);

        assert!(repo.get_all().is_empty());

        repo.insert("0xabc123", "Alice", Some("{\"raw\":\"test\"}"), Some("client"));
        let contacts = repo.get_all();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Alice");
        assert_eq!(contacts[0].address, "0xabc123");
        assert_eq!(contacts[0].instance_type.as_deref(), Some("client"));

        // Upsert
        repo.insert("0xabc123", "Alice Updated", None, Some("server"));
        let contacts = repo.get_all();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Alice Updated");

        assert!(repo.delete("0xabc123"));
        assert!(repo.get_all().is_empty());
        assert!(!repo.delete("nonexistent"));
    }
}

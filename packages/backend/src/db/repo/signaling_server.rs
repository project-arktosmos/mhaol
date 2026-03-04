use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingServerRow {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct SignalingServerRepo {
    db: DbPool,
}

impl SignalingServerRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<SignalingServerRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, name, url, enabled, created_at, updated_at FROM signaling_servers WHERE id = ?1",
            params![id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<SignalingServerRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, url, enabled, created_at, updated_at FROM signaling_servers ORDER BY created_at ASC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_enabled(&self) -> Vec<SignalingServerRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, url, enabled, created_at, updated_at FROM signaling_servers WHERE enabled = 1 ORDER BY created_at ASC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn insert(&self, id: &str, name: &str, url: &str, enabled: bool) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO signaling_servers (id, name, url, enabled) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, url, enabled],
        )
        .unwrap();
    }

    pub fn update(&self, id: &str, name: &str, url: &str, enabled: bool) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE signaling_servers SET name = ?2, url = ?3, enabled = ?4 WHERE id = ?1",
            params![id, name, url, enabled],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM signaling_servers WHERE id = ?1", params![id])
            .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<SignalingServerRow> {
        Ok(SignalingServerRow {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            enabled: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_signaling_server_crud() {
        let db = open_test_database();
        let repo = SignalingServerRepo::new(db);

        repo.insert("srv1", "Test Server", "https://test.partykit.dev", true);

        let all = repo.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Test Server");
        assert!(all[0].enabled);

        let got = repo.get("srv1").unwrap();
        assert_eq!(got.url, "https://test.partykit.dev");

        repo.update("srv1", "Renamed", "https://new.partykit.dev", false);
        let updated = repo.get("srv1").unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.url, "https://new.partykit.dev");
        assert!(!updated.enabled);

        assert!(repo.get_enabled().is_empty());

        repo.delete("srv1");
        assert!(repo.get_all().is_empty());
    }
}

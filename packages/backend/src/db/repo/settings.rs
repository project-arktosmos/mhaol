use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRow {
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct SettingsRepo {
    db: DbPool,
}

impl SettingsRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn get_row(&self, key: &str) -> Option<SettingRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT key, value, created_at, updated_at FROM settings WHERE key = ?1",
            params![key],
            |row| {
                Ok(SettingRow {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<SettingRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT key, value, created_at, updated_at FROM settings ORDER BY key")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(SettingRow {
                key: row.get(0)?,
                value: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_by_prefix(&self, prefix: &str) -> Vec<SettingRow> {
        let conn = self.db.lock();
        let pattern = format!("{}%", prefix);
        let mut stmt = conn
            .prepare("SELECT key, value, created_at, updated_at FROM settings WHERE key LIKE ?1 ORDER BY key")
            .unwrap();
        stmt.query_map(params![pattern], |row| {
            Ok(SettingRow {
                key: row.get(0)?,
                value: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn set(&self, key: &str, value: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )
        .unwrap();
    }

    pub fn delete(&self, key: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .unwrap();
    }

    pub fn exists(&self, key: &str) -> bool {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT 1 FROM settings WHERE key = ?1")
            .unwrap();
        stmt.exists(params![key]).unwrap_or(false)
    }

    pub fn set_many(&self, entries: &HashMap<String, String>) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        for (key, value) in entries {
            tx.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, value],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_settings_crud() {
        let db = open_test_database();
        let repo = SettingsRepo::new(db);

        assert!(!repo.exists("test_key"));
        repo.set("test_key", "test_value");
        assert!(repo.exists("test_key"));
        assert_eq!(repo.get("test_key"), Some("test_value".to_string()));

        repo.set("test_key", "updated");
        assert_eq!(repo.get("test_key"), Some("updated".to_string()));

        repo.delete("test_key");
        assert!(!repo.exists("test_key"));
    }

    #[test]
    fn test_settings_set_many() {
        let db = open_test_database();
        let repo = SettingsRepo::new(db);

        let mut entries = HashMap::new();
        entries.insert("a".to_string(), "1".to_string());
        entries.insert("b".to_string(), "2".to_string());
        repo.set_many(&entries);

        assert_eq!(repo.get("a"), Some("1".to_string()));
        assert_eq!(repo.get("b"), Some("2".to_string()));
    }

    #[test]
    fn test_settings_get_by_prefix() {
        let db = open_test_database();
        let repo = SettingsRepo::new(db);

        repo.set("plugin.foo.enabled", "true");
        repo.set("plugin.foo.path", "/tmp");
        repo.set("other.key", "val");

        let results = repo.get_by_prefix("plugin.foo.");
        assert_eq!(results.len(), 2);
    }
}

use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRow {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct MetadataRepo {
    db: DbPool,
}

impl MetadataRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> Option<MetadataRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT key, value, type, created_at, updated_at FROM metadata WHERE key = ?1",
            params![key],
            |row| {
                Ok(MetadataRow {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    value_type: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .ok()
    }

    /// Get a typed value. Deserializes based on stored type.
    pub fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        let row = self.get(key)?;
        match row.value_type.as_str() {
            "number" => row.value.parse::<f64>().ok().map(serde_json::Value::from),
            "boolean" => Some(serde_json::Value::Bool(row.value == "true")),
            "json" => serde_json::from_str(&row.value).ok(),
            _ => Some(serde_json::Value::String(row.value)),
        }
    }

    /// Get value as a string directly.
    pub fn get_value_string(&self, key: &str) -> Option<String> {
        let row = self.get(key)?;
        Some(row.value)
    }

    pub fn get_all(&self) -> Vec<MetadataRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT key, value, type, created_at, updated_at FROM metadata ORDER BY key")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(MetadataRow {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_by_type(&self, value_type: &str) -> Vec<MetadataRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT key, value, type, created_at, updated_at FROM metadata WHERE type = ?1 ORDER BY key")
            .unwrap();
        stmt.query_map(params![value_type], |row| {
            Ok(MetadataRow {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn set_string(&self, key: &str, value: &str) {
        self.set_raw(key, value, "string");
    }

    pub fn set_number(&self, key: &str, value: f64) {
        self.set_raw(key, &value.to_string(), "number");
    }

    pub fn set_boolean(&self, key: &str, value: bool) {
        self.set_raw(key, &value.to_string(), "boolean");
    }

    pub fn set_json(&self, key: &str, value: &serde_json::Value) {
        self.set_raw(key, &value.to_string(), "json");
    }

    fn set_raw(&self, key: &str, value: &str, value_type: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO metadata (key, value, type) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, type = ?3",
            params![key, value, value_type],
        )
        .unwrap();
    }

    pub fn delete(&self, key: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM metadata WHERE key = ?1", params![key])
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_metadata_types() {
        let db = open_test_database();
        let repo = MetadataRepo::new(db);

        repo.set_string("name", "test");
        repo.set_number("count", 42.0);
        repo.set_boolean("enabled", true);
        repo.set_json("config", &serde_json::json!({"a": 1}));

        assert_eq!(
            repo.get_value("name"),
            Some(serde_json::Value::String("test".to_string()))
        );
        assert_eq!(
            repo.get_value("count"),
            Some(serde_json::Value::from(42.0))
        );
        assert_eq!(
            repo.get_value("enabled"),
            Some(serde_json::Value::Bool(true))
        );
        assert_eq!(
            repo.get_value("config"),
            Some(serde_json::json!({"a": 1}))
        );
    }
}

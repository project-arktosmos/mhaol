use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRow {
    pub username: String,
    pub wallet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_picture_url: Option<String>,
    pub added_at: String,
}

#[derive(Clone)]
pub struct ProfileRepo {
    db: DbPool,
}

impl ProfileRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Vec<ProfileRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT username, wallet, profile_picture_url, added_at FROM profiles ORDER BY added_at DESC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(ProfileRow {
                username: row.get(0)?,
                wallet: row.get(1)?,
                profile_picture_url: row.get(2)?,
                added_at: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_by_wallet(&self, wallet: &str) -> Option<ProfileRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT username, wallet, profile_picture_url, added_at FROM profiles WHERE wallet = ?1",
            params![wallet],
            |row| {
                Ok(ProfileRow {
                    username: row.get(0)?,
                    wallet: row.get(1)?,
                    profile_picture_url: row.get(2)?,
                    added_at: row.get(3)?,
                })
            },
        )
        .ok()
    }

    pub fn upsert(&self, username: &str, wallet: &str, profile_picture_url: Option<&str>) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO profiles (wallet, username, profile_picture_url) VALUES (?1, ?2, ?3)
             ON CONFLICT(wallet) DO UPDATE SET username = ?2, profile_picture_url = ?3",
            params![wallet, username, profile_picture_url],
        )
        .unwrap();
    }

    pub fn delete(&self, wallet: &str) -> bool {
        let conn = self.db.lock();
        conn.execute("DELETE FROM profiles WHERE wallet = ?1", params![wallet])
            .map(|n| n > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_profile_crud() {
        let db = open_test_database();
        let repo = ProfileRepo::new(db);

        assert!(repo.get_all().is_empty());

        repo.upsert("Alice", "0xabc123", None);
        let profiles = repo.get_all();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].username, "Alice");
        assert_eq!(profiles[0].wallet, "0xabc123");
        assert!(profiles[0].profile_picture_url.is_none());

        // Upsert same wallet, different name + profile picture
        repo.upsert("Alice Updated", "0xabc123", Some("https://example.com/pic.png"));
        let profiles = repo.get_all();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].username, "Alice Updated");
        assert_eq!(profiles[0].profile_picture_url.as_deref(), Some("https://example.com/pic.png"));

        assert!(repo.delete("0xabc123"));
        assert!(repo.get_all().is_empty());
        assert!(!repo.delete("nonexistent"));
    }
}

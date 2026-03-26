use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConversationRow {
    pub id: String,
    pub title: String,
    pub system_prompt: Option<String>,
    pub messages: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct LlmConversationRepo {
    db: DbPool,
}

impl LlmConversationRepo {
    pub fn new(db: Arc<parking_lot::Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Vec<LlmConversationRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, system_prompt, messages, created_at, updated_at
                 FROM llm_conversations ORDER BY updated_at DESC",
            )
            .unwrap();
        stmt.query_map([], |row| {
            Ok(LlmConversationRow {
                id: row.get(0)?,
                title: row.get(1)?,
                system_prompt: row.get(2)?,
                messages: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_by_id(&self, id: &str) -> Option<LlmConversationRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, title, system_prompt, messages, created_at, updated_at
             FROM llm_conversations WHERE id = ?1",
            params![id],
            |row| {
                Ok(LlmConversationRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    system_prompt: row.get(2)?,
                    messages: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .ok()
    }

    pub fn insert(&self, id: &str, title: &str, system_prompt: Option<&str>, messages: &str) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO llm_conversations (id, title, system_prompt, messages) VALUES (?1, ?2, ?3, ?4)",
            params![id, title, system_prompt, messages],
        )
        .unwrap();
    }

    pub fn update(&self, id: &str, title: &str, messages: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE llm_conversations SET title = ?2, messages = ?3 WHERE id = ?1",
            params![id, title, messages],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM llm_conversations WHERE id = ?1", params![id])
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_llm_conversation_crud() {
        let db = open_test_database();
        let repo = LlmConversationRepo::new(db);

        repo.insert("c1", "Test Chat", Some("You are helpful."), "[]");
        let row = repo.get_by_id("c1").unwrap();
        assert_eq!(row.title, "Test Chat");
        assert_eq!(row.system_prompt, Some("You are helpful.".to_string()));
        assert_eq!(row.messages, "[]");

        repo.update(
            "c1",
            "Updated Chat",
            "[{\"role\":\"user\",\"content\":\"hi\"}]",
        );
        let row = repo.get_by_id("c1").unwrap();
        assert_eq!(row.title, "Updated Chat");

        let all = repo.get_all();
        assert_eq!(all.len(), 1);

        repo.delete("c1");
        assert!(repo.get_by_id("c1").is_none());
    }
}

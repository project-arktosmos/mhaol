use crate::db::DbPool;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationLabelRow {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationLabelAssignmentRow {
    pub id: String,
    pub wallet: String,
    pub source: String,
    pub source_id: String,
    pub source_type: String,
    pub label_id: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct RecommendationLabelRepo {
    db: DbPool,
}

impl RecommendationLabelRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn list_labels(&self) -> Vec<RecommendationLabelRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, name, emoji, sort_order FROM recommendation_labels ORDER BY sort_order ASC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(RecommendationLabelRow {
                id: row.get(0)?,
                name: row.get(1)?,
                emoji: row.get(2)?,
                sort_order: row.get(3)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_assignments_by_wallet(&self, wallet: &str) -> Vec<RecommendationLabelAssignmentRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, wallet, source, source_id, source_type, label_id, created_at
                 FROM recommendation_label_assignments
                 WHERE wallet = ?1
                 ORDER BY created_at DESC",
            )
            .unwrap();
        stmt.query_map(params![wallet], |row| {
            Ok(RecommendationLabelAssignmentRow {
                id: row.get(0)?,
                wallet: row.get(1)?,
                source: row.get(2)?,
                source_id: row.get(3)?,
                source_type: row.get(4)?,
                label_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_assignments_by_wallet_and_source(
        &self,
        wallet: &str,
        source: &str,
    ) -> Vec<RecommendationLabelAssignmentRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, wallet, source, source_id, source_type, label_id, created_at
                 FROM recommendation_label_assignments
                 WHERE wallet = ?1 AND source = ?2
                 ORDER BY created_at DESC",
            )
            .unwrap();
        stmt.query_map(params![wallet, source], |row| {
            Ok(RecommendationLabelAssignmentRow {
                id: row.get(0)?,
                wallet: row.get(1)?,
                source: row.get(2)?,
                source_id: row.get(3)?,
                source_type: row.get(4)?,
                label_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn upsert(
        &self,
        wallet: &str,
        source: &str,
        source_id: &str,
        source_type: &str,
        label_id: &str,
    ) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO recommendation_label_assignments (id, wallet, source, source_id, source_type, label_id)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(wallet, source, source_id, source_type)
             DO UPDATE SET label_id = ?5, created_at = datetime('now')",
            params![wallet, source, source_id, source_type, label_id],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }

    pub fn delete(&self, wallet: &str, source: &str, source_id: &str, source_type: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM recommendation_label_assignments
             WHERE wallet = ?1 AND source = ?2 AND source_id = ?3 AND source_type = ?4",
            params![wallet, source, source_id, source_type],
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
    fn test_list_labels() {
        let db = open_test_database();
        let repo = RecommendationLabelRepo::new(db);
        let labels = repo.list_labels();
        assert_eq!(labels.len(), 4);
        assert_eq!(labels[0].id, "thumbs-up");
        assert_eq!(labels[1].id, "thumbs-down");
        assert_eq!(labels[2].id, "love");
        assert_eq!(labels[3].id, "hate");
    }

    #[test]
    fn test_upsert_and_get() {
        let db = open_test_database();
        let profiles = ProfileRepo::new(db.clone());
        let repo = RecommendationLabelRepo::new(db);

        profiles.upsert("Alice", "0xabc123", None);

        assert!(repo.get_assignments_by_wallet("0xabc123").is_empty());

        assert!(repo.upsert("0xabc123", "tmdb", "550", "movie", "love"));
        let assignments = repo.get_assignments_by_wallet("0xabc123");
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].source_id, "550");
        assert_eq!(assignments[0].label_id, "love");
    }

    #[test]
    fn test_upsert_replaces_label() {
        let db = open_test_database();
        let profiles = ProfileRepo::new(db.clone());
        let repo = RecommendationLabelRepo::new(db);

        profiles.upsert("Alice", "0xabc123", None);

        repo.upsert("0xabc123", "tmdb", "550", "movie", "love");
        repo.upsert("0xabc123", "tmdb", "550", "movie", "hate");

        let assignments = repo.get_assignments_by_wallet("0xabc123");
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].label_id, "hate");
    }

    #[test]
    fn test_delete() {
        let db = open_test_database();
        let profiles = ProfileRepo::new(db.clone());
        let repo = RecommendationLabelRepo::new(db);

        profiles.upsert("Alice", "0xabc123", None);

        repo.upsert("0xabc123", "tmdb", "550", "movie", "thumbs-up");
        assert!(repo.delete("0xabc123", "tmdb", "550", "movie"));
        assert!(repo.get_assignments_by_wallet("0xabc123").is_empty());

        // Delete non-existent returns false
        assert!(!repo.delete("0xabc123", "tmdb", "550", "movie"));
    }

    #[test]
    fn test_different_users_different_labels() {
        let db = open_test_database();
        let profiles = ProfileRepo::new(db.clone());
        let repo = RecommendationLabelRepo::new(db);

        profiles.upsert("Alice", "0xabc123", None);
        profiles.upsert("Bob", "0xdef456", None);

        repo.upsert("0xabc123", "tmdb", "550", "movie", "love");
        repo.upsert("0xdef456", "tmdb", "550", "movie", "hate");

        let alice = repo.get_assignments_by_wallet("0xabc123");
        assert_eq!(alice.len(), 1);
        assert_eq!(alice[0].label_id, "love");

        let bob = repo.get_assignments_by_wallet("0xdef456");
        assert_eq!(bob.len(), 1);
        assert_eq!(bob[0].label_id, "hate");
    }
}

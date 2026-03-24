use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSourceRow {
    pub id: String,
    pub plugin: String,
    pub service: String,
    pub label: String,
    pub media_type_id: String,
    pub category_id: Option<String>,
}

#[derive(Clone)]
pub struct LinkSourceRepo {
    db: DbPool,
}

impl LinkSourceRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Vec<LinkSourceRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, plugin, service, label, media_type_id, category_id FROM link_sources ORDER BY media_type_id, label ASC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_media_type(&self, media_type_id: &str) -> Vec<LinkSourceRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, plugin, service, label, media_type_id, category_id FROM link_sources WHERE media_type_id = ?1 AND category_id IS NULL ORDER BY label ASC")
            .unwrap();
        stmt.query_map(params![media_type_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_media_type_and_category(
        &self,
        media_type_id: &str,
        category_id: &str,
    ) -> Vec<LinkSourceRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, plugin, service, label, media_type_id, category_id FROM link_sources WHERE media_type_id = ?1 AND (category_id IS NULL OR category_id = ?2) ORDER BY label ASC")
            .unwrap();
        stmt.query_map(params![media_type_id, category_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn upsert(&self, row: &LinkSourceRow) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO link_sources (id, plugin, service, label, media_type_id, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(service, media_type_id, category_id) DO UPDATE SET
                plugin = excluded.plugin,
                label = excluded.label",
            params![row.id, row.plugin, row.service, row.label, row.media_type_id, row.category_id],
        )
        .unwrap();
    }

    pub fn delete_by_plugin(&self, plugin: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM link_sources WHERE plugin = ?1",
            params![plugin],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<LinkSourceRow> {
        Ok(LinkSourceRow {
            id: row.get(0)?,
            plugin: row.get(1)?,
            service: row.get(2)?,
            label: row.get(3)?,
            media_type_id: row.get(4)?,
            category_id: row.get(5)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_link_source_upsert() {
        let db = open_test_database();
        let repo = LinkSourceRepo::new(db);

        repo.upsert(&LinkSourceRow {
            id: "ls1".into(),
            plugin: "tmdb".into(),
            service: "tmdb".into(),
            label: "TMDB".into(),
            media_type_id: "video".into(),
            category_id: Some("movies".into()),
        });

        let all = repo.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].label, "TMDB");

        // Upsert updates label
        repo.upsert(&LinkSourceRow {
            id: "ls2".into(),
            plugin: "tmdb".into(),
            service: "tmdb".into(),
            label: "The Movie DB".into(),
            media_type_id: "video".into(),
            category_id: Some("movies".into()),
        });

        let all = repo.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].label, "The Movie DB");
    }
}

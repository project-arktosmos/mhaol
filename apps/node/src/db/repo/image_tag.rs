use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTagRow {
    pub id: String,
    pub library_item_id: String,
    pub tag: String,
    pub score: f64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct ImageTagRepo {
    db: DbPool,
}

impl ImageTagRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_item(&self, library_item_id: &str) -> Vec<ImageTagRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, tag, score, created_at FROM image_tags WHERE library_item_id = ?1 ORDER BY score DESC")
            .unwrap();
        stmt.query_map(params![library_item_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_items(&self, library_item_ids: &[&str]) -> Vec<ImageTagRow> {
        let mut result = Vec::new();
        for id in library_item_ids {
            result.extend(self.get_by_item(id));
        }
        result
    }

    pub fn replace_for_item(&self, library_item_id: &str, tags: &[(&str, f64)]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        tx.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1",
            params![library_item_id],
        )
        .unwrap();
        for (tag, score) in tags {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO image_tags (id, library_item_id, tag, score) VALUES (?1, ?2, ?3, ?4)",
                params![id, library_item_id, tag, score],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    pub fn add_tag(&self, library_item_id: &str, tag: &str, score: f64) {
        let conn = self.db.lock();
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO image_tags (id, library_item_id, tag, score) VALUES (?1, ?2, ?3, ?4)",
            params![id, library_item_id, tag, score],
        )
        .unwrap();
    }

    pub fn delete_tag(&self, library_item_id: &str, tag: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1 AND tag = ?2",
            params![library_item_id, tag],
        )
        .unwrap();
    }

    pub fn delete_by_item(&self, library_item_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM image_tags WHERE library_item_id = ?1",
            params![library_item_id],
        )
        .unwrap();
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<ImageTagRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, tag, score, created_at FROM image_tags WHERE tag = ?1 ORDER BY score DESC")
            .unwrap();
        stmt.query_map(params![tag], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImageTagRow> {
        Ok(ImageTagRow {
            id: row.get(0)?,
            library_item_id: row.get(1)?,
            tag: row.get(2)?,
            score: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn setup() -> (ImageTagRepo, crate::db::DbPool) {
        let db = open_test_database();
        // Insert parent records required by foreign keys
        {
            let conn = db.lock();
            conn.execute(
                "INSERT INTO libraries (id, name, path, media_types, date_added) VALUES ('lib1', 'Test', '/tmp', '[]', 0)",
                [],
            ).unwrap();
        }
        let repo = ImageTagRepo::new(db.clone());
        (repo, db)
    }

    fn insert_library_item(db: &crate::db::DbPool, id: &str) {
        let conn = db.lock();
        conn.execute(
            "INSERT OR IGNORE INTO library_items (id, library_id, path, extension, media_type) VALUES (?1, 'lib1', ?2, 'jpg', 'image')",
            rusqlite::params![id, format!("/tmp/{}.jpg", id)],
        ).unwrap();
    }

    #[test]
    fn test_add_tag_and_get_by_item() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");

        repo.add_tag("item1", "landscape", 0.95);
        repo.add_tag("item1", "sunset", 0.80);

        let tags = repo.get_by_item("item1");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag, "landscape");
        assert_eq!(tags[1].tag, "sunset");
    }

    #[test]
    fn test_get_by_item_empty() {
        let (repo, _db) = setup();
        assert!(repo.get_by_item("nonexistent").is_empty());
    }

    #[test]
    fn test_replace_for_item() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");

        repo.add_tag("item1", "old_tag", 0.5);
        repo.replace_for_item("item1", &[("new_tag1", 0.9), ("new_tag2", 0.7)]);

        let tags = repo.get_by_item("item1");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag, "new_tag1");
        assert_eq!(tags[1].tag, "new_tag2");
    }

    #[test]
    fn test_delete_tag() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");

        repo.add_tag("item1", "landscape", 0.9);
        repo.add_tag("item1", "sunset", 0.8);

        repo.delete_tag("item1", "landscape");
        let tags = repo.get_by_item("item1");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, "sunset");
    }

    #[test]
    fn test_delete_by_item() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");
        insert_library_item(&db, "item2");

        repo.add_tag("item1", "tag1", 0.9);
        repo.add_tag("item1", "tag2", 0.8);
        repo.add_tag("item2", "tag1", 0.7);

        repo.delete_by_item("item1");
        assert!(repo.get_by_item("item1").is_empty());
        assert_eq!(repo.get_by_item("item2").len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");
        insert_library_item(&db, "item2");
        insert_library_item(&db, "item3");

        repo.add_tag("item1", "landscape", 0.95);
        repo.add_tag("item2", "landscape", 0.80);
        repo.add_tag("item3", "portrait", 0.90);

        let results = repo.search_by_tag("landscape");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].score, 0.95);

        assert!(repo.search_by_tag("nonexistent").is_empty());
    }

    #[test]
    fn test_get_by_items() {
        let (repo, db) = setup();
        insert_library_item(&db, "item1");
        insert_library_item(&db, "item2");
        insert_library_item(&db, "item3");

        repo.add_tag("item1", "tag1", 0.9);
        repo.add_tag("item2", "tag2", 0.8);
        repo.add_tag("item3", "tag3", 0.7);

        let results = repo.get_by_items(&["item1", "item3"]);
        assert_eq!(results.len(), 2);
    }
}

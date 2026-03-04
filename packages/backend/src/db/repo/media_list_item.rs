use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListItemRow {
    pub id: String,
    pub list_id: String,
    pub library_item_id: String,
    pub position: i64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MediaListItemRepo {
    db: DbPool,
}

impl MediaListItemRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_list(&self, list_id: &str) -> Vec<MediaListItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, list_id, library_item_id, position, created_at FROM media_list_items WHERE list_id = ?1 ORDER BY position ASC")
            .unwrap();
        stmt.query_map(params![list_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn delete_by_list(&self, list_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM media_list_items WHERE list_id = ?1",
            params![list_id],
        )
        .unwrap();
    }

    /// Replace all items in a list with the given items (delete + insert in one lock).
    pub fn sync_list(&self, list_id: &str, items: &[(String, String, i64)]) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM media_list_items WHERE list_id = ?1",
            params![list_id],
        )
        .unwrap();
        for (id, library_item_id, position) in items {
            conn.execute(
                "INSERT OR IGNORE INTO media_list_items (id, list_id, library_item_id, position) VALUES (?1, ?2, ?3, ?4)",
                params![id, list_id, library_item_id, position],
            )
            .unwrap();
        }
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaListItemRow> {
        Ok(MediaListItemRow {
            id: row.get(0)?,
            list_id: row.get(1)?,
            library_item_id: row.get(2)?,
            position: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::{LibraryItemRepo, LibraryRepo, MediaListRepo};
    use crate::db::repo::library_item::InsertLibraryItem;

    fn setup() -> (MediaListRepo, MediaListItemRepo) {
        let db = open_test_database();
        let lib_repo = LibraryRepo::new(db.clone());
        let item_repo = LibraryItemRepo::new(db.clone());
        let list_repo = MediaListRepo::new(db.clone());
        let list_item_repo = MediaListItemRepo::new(db);

        lib_repo.insert("lib1", "Test", "/tmp", "[\"video\"]", 1000);
        item_repo.insert(&InsertLibraryItem {
            id: "item1".into(),
            library_id: "lib1".into(),
            path: "/tmp/ep01.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });
        item_repo.insert(&InsertLibraryItem {
            id: "item2".into(),
            library_id: "lib1".into(),
            path: "/tmp/ep02.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });
        list_repo.insert("list1", "lib1", "Test Show", None, None, "video", "auto", Some("/tmp:video"));

        (list_repo, list_item_repo)
    }

    #[test]
    fn test_sync_list() {
        let (_list_repo, repo) = setup();

        repo.sync_list("list1", &[
            ("li1".into(), "item1".into(), 0),
            ("li2".into(), "item2".into(), 1),
        ]);

        let items = repo.get_by_list("list1");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].library_item_id, "item1");
        assert_eq!(items[0].position, 0);
        assert_eq!(items[1].library_item_id, "item2");
        assert_eq!(items[1].position, 1);

        // Re-sync with only one item
        repo.sync_list("list1", &[
            ("li3".into(), "item2".into(), 0),
        ]);
        let items = repo.get_by_list("list1");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].library_item_id, "item2");
    }
}

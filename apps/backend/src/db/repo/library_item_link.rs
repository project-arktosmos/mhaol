use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemLinkRow {
    pub id: String,
    pub library_item_id: String,
    pub service: String,
    pub service_id: String,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct LibraryItemLinkRepo {
    db: DbPool,
}

impl LibraryItemLinkRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_item(&self, library_item_id: &str) -> Vec<LibraryItemLinkRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, service, service_id, season_number, episode_number, created_at FROM library_item_links WHERE library_item_id = ?1 ORDER BY service ASC")
            .unwrap();
        stmt.query_map(params![library_item_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_item_and_service(
        &self,
        library_item_id: &str,
        service: &str,
    ) -> Option<LibraryItemLinkRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, library_item_id, service, service_id, season_number, episode_number, created_at FROM library_item_links WHERE library_item_id = ?1 AND service = ?2",
            params![library_item_id, service],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_by_service_id(&self, service: &str, service_id: &str) -> Vec<LibraryItemLinkRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_item_id, service, service_id, season_number, episode_number, created_at FROM library_item_links WHERE service = ?1 AND service_id = ?2 ORDER BY library_item_id ASC")
            .unwrap();
        stmt.query_map(params![service, service_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn upsert(
        &self,
        id: &str,
        library_item_id: &str,
        service: &str,
        service_id: &str,
        season_number: Option<i64>,
        episode_number: Option<i64>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO library_item_links (id, library_item_id, service, service_id, season_number, episode_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(library_item_id, service) DO UPDATE SET
                service_id = excluded.service_id,
                season_number = excluded.season_number,
                episode_number = excluded.episode_number",
            params![id, library_item_id, service, service_id, season_number, episode_number],
        )
        .unwrap();
    }

    pub fn delete(&self, library_item_id: &str, service: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM library_item_links WHERE library_item_id = ?1 AND service = ?2",
            params![library_item_id, service],
        )
        .unwrap();
    }

    pub fn delete_by_item(&self, library_item_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM library_item_links WHERE library_item_id = ?1",
            params![library_item_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryItemLinkRow> {
        Ok(LibraryItemLinkRow {
            id: row.get(0)?,
            library_item_id: row.get(1)?,
            service: row.get(2)?,
            service_id: row.get(3)?,
            season_number: row.get(4)?,
            episode_number: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::{LibraryItemRepo, LibraryRepo};
    use crate::db::repo::library_item::InsertLibraryItem;

    fn setup() -> LibraryItemLinkRepo {
        let db = open_test_database();
        let lib_repo = LibraryRepo::new(db.clone());
        let item_repo = LibraryItemRepo::new(db.clone());
        lib_repo.insert("lib1", "Test", "/tmp", "[\"video\"]", 1000);
        item_repo.insert(&InsertLibraryItem {
            id: "item1".into(),
            library_id: "lib1".into(),
            path: "/tmp/movie.mp4".into(),
            extension: "mp4".into(),
            media_type: "video".into(),
            category_id: None,
        });
        LibraryItemLinkRepo::new(db)
    }

    #[test]
    fn test_link_upsert() {
        let repo = setup();

        repo.upsert("link1", "item1", "tmdb", "12345", Some(1), Some(3));
        let link = repo.get_by_item_and_service("item1", "tmdb").unwrap();
        assert_eq!(link.service_id, "12345");
        assert_eq!(link.season_number, Some(1));

        // Upsert updates
        repo.upsert("link2", "item1", "tmdb", "99999", None, None);
        let link = repo.get_by_item_and_service("item1", "tmdb").unwrap();
        assert_eq!(link.service_id, "99999");
        assert!(link.season_number.is_none());
    }
}

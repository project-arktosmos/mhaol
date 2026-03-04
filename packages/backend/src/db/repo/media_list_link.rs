use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListLinkRow {
    pub id: String,
    pub list_id: String,
    pub service: String,
    pub service_id: String,
    pub season_number: Option<i64>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MediaListLinkRepo {
    db: DbPool,
}

impl MediaListLinkRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_list(&self, list_id: &str) -> Vec<MediaListLinkRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, list_id, service, service_id, season_number, created_at FROM media_list_links WHERE list_id = ?1 ORDER BY service ASC")
            .unwrap();
        stmt.query_map(params![list_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_list_and_service(
        &self,
        list_id: &str,
        service: &str,
    ) -> Option<MediaListLinkRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, list_id, service, service_id, season_number, created_at FROM media_list_links WHERE list_id = ?1 AND service = ?2",
            params![list_id, service],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn upsert(
        &self,
        id: &str,
        list_id: &str,
        service: &str,
        service_id: &str,
        season_number: Option<i64>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO media_list_links (id, list_id, service, service_id, season_number)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(list_id, service) DO UPDATE SET
                service_id = excluded.service_id,
                season_number = excluded.season_number",
            params![id, list_id, service, service_id, season_number],
        )
        .unwrap();
    }

    pub fn delete(&self, list_id: &str, service: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM media_list_links WHERE list_id = ?1 AND service = ?2",
            params![list_id, service],
        )
        .unwrap();
    }

    pub fn delete_by_list(&self, list_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM media_list_links WHERE list_id = ?1",
            params![list_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaListLinkRow> {
        Ok(MediaListLinkRow {
            id: row.get(0)?,
            list_id: row.get(1)?,
            service: row.get(2)?,
            service_id: row.get(3)?,
            season_number: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::{LibraryRepo, MediaListRepo};

    fn setup() -> MediaListLinkRepo {
        let db = open_test_database();
        let lib_repo = LibraryRepo::new(db.clone());
        let list_repo = MediaListRepo::new(db.clone());
        lib_repo.insert("lib1", "Test", "/tmp", "[\"video\"]", 1000);
        list_repo.insert("list1", "lib1", "My List", None, None, "video", "auto", None);
        MediaListLinkRepo::new(db)
    }

    #[test]
    fn test_link_upsert() {
        let repo = setup();

        repo.upsert("link1", "list1", "tmdb", "12345", Some(2));
        let link = repo.get_by_list_and_service("list1", "tmdb").unwrap();
        assert_eq!(link.service_id, "12345");
        assert_eq!(link.season_number, Some(2));

        // Upsert updates
        repo.upsert("link2", "list1", "tmdb", "99999", None);
        let link = repo.get_by_list_and_service("list1", "tmdb").unwrap();
        assert_eq!(link.service_id, "99999");
        assert_eq!(link.season_number, None);
    }

    #[test]
    fn test_link_delete() {
        let repo = setup();

        repo.upsert("link1", "list1", "tmdb", "12345", None);
        assert!(repo.get_by_list_and_service("list1", "tmdb").is_some());

        repo.delete("list1", "tmdb");
        assert!(repo.get_by_list_and_service("list1", "tmdb").is_none());
    }
}

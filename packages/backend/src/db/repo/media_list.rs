use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListRow {
    pub id: String,
    pub library_id: String,
    pub title: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub media_type: String,
    pub source: String,
    pub source_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct MediaListRepo {
    db: DbPool,
}

impl MediaListRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_id(&self, id: &str) -> Option<MediaListRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, library_id, title, description, cover_image, media_type, source, source_path, created_at, updated_at FROM media_lists WHERE id = ?1",
            params![id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<MediaListRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, title, description, cover_image, media_type, source, source_path, created_at, updated_at FROM media_lists ORDER BY title ASC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_library(&self, library_id: &str) -> Vec<MediaListRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, title, description, cover_image, media_type, source, source_path, created_at, updated_at FROM media_lists WHERE library_id = ?1 ORDER BY title ASC")
            .unwrap();
        stmt.query_map(params![library_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_auto_by_library(&self, library_id: &str) -> Vec<MediaListRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT id, library_id, title, description, cover_image, media_type, source, source_path, created_at, updated_at FROM media_lists WHERE library_id = ?1 AND source = 'auto' ORDER BY title ASC")
            .unwrap();
        stmt.query_map(params![library_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_source_path(&self, source_path: &str) -> Option<MediaListRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, library_id, title, description, cover_image, media_type, source, source_path, created_at, updated_at FROM media_lists WHERE source_path = ?1",
            params![source_path],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn insert(
        &self,
        id: &str,
        library_id: &str,
        title: &str,
        description: Option<&str>,
        cover_image: Option<&str>,
        media_type: &str,
        source: &str,
        source_path: Option<&str>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO media_lists (id, library_id, title, description, cover_image, media_type, source, source_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, library_id, title, description, cover_image, media_type, source, source_path],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM media_lists WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn delete_by_library(&self, library_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM media_lists WHERE library_id = ?1",
            params![library_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaListRow> {
        Ok(MediaListRow {
            id: row.get(0)?,
            library_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            cover_image: row.get(4)?,
            media_type: row.get(5)?,
            source: row.get(6)?,
            source_path: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::LibraryRepo;

    fn setup() -> (LibraryRepo, MediaListRepo) {
        let db = open_test_database();
        let lib_repo = LibraryRepo::new(db.clone());
        let list_repo = MediaListRepo::new(db);
        lib_repo.insert("lib1", "Test", "/tmp", "[\"video\"]", 1000);
        (lib_repo, list_repo)
    }

    #[test]
    fn test_media_list_crud() {
        let (_lib_repo, repo) = setup();

        repo.insert("list1", "lib1", "My TV Show", None, None, "video", "auto", Some("/tmp/show:video"));

        let all = repo.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "My TV Show");
        assert_eq!(all[0].source, "auto");

        let by_source = repo.get_by_source_path("/tmp/show:video").unwrap();
        assert_eq!(by_source.id, "list1");

        let auto = repo.get_auto_by_library("lib1");
        assert_eq!(auto.len(), 1);

        repo.delete("list1");
        assert!(repo.get_all().is_empty());
    }
}

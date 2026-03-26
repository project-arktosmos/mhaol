use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItemRow {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub sort_title: String,
    pub year: Option<String>,
    pub overview: Option<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i64>,
    pub parent_id: Option<String>,
    pub position: Option<i64>,
    pub source: String,
    pub source_id: String,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct CatalogRepo {
    db: DbPool,
}

impl CatalogRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_id(&self, id: &str) -> Option<CatalogItemRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                    vote_average, vote_count, parent_id, position, source, source_id,
                    metadata, created_at, updated_at
             FROM catalog_items WHERE id = ?1",
            params![id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_by_source(&self, source: &str, source_id: &str, kind: &str) -> Option<CatalogItemRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                    vote_average, vote_count, parent_id, position, source, source_id,
                    metadata, created_at, updated_at
             FROM catalog_items WHERE source = ?1 AND source_id = ?2 AND kind = ?3",
            params![source, source_id, kind],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_by_kind(&self, kind: &str, limit: i64, offset: i64) -> Vec<CatalogItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                        vote_average, vote_count, parent_id, position, source, source_id,
                        metadata, created_at, updated_at
                 FROM catalog_items WHERE kind = ?1
                 ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3",
            )
            .unwrap();
        stmt.query_map(params![kind, limit, offset], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_children(&self, parent_id: &str) -> Vec<CatalogItemRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                        vote_average, vote_count, parent_id, position, source, source_id,
                        metadata, created_at, updated_at
                 FROM catalog_items WHERE parent_id = ?1
                 ORDER BY position ASC, title ASC",
            )
            .unwrap();
        stmt.query_map(params![parent_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn count_by_kind(&self, kind: &str) -> i64 {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT COUNT(*) FROM catalog_items WHERE kind = ?1",
            params![kind],
            |row| row.get(0),
        )
        .unwrap_or(0)
    }

    pub fn search(&self, query: &str, kind: Option<&str>, limit: i64, offset: i64) -> Vec<CatalogItemRow> {
        let conn = self.db.lock();
        let like = format!("%{}%", query.to_lowercase());
        if let Some(k) = kind {
            let mut stmt = conn
                .prepare(
                    "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                            vote_average, vote_count, parent_id, position, source, source_id,
                            metadata, created_at, updated_at
                     FROM catalog_items WHERE kind = ?1 AND sort_title LIKE ?2
                     ORDER BY vote_average DESC NULLS LAST, title ASC
                     LIMIT ?3 OFFSET ?4",
                )
                .unwrap();
            stmt.query_map(params![k, like, limit, offset], Self::row_mapper)
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id, kind, title, sort_title, year, overview, poster_url, backdrop_url,
                            vote_average, vote_count, parent_id, position, source, source_id,
                            metadata, created_at, updated_at
                     FROM catalog_items WHERE sort_title LIKE ?1
                     ORDER BY vote_average DESC NULLS LAST, title ASC
                     LIMIT ?2 OFFSET ?3",
                )
                .unwrap();
            stmt.query_map(params![like, limit, offset], Self::row_mapper)
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        }
    }

    pub fn upsert(&self, item: &CatalogItemRow) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO catalog_items (id, kind, title, sort_title, year, overview,
                poster_url, backdrop_url, vote_average, vote_count, parent_id, position,
                source, source_id, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(source, source_id, kind) DO UPDATE SET
                title = excluded.title,
                sort_title = excluded.sort_title,
                year = excluded.year,
                overview = excluded.overview,
                poster_url = excluded.poster_url,
                backdrop_url = excluded.backdrop_url,
                vote_average = excluded.vote_average,
                vote_count = excluded.vote_count,
                parent_id = excluded.parent_id,
                position = excluded.position,
                metadata = excluded.metadata,
                updated_at = datetime('now')",
            params![
                item.id,
                item.kind,
                item.title,
                item.sort_title,
                item.year,
                item.overview,
                item.poster_url,
                item.backdrop_url,
                item.vote_average,
                item.vote_count,
                item.parent_id,
                item.position,
                item.source,
                item.source_id,
                item.metadata,
            ],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute("DELETE FROM catalog_items WHERE id = ?1", params![id])
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<CatalogItemRow> {
        Ok(CatalogItemRow {
            id: row.get(0)?,
            kind: row.get(1)?,
            title: row.get(2)?,
            sort_title: row.get(3)?,
            year: row.get(4)?,
            overview: row.get(5)?,
            poster_url: row.get(6)?,
            backdrop_url: row.get(7)?,
            vote_average: row.get(8)?,
            vote_count: row.get(9)?,
            parent_id: row.get(10)?,
            position: row.get(11)?,
            source: row.get(12)?,
            source_id: row.get(13)?,
            metadata: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    #[test]
    fn test_catalog_crud() {
        let db = open_test_database();
        let repo = CatalogRepo::new(db);

        let item = CatalogItemRow {
            id: "test-1".to_string(),
            kind: "movie".to_string(),
            title: "Fight Club".to_string(),
            sort_title: "fight club".to_string(),
            year: Some("1999".to_string()),
            overview: Some("An insomniac office worker...".to_string()),
            poster_url: Some("/poster.jpg".to_string()),
            backdrop_url: Some("/backdrop.jpg".to_string()),
            vote_average: Some(8.4),
            vote_count: Some(25000),
            parent_id: None,
            position: None,
            source: "tmdb".to_string(),
            source_id: "550".to_string(),
            metadata: r#"{"tmdbId":550,"genres":["Drama"]}"#.to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };

        repo.upsert(&item);

        let found = repo.get_by_id("test-1").unwrap();
        assert_eq!(found.title, "Fight Club");
        assert_eq!(found.source_id, "550");

        let by_source = repo.get_by_source("tmdb", "550", "movie").unwrap();
        assert_eq!(by_source.id, "test-1");

        let by_kind = repo.get_by_kind("movie", 10, 0);
        assert_eq!(by_kind.len(), 1);

        assert_eq!(repo.count_by_kind("movie"), 1);
        assert_eq!(repo.count_by_kind("tv_show"), 0);

        let results = repo.search("fight", None, 10, 0);
        assert_eq!(results.len(), 1);

        assert!(repo.delete("test-1"));
        assert!(repo.get_by_id("test-1").is_none());
    }

    #[test]
    fn test_catalog_hierarchy() {
        let db = open_test_database();
        let repo = CatalogRepo::new(db);

        let show = CatalogItemRow {
            id: "show-1".to_string(),
            kind: "tv_show".to_string(),
            title: "Breaking Bad".to_string(),
            sort_title: "breaking bad".to_string(),
            year: Some("2008".to_string()),
            overview: None,
            poster_url: None,
            backdrop_url: None,
            vote_average: Some(9.5),
            vote_count: Some(10000),
            parent_id: None,
            position: None,
            source: "tmdb".to_string(),
            source_id: "1396".to_string(),
            metadata: "{}".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        repo.upsert(&show);

        let season = CatalogItemRow {
            id: "season-1".to_string(),
            kind: "tv_season".to_string(),
            title: "Season 1".to_string(),
            sort_title: "season 1".to_string(),
            year: Some("2008".to_string()),
            overview: None,
            poster_url: None,
            backdrop_url: None,
            vote_average: None,
            vote_count: None,
            parent_id: Some("show-1".to_string()),
            position: Some(1),
            source: "tmdb".to_string(),
            source_id: "3572".to_string(),
            metadata: "{}".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        repo.upsert(&season);

        let children = repo.get_children("show-1");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].title, "Season 1");
        assert_eq!(children[0].position, Some(1));
    }

    #[test]
    fn test_catalog_upsert_conflict() {
        let db = open_test_database();
        let repo = CatalogRepo::new(db);

        let item = CatalogItemRow {
            id: "item-1".to_string(),
            kind: "movie".to_string(),
            title: "Old Title".to_string(),
            sort_title: "old title".to_string(),
            year: None,
            overview: None,
            poster_url: None,
            backdrop_url: None,
            vote_average: None,
            vote_count: None,
            parent_id: None,
            position: None,
            source: "tmdb".to_string(),
            source_id: "100".to_string(),
            metadata: "{}".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        repo.upsert(&item);

        let updated = CatalogItemRow {
            id: "item-2".to_string(),
            title: "New Title".to_string(),
            sort_title: "new title".to_string(),
            ..item
        };
        repo.upsert(&updated);

        // Should update existing row, not create a second
        assert_eq!(repo.count_by_kind("movie"), 1);
        let found = repo.get_by_source("tmdb", "100", "movie").unwrap();
        assert_eq!(found.title, "New Title");
    }
}

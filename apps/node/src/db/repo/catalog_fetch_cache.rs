use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogFetchCacheRow {
    pub id: String,
    pub catalog_item_id: String,
    pub scope: String,
    pub scope_key: String,
    pub candidate_json: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct CatalogFetchCacheRepo {
    db: DbPool,
}

impl CatalogFetchCacheRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(
        &self,
        catalog_item_id: &str,
        scope: &str,
        scope_key: &str,
    ) -> Option<CatalogFetchCacheRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT id, catalog_item_id, scope, scope_key, candidate_json, created_at
             FROM catalog_fetch_cache
             WHERE catalog_item_id = ?1 AND scope = ?2 AND scope_key = ?3",
            params![catalog_item_id, scope, scope_key],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_by_item(&self, catalog_item_id: &str) -> Vec<CatalogFetchCacheRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, catalog_item_id, scope, scope_key, candidate_json, created_at
                 FROM catalog_fetch_cache WHERE catalog_item_id = ?1
                 ORDER BY created_at DESC",
            )
            .unwrap();
        stmt.query_map(params![catalog_item_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn upsert(
        &self,
        catalog_item_id: &str,
        scope: &str,
        scope_key: &str,
        candidate_json: &str,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO catalog_fetch_cache (id, catalog_item_id, scope, scope_key, candidate_json)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4)
             ON CONFLICT(catalog_item_id, scope, scope_key) DO UPDATE SET
                candidate_json = excluded.candidate_json,
                created_at = datetime('now')",
            params![catalog_item_id, scope, scope_key, candidate_json],
        )
        .unwrap();
    }

    pub fn delete_by_item(&self, catalog_item_id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM catalog_fetch_cache WHERE catalog_item_id = ?1",
            params![catalog_item_id],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }

    pub fn get_all_info_hashes(&self) -> Vec<(String, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT catalog_item_id, candidate_json FROM catalog_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| {
            let item_id: String = row.get(0)?;
            let json: String = row.get(1)?;
            Ok((item_id, json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(item_id, json)| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            let hash = v.get("infoHash")?.as_str()?.to_lowercase();
            Some((item_id, hash))
        })
        .collect()
    }

    pub fn get_all_summaries(&self) -> Vec<(String, String, String)> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT catalog_item_id, scope, candidate_json FROM catalog_fetch_cache")
            .unwrap();
        stmt.query_map([], |row| {
            let item_id: String = row.get(0)?;
            let scope: String = row.get(1)?;
            let json: String = row.get(2)?;
            Ok((item_id, scope, json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(item_id, scope, json)| {
            let v: serde_json::Value = serde_json::from_str(&json).ok()?;
            let name = v.get("name")?.as_str()?.to_string();
            Some((item_id, scope, name))
        })
        .collect()
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<CatalogFetchCacheRow> {
        Ok(CatalogFetchCacheRow {
            id: row.get(0)?,
            catalog_item_id: row.get(1)?,
            scope: row.get(2)?,
            scope_key: row.get(3)?,
            candidate_json: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;
    use crate::db::repo::catalog::CatalogItemRow;
    use crate::db::repo::CatalogRepo;

    fn insert_test_item(catalog: &CatalogRepo) {
        catalog.upsert(&CatalogItemRow {
            id: "movie-1".to_string(),
            kind: "movie".to_string(),
            title: "Test Movie".to_string(),
            sort_title: "test movie".to_string(),
            year: None,
            overview: None,
            poster_url: None,
            backdrop_url: None,
            vote_average: None,
            vote_count: None,
            parent_id: None,
            position: None,
            source: "tmdb".to_string(),
            source_id: "550".to_string(),
            metadata: "{}".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        });
    }

    #[test]
    fn test_fetch_cache_crud() {
        let db = open_test_database();
        let catalog = CatalogRepo::new(db.clone());
        let repo = CatalogFetchCacheRepo::new(db);

        insert_test_item(&catalog);

        repo.upsert(
            "movie-1",
            "default",
            "",
            r#"{"name":"Test.Torrent","infoHash":"abc123"}"#,
        );

        let found = repo.get("movie-1", "default", "").unwrap();
        assert_eq!(found.catalog_item_id, "movie-1");
        assert!(found.candidate_json.contains("Test.Torrent"));

        let by_item = repo.get_by_item("movie-1");
        assert_eq!(by_item.len(), 1);

        let hashes = repo.get_all_info_hashes();
        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].1, "abc123");

        assert!(repo.delete_by_item("movie-1"));
        assert!(repo.get_by_item("movie-1").is_empty());
    }

    #[test]
    fn test_fetch_cache_scoped() {
        let db = open_test_database();
        let catalog = CatalogRepo::new(db.clone());
        let repo = CatalogFetchCacheRepo::new(db);

        insert_test_item(&catalog);

        repo.upsert("movie-1", "complete", "", r#"{"name":"Complete"}"#);
        repo.upsert("movie-1", "season", "3", r#"{"name":"Season 3"}"#);
        repo.upsert("movie-1", "episode", "3:5", r#"{"name":"S03E05"}"#);

        let by_item = repo.get_by_item("movie-1");
        assert_eq!(by_item.len(), 3);

        let season = repo.get("movie-1", "season", "3").unwrap();
        assert!(season.candidate_json.contains("Season 3"));

        let episode = repo.get("movie-1", "episode", "3:5").unwrap();
        assert!(episode.candidate_json.contains("S03E05"));
    }
}

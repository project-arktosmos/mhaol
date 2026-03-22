use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentDownloadRow {
    pub info_hash: String,
    pub name: String,
    pub size: i64,
    pub progress: f64,
    pub state: String,
    pub download_speed: i64,
    pub upload_speed: i64,
    pub peers: i64,
    pub seeds: i64,
    pub added_at: i64,
    pub eta: Option<i64>,
    pub output_path: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct TorrentDownloadRepo {
    db: DbPool,
}

impl TorrentDownloadRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, info_hash: &str) -> Option<TorrentDownloadRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT info_hash, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, source, created_at, updated_at FROM torrent_downloads WHERE info_hash = ?1",
            params![info_hash],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<TorrentDownloadRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT info_hash, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, source, created_at, updated_at FROM torrent_downloads ORDER BY added_at DESC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert(
        &self,
        info_hash: &str,
        name: &str,
        size: i64,
        progress: f64,
        state: &str,
        download_speed: i64,
        upload_speed: i64,
        peers: i64,
        seeds: i64,
        added_at: i64,
        eta: Option<i64>,
        output_path: Option<&str>,
        source: &str,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO torrent_downloads (info_hash, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(info_hash) DO UPDATE SET
                name = excluded.name,
                size = excluded.size,
                progress = excluded.progress,
                state = excluded.state,
                download_speed = excluded.download_speed,
                upload_speed = excluded.upload_speed,
                peers = excluded.peers,
                seeds = excluded.seeds,
                eta = excluded.eta,
                output_path = excluded.output_path",
            params![info_hash, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, source],
        )
        .unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_state(
        &self,
        info_hash: &str,
        progress: f64,
        state: &str,
        download_speed: i64,
        upload_speed: i64,
        peers: i64,
        seeds: i64,
        eta: Option<i64>,
        output_path: Option<&str>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE torrent_downloads SET progress = ?2, state = ?3, download_speed = ?4, upload_speed = ?5, peers = ?6, seeds = ?7, eta = ?8, output_path = ?9 WHERE info_hash = ?1",
            params![info_hash, progress, state, download_speed, upload_speed, peers, seeds, eta, output_path],
        )
        .unwrap();
    }

    pub fn delete(&self, info_hash: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM torrent_downloads WHERE info_hash = ?1",
            params![info_hash],
        )
        .unwrap();
    }

    pub fn delete_all(&self) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM torrent_downloads", []).unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<TorrentDownloadRow> {
        Ok(TorrentDownloadRow {
            info_hash: row.get(0)?,
            name: row.get(1)?,
            size: row.get(2)?,
            progress: row.get(3)?,
            state: row.get(4)?,
            download_speed: row.get(5)?,
            upload_speed: row.get(6)?,
            peers: row.get(7)?,
            seeds: row.get(8)?,
            added_at: row.get(9)?,
            eta: row.get(10)?,
            output_path: row.get(11)?,
            source: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn make_repo() -> TorrentDownloadRepo {
        TorrentDownloadRepo::new(open_test_database())
    }

    fn insert_sample(repo: &TorrentDownloadRepo, hash: &str, state: &str) {
        repo.upsert(
            hash, &format!("Torrent {}", hash), 1024000, 0.0, state,
            0, 0, 5, 10, 1000, None, None, "magnet",
        );
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = make_repo();
        insert_sample(&repo, "abc123", "downloading");

        let row = repo.get("abc123").unwrap();
        assert_eq!(row.info_hash, "abc123");
        assert_eq!(row.name, "Torrent abc123");
        assert_eq!(row.state, "downloading");
        assert_eq!(row.size, 1024000);
        assert_eq!(row.source, "magnet");
    }

    #[test]
    fn test_get_not_found() {
        let repo = make_repo();
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_upsert_updates_existing() {
        let repo = make_repo();
        insert_sample(&repo, "abc123", "downloading");
        repo.upsert(
            "abc123", "Updated Name", 2048000, 0.5, "seeding",
            100, 50, 3, 8, 1000, Some(60), Some("/out"), "magnet",
        );

        let row = repo.get("abc123").unwrap();
        assert_eq!(row.name, "Updated Name");
        assert_eq!(row.state, "seeding");
        assert_eq!(row.progress, 0.5);
        assert_eq!(row.output_path, Some("/out".to_string()));
    }

    #[test]
    fn test_get_all() {
        let repo = make_repo();
        insert_sample(&repo, "hash1", "downloading");
        insert_sample(&repo, "hash2", "seeding");

        assert_eq!(repo.get_all().len(), 2);
    }

    #[test]
    fn test_update_state() {
        let repo = make_repo();
        insert_sample(&repo, "hash1", "downloading");

        repo.update_state("hash1", 1.0, "completed", 0, 0, 0, 0, None, Some("/done.mkv"));
        let row = repo.get("hash1").unwrap();
        assert_eq!(row.state, "completed");
        assert_eq!(row.progress, 1.0);
        assert_eq!(row.output_path, Some("/done.mkv".to_string()));
    }

    #[test]
    fn test_delete_and_delete_all() {
        let repo = make_repo();
        insert_sample(&repo, "hash1", "downloading");
        insert_sample(&repo, "hash2", "seeding");

        repo.delete("hash1");
        assert!(repo.get("hash1").is_none());
        assert_eq!(repo.get_all().len(), 1);

        repo.delete_all();
        assert!(repo.get_all().is_empty());
    }
}

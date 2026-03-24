use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRow {
    pub id: String,
    #[serde(rename = "type")]
    pub download_type: String,
    pub name: String,
    pub size: i64,
    pub progress: f64,
    pub state: String,
    pub download_speed: i64,
    pub upload_speed: i64,
    pub peers: i64,
    pub seeds: i64,
    pub added_at: Option<i64>,
    pub eta: Option<i64>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub source: Option<String>,
    pub url: Option<String>,
    pub video_id: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

const SELECT_COLS: &str = "id, type, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, error, source, url, video_id, thumbnail_url, duration_seconds, created_at, updated_at";

#[derive(Clone)]
pub struct DownloadRepo {
    db: DbPool,
}

impl DownloadRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<DownloadRow> {
        let conn = self.db.lock();
        conn.query_row(
            &format!("SELECT {SELECT_COLS} FROM downloads WHERE id = ?1"),
            params![id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<DownloadRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {SELECT_COLS} FROM downloads ORDER BY updated_at DESC"
            ))
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_type(&self, download_type: &str) -> Vec<DownloadRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {SELECT_COLS} FROM downloads WHERE type = ?1 ORDER BY updated_at DESC"
            ))
            .unwrap();
        stmt.query_map(params![download_type], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_torrent(
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
            "INSERT INTO downloads (id, type, name, size, progress, state, download_speed, upload_speed, peers, seeds, added_at, eta, output_path, source)
             VALUES (?1, 'torrent', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
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
    pub fn update_torrent_state(
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
            "UPDATE downloads SET progress = ?2, state = ?3, download_speed = ?4, upload_speed = ?5, peers = ?6, seeds = ?7, eta = ?8, output_path = ?9 WHERE id = ?1",
            params![info_hash, progress, state, download_speed, upload_speed, peers, seeds, eta, output_path],
        )
        .unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_youtube(
        &self,
        download_id: &str,
        download_type: &str,
        name: &str,
        size: i64,
        url: &str,
        video_id: &str,
        state: &str,
        progress: f64,
        output_path: Option<&str>,
        error: Option<&str>,
        thumbnail_url: Option<&str>,
        duration_seconds: Option<i64>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO downloads (id, type, name, size, url, video_id, state, progress, output_path, error, thumbnail_url, duration_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                size = excluded.size,
                state = excluded.state,
                progress = excluded.progress,
                output_path = excluded.output_path,
                error = excluded.error,
                thumbnail_url = excluded.thumbnail_url,
                duration_seconds = excluded.duration_seconds",
            params![download_id, download_type, name, size, url, video_id, state, progress, output_path, error, thumbnail_url, duration_seconds],
        )
        .unwrap();
    }

    pub fn update_youtube_state(
        &self,
        download_id: &str,
        state: &str,
        progress: f64,
        output_path: Option<&str>,
        error: Option<&str>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE downloads SET state = ?2, progress = ?3, output_path = ?4, error = ?5 WHERE id = ?1",
            params![download_id, state, progress, output_path, error],
        )
        .unwrap();
    }

    pub fn delete(&self, id: &str) {
        let conn = self.db.lock();
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])
            .unwrap();
    }

    pub fn delete_all_by_type(&self, download_type: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM downloads WHERE type = ?1",
            params![download_type],
        )
        .unwrap();
    }

    pub fn delete_by_type_and_states(&self, download_type: &str, states: &[&str]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        for state in states {
            tx.execute(
                "DELETE FROM downloads WHERE type = ?1 AND state = ?2",
                params![download_type, state],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    pub fn delete_youtube_by_states(&self, states: &[&str]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        for state in states {
            tx.execute(
                "DELETE FROM downloads WHERE type IN ('youtube-video', 'youtube-audio') AND state = ?1",
                params![state],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadRow> {
        Ok(DownloadRow {
            id: row.get(0)?,
            download_type: row.get(1)?,
            name: row.get(2)?,
            size: row.get(3)?,
            progress: row.get(4)?,
            state: row.get(5)?,
            download_speed: row.get(6)?,
            upload_speed: row.get(7)?,
            peers: row.get(8)?,
            seeds: row.get(9)?,
            added_at: row.get(10)?,
            eta: row.get(11)?,
            output_path: row.get(12)?,
            error: row.get(13)?,
            source: row.get(14)?,
            url: row.get(15)?,
            video_id: row.get(16)?,
            thumbnail_url: row.get(17)?,
            duration_seconds: row.get(18)?,
            created_at: row.get(19)?,
            updated_at: row.get(20)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn make_repo() -> DownloadRepo {
        DownloadRepo::new(open_test_database())
    }

    fn insert_torrent(repo: &DownloadRepo, hash: &str, state: &str) {
        repo.upsert_torrent(
            hash,
            &format!("Torrent {}", hash),
            1024000,
            0.0,
            state,
            0,
            0,
            5,
            10,
            1000,
            None,
            None,
            "magnet",
        );
    }

    fn insert_youtube(repo: &DownloadRepo, id: &str, dl_type: &str, state: &str) {
        repo.upsert_youtube(
            id,
            dl_type,
            "Test Video",
            5000000,
            "https://youtube.com/watch?v=abc",
            "abc",
            state,
            0.0,
            None,
            None,
            None,
            None,
        );
    }

    #[test]
    fn test_upsert_torrent_and_get() {
        let repo = make_repo();
        insert_torrent(&repo, "abc123", "downloading");

        let row = repo.get("abc123").unwrap();
        assert_eq!(row.id, "abc123");
        assert_eq!(row.download_type, "torrent");
        assert_eq!(row.name, "Torrent abc123");
        assert_eq!(row.state, "downloading");
        assert_eq!(row.size, 1024000);
        assert_eq!(row.source, Some("magnet".to_string()));
    }

    #[test]
    fn test_upsert_youtube_and_get() {
        let repo = make_repo();
        insert_youtube(&repo, "dl1", "youtube-video", "pending");

        let row = repo.get("dl1").unwrap();
        assert_eq!(row.id, "dl1");
        assert_eq!(row.download_type, "youtube-video");
        assert_eq!(row.name, "Test Video");
        assert_eq!(row.state, "pending");
        assert_eq!(row.url, Some("https://youtube.com/watch?v=abc".to_string()));
    }

    #[test]
    fn test_get_not_found() {
        let repo = make_repo();
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_upsert_torrent_updates_existing() {
        let repo = make_repo();
        insert_torrent(&repo, "abc123", "downloading");
        repo.upsert_torrent(
            "abc123",
            "Updated Name",
            2048000,
            0.5,
            "seeding",
            100,
            50,
            3,
            8,
            1000,
            Some(60),
            Some("/out"),
            "magnet",
        );

        let row = repo.get("abc123").unwrap();
        assert_eq!(row.name, "Updated Name");
        assert_eq!(row.state, "seeding");
        assert_eq!(row.progress, 0.5);
        assert_eq!(row.output_path, Some("/out".to_string()));
    }

    #[test]
    fn test_upsert_youtube_updates_existing() {
        let repo = make_repo();
        insert_youtube(&repo, "dl1", "youtube-video", "pending");
        repo.upsert_youtube(
            "dl1",
            "youtube-video",
            "Updated Title",
            5000000,
            "https://youtube.com/watch?v=abc",
            "abc",
            "downloading",
            0.5,
            None,
            None,
            Some("https://img.youtube.com/thumb.jpg"),
            Some(300),
        );

        let row = repo.get("dl1").unwrap();
        assert_eq!(row.name, "Updated Title");
        assert_eq!(row.state, "downloading");
        assert_eq!(
            row.thumbnail_url,
            Some("https://img.youtube.com/thumb.jpg".to_string())
        );
        assert_eq!(row.duration_seconds, Some(300));
    }

    #[test]
    fn test_get_all_and_get_by_type() {
        let repo = make_repo();
        insert_torrent(&repo, "hash1", "downloading");
        insert_torrent(&repo, "hash2", "seeding");
        insert_youtube(&repo, "dl1", "youtube-video", "pending");
        insert_youtube(&repo, "dl2", "youtube-audio", "completed");

        assert_eq!(repo.get_all().len(), 4);
        assert_eq!(repo.get_by_type("torrent").len(), 2);
        assert_eq!(repo.get_by_type("youtube-video").len(), 1);
        assert_eq!(repo.get_by_type("youtube-audio").len(), 1);
    }

    #[test]
    fn test_update_torrent_state() {
        let repo = make_repo();
        insert_torrent(&repo, "hash1", "downloading");

        repo.update_torrent_state("hash1", 1.0, "completed", 0, 0, 0, 0, None, Some("/done.mkv"));
        let row = repo.get("hash1").unwrap();
        assert_eq!(row.state, "completed");
        assert_eq!(row.progress, 1.0);
        assert_eq!(row.output_path, Some("/done.mkv".to_string()));
    }

    #[test]
    fn test_update_youtube_state() {
        let repo = make_repo();
        insert_youtube(&repo, "dl1", "youtube-video", "pending");

        repo.update_youtube_state("dl1", "completed", 1.0, Some("/out.mp4"), None);
        let row = repo.get("dl1").unwrap();
        assert_eq!(row.state, "completed");
        assert_eq!(row.progress, 1.0);
        assert_eq!(row.output_path, Some("/out.mp4".to_string()));
    }

    #[test]
    fn test_delete() {
        let repo = make_repo();
        insert_torrent(&repo, "hash1", "downloading");
        insert_youtube(&repo, "dl1", "youtube-video", "pending");

        repo.delete("hash1");
        assert!(repo.get("hash1").is_none());
        assert_eq!(repo.get_all().len(), 1);
    }

    #[test]
    fn test_delete_all_by_type() {
        let repo = make_repo();
        insert_torrent(&repo, "hash1", "downloading");
        insert_torrent(&repo, "hash2", "seeding");
        insert_youtube(&repo, "dl1", "youtube-video", "pending");

        repo.delete_all_by_type("torrent");
        assert_eq!(repo.get_all().len(), 1);
        assert_eq!(repo.get_by_type("torrent").len(), 0);
    }

    #[test]
    fn test_delete_youtube_by_states() {
        let repo = make_repo();
        insert_youtube(&repo, "dl1", "youtube-video", "completed");
        insert_youtube(&repo, "dl2", "youtube-audio", "failed");
        insert_youtube(&repo, "dl3", "youtube-video", "pending");
        insert_torrent(&repo, "hash1", "completed");

        repo.delete_youtube_by_states(&["completed", "failed"]);
        assert_eq!(repo.get_all().len(), 2); // dl3 + hash1
        assert!(repo.get("dl1").is_none());
        assert!(repo.get("dl2").is_none());
        assert!(repo.get("dl3").is_some());
        assert!(repo.get("hash1").is_some());
    }
}

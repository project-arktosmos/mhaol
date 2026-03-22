use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeDownloadRow {
    pub download_id: String,
    pub url: String,
    pub video_id: String,
    pub title: String,
    pub state: String,
    pub progress: f64,
    pub downloaded_bytes: i64,
    pub total_bytes: i64,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub mode: String,
    pub quality: String,
    pub format: String,
    pub video_quality: Option<String>,
    pub video_format: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct YouTubeDownloadRepo {
    db: DbPool,
}

impl YouTubeDownloadRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, download_id: &str) -> Option<YouTubeDownloadRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT download_id, url, video_id, title, state, progress, downloaded_bytes, total_bytes, output_path, error, mode, quality, format, video_quality, video_format, thumbnail_url, duration_seconds, created_at, updated_at FROM youtube_downloads WHERE download_id = ?1",
            params![download_id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<YouTubeDownloadRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT download_id, url, video_id, title, state, progress, downloaded_bytes, total_bytes, output_path, error, mode, quality, format, video_quality, video_format, thumbnail_url, duration_seconds, created_at, updated_at FROM youtube_downloads ORDER BY created_at DESC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_state(&self, state: &str) -> Vec<YouTubeDownloadRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT download_id, url, video_id, title, state, progress, downloaded_bytes, total_bytes, output_path, error, mode, quality, format, video_quality, video_format, thumbnail_url, duration_seconds, created_at, updated_at FROM youtube_downloads WHERE state = ?1 ORDER BY created_at DESC")
            .unwrap();
        stmt.query_map(params![state], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert(
        &self,
        download_id: &str,
        url: &str,
        video_id: &str,
        title: &str,
        state: &str,
        progress: f64,
        downloaded_bytes: i64,
        total_bytes: i64,
        output_path: Option<&str>,
        error: Option<&str>,
        mode: &str,
        quality: &str,
        format: &str,
        video_quality: Option<&str>,
        video_format: Option<&str>,
        thumbnail_url: Option<&str>,
        duration_seconds: Option<i64>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO youtube_downloads (download_id, url, video_id, title, state, progress, downloaded_bytes, total_bytes, output_path, error, mode, quality, format, video_quality, video_format, thumbnail_url, duration_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
             ON CONFLICT(download_id) DO UPDATE SET
                title = excluded.title,
                state = excluded.state,
                progress = excluded.progress,
                downloaded_bytes = excluded.downloaded_bytes,
                total_bytes = excluded.total_bytes,
                output_path = excluded.output_path,
                error = excluded.error,
                video_quality = excluded.video_quality,
                video_format = excluded.video_format,
                thumbnail_url = excluded.thumbnail_url,
                duration_seconds = excluded.duration_seconds",
            params![download_id, url, video_id, title, state, progress, downloaded_bytes, total_bytes, output_path, error, mode, quality, format, video_quality, video_format, thumbnail_url, duration_seconds],
        )
        .unwrap();
    }

    pub fn update_state(
        &self,
        download_id: &str,
        state: &str,
        progress: f64,
        output_path: Option<&str>,
        error: Option<&str>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_downloads SET state = ?2, progress = ?3, output_path = ?4, error = ?5 WHERE download_id = ?1",
            params![download_id, state, progress, output_path, error],
        )
        .unwrap();
    }

    pub fn delete(&self, download_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM youtube_downloads WHERE download_id = ?1",
            params![download_id],
        )
        .unwrap();
    }

    pub fn delete_by_states(&self, states: &[&str]) {
        let conn = self.db.lock();
        let tx = conn.unchecked_transaction().unwrap();
        for state in states {
            tx.execute(
                "DELETE FROM youtube_downloads WHERE state = ?1",
                params![state],
            )
            .unwrap();
        }
        tx.commit().unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<YouTubeDownloadRow> {
        Ok(YouTubeDownloadRow {
            download_id: row.get(0)?,
            url: row.get(1)?,
            video_id: row.get(2)?,
            title: row.get(3)?,
            state: row.get(4)?,
            progress: row.get(5)?,
            downloaded_bytes: row.get(6)?,
            total_bytes: row.get(7)?,
            output_path: row.get(8)?,
            error: row.get(9)?,
            mode: row.get(10)?,
            quality: row.get(11)?,
            format: row.get(12)?,
            video_quality: row.get(13)?,
            video_format: row.get(14)?,
            thumbnail_url: row.get(15)?,
            duration_seconds: row.get(16)?,
            created_at: row.get(17)?,
            updated_at: row.get(18)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn make_repo() -> YouTubeDownloadRepo {
        YouTubeDownloadRepo::new(open_test_database())
    }

    fn insert_sample(repo: &YouTubeDownloadRepo, id: &str, state: &str) {
        repo.upsert(
            id, "https://youtube.com/watch?v=abc", "abc", "Test Video",
            state, 0.0, 0, 1000, None, None,
            "video", "best", "mp4", None, None, None, None,
        );
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = make_repo();
        insert_sample(&repo, "dl1", "pending");

        let row = repo.get("dl1").unwrap();
        assert_eq!(row.download_id, "dl1");
        assert_eq!(row.title, "Test Video");
        assert_eq!(row.state, "pending");
        assert_eq!(row.mode, "video");
        assert_eq!(row.format, "mp4");
    }

    #[test]
    fn test_get_not_found() {
        let repo = make_repo();
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_upsert_updates_existing() {
        let repo = make_repo();
        insert_sample(&repo, "dl1", "pending");
        repo.upsert(
            "dl1", "https://youtube.com/watch?v=abc", "abc", "Updated Title",
            "downloading", 0.5, 500, 1000, None, None,
            "video", "best", "mp4", Some("720p"), None, None, None,
        );

        let row = repo.get("dl1").unwrap();
        assert_eq!(row.title, "Updated Title");
        assert_eq!(row.state, "downloading");
        assert_eq!(row.video_quality, Some("720p".to_string()));
    }

    #[test]
    fn test_get_all_and_get_by_state() {
        let repo = make_repo();
        insert_sample(&repo, "dl1", "pending");
        insert_sample(&repo, "dl2", "downloading");
        insert_sample(&repo, "dl3", "pending");

        assert_eq!(repo.get_all().len(), 3);
        assert_eq!(repo.get_by_state("pending").len(), 2);
        assert_eq!(repo.get_by_state("downloading").len(), 1);
        assert_eq!(repo.get_by_state("completed").len(), 0);
    }

    #[test]
    fn test_update_state() {
        let repo = make_repo();
        insert_sample(&repo, "dl1", "pending");

        repo.update_state("dl1", "completed", 1.0, Some("/out.mp4"), None);
        let row = repo.get("dl1").unwrap();
        assert_eq!(row.state, "completed");
        assert_eq!(row.progress, 1.0);
        assert_eq!(row.output_path, Some("/out.mp4".to_string()));
    }

    #[test]
    fn test_delete_and_delete_by_states() {
        let repo = make_repo();
        insert_sample(&repo, "dl1", "completed");
        insert_sample(&repo, "dl2", "failed");
        insert_sample(&repo, "dl3", "pending");

        repo.delete("dl1");
        assert!(repo.get("dl1").is_none());
        assert_eq!(repo.get_all().len(), 2);

        repo.delete_by_states(&["failed", "pending"]);
        assert!(repo.get_all().is_empty());
    }
}

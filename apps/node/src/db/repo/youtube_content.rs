use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeContentRow {
    pub youtube_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i64>,
    pub channel_name: Option<String>,
    pub channel_id: Option<String>,
    pub video_path: Option<String>,
    pub audio_path: Option<String>,
    pub is_favorite: bool,
    pub favorited_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct YouTubeContentRepo {
    db: DbPool,
}

impl YouTubeContentRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, youtube_id: &str) -> Option<YouTubeContentRow> {
        let conn = self.db.lock();
        conn.query_row(
            "SELECT youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path, is_favorite, favorited_at, created_at, updated_at FROM youtube_content WHERE youtube_id = ?1",
            params![youtube_id],
            Self::row_mapper,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<YouTubeContentRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path, is_favorite, favorited_at, created_at, updated_at FROM youtube_content ORDER BY created_at DESC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn get_by_channel(&self, channel_id: &str) -> Vec<YouTubeContentRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path, is_favorite, favorited_at, created_at, updated_at FROM youtube_content WHERE channel_id = ?1 ORDER BY created_at DESC")
            .unwrap();
        stmt.query_map(params![channel_id], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert(
        &self,
        youtube_id: &str,
        title: &str,
        thumbnail_url: Option<&str>,
        duration_seconds: Option<i64>,
        channel_name: Option<&str>,
        channel_id: Option<&str>,
        video_path: Option<&str>,
        audio_path: Option<&str>,
    ) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO youtube_content (youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(youtube_id) DO UPDATE SET
                title = excluded.title,
                thumbnail_url = COALESCE(excluded.thumbnail_url, youtube_content.thumbnail_url),
                duration_seconds = COALESCE(excluded.duration_seconds, youtube_content.duration_seconds),
                channel_name = COALESCE(excluded.channel_name, youtube_content.channel_name),
                channel_id = COALESCE(excluded.channel_id, youtube_content.channel_id),
                video_path = COALESCE(excluded.video_path, youtube_content.video_path),
                audio_path = COALESCE(excluded.audio_path, youtube_content.audio_path)",
            params![youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path],
        )
        .unwrap();
    }

    pub fn get_ids_missing_duration(&self) -> Vec<String> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT youtube_id FROM youtube_content WHERE duration_seconds IS NULL")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn update_duration(&self, youtube_id: &str, duration_seconds: i64) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET duration_seconds = ?2 WHERE youtube_id = ?1",
            params![youtube_id, duration_seconds],
        )
        .unwrap();
    }

    pub fn update_video_path(&self, youtube_id: &str, path: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET video_path = ?2 WHERE youtube_id = ?1",
            params![youtube_id, path],
        )
        .unwrap();
    }

    pub fn update_audio_path(&self, youtube_id: &str, path: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET audio_path = ?2 WHERE youtube_id = ?1",
            params![youtube_id, path],
        )
        .unwrap();
    }

    pub fn clear_video_path(&self, youtube_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET video_path = NULL WHERE youtube_id = ?1",
            params![youtube_id],
        )
        .unwrap();
    }

    pub fn clear_audio_path(&self, youtube_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET audio_path = NULL WHERE youtube_id = ?1",
            params![youtube_id],
        )
        .unwrap();
    }

    pub fn toggle_favorite(&self, youtube_id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "UPDATE youtube_content SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END, favorited_at = CASE WHEN is_favorite = 0 THEN datetime('now') ELSE NULL END WHERE youtube_id = ?1",
            params![youtube_id],
        )
        .unwrap();
        let is_fav: i32 = conn
            .query_row(
                "SELECT is_favorite FROM youtube_content WHERE youtube_id = ?1",
                params![youtube_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        is_fav != 0
    }

    pub fn get_favorites(&self) -> Vec<YouTubeContentRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare("SELECT youtube_id, title, thumbnail_url, duration_seconds, channel_name, channel_id, video_path, audio_path, is_favorite, favorited_at, created_at, updated_at FROM youtube_content WHERE is_favorite = 1 ORDER BY favorited_at DESC")
            .unwrap();
        stmt.query_map([], Self::row_mapper)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn delete(&self, youtube_id: &str) {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM youtube_content WHERE youtube_id = ?1",
            params![youtube_id],
        )
        .unwrap();
    }

    fn row_mapper(row: &rusqlite::Row<'_>) -> rusqlite::Result<YouTubeContentRow> {
        Ok(YouTubeContentRow {
            youtube_id: row.get(0)?,
            title: row.get(1)?,
            thumbnail_url: row.get(2)?,
            duration_seconds: row.get(3)?,
            channel_name: row.get(4)?,
            channel_id: row.get(5)?,
            video_path: row.get(6)?,
            audio_path: row.get(7)?,
            is_favorite: row.get::<_, i32>(8)? != 0,
            favorited_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn make_repo() -> YouTubeContentRepo {
        YouTubeContentRepo::new(open_test_database())
    }

    #[test]
    fn test_upsert_and_get() {
        let repo = make_repo();
        repo.upsert(
            "vid1",
            "Title 1",
            Some("http://thumb.jpg"),
            Some(120),
            Some("Channel"),
            Some("ch1"),
            None,
            None,
        );

        let row = repo.get("vid1").unwrap();
        assert_eq!(row.title, "Title 1");
        assert_eq!(row.thumbnail_url, Some("http://thumb.jpg".to_string()));
        assert_eq!(row.duration_seconds, Some(120));
        assert_eq!(row.channel_name, Some("Channel".to_string()));
        assert_eq!(row.channel_id, Some("ch1".to_string()));
        assert!(!row.is_favorite);
    }

    #[test]
    fn test_get_not_found() {
        let repo = make_repo();
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_upsert_updates_existing() {
        let repo = make_repo();
        repo.upsert(
            "vid1",
            "Title 1",
            Some("http://thumb.jpg"),
            Some(120),
            None,
            None,
            None,
            None,
        );
        repo.upsert(
            "vid1",
            "Title Updated",
            None,
            None,
            Some("New Channel"),
            None,
            None,
            None,
        );

        let row = repo.get("vid1").unwrap();
        assert_eq!(row.title, "Title Updated");
        // COALESCE keeps old thumbnail when new is NULL
        assert_eq!(row.thumbnail_url, Some("http://thumb.jpg".to_string()));
        assert_eq!(row.channel_name, Some("New Channel".to_string()));
    }

    #[test]
    fn test_get_all_and_get_by_channel() {
        let repo = make_repo();
        repo.upsert("vid1", "A", None, None, None, Some("ch1"), None, None);
        repo.upsert("vid2", "B", None, None, None, Some("ch1"), None, None);
        repo.upsert("vid3", "C", None, None, None, Some("ch2"), None, None);

        assert_eq!(repo.get_all().len(), 3);
        assert_eq!(repo.get_by_channel("ch1").len(), 2);
        assert_eq!(repo.get_by_channel("ch2").len(), 1);
    }

    #[test]
    fn test_update_paths_and_clear() {
        let repo = make_repo();
        repo.upsert("vid1", "Title", None, None, None, None, None, None);

        repo.update_video_path("vid1", "/video.mp4");
        repo.update_audio_path("vid1", "/audio.mp3");
        let row = repo.get("vid1").unwrap();
        assert_eq!(row.video_path, Some("/video.mp4".to_string()));
        assert_eq!(row.audio_path, Some("/audio.mp3".to_string()));

        repo.clear_video_path("vid1");
        repo.clear_audio_path("vid1");
        let row = repo.get("vid1").unwrap();
        assert!(row.video_path.is_none());
        assert!(row.audio_path.is_none());
    }

    #[test]
    fn test_toggle_favorite_and_get_favorites() {
        let repo = make_repo();
        repo.upsert("vid1", "Title", None, None, None, None, None, None);

        assert!(repo.get_favorites().is_empty());

        let is_fav = repo.toggle_favorite("vid1");
        assert!(is_fav);
        assert_eq!(repo.get_favorites().len(), 1);

        let is_fav = repo.toggle_favorite("vid1");
        assert!(!is_fav);
        assert!(repo.get_favorites().is_empty());
    }

    #[test]
    fn test_delete_and_missing_duration() {
        let repo = make_repo();
        repo.upsert("vid1", "A", None, None, None, None, None, None);
        repo.upsert("vid2", "B", None, Some(60), None, None, None, None);

        let missing = repo.get_ids_missing_duration();
        assert_eq!(missing, vec!["vid1".to_string()]);

        repo.update_duration("vid1", 90);
        assert!(repo.get_ids_missing_duration().is_empty());

        repo.delete("vid1");
        assert!(repo.get("vid1").is_none());
        assert_eq!(repo.get_all().len(), 1);
    }
}

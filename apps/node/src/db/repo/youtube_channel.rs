use crate::db::DbPool;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeChannelRow {
    pub id: String,
    pub handle: String,
    pub name: String,
    pub url: String,
    pub subscriber_text: Option<String>,
    pub image_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeChannelUpdate {
    pub name: Option<String>,
    pub subscriber_text: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Clone)]
pub struct YouTubeChannelRepo {
    db: DbPool,
}

const SELECT_COLS: &str =
    "id, handle, name, url, subscriber_text, image_url, created_at, updated_at";

fn row_to_channel(row: &rusqlite::Row) -> rusqlite::Result<YouTubeChannelRow> {
    Ok(YouTubeChannelRow {
        id: row.get(0)?,
        handle: row.get(1)?,
        name: row.get(2)?,
        url: row.get(3)?,
        subscriber_text: row.get(4)?,
        image_url: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

impl YouTubeChannelRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get(&self, id: &str) -> Option<YouTubeChannelRow> {
        let conn = self.db.lock();
        conn.query_row(
            &format!("SELECT {} FROM youtube_channels WHERE id = ?1", SELECT_COLS),
            params![id],
            row_to_channel,
        )
        .ok()
    }

    pub fn get_all(&self) -> Vec<YouTubeChannelRow> {
        let conn = self.db.lock();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM youtube_channels ORDER BY name ASC",
                SELECT_COLS
            ))
            .unwrap();
        stmt.query_map([], row_to_channel)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    pub fn insert(&self, channel: &YouTubeChannelRow) -> bool {
        let conn = self.db.lock();
        conn.execute(
            "INSERT OR IGNORE INTO youtube_channels (id, handle, name, url, subscriber_text, image_url) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                channel.id,
                channel.handle,
                channel.name,
                channel.url,
                channel.subscriber_text,
                channel.image_url,
            ],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }

    pub fn update(&self, id: &str, update: &YouTubeChannelUpdate) -> bool {
        let conn = self.db.lock();
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref name) = update.name {
            sets.push("name = ?");
            values.push(Box::new(name.clone()));
        }
        if let Some(ref subscriber_text) = update.subscriber_text {
            sets.push("subscriber_text = ?");
            values.push(Box::new(subscriber_text.clone()));
        }
        if let Some(ref image_url) = update.image_url {
            sets.push("image_url = ?");
            values.push(Box::new(image_url.clone()));
        }

        if sets.is_empty() {
            return false;
        }

        values.push(Box::new(id.to_string()));
        let sql = format!(
            "UPDATE youtube_channels SET {} WHERE id = ?",
            sets.join(", ")
        );
        let params: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params.as_slice())
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    pub fn delete(&self, id: &str) -> bool {
        let conn = self.db.lock();
        conn.execute("DELETE FROM youtube_channels WHERE id = ?1", params![id])
            .map(|n| n > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_database;

    fn make_repo() -> YouTubeChannelRepo {
        YouTubeChannelRepo::new(open_test_database())
    }

    fn sample_channel(id: &str, name: &str) -> YouTubeChannelRow {
        YouTubeChannelRow {
            id: id.to_string(),
            handle: format!("@{}", name.to_lowercase()),
            name: name.to_string(),
            url: format!("https://youtube.com/channel/{}", id),
            subscriber_text: Some("1M subscribers".to_string()),
            image_url: Some("https://img.com/avatar.jpg".to_string()),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let repo = make_repo();
        let channel = sample_channel("ch1", "TestChannel");

        let inserted = repo.insert(&channel);
        assert!(inserted);

        let row = repo.get("ch1").unwrap();
        assert_eq!(row.name, "TestChannel");
        assert_eq!(row.handle, "@testchannel");
        assert_eq!(row.subscriber_text, Some("1M subscribers".to_string()));
    }

    #[test]
    fn test_get_not_found() {
        let repo = make_repo();
        assert!(repo.get("nonexistent").is_none());
    }

    #[test]
    fn test_insert_duplicate_ignored() {
        let repo = make_repo();
        let channel = sample_channel("ch1", "TestChannel");

        assert!(repo.insert(&channel));
        let second = repo.insert(&channel);
        assert!(!second);
        assert_eq!(repo.get_all().len(), 1);
    }

    #[test]
    fn test_get_all_ordered_by_name() {
        let repo = make_repo();
        repo.insert(&sample_channel("ch2", "Bravo"));
        repo.insert(&sample_channel("ch1", "Alpha"));
        repo.insert(&sample_channel("ch3", "Charlie"));

        let all = repo.get_all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].name, "Alpha");
        assert_eq!(all[1].name, "Bravo");
        assert_eq!(all[2].name, "Charlie");
    }

    #[test]
    fn test_update() {
        let repo = make_repo();
        repo.insert(&sample_channel("ch1", "OldName"));

        let updated = repo.update(
            "ch1",
            &YouTubeChannelUpdate {
                name: Some("NewName".to_string()),
                subscriber_text: Some("2M subscribers".to_string()),
                image_url: None,
            },
        );
        assert!(updated);

        let row = repo.get("ch1").unwrap();
        assert_eq!(row.name, "NewName");
        assert_eq!(row.subscriber_text, Some("2M subscribers".to_string()));
    }

    #[test]
    fn test_update_empty_returns_false() {
        let repo = make_repo();
        repo.insert(&sample_channel("ch1", "Name"));

        let updated = repo.update(
            "ch1",
            &YouTubeChannelUpdate {
                name: None,
                subscriber_text: None,
                image_url: None,
            },
        );
        assert!(!updated);
    }

    #[test]
    fn test_delete() {
        let repo = make_repo();
        repo.insert(&sample_channel("ch1", "Name"));

        assert!(repo.delete("ch1"));
        assert!(repo.get("ch1").is_none());
        assert!(!repo.delete("ch1"));
    }
}

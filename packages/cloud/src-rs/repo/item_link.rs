use crate::types::{CloudItemLinkRow, DbPool};
use rusqlite::params;

#[derive(Clone)]
pub struct CloudItemLinkRepo {
    db: DbPool,
}

impl CloudItemLinkRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_by_item(&self, item_id: &str) -> Vec<CloudItemLinkRow> {
        let conn = self.db.lock();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, service, service_id, extra, created_at FROM cloud_item_links WHERE item_id = ?1 ORDER BY service ASC"
        ).unwrap();
        stmt.query_map(params![item_id], |row| Ok(CloudItemLinkRow {
            id: row.get(0)?, item_id: row.get(1)?, service: row.get(2)?,
            service_id: row.get(3)?, extra: row.get(4)?, created_at: row.get(5)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn upsert(&self, id: &str, item_id: &str, service: &str, service_id: &str, extra: Option<&str>) {
        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO cloud_item_links (id, item_id, service, service_id, extra) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(item_id, service) DO UPDATE SET service_id = ?4, extra = ?5",
            params![id, item_id, service, service_id, extra],
        ).unwrap();
    }
}

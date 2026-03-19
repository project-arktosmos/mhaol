use serde::{Deserialize, Serialize};

pub type DbPool = std::sync::Arc<parking_lot::Mutex<rusqlite::Connection>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudLibraryRow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub scan_status: String,
    pub scan_error: Option<String>,
    pub item_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudItemRow {
    pub id: String,
    pub library_id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct InsertCloudItem {
    pub id: String,
    pub library_id: String,
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: Option<i64>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudItemAttributeRow {
    pub id: String,
    pub item_id: String,
    pub key: String,
    pub value: String,
    pub attribute_type_id: String,
    pub source: String,
    pub confidence: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudItemLinkRow {
    pub id: String,
    pub item_id: String,
    pub service: String,
    pub service_id: String,
    pub extra: Option<String>,
    pub created_at: String,
}

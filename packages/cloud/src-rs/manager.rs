use crate::repo::{CloudItemAttributeRepo, CloudItemLinkRepo, CloudItemRepo, CloudLibraryRepo};
use crate::scanner;
use crate::types::*;
use std::sync::Arc;

/// CloudManager owns all cloud repositories and provides the business logic.
/// This is the single entry point for the backend to interact with cloud functionality.
#[derive(Clone)]
pub struct CloudManager {
    pub libraries: CloudLibraryRepo,
    pub items: CloudItemRepo,
    pub attributes: CloudItemAttributeRepo,
    pub links: CloudItemLinkRepo,
}

impl CloudManager {
    pub fn new(db: DbPool) -> Self {
        Self {
            libraries: CloudLibraryRepo::new(Arc::clone(&db)),
            items: CloudItemRepo::new(Arc::clone(&db)),
            attributes: CloudItemAttributeRepo::new(Arc::clone(&db)),
            links: CloudItemLinkRepo::new(Arc::clone(&db)),
        }
    }

    pub fn list_libraries(&self) -> Vec<CloudLibraryRow> {
        self.libraries.get_all()
    }

    pub fn get_library(&self, id: &str) -> Option<CloudLibraryRow> {
        self.libraries.get(id)
    }

    pub fn create_library(&self, name: &str, path: &str, kind: &str) -> CloudLibraryRow {
        let id = uuid::Uuid::new_v4().to_string();
        self.libraries.insert(&id, name, path, kind);
        self.libraries.get(&id).unwrap()
    }

    pub fn delete_library(&self, id: &str) {
        self.items.delete_by_library(id);
        self.libraries.delete(id);
    }

    pub fn scan_library(&self, id: &str) -> Option<ScanResult> {
        let library = self.libraries.get(id)?;

        self.libraries.update_scan_status(id, "scanning", None);

        let mut scanned_files = Vec::new();
        scanner::scan_dir_recursive(&library.path, id, &mut scanned_files);
        self.items.sync_library(id, &scanned_files);

        let items = self.items.get_by_library(id);
        let count = items.len() as i64;
        self.libraries.update_item_count(id, count);
        self.libraries.update_scan_status(id, "idle", None);

        // Auto-extract basic attributes for new items
        for item in &items {
            let existing = self.attributes.get_by_item(&item.id);
            if existing.is_empty() {
                if let Some(ref mime) = item.mime_type {
                    self.attributes.set(
                        &uuid::Uuid::new_v4().to_string(),
                        &item.id,
                        "mime_type",
                        mime,
                        "string",
                        "system",
                        None,
                    );
                }
                if let Some(size) = item.size_bytes {
                    self.attributes.set(
                        &uuid::Uuid::new_v4().to_string(),
                        &item.id,
                        "size_bytes",
                        &size.to_string(),
                        "bytes",
                        "system",
                        None,
                    );
                }
            }
        }

        Some(ScanResult {
            library_id: id.to_string(),
            library_path: library.path,
            item_count: count,
            items,
        })
    }

    pub fn get_library_items(&self, library_id: &str) -> Vec<CloudItemRow> {
        self.items.get_by_library(library_id)
    }

    pub fn get_item(&self, id: &str) -> Option<CloudItemRow> {
        self.items.get(id)
    }

    pub fn get_item_attributes(&self, item_id: &str) -> Vec<CloudItemAttributeRow> {
        self.attributes.get_by_item(item_id)
    }

    pub fn get_item_links(&self, item_id: &str) -> Vec<CloudItemLinkRow> {
        self.links.get_by_item(item_id)
    }

    pub fn set_attribute(
        &self,
        item_id: &str,
        key: &str,
        value: &str,
        type_id: &str,
        source: &str,
        confidence: Option<f64>,
    ) {
        self.attributes.set(
            &uuid::Uuid::new_v4().to_string(),
            item_id,
            key,
            value,
            type_id,
            source,
            confidence,
        );
    }

    pub fn delete_attribute(&self, item_id: &str, key: &str) {
        self.attributes.delete_by_item_and_key(item_id, key);
    }

    pub fn distinct_keys(&self) -> Vec<String> {
        self.attributes.distinct_keys()
    }

    pub fn distinct_values(&self, key: &str) -> Vec<String> {
        self.attributes.distinct_values(key)
    }

    pub fn search_by_attribute(&self, key: &str, value: &str) -> Vec<CloudItemRow> {
        let attr_rows = self.attributes.get_by_key_and_value(key, value);
        let mut items = Vec::new();
        for attr in &attr_rows {
            if let Some(item) = self.items.get(&attr.item_id) {
                items.push(item);
            }
        }
        items
    }

    pub fn search_by_filename(&self, query: &str) -> Vec<CloudItemRow> {
        self.items.search(query)
    }
}

pub struct ScanResult {
    pub library_id: String,
    pub library_path: String,
    pub item_count: i64,
    pub items: Vec<CloudItemRow>,
}

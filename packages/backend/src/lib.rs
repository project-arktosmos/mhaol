pub mod api;
pub mod db;

use db::repo::*;
use db::DbPool;
use std::path::Path;
use std::sync::Arc;

/// Shared application state available to all API handlers and modules.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub settings: SettingsRepo,
    pub metadata: MetadataRepo,
    pub libraries: LibraryRepo,
    pub library_items: LibraryItemRepo,
    pub library_item_links: LibraryItemLinkRepo,
    pub media_types: MediaTypeRepo,
    pub categories: CategoryRepo,
    pub link_sources: LinkSourceRepo,
    pub youtube_downloads: YouTubeDownloadRepo,
    pub torrent_downloads: TorrentDownloadRepo,
    pub image_tags: ImageTagRepo,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;
        Ok(Self::from_pool(db))
    }

    /// Create AppState from an existing database pool.
    pub fn from_pool(db: DbPool) -> Self {
        Self {
            settings: SettingsRepo::new(Arc::clone(&db)),
            metadata: MetadataRepo::new(Arc::clone(&db)),
            libraries: LibraryRepo::new(Arc::clone(&db)),
            library_items: LibraryItemRepo::new(Arc::clone(&db)),
            library_item_links: LibraryItemLinkRepo::new(Arc::clone(&db)),
            media_types: MediaTypeRepo::new(Arc::clone(&db)),
            categories: CategoryRepo::new(Arc::clone(&db)),
            link_sources: LinkSourceRepo::new(Arc::clone(&db)),
            youtube_downloads: YouTubeDownloadRepo::new(Arc::clone(&db)),
            torrent_downloads: TorrentDownloadRepo::new(Arc::clone(&db)),
            image_tags: ImageTagRepo::new(Arc::clone(&db)),
            db,
        }
    }

    /// Seed a default "Downloads" library if no libraries exist.
    pub fn seed_default_library(&self) {
        if self.libraries.get_all().is_empty() {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let downloads_path = format!("{}/Downloads", home);
            self.libraries.insert(
                &uuid::Uuid::new_v4().to_string(),
                "Downloads",
                &downloads_path,
                "[\"video\",\"image\",\"audio\"]",
                chrono::Utc::now().timestamp_millis(),
            );
            tracing::info!("Created default library at {}", downloads_path);
        }
    }
}

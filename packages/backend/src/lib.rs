pub mod api;
pub mod db;
pub mod identity;
pub mod modules;
pub mod signaling_dev;
pub mod worker_bridge;

use db::repo::*;
use db::DbPool;
use identity::IdentityManager;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;
#[cfg(not(target_os = "android"))]
use modules::image_tagger::ImageTaggerManager;
use modules::ModuleRegistry;
use parking_lot::RwLock;
use signaling_dev::SignalingDevServer;
use worker_bridge::WorkerBridge;
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
    pub identity_manager: IdentityManager,
    pub module_registry: Arc<RwLock<ModuleRegistry>>,
    #[cfg(not(target_os = "android"))]
    pub ytdl_manager: Arc<DownloadManager>,
    #[cfg(not(target_os = "android"))]
    pub torrent_manager: Arc<TorrentManager>,
    #[cfg(not(target_os = "android"))]
    pub image_tagger_manager: Arc<ImageTaggerManager>,
    pub signaling_dev: Arc<SignalingDevServer>,
    pub worker_bridge: Arc<WorkerBridge>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;
        let identities_path = identity::default_identities_path();

        Ok(Self {
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
            identity_manager: IdentityManager::new(identities_path),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            #[cfg(not(target_os = "android"))]
            ytdl_manager: {
                let config = mhaol_yt_dlp::YtDownloadConfig::from_env();
                Arc::new(DownloadManager::new(config))
            },
            #[cfg(not(target_os = "android"))]
            torrent_manager: Arc::new(TorrentManager::new()),
            #[cfg(not(target_os = "android"))]
            image_tagger_manager: Arc::new(ImageTaggerManager::new()),
            signaling_dev: Arc::new(SignalingDevServer::new()),
            worker_bridge: Arc::new(WorkerBridge::new()),
            db,
        })
    }

    /// Register and initialize all built-in modules (addons + core modules).
    pub fn initialize_modules(&self) {
        use modules::{
            lyrics::LyricsModule, musicbrainz::MusicbrainzModule,
            signaling::SignalingModule, signaling_deploy::SignalingDeployModule,
            tmdb::TmdbModule, torrent_search::TorrentSearchModule,
            youtube_meta::YoutubeMetaModule,
        };
        #[cfg(not(target_os = "android"))]
        use modules::{
            image_tagger::ImageTaggerModule, p2p_stream::P2pStreamModule,
            torrent::TorrentModule, ytdl::YtdlModule,
        };

        let mut registry = self.module_registry.write();

        // Addons
        registry.register(Box::new(TmdbModule));
        registry.register(Box::new(MusicbrainzModule));
        registry.register(Box::new(YoutubeMetaModule));
        registry.register(Box::new(LyricsModule));
        registry.register(Box::new(TorrentSearchModule));

        // Signaling modules
        registry.register(Box::new(SignalingModule {
            dev_server: Arc::clone(&self.signaling_dev),
        }));
        registry.register(Box::new(SignalingDeployModule));

        // Core modules (desktop only)
        #[cfg(not(target_os = "android"))]
        {
            registry.register(Box::new(YtdlModule {
                manager: Arc::clone(&self.ytdl_manager),
            }));
            registry.register(Box::new(TorrentModule {
                manager: Arc::clone(&self.torrent_manager),
            }));
            registry.register(Box::new(P2pStreamModule::new()));
            registry.register(Box::new(ImageTaggerModule {
                manager: Arc::clone(&self.image_tagger_manager),
            }));
        }

        // Initialize all registered modules (applies schemas, seeds settings, registers link sources)
        registry.initialize(self);
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

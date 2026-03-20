pub mod api;
pub mod db;
pub mod modules;
pub mod signaling_rooms;
pub mod worker_bridge;

use db::repo::*;
use db::DbPool;
use mhaol_identity::IdentityManager;
#[cfg(not(target_os = "android"))]
use mhaol_llm::LlmEngine;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;
use mhaol_cloud::CloudManager;
use modules::ModuleRegistry;
use parking_lot::RwLock;
use signaling_rooms::SignalingRoomManager;
use worker_bridge::WorkerBridge;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Return the default mhaol data directory: `<Documents>/mhaol`.
/// Creates the directory if it does not exist.
pub fn default_data_dir() -> PathBuf {
    let doc_dir = dirs::document_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Documents")
    });
    let data_dir = doc_dir.join("mhaol");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}

/// Load .env.app from the workspace root into process environment variables.
/// Only sets variables that are not already present in the environment.
pub fn load_env_app() {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env.app");
        }
        if !dir.pop() {
            break PathBuf::from(".env.app");
        }
    };

    let content = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let value = trimmed[eq_idx + 1..].trim();
            if !key.is_empty() && std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}

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
    pub torrent_downloads: TorrentDownloadRepo,
    pub youtube_content: YouTubeContentRepo,
    pub youtube_downloads: YouTubeDownloadRepo,
    pub youtube_channels: YouTubeChannelRepo,
    pub media_lists: MediaListRepo,
    pub media_list_items: MediaListItemRepo,
    pub media_list_links: MediaListLinkRepo,
    pub identity_manager: IdentityManager,
    pub module_registry: Arc<RwLock<ModuleRegistry>>,
    #[cfg(not(target_os = "android"))]
    pub ytdl_manager: Arc<DownloadManager>,
    #[cfg(not(target_os = "android"))]
    pub torrent_manager: Arc<TorrentManager>,
    #[cfg(not(target_os = "android"))]
    pub llm_engine: Arc<LlmEngine>,
    pub data_dir: PathBuf,
    pub llm_conversations: LlmConversationRepo,
    pub signaling_servers: SignalingServerRepo,
    pub signaling_rooms: Arc<SignalingRoomManager>,
    pub worker_bridge: Arc<WorkerBridge>,
    pub cloud: Arc<CloudManager>,
    pub hub: Arc<api::hub::HubManager>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;
        let identities_dir = mhaol_identity::default_identities_dir();
        let identity_manager = IdentityManager::new(identities_dir);

        // One-time migration from old .env.identities format
        let old_env_path = find_env_identities_path();
        if old_env_path.exists() {
            let count = mhaol_identity::migrate_from_env_file(&old_env_path, &identity_manager);
            if count > 0 {
                tracing::info!(
                    "Migrated {} identities from {} to ~/.mhaol-identities/",
                    count,
                    old_env_path.display()
                );
            }
        }

        let data_dir = default_data_dir();

        #[cfg(not(target_os = "android"))]
        let llm_models_dir = {
            let base = db_path
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            base.join("models")
        };

        Ok(Self {
            settings: SettingsRepo::new(Arc::clone(&db)),
            metadata: MetadataRepo::new(Arc::clone(&db)),
            libraries: LibraryRepo::new(Arc::clone(&db)),
            library_items: LibraryItemRepo::new(Arc::clone(&db)),
            library_item_links: LibraryItemLinkRepo::new(Arc::clone(&db)),
            media_types: MediaTypeRepo::new(Arc::clone(&db)),
            categories: CategoryRepo::new(Arc::clone(&db)),
            link_sources: LinkSourceRepo::new(Arc::clone(&db)),
            torrent_downloads: TorrentDownloadRepo::new(Arc::clone(&db)),
            youtube_content: YouTubeContentRepo::new(Arc::clone(&db)),
            youtube_downloads: YouTubeDownloadRepo::new(Arc::clone(&db)),
            youtube_channels: YouTubeChannelRepo::new(Arc::clone(&db)),
            media_lists: MediaListRepo::new(Arc::clone(&db)),
            media_list_items: MediaListItemRepo::new(Arc::clone(&db)),
            media_list_links: MediaListLinkRepo::new(Arc::clone(&db)),
            identity_manager,
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            #[cfg(not(target_os = "android"))]
            ytdl_manager: {
                let config = mhaol_yt_dlp::YtDownloadConfig::from_env();
                Arc::new(DownloadManager::new(config))
            },
            #[cfg(not(target_os = "android"))]
            torrent_manager: Arc::new(TorrentManager::new()),
            #[cfg(not(target_os = "android"))]
            llm_engine: Arc::new(LlmEngine::new(llm_models_dir)),
            llm_conversations: LlmConversationRepo::new(Arc::clone(&db)),
            signaling_servers: SignalingServerRepo::new(Arc::clone(&db)),
            signaling_rooms: Arc::new(SignalingRoomManager::new()),
            worker_bridge: Arc::new(WorkerBridge::new()),
            cloud: Arc::new(CloudManager::new(Arc::clone(&db))),
            hub: Arc::new(api::hub::HubManager::new()),
            data_dir,
            db,
        })
    }

    /// The fixed ID for the single default library.
    pub const DEFAULT_LIBRARY_ID: &'static str = "default";

    /// Register and initialize all built-in modules (addons + core modules).
    pub fn initialize_modules(&self) {
        use modules::{
            jackett::JackettModule, signaling::SignalingModule,
            signaling_deploy::SignalingDeployModule, tmdb::TmdbModule,
            torrent_search::TorrentSearchModule,
            youtube_meta::YoutubeMetaModule,
        };
        #[cfg(not(target_os = "android"))]
        use modules::{
            p2p_stream::P2pStreamModule,
            torrent::TorrentModule,
            ytdl::YtdlModule,
        };

        let mut registry = self.module_registry.write();

        // YouTube
        registry.register(Box::new(YoutubeMetaModule));
        #[cfg(not(target_os = "android"))]
        registry.register(Box::new(YtdlModule {
            manager: Arc::clone(&self.ytdl_manager),
        }));

        // Addons
        registry.register(Box::new(TmdbModule));
        registry.register(Box::new(TorrentSearchModule));
        registry.register(Box::new(JackettModule));

        // Signaling modules
        registry.register(Box::new(SignalingModule {
            rooms: Arc::clone(&self.signaling_rooms),
        }));
        registry.register(Box::new(SignalingDeployModule));

        // Core modules (desktop only)
        #[cfg(not(target_os = "android"))]
        {
            registry.register(Box::new(TorrentModule {
                manager: Arc::clone(&self.torrent_manager),
            }));
            registry.register(Box::new(P2pStreamModule::new()));
        }

        // Initialize all registered modules (applies schemas, seeds settings, registers link sources)
        registry.initialize(self);
    }

    /// Seed a default "Downloads" library if no libraries exist.
    pub fn seed_default_library(&self) {
        if self.libraries.get_all().is_empty() {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let downloads_path = format!("{}/Downloads", home);
            let library_id = uuid::Uuid::new_v4().to_string();
            self.libraries.insert(
                &library_id,
                "Downloads",
                &downloads_path,
                "[\"movies\"]",
                chrono::Utc::now().timestamp_millis(),
            );
            // Point the torrent module at this library so downloads land where scans look
            self.metadata.set_string("torrent.libraryId", &library_id);
            tracing::info!("Created default library at {}", downloads_path);
        }
    }
}

/// Find the legacy .env.identities file by searching up for the repo root.
fn find_env_identities_path() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir.join(".env.identities");
        }
        if !dir.pop() {
            break;
        }
    }
    Path::new(".env.identities").to_path_buf()
}

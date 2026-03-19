pub mod api;
pub mod db;
pub mod identity;
#[cfg(not(target_os = "android"))]
pub mod llm_engine;
pub mod modules;
pub mod signaling_rooms;
pub mod worker_bridge;

use db::repo::*;
use db::DbPool;
use identity::IdentityManager;
#[cfg(not(target_os = "android"))]
use llm_engine::LlmEngine;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
use modules::ModuleRegistry;
use parking_lot::RwLock;
use signaling_rooms::SignalingRoomManager;
use worker_bridge::WorkerBridge;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    pub media_lists: MediaListRepo,
    pub media_list_items: MediaListItemRepo,
    pub media_list_links: MediaListLinkRepo,
    pub identity_manager: IdentityManager,
    pub module_registry: Arc<RwLock<ModuleRegistry>>,
    #[cfg(not(target_os = "android"))]
    pub torrent_manager: Arc<TorrentManager>,
    #[cfg(not(target_os = "android"))]
    pub llm_engine: Arc<LlmEngine>,
    pub llm_conversations: LlmConversationRepo,
    pub signaling_servers: SignalingServerRepo,
    pub signaling_rooms: Arc<SignalingRoomManager>,
    pub worker_bridge: Arc<WorkerBridge>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;
        let identities_path = identity::default_identities_path();

        #[cfg(not(target_os = "android"))]
        let llm_models_dir = {
            let base = db_path
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            base.join("llm").join("models")
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
            media_lists: MediaListRepo::new(Arc::clone(&db)),
            media_list_items: MediaListItemRepo::new(Arc::clone(&db)),
            media_list_links: MediaListLinkRepo::new(Arc::clone(&db)),
            identity_manager: IdentityManager::new(identities_path),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            #[cfg(not(target_os = "android"))]
            torrent_manager: Arc::new(TorrentManager::new()),
            #[cfg(not(target_os = "android"))]
            llm_engine: Arc::new(LlmEngine::new(llm_models_dir)),
            llm_conversations: LlmConversationRepo::new(Arc::clone(&db)),
            signaling_servers: SignalingServerRepo::new(Arc::clone(&db)),
            signaling_rooms: Arc::new(SignalingRoomManager::new()),
            worker_bridge: Arc::new(WorkerBridge::new()),
            db,
        })
    }

    /// Register and initialize all built-in modules (addons + core modules).
    pub fn initialize_modules(&self) {
        use modules::{
            jackett::JackettModule, signaling::SignalingModule,
            signaling_deploy::SignalingDeployModule, tmdb::TmdbModule,
            torrent_search::TorrentSearchModule,
        };
        #[cfg(not(target_os = "android"))]
        use modules::{
            p2p_stream::P2pStreamModule,
            torrent::TorrentModule,
        };

        let mut registry = self.module_registry.write();

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

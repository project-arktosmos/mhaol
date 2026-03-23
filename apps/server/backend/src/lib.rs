pub mod api;
pub mod db;
#[cfg(not(target_os = "android"))]
pub mod http_stream;
#[cfg(not(target_os = "android"))]
pub mod llm_worker;
pub mod modules;
#[cfg(not(target_os = "android"))]
pub mod peer_service;
pub mod signaling_rooms;
pub mod worker_bridge;

use db::repo::*;
use db::DbPool;
use mhaol_identity::IdentityManager;
use mhaol_queue::QueueManager;
#[cfg(not(target_os = "android"))]
use mhaol_llm::LlmEngine;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;
use modules::ModuleRegistry;
use parking_lot::RwLock;
use signaling_rooms::SignalingRoomManager;
use worker_bridge::WorkerBridge;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Return the default mhaol data directory: `<Documents>/mhaol`.
/// Creates the directory if it does not exist.
pub fn default_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DATA_DIR") {
        let p = PathBuf::from(dir);
        std::fs::create_dir_all(&p).ok();
        return p;
    }
    let doc_dir = dirs::document_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Documents")
    });
    let data_dir = doc_dir.join("mhaol");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}

/// Load .env from the workspace root into process environment variables.
/// Only sets variables that are not already present in the environment.
pub fn load_env() {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env");
        }
        if !dir.pop() {
            break PathBuf::from(".env");
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
    pub downloads: DownloadRepo,
    pub youtube_content: YouTubeContentRepo,
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
    pub image_tags: ImageTagRepo,
    #[cfg(not(target_os = "android"))]
    pub image_tagger_manager: Arc<modules::image_tagger::ImageTaggerManager>,
    pub data_dir: PathBuf,
    pub llm_conversations: LlmConversationRepo,
    pub torrent_fetch_cache: TorrentFetchCacheRepo,
    pub tv_torrent_fetch_cache: TvTorrentFetchCacheRepo,
    pub tmdb_api_cache: TmdbApiCacheRepo,
    pub tmdb_image_overrides: TmdbImageOverrideRepo,
    pub roster_contacts: RosterContactRepo,
    pub openlibrary_api_cache: OpenLibraryApiCacheRepo,
    pub book_torrent_fetch_cache: BookTorrentFetchCacheRepo,
    pub signaling_rooms: Arc<SignalingRoomManager>,
    pub worker_bridge: Arc<WorkerBridge>,
    pub hub: Arc<api::hub::HubManager>,
    pub queue: Arc<QueueManager>,
}

impl AppState {
    /// Create a new AppState with a database at the given path (or in-memory if None).
    pub fn new(db_path: Option<&Path>) -> Result<Self, rusqlite::Error> {
        let db = db::open_database(db_path)?;
        let identities_dir = std::env::var("DATA_DIR")
            .ok()
            .map(|d| PathBuf::from(d).join("identities"))
            .unwrap_or_else(mhaol_identity::default_identities_dir);
        let signaling_url = std::env::var("SIGNALING_URL")
            .unwrap_or_else(|_| "https://mhaol-signaling.project-arktosmos.partykit.dev".to_string());
        let identity_manager = IdentityManager::new(identities_dir, "server".to_string(), signaling_url);

        let data_dir = default_data_dir();

        #[cfg(not(target_os = "android"))]
        let llm_models_dir = find_workspace_root().join("models");

        Ok(Self {
            settings: SettingsRepo::new(Arc::clone(&db)),
            metadata: MetadataRepo::new(Arc::clone(&db)),
            libraries: LibraryRepo::new(Arc::clone(&db)),
            library_items: LibraryItemRepo::new(Arc::clone(&db)),
            library_item_links: LibraryItemLinkRepo::new(Arc::clone(&db)),
            media_types: MediaTypeRepo::new(Arc::clone(&db)),
            categories: CategoryRepo::new(Arc::clone(&db)),
            link_sources: LinkSourceRepo::new(Arc::clone(&db)),
            downloads: DownloadRepo::new(Arc::clone(&db)),
            youtube_content: YouTubeContentRepo::new(Arc::clone(&db)),
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
            image_tags: ImageTagRepo::new(Arc::clone(&db)),
            #[cfg(not(target_os = "android"))]
            image_tagger_manager: Arc::new(modules::image_tagger::ImageTaggerManager::new()),
            llm_conversations: LlmConversationRepo::new(Arc::clone(&db)),
            torrent_fetch_cache: TorrentFetchCacheRepo::new(Arc::clone(&db)),
            tv_torrent_fetch_cache: TvTorrentFetchCacheRepo::new(Arc::clone(&db)),
            tmdb_api_cache: TmdbApiCacheRepo::new(Arc::clone(&db)),
            tmdb_image_overrides: TmdbImageOverrideRepo::new(Arc::clone(&db)),
            roster_contacts: RosterContactRepo::new(Arc::clone(&db)),
            openlibrary_api_cache: OpenLibraryApiCacheRepo::new(Arc::clone(&db)),
            book_torrent_fetch_cache: BookTorrentFetchCacheRepo::new(Arc::clone(&db)),
            signaling_rooms: Arc::new(SignalingRoomManager::new()),
            worker_bridge: Arc::new(WorkerBridge::new()),
            hub: Arc::new(api::hub::HubManager::new()),
            queue: Arc::new(QueueManager::new(Arc::clone(&db))),
            data_dir,
            db,
        })
    }

    /// The fixed ID for the single default library.
    pub const DEFAULT_LIBRARY_ID: &'static str = "default";

    /// Register and initialize all built-in modules (addons + core modules).
    pub fn initialize_modules(&self) {
        use modules::{
            lyrics::LyricsModule,
            musicbrainz::MusicbrainzModule, retroachievements::RetroachievementsModule,
            signaling::SignalingModule, tmdb::TmdbModule,
            torrent_search::TorrentSearchModule,
            youtube_meta::YoutubeMetaModule,
        };
        #[cfg(not(target_os = "android"))]
        use modules::{
            image_tagger::ImageTaggerModule,
            p2p_stream::P2pStreamModule,
            torrent::TorrentModule,
            ytdl::YtdlModule,
        };

        let mut registry = self.module_registry.write();
        let is_server = std::env::var("APP_ID").ok().as_deref() == Some("server");

        // Addons (packages/addons/*)
        registry.register(Box::new(LyricsModule));
        registry.register(Box::new(MusicbrainzModule));
        registry.register(Box::new(RetroachievementsModule));
        registry.register(Box::new(TmdbModule));
        registry.register(Box::new(TorrentSearchModule));
        registry.register(Box::new(YoutubeMetaModule));

        // Non-server modules
        if !is_server {
            #[cfg(not(target_os = "android"))]
            registry.register(Box::new(YtdlModule {
                manager: Arc::clone(&self.ytdl_manager),
            }));
            #[cfg(not(target_os = "android"))]
            registry.register(Box::new(ImageTaggerModule {
                manager: Arc::clone(&self.image_tagger_manager),
            }));
        }



        // Signaling modules
        registry.register(Box::new(SignalingModule {
            rooms: Arc::clone(&self.signaling_rooms),
        }));

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

    /// Seed default libraries (one per media category) if no libraries exist.
    pub fn seed_default_libraries(&self) {
        if !self.libraries.get_all().is_empty() {
            return;
        }

        let downloads_dir = self.data_dir.join("downloads");
        let defaults = [
            ("Movies", "movies"),
            ("TV", "tv"),
            ("Music", "music"),
            ("Games", "games"),
            ("YouTube", "youtube"),
        ];
        let now = chrono::Utc::now().timestamp_millis();

        for (name, kind) in &defaults {
            let path = downloads_dir.join(kind);
            std::fs::create_dir_all(&path).ok();
            let library_id = uuid::Uuid::new_v4().to_string();
            let media_types = format!("[\"{}\"]", kind);
            self.libraries.insert(
                &library_id,
                name,
                &path.to_string_lossy(),
                &media_types,
                now,
            );
            // Point the torrent module at the Movies library by default
            if *kind == "movies" {
                self.metadata.set_string("torrent.libraryId", &library_id);
            }
            tracing::info!("Created default {} library at {}", name, path.display());
        }
    }
}

/// Find the monorepo workspace root by checking CARGO_MANIFEST_DIR at compile
/// time, then falling back to walking up from the current working directory.
pub fn find_workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is apps/server/backend — go up three levels to repo root
    let compile_time_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    if let Ok(root) = compile_time_root.canonicalize() {
        if root.join("pnpm-workspace.yaml").exists() {
            return root;
        }
    }

    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

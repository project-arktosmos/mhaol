#[cfg(feature = "embed-frontend")]
pub mod frontend;
pub mod addons;
pub mod database;
pub mod downloads;
pub mod health;
pub mod hub;
pub mod images;
pub mod identities;
pub mod libraries;
pub mod lyrics;

pub mod media;
pub mod media_lists;
pub mod musicbrainz;
pub mod network;
pub mod p2p_stream;
pub mod retroachievements;
pub mod player;
pub mod plugins;
pub mod queue;
pub mod roster;
pub mod signaling;
pub mod signaling_ws;
pub mod openlibrary;
pub mod smart_search;
pub mod tmdb;
pub mod youtube;
pub mod youtube_search;
#[cfg(not(target_os = "android"))]
pub mod ytdl;
#[cfg(not(target_os = "android"))]
pub mod llm;
#[cfg(not(target_os = "android"))]
pub mod torrent;

use crate::AppState;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

/// Build the complete API router with all route groups.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .nest("/api/health", health::router())
        .nest("/api/hub", hub::router())
        .nest("/api/libraries", libraries::router())
        .nest("/api/media", media::router())
        .nest("/api/media-lists", media_lists::router())
        .nest("/api/downloads", downloads::router())
        .nest("/api/database", database::router())
        .nest("/api/p2p-stream", p2p_stream::router())
        .nest("/api/identities", identities::router())
        .nest("/api/plugins", plugins::router())
        .nest("/api/player", player::router())
        .nest("/api/tmdb", tmdb::router())
        .nest("/api/addons", addons::router())
        .nest("/api/network", network::router())
        .nest("/api/roster", roster::router())
        .nest("/api/signaling", signaling::router())
        .nest("/api/images", images::router())
        .nest("/api/lyrics", lyrics::router())
        .nest("/api/musicbrainz", musicbrainz::router())
        .nest("/api/retroachievements", retroachievements::router())
        .nest("/api/youtube", youtube::router())
        .nest("/api/youtube-search", youtube_search::router())
        .nest("/api/openlibrary", openlibrary::router())
        .nest("/api/queue", queue::router())
        .nest("/api/smart-search", smart_search::router())
        .merge(signaling_ws::signaling_routes());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/ytdl", ytdl::router());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/torrent", torrent::router());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/llm", llm::router());

    #[cfg(feature = "embed-frontend")]
    let router = router.fallback(frontend::serve_frontend);

    // When STATIC_DIR is set, serve static files as a fallback (for headless mode).
    // API routes take priority; unmatched paths fall back to static files with SPA index.html.
    #[cfg(not(feature = "embed-frontend"))]
    let router = if let Ok(static_dir) = std::env::var("STATIC_DIR") {
        let index = std::path::PathBuf::from(&static_dir).join("index.html");
        router.fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(index)))
    } else {
        router
    };

    router.with_state(state).layer(cors)
}

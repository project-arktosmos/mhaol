#[cfg(feature = "embed-frontend")]
pub mod frontend;
pub mod addons;
pub mod jackett;
pub mod database;
pub mod downloads;
pub mod identities;
pub mod libraries;
pub mod media;
pub mod media_lists;
pub mod p2p_stream;
pub mod player;
pub mod plugins;
pub mod signaling;
pub mod signaling_ws;
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

/// Build the complete API router with all route groups.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
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
        .nest("/api/jackett", jackett::router())
        .nest("/api/signaling", signaling::router())
        .nest("/api/youtube", youtube::router())
        .nest("/api/youtube-search", youtube_search::router())
        .merge(signaling_ws::signaling_routes());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/ytdl", ytdl::router());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/torrent", torrent::router());

    #[cfg(not(target_os = "android"))]
    let router = router.nest("/api/llm", llm::router());

    #[cfg(feature = "embed-frontend")]
    let router = router.fallback(frontend::serve_frontend);

    router.with_state(state).layer(cors)
}

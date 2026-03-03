pub mod addons;
pub mod database;
pub mod downloads;
pub mod identities;
pub mod images;
pub mod libraries;
pub mod lyrics;
pub mod media;
pub mod musicbrainz;
pub mod p2p_stream;
pub mod player;
pub mod plugins;
pub mod signaling;
pub mod tmdb;
#[cfg(not(target_os = "android"))]
pub mod torrent;
pub mod youtube;
pub mod youtube_search;
#[cfg(not(target_os = "android"))]
pub mod ytdl;

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
        .nest("/api/downloads", downloads::router())
        .nest("/api/database", database::router())
        .nest("/api/p2p-stream", p2p_stream::router())
        .nest("/api/identities", identities::router())
        .nest("/api/plugins", plugins::router())
        .nest("/api/player", player::router())
        .nest("/api/tmdb", tmdb::router())
        .nest("/api/musicbrainz", musicbrainz::router())
        .nest("/api/youtube", youtube::router())
        .nest("/api/youtube-search", youtube_search::router())
        .nest("/api/lyrics", lyrics::router())
        .nest("/api/addons", addons::router())
        .nest("/api/signaling", signaling::router())
        .nest("/api/images", images::router());

    #[cfg(not(target_os = "android"))]
    let router = router
        .nest("/api/ytdl", ytdl::router())
        .nest("/api/torrent", torrent::router());

    router.with_state(state).layer(cors)
}

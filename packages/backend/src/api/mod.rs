pub mod addons;
pub mod database;
pub mod downloads;
pub mod identities;
pub mod libraries;
pub mod lyrics;
pub mod media;
pub mod musicbrainz;
pub mod p2p_stream;
pub mod player;
pub mod plugins;
pub mod tmdb;
pub mod torrent;
pub mod youtube;
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

    Router::new()
        .nest("/api/libraries", libraries::router())
        .nest("/api/media", media::router())
        .nest("/api/downloads", downloads::router())
        .nest("/api/database", database::router())
        .nest("/api/p2p-stream", p2p_stream::router())
        .nest("/api/identities", identities::router())
        .nest("/api/plugins", plugins::router())
        .nest("/api/ytdl", ytdl::router())
        .nest("/api/torrent", torrent::router())
        .nest("/api/player", player::router())
        .nest("/api/tmdb", tmdb::router())
        .nest("/api/musicbrainz", musicbrainz::router())
        .nest("/api/youtube", youtube::router())
        .nest("/api/lyrics", lyrics::router())
        .nest("/api/addons", addons::router())
        .with_state(state)
        .layer(cors)
}

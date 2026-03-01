pub mod database;
pub mod downloads;
pub mod libraries;
pub mod media;
pub mod p2p_stream;

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
        .with_state(state)
        .layer(cors)
}

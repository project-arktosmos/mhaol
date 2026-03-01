pub mod libraries;

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
        .with_state(state)
        .layer(cors)
}

use mhaol_identity::IdentityManager;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Shared application state for the cloud server.
///
/// Backed by SurrealDB (embedded SurrealKV) — independent from the
/// SQLite-backed `mhaol-node` data layer.
#[derive(Clone)]
pub struct CloudState {
    pub db: Surreal<Db>,
    pub identity_manager: IdentityManager,
}

impl CloudState {
    pub fn new(db: Surreal<Db>, identity_manager: IdentityManager) -> Self {
        Self {
            db,
            identity_manager,
        }
    }
}

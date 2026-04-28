use std::path::Path;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

pub const NAMESPACE: &str = "mhaol";
pub const DATABASE: &str = "cloud";

/// Open (or create) the cloud's SurrealDB store at the given path.
///
/// Uses the embedded SurrealKV engine — pure Rust, no external server.
/// The path is treated as a directory: SurrealKV creates and manages
/// its files inside it.
pub async fn open(path: &Path) -> surrealdb::Result<Surreal<Db>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let db = Surreal::new::<SurrealKv>(path.to_string_lossy().to_string()).await?;
    db.use_ns(NAMESPACE).use_db(DATABASE).await?;
    Ok(db)
}

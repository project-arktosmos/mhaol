use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

pub const NAMESPACE: &str = "mhaol";
pub const DATABASE: &str = "cloud";
pub const ENGINE: &str = "rocksdb";

/// Open (or create) the cloud's SurrealDB store at the given path.
///
/// Uses the embedded RocksDB engine. SurrealKV was tried first but
/// hit surrealdb/surrealdb#5064 — concurrent writes corrupt the on-disk
/// format and subsequent reads panic with `Invalid revision N for type Value`.
/// The path is treated as a directory: RocksDB creates and manages its
/// files inside it.
pub async fn open(path: &Path) -> surrealdb::Result<Surreal<Db>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let db = Surreal::new::<RocksDb>(path.to_string_lossy().to_string()).await?;
    db.use_ns(NAMESPACE).use_db(DATABASE).await?;
    Ok(db)
}

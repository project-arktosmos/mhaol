pub mod repo;
pub mod schema;

use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;

/// Shared database handle used across the application.
pub type DbPool = Arc<Mutex<Connection>>;

/// Open (or create) an SQLite database at the given path, or in-memory if `None`.
/// Applies pragmas, schema, migrations, and seed data.
pub fn open_database(path: Option<&Path>) -> Result<DbPool, rusqlite::Error> {
    let conn = match path {
        Some(p) => Connection::open(p)?,
        None => Connection::open_in_memory()?,
    };

    // Pragmas
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA journal_mode = WAL;",
    )?;

    // Schema + migrations + seed
    schema::initialize_schema(&conn)?;
    schema::initialize_module_schemas(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}

#[cfg(test)]
pub fn open_test_database() -> DbPool {
    open_database(None).expect("failed to open test database")
}

use rusqlite::Connection;

pub const CLOUD_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS cloud_attribute_types (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cloud_libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'filesystem',
    scan_status TEXT NOT NULL DEFAULT 'idle'
        CHECK (scan_status IN ('idle', 'scanning', 'error')),
    scan_error TEXT,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS cloud_libraries_updated_at
    AFTER UPDATE ON cloud_libraries FOR EACH ROW
BEGIN
    UPDATE cloud_libraries SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS cloud_items (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES cloud_libraries(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    extension TEXT NOT NULL,
    size_bytes INTEGER,
    mime_type TEXT,
    checksum TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(library_id, path)
);

CREATE INDEX IF NOT EXISTS idx_cloud_items_library_id ON cloud_items(library_id);
CREATE INDEX IF NOT EXISTS idx_cloud_items_extension ON cloud_items(extension);

CREATE TRIGGER IF NOT EXISTS cloud_items_updated_at
    AFTER UPDATE ON cloud_items FOR EACH ROW
BEGIN
    UPDATE cloud_items SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS cloud_item_attributes (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES cloud_items(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    attribute_type_id TEXT NOT NULL REFERENCES cloud_attribute_types(id) DEFAULT 'string',
    source TEXT NOT NULL DEFAULT 'system',
    confidence REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, key, source)
);

CREATE INDEX IF NOT EXISTS idx_cloud_item_attributes_item_id ON cloud_item_attributes(item_id);
CREATE INDEX IF NOT EXISTS idx_cloud_item_attributes_key ON cloud_item_attributes(key);
CREATE INDEX IF NOT EXISTS idx_cloud_item_attributes_key_value ON cloud_item_attributes(key, value);

CREATE TRIGGER IF NOT EXISTS cloud_item_attributes_updated_at
    AFTER UPDATE ON cloud_item_attributes FOR EACH ROW
BEGIN
    UPDATE cloud_item_attributes SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS cloud_item_links (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES cloud_items(id) ON DELETE CASCADE,
    service TEXT NOT NULL,
    service_id TEXT NOT NULL,
    extra TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, service)
);

CREATE INDEX IF NOT EXISTS idx_cloud_item_links_item_id ON cloud_item_links(item_id);

CREATE TABLE IF NOT EXISTS cloud_collections (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES cloud_libraries(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    cover_path TEXT,
    kind TEXT NOT NULL DEFAULT 'manual' CHECK (kind IN ('manual', 'auto', 'smart')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS cloud_collections_updated_at
    AFTER UPDATE ON cloud_collections FOR EACH ROW
BEGIN
    UPDATE cloud_collections SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS cloud_collection_items (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL REFERENCES cloud_collections(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES cloud_items(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(collection_id, item_id)
);
";

pub const CLOUD_SEED_SQL: &str = "
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('string', 'String');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('number', 'Number');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('boolean', 'Boolean');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('json', 'JSON');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('date', 'Date');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('url', 'URL');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('duration', 'Duration');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('bytes', 'Bytes');
INSERT OR IGNORE INTO cloud_attribute_types (id, label) VALUES ('tags', 'Tags');
";

/// Initialize cloud schema and seed data on the given connection.
pub fn initialize_cloud_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(CLOUD_SCHEMA_SQL)?;
    conn.execute_batch(CLOUD_SEED_SQL)?;
    Ok(())
}

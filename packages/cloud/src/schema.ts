import type { Database as DatabaseType } from 'better-sqlite3';

const SCHEMA_SQL = `
CREATE TABLE IF NOT EXISTS cloud_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS cloud_settings_updated_at
    AFTER UPDATE ON cloud_settings
    FOR EACH ROW
BEGIN
    UPDATE cloud_settings SET updated_at = datetime('now') WHERE key = OLD.key;
END;

CREATE TABLE IF NOT EXISTS attribute_types (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS libraries (
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

CREATE TRIGGER IF NOT EXISTS libraries_updated_at
    AFTER UPDATE ON libraries
    FOR EACH ROW
BEGIN
    UPDATE libraries SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
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

CREATE INDEX IF NOT EXISTS idx_items_library_id ON items(library_id);
CREATE INDEX IF NOT EXISTS idx_items_extension ON items(extension);
CREATE INDEX IF NOT EXISTS idx_items_mime_type ON items(mime_type);

CREATE TRIGGER IF NOT EXISTS items_updated_at
    AFTER UPDATE ON items
    FOR EACH ROW
BEGIN
    UPDATE items SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS item_attributes (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    attribute_type_id TEXT NOT NULL REFERENCES attribute_types(id)
        DEFAULT 'string',
    source TEXT NOT NULL DEFAULT 'system',
    confidence REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, key, source)
);

CREATE INDEX IF NOT EXISTS idx_item_attributes_item_id ON item_attributes(item_id);
CREATE INDEX IF NOT EXISTS idx_item_attributes_key ON item_attributes(key);
CREATE INDEX IF NOT EXISTS idx_item_attributes_key_value ON item_attributes(key, value);
CREATE INDEX IF NOT EXISTS idx_item_attributes_source ON item_attributes(source);

CREATE TRIGGER IF NOT EXISTS item_attributes_updated_at
    AFTER UPDATE ON item_attributes
    FOR EACH ROW
BEGIN
    UPDATE item_attributes SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS item_links (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    service TEXT NOT NULL,
    service_id TEXT NOT NULL,
    extra TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(item_id, service)
);

CREATE INDEX IF NOT EXISTS idx_item_links_item_id ON item_links(item_id);
CREATE INDEX IF NOT EXISTS idx_item_links_service ON item_links(service, service_id);

CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    cover_path TEXT,
    kind TEXT NOT NULL DEFAULT 'manual'
        CHECK (kind IN ('manual', 'auto', 'smart')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_collections_library_id ON collections(library_id);

CREATE TRIGGER IF NOT EXISTS collections_updated_at
    AFTER UPDATE ON collections
    FOR EACH ROW
BEGIN
    UPDATE collections SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS collection_items (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(collection_id, item_id)
);

CREATE INDEX IF NOT EXISTS idx_collection_items_collection_id ON collection_items(collection_id);

CREATE TABLE IF NOT EXISTS signaling_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS signaling_servers_updated_at
    AFTER UPDATE ON signaling_servers
    FOR EACH ROW
BEGIN
    UPDATE signaling_servers SET updated_at = datetime('now') WHERE id = OLD.id;
END;
`;

const SEED_SQL = `
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('string', 'String');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('number', 'Number');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('boolean', 'Boolean');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('json', 'JSON');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('date', 'Date');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('url', 'URL');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('duration', 'Duration');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('bytes', 'Bytes');
INSERT OR IGNORE INTO attribute_types (id, label) VALUES ('tags', 'Tags');
`;

export function initializeCloudSchema(db: DatabaseType): void {
	db.exec(SCHEMA_SQL);
	db.exec(SEED_SQL);
}

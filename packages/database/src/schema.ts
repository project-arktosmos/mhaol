import type { Database as DatabaseType } from 'better-sqlite3';

const SCHEMA_SQL = `
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS settings_updated_at
    AFTER UPDATE ON settings
    FOR EACH ROW
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE key = OLD.key;
END;

CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'string' CHECK (type IN ('string', 'number', 'boolean', 'json')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS metadata_updated_at
    AFTER UPDATE ON metadata
    FOR EACH ROW
BEGIN
    UPDATE metadata SET updated_at = datetime('now') WHERE key = OLD.key;
END;

CREATE TABLE IF NOT EXISTS youtube_downloads (
    download_id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    video_id TEXT NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'pending',
    progress REAL NOT NULL DEFAULT 0,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    output_path TEXT,
    error TEXT,
    mode TEXT NOT NULL DEFAULT 'audio',
    quality TEXT NOT NULL DEFAULT 'high',
    format TEXT NOT NULL DEFAULT 'aac',
    video_quality TEXT,
    video_format TEXT,
    thumbnail_url TEXT,
    duration_seconds REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS youtube_downloads_updated_at
    AFTER UPDATE ON youtube_downloads
    FOR EACH ROW
BEGIN
    UPDATE youtube_downloads SET updated_at = datetime('now') WHERE download_id = OLD.download_id;
END;

CREATE TABLE IF NOT EXISTS torrent_downloads (
    info_hash TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    progress REAL NOT NULL DEFAULT 0,
    state TEXT NOT NULL DEFAULT 'initializing',
    download_speed INTEGER NOT NULL DEFAULT 0,
    upload_speed INTEGER NOT NULL DEFAULT 0,
    peers INTEGER NOT NULL DEFAULT 0,
    seeds INTEGER NOT NULL DEFAULT 0,
    added_at INTEGER NOT NULL,
    eta INTEGER,
    output_path TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS torrent_downloads_updated_at
    AFTER UPDATE ON torrent_downloads
    FOR EACH ROW
BEGIN
    UPDATE torrent_downloads SET updated_at = datetime('now') WHERE info_hash = OLD.info_hash;
END;

CREATE TABLE IF NOT EXISTS libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    media_types TEXT NOT NULL DEFAULT '[]',
    date_added INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS libraries_updated_at
    AFTER UPDATE ON libraries
    FOR EACH ROW
BEGIN
    UPDATE libraries SET updated_at = datetime('now') WHERE id = OLD.id;
END;
`;

const SEED_SQL = `
INSERT OR REPLACE INTO metadata (key, value, type) VALUES ('db_version', '3', 'number');
INSERT OR IGNORE INTO metadata (key, value, type) VALUES ('created_at', datetime('now'), 'string');
`;

export function initializeSchema(db: DatabaseType): void {
	db.exec(SCHEMA_SQL);
	db.exec(SEED_SQL);
}

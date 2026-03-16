import type { Database as DatabaseType } from "better-sqlite3";

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

CREATE TABLE IF NOT EXISTS media_types (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    media_type_id TEXT NOT NULL REFERENCES media_types(id),
    label TEXT NOT NULL
);

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

CREATE TABLE IF NOT EXISTS library_items (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    path TEXT NOT NULL UNIQUE,
    extension TEXT NOT NULL,
    media_type TEXT NOT NULL REFERENCES media_types(id),
    category_id TEXT REFERENCES categories(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS library_items_updated_at
    AFTER UPDATE ON library_items
    FOR EACH ROW
BEGIN
    UPDATE library_items SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS library_item_links (
    id TEXT PRIMARY KEY,
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    service TEXT NOT NULL,
    service_id TEXT NOT NULL,
    season_number INTEGER,
    episode_number INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(library_item_id, service)
);

CREATE TABLE IF NOT EXISTS link_sources (
    id TEXT PRIMARY KEY,
    plugin TEXT NOT NULL,
    service TEXT NOT NULL,
    label TEXT NOT NULL,
    media_type_id TEXT NOT NULL REFERENCES media_types(id),
    category_id TEXT REFERENCES categories(id),
    UNIQUE(service, media_type_id, category_id)
);

CREATE TABLE IF NOT EXISTS media_lists (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    cover_image TEXT,
    media_type TEXT NOT NULL REFERENCES media_types(id),
    source TEXT NOT NULL DEFAULT 'auto' CHECK (source IN ('auto', 'user')),
    source_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS media_lists_updated_at
    AFTER UPDATE ON media_lists
    FOR EACH ROW
BEGIN
    UPDATE media_lists SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TABLE IF NOT EXISTS media_list_items (
    id TEXT PRIMARY KEY,
    list_id TEXT NOT NULL REFERENCES media_lists(id) ON DELETE CASCADE,
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(list_id, library_item_id)
);

CREATE INDEX IF NOT EXISTS idx_media_list_items_list_id ON media_list_items(list_id);
CREATE INDEX IF NOT EXISTS idx_media_lists_source_path ON media_lists(source_path);

CREATE TABLE IF NOT EXISTS media_list_links (
    id TEXT PRIMARY KEY,
    list_id TEXT NOT NULL REFERENCES media_lists(id) ON DELETE CASCADE,
    service TEXT NOT NULL,
    service_id TEXT NOT NULL,
    season_number INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(list_id, service)
);

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
INSERT OR REPLACE INTO metadata (key, value, type) VALUES ('db_version', '19', 'number');
INSERT OR IGNORE INTO metadata (key, value, type) VALUES ('created_at', datetime('now'), 'string');

INSERT OR IGNORE INTO media_types (id, label) VALUES ('video', 'Video');
INSERT OR IGNORE INTO media_types (id, label) VALUES ('image', 'Image');
INSERT OR IGNORE INTO media_types (id, label) VALUES ('audio', 'Audio');

INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('tv', 'video', 'TV');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('movies', 'video', 'Movies');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('video-uncategorized', 'video', 'Uncategorized');

INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('music', 'audio', 'Music');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('podcast', 'audio', 'Podcast');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('audio-uncategorized', 'audio', 'Uncategorized');

INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('photos', 'image', 'Photos');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('memes', 'image', 'Memes');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('image-uncategorized', 'image', 'Uncategorized');
`;

function runMigrations(db: DatabaseType): void {
  const hasColumn = (table: string, column: string): boolean => {
    const columns = db.pragma(`table_info(${table})`) as Array<{ name: string }>;
    return columns.some((c) => c.name === column);
  };

  const hasTable = (name: string): boolean => {
    const row = db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?").get(name) as { name: string } | undefined;
    return !!row;
  };

  // Migration: add youtube_id to library_items (added in db_version 7)
  if (!hasColumn('library_items', 'youtube_id')) {
    db.exec('ALTER TABLE library_items ADD COLUMN youtube_id TEXT');
  }

  // Migration: add musicbrainz_id to library_items (added in db_version 9)
  if (!hasColumn('library_items', 'musicbrainz_id')) {
    db.exec('ALTER TABLE library_items ADD COLUMN musicbrainz_id TEXT');
  }

  // Migration: add media_types and categories reference tables (db_version 12)
  if (!hasTable('media_types')) {
    db.exec(`
      CREATE TABLE media_types (
        id TEXT PRIMARY KEY,
        label TEXT NOT NULL
      );
      CREATE TABLE categories (
        id TEXT PRIMARY KEY,
        media_type_id TEXT NOT NULL REFERENCES media_types(id),
        label TEXT NOT NULL
      );
    `);
  }

  // Migration: rename 'images' → 'image' and 'music' → 'audio' in library_items
  db.exec("UPDATE library_items SET media_type = 'image' WHERE media_type = 'images'");
  db.exec("UPDATE library_items SET media_type = 'audio' WHERE media_type = 'music'");

  // Migration: rename in libraries.media_types JSON arrays
  db.exec("UPDATE libraries SET media_types = REPLACE(REPLACE(media_types, '\"images\"', '\"image\"'), '\"music\"', '\"audio\"')");

  // Migration: add category_id to library_items (db_version 12)
  if (!hasColumn('library_items', 'category_id')) {
    db.exec('ALTER TABLE library_items ADD COLUMN category_id TEXT REFERENCES categories(id)');
    db.exec("UPDATE library_items SET category_id = 'video-uncategorized' WHERE media_type = 'video' AND category_id IS NULL");
  }

  // Migration: rename 'uncategorized' category to 'video-uncategorized' (db_version 13)
  const oldCat = db.prepare("SELECT id FROM categories WHERE id = 'uncategorized'").get() as { id: string } | undefined;
  if (oldCat) {
    db.exec("UPDATE library_items SET category_id = 'video-uncategorized' WHERE category_id = 'uncategorized'");
    db.exec("DELETE FROM categories WHERE id = 'uncategorized'");
  }

  // Migration: extract external service links into library_item_links table (db_version 14)
  if (!hasTable('library_item_links')) {
    db.exec(`
      CREATE TABLE library_item_links (
        id TEXT PRIMARY KEY,
        library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
        service TEXT NOT NULL,
        service_id TEXT NOT NULL,
        season_number INTEGER,
        episode_number INTEGER,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(library_item_id, service)
      );
    `);
  }

  // Migration: add link_sources table (db_version 15)
  if (!hasTable('link_sources')) {
    db.exec(`
      CREATE TABLE link_sources (
        id TEXT PRIMARY KEY,
        plugin TEXT NOT NULL,
        service TEXT NOT NULL,
        label TEXT NOT NULL,
        media_type_id TEXT NOT NULL REFERENCES media_types(id),
        category_id TEXT REFERENCES categories(id),
        UNIQUE(service, media_type_id, category_id)
      );
    `);
  }

  // Migration: add media_lists and media_list_items tables (db_version 16)
  if (!hasTable('media_lists')) {
    db.exec(`
      CREATE TABLE media_lists (
        id TEXT PRIMARY KEY,
        library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        description TEXT,
        cover_image TEXT,
        media_type TEXT NOT NULL REFERENCES media_types(id),
        source TEXT NOT NULL DEFAULT 'auto' CHECK (source IN ('auto', 'user')),
        source_path TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
      );
      CREATE TRIGGER IF NOT EXISTS media_lists_updated_at
        AFTER UPDATE ON media_lists FOR EACH ROW
      BEGIN UPDATE media_lists SET updated_at = datetime('now') WHERE id = OLD.id; END;
      CREATE TABLE media_list_items (
        id TEXT PRIMARY KEY,
        list_id TEXT NOT NULL REFERENCES media_lists(id) ON DELETE CASCADE,
        library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
        position INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(list_id, library_item_id)
      );
      CREATE INDEX IF NOT EXISTS idx_media_list_items_list_id ON media_list_items(list_id);
      CREATE INDEX IF NOT EXISTS idx_media_lists_source_path ON media_lists(source_path);
    `);
  }

  // Migration: add media_list_links table (db_version 17)
  if (!hasTable('media_list_links')) {
    db.exec(`
      CREATE TABLE media_list_links (
        id TEXT PRIMARY KEY,
        list_id TEXT NOT NULL REFERENCES media_lists(id) ON DELETE CASCADE,
        service TEXT NOT NULL,
        service_id TEXT NOT NULL,
        season_number INTEGER,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        UNIQUE(list_id, service)
      );
    `);
  }

  // Migration: add season_number to media_list_links (db_version 18)
  if (hasTable('media_list_links') && !hasColumn('media_list_links', 'season_number')) {
    db.exec('ALTER TABLE media_list_links ADD COLUMN season_number INTEGER');
  }

  // Migration: add signaling_servers table (db_version 19)
  if (!hasTable('signaling_servers')) {
    db.exec(`
      CREATE TABLE signaling_servers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        url TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
      );
      CREATE TRIGGER IF NOT EXISTS signaling_servers_updated_at
        AFTER UPDATE ON signaling_servers FOR EACH ROW
      BEGIN UPDATE signaling_servers SET updated_at = datetime('now') WHERE id = OLD.id; END;
    `);
  }

  // Migrate existing signaling.partyUrl setting into signaling_servers table
  if (hasTable('signaling_servers')) {
    const existingUrl = db.prepare("SELECT value FROM settings WHERE key = 'signaling.partyUrl'").get() as { value: string } | undefined;
    if (existingUrl && existingUrl.value) {
      const existingName = db.prepare("SELECT value FROM settings WHERE key = 'signaling.deployName'").get() as { value: string } | undefined;
      const name = existingName?.value || 'PartyKit Server';
      db.exec(`
        INSERT OR IGNORE INTO signaling_servers (id, name, url)
        SELECT lower(hex(randomblob(16))), '${name.replace(/'/g, "''")}', '${existingUrl.value.replace(/'/g, "''")}'
        WHERE NOT EXISTS (SELECT 1 FROM signaling_servers WHERE url = '${existingUrl.value.replace(/'/g, "''")}');
      `);
      db.exec("DELETE FROM settings WHERE key IN ('signaling.partyUrl', 'signaling.deployName')");
    }
  }

  if (hasColumn('library_items', 'tmdb_id')) {
    // Migrate tmdb links
    db.exec(`
      INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id, season_number, episode_number)
      SELECT lower(hex(randomblob(16))), id, 'tmdb', CAST(tmdb_id AS TEXT), tmdb_season_number, tmdb_episode_number
      FROM library_items WHERE tmdb_id IS NOT NULL;
    `);

    // Migrate youtube links
    db.exec(`
      INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id)
      SELECT lower(hex(randomblob(16))), id, 'youtube', youtube_id
      FROM library_items WHERE youtube_id IS NOT NULL;
    `);

    // Migrate musicbrainz links
    db.exec(`
      INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id)
      SELECT lower(hex(randomblob(16))), id, 'musicbrainz', musicbrainz_id
      FROM library_items WHERE musicbrainz_id IS NOT NULL;
    `);

    // Rebuild library_items without the old columns
    db.exec(`
      CREATE TABLE library_items_new (
        id TEXT PRIMARY KEY,
        library_id TEXT NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
        path TEXT NOT NULL UNIQUE,
        extension TEXT NOT NULL,
        media_type TEXT NOT NULL REFERENCES media_types(id),
        category_id TEXT REFERENCES categories(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
      );

      INSERT INTO library_items_new (id, library_id, path, extension, media_type, category_id, created_at, updated_at)
      SELECT id, library_id, path, extension, media_type, category_id, created_at, updated_at
      FROM library_items;

      DROP TABLE library_items;
      ALTER TABLE library_items_new RENAME TO library_items;

      CREATE TRIGGER IF NOT EXISTS library_items_updated_at
        AFTER UPDATE ON library_items
        FOR EACH ROW
      BEGIN
        UPDATE library_items SET updated_at = datetime('now') WHERE id = OLD.id;
      END;
    `);
  }
}

export function initializeSchema(db: DatabaseType): void {
  db.exec(SCHEMA_SQL);
  runMigrations(db);
  db.exec(SEED_SQL);
}

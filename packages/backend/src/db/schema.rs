use rusqlite::Connection;

/// Core tables needed by every app.
const CORE_SCHEMA_SQL: &str = "
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

CREATE TABLE IF NOT EXISTS torrent_downloads (
    info_hash TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    size INTEGER NOT NULL,
    progress REAL NOT NULL,
    state TEXT NOT NULL,
    download_speed INTEGER NOT NULL,
    upload_speed INTEGER NOT NULL,
    peers INTEGER NOT NULL,
    seeds INTEGER NOT NULL,
    added_at INTEGER NOT NULL,
    eta INTEGER,
    output_path TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS llm_conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    system_prompt TEXT,
    messages TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS llm_conversations_updated_at
    AFTER UPDATE ON llm_conversations
    FOR EACH ROW
BEGIN
    UPDATE llm_conversations SET updated_at = datetime('now') WHERE id = OLD.id;
END;
";

/// Media lists and signaling servers (video-cloud, tunes apps).
const MEDIA_LISTS_SQL: &str = "
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
";

/// YouTube tables (tube app).
const YOUTUBE_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS youtube_content (
    youtube_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    thumbnail_url TEXT,
    duration_seconds INTEGER,
    channel_name TEXT,
    channel_id TEXT,
    video_path TEXT,
    audio_path TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    favorited_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS youtube_content_updated_at
    AFTER UPDATE ON youtube_content
    FOR EACH ROW
BEGIN
    UPDATE youtube_content SET updated_at = datetime('now') WHERE youtube_id = OLD.youtube_id;
END;

CREATE TABLE IF NOT EXISTS youtube_downloads (
    download_id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    video_id TEXT NOT NULL,
    title TEXT NOT NULL,
    state TEXT NOT NULL,
    progress REAL NOT NULL,
    downloaded_bytes INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    output_path TEXT,
    error TEXT,
    mode TEXT NOT NULL,
    quality TEXT NOT NULL,
    format TEXT NOT NULL,
    video_quality TEXT,
    video_format TEXT,
    thumbnail_url TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS youtube_channels (
    id TEXT PRIMARY KEY,
    handle TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    subscriber_text TEXT,
    image_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS youtube_channels_updated_at
    AFTER UPDATE ON youtube_channels
    FOR EACH ROW
BEGIN
    UPDATE youtube_channels SET updated_at = datetime('now') WHERE id = OLD.id;
END;
";

/// Image tagging tables (photos app).
const IMAGE_TAGS_SQL: &str = "
CREATE TABLE IF NOT EXISTS image_tags (
    id TEXT PRIMARY KEY,
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    score REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_image_tags_library_item_id ON image_tags(library_item_id);
CREATE INDEX IF NOT EXISTS idx_image_tags_tag ON image_tags(tag);
";

const SEED_SQL: &str = "
INSERT OR REPLACE INTO metadata (key, value, type) VALUES ('db_version', '21', 'number');
INSERT OR IGNORE INTO metadata (key, value, type) VALUES ('created_at', datetime('now'), 'string');

INSERT OR IGNORE INTO media_types (id, label) VALUES ('video', 'Video');
INSERT OR IGNORE INTO media_types (id, label) VALUES ('audio', 'Audio');
INSERT OR IGNORE INTO media_types (id, label) VALUES ('image', 'Image');

INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('tv', 'video', 'TV');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('movies', 'video', 'Movies');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('video-uncategorized', 'video', 'Uncategorized');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('audio-uncategorized', 'audio', 'Uncategorized');
INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('image-uncategorized', 'image', 'Uncategorized');
";

/// YouTube video metadata cache (module schema).
pub const YOUTUBE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS youtube_videos (
    video_id TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const MUSICBRAINZ_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS musicbrainz_artists (
    mbid TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS musicbrainz_release_groups (
    mbid TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS musicbrainz_releases (
    mbid TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS musicbrainz_recordings (
    mbid TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS musicbrainz_popular_cache (
    genre TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const LYRICS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS lrclib_lyrics (
    lrclib_id INTEGER PRIMARY KEY,
    track_name TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    album_name TEXT NOT NULL DEFAULT '',
    duration REAL NOT NULL DEFAULT 0,
    instrumental INTEGER NOT NULL DEFAULT 0,
    plain_lyrics TEXT,
    synced_lyrics TEXT,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS lrclib_lookups (
    library_item_id TEXT PRIMARY KEY,
    lrclib_id INTEGER REFERENCES lrclib_lyrics(lrclib_id),
    status TEXT NOT NULL CHECK (status IN ('found', 'not_found')),
    looked_up_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

/// Module SQL schemas for addon tables
pub const TMDB_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS tmdb_movies (
    tmdb_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tmdb_tv_shows (
    tmdb_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tmdb_seasons (
    tmdb_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tmdb_id, season_number)
);

CREATE TABLE IF NOT EXISTS tmdb_api_cache (
    cache_key TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";


fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql).unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    columns.iter().any(|c| c == column)
}

fn has_table(conn: &Connection, name: &str) -> bool {
    conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")
        .unwrap()
        .exists(rusqlite::params![name])
        .unwrap_or(false)
}

fn run_migrations(conn: &Connection) {
    // Migration: add youtube_id to library_items (db_version 7)
    if !has_column(conn, "library_items", "youtube_id") && has_table(conn, "library_items") {
        let _ = conn.execute_batch("ALTER TABLE library_items ADD COLUMN youtube_id TEXT");
    }

    // Migration: add musicbrainz_id to library_items (db_version 9)
    if !has_column(conn, "library_items", "musicbrainz_id") && has_table(conn, "library_items") {
        let _ = conn.execute_batch("ALTER TABLE library_items ADD COLUMN musicbrainz_id TEXT");
    }

    // Migration: rename 'images' -> 'image' and 'music' -> 'audio'
    if has_table(conn, "library_items") {
        let _ = conn.execute_batch(
            "UPDATE library_items SET media_type = 'image' WHERE media_type = 'images';
             UPDATE library_items SET media_type = 'audio' WHERE media_type = 'music';",
        );
    }

    // Migration: rename in libraries.media_types JSON arrays
    if has_table(conn, "libraries") {
        let _ = conn.execute_batch(
            "UPDATE libraries SET media_types = REPLACE(REPLACE(media_types, '\"images\"', '\"image\"'), '\"music\"', '\"audio\"')",
        );
    }

    // Migration: add category_id to library_items (db_version 12)
    if has_table(conn, "library_items") && !has_column(conn, "library_items", "category_id") {
        let _ = conn.execute_batch(
            "ALTER TABLE library_items ADD COLUMN category_id TEXT REFERENCES categories(id);
             UPDATE library_items SET category_id = 'video-uncategorized' WHERE media_type = 'video' AND category_id IS NULL;",
        );
    }

    // Migration: rename 'uncategorized' category to 'video-uncategorized' (db_version 13)
    if has_table(conn, "categories") {
        let has_old: bool = conn
            .prepare("SELECT id FROM categories WHERE id = 'uncategorized'")
            .and_then(|mut s| s.exists([]))
            .unwrap_or(false);
        if has_old {
            let _ = conn.execute_batch(
                "UPDATE library_items SET category_id = 'video-uncategorized' WHERE category_id = 'uncategorized';
                 DELETE FROM categories WHERE id = 'uncategorized';",
            );
        }
    }

    // Migration: extract external service links into library_item_links (db_version 14)
    if !has_table(conn, "library_item_links") {
        let _ = conn.execute_batch(
            "CREATE TABLE library_item_links (
                id TEXT PRIMARY KEY,
                library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
                service TEXT NOT NULL,
                service_id TEXT NOT NULL,
                season_number INTEGER,
                episode_number INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(library_item_id, service)
            );",
        );
    }

    // Migration: add link_sources table (db_version 15)
    if !has_table(conn, "link_sources") {
        let _ = conn.execute_batch(
            "CREATE TABLE link_sources (
                id TEXT PRIMARY KEY,
                plugin TEXT NOT NULL,
                service TEXT NOT NULL,
                label TEXT NOT NULL,
                media_type_id TEXT NOT NULL REFERENCES media_types(id),
                category_id TEXT REFERENCES categories(id),
                UNIQUE(service, media_type_id, category_id)
            );",
        );
    }

    // Migration: add media_lists and media_list_items tables (db_version 16)
    if !has_table(conn, "media_lists") {
        let _ = conn.execute_batch(
            "CREATE TABLE media_lists (
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
            CREATE INDEX IF NOT EXISTS idx_media_lists_source_path ON media_lists(source_path);",
        );
    }

    // Migration: add media_list_links table (db_version 17)
    if !has_table(conn, "media_list_links") {
        let _ = conn.execute_batch(
            "CREATE TABLE media_list_links (
                id TEXT PRIMARY KEY,
                list_id TEXT NOT NULL REFERENCES media_lists(id) ON DELETE CASCADE,
                service TEXT NOT NULL,
                service_id TEXT NOT NULL,
                season_number INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(list_id, service)
            );",
        );
    }

    // Migration: add season_number to media_list_links (db_version 18)
    if has_table(conn, "media_list_links") && !has_column(conn, "media_list_links", "season_number")
    {
        let _ =
            conn.execute_batch("ALTER TABLE media_list_links ADD COLUMN season_number INTEGER");
    }

    // Migration: add signaling_servers table (db_version 19)
    if !has_table(conn, "signaling_servers") {
        let _ = conn.execute_batch(
            "CREATE TABLE signaling_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TRIGGER IF NOT EXISTS signaling_servers_updated_at
                AFTER UPDATE ON signaling_servers FOR EACH ROW
            BEGIN UPDATE signaling_servers SET updated_at = datetime('now') WHERE id = OLD.id; END;",
        );
    }

    // Migrate existing signaling.partyUrl setting into signaling_servers table
    if has_table(conn, "signaling_servers") {
        let url: Result<String, _> = conn
            .prepare("SELECT value FROM settings WHERE key = 'signaling.partyUrl'")
            .and_then(|mut s| s.query_row([], |r| r.get::<_, String>(0)));
        if let Ok(url) = url {
            if !url.is_empty() {
                let name: String = conn
                    .prepare("SELECT value FROM settings WHERE key = 'signaling.deployName'")
                    .and_then(|mut s| s.query_row([], |r| r.get::<_, String>(0)))
                    .unwrap_or_else(|_| "PartyKit Server".to_string());
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO signaling_servers (id, name, url) VALUES (lower(hex(randomblob(16))), ?1, ?2)",
                    rusqlite::params![name, url],
                );
                let _ = conn.execute_batch(
                    "DELETE FROM settings WHERE key IN ('signaling.partyUrl', 'signaling.deployName')",
                );
            }
        }
    }

    // Migration: re-add audio and image media types (db_version 22, reverses db_version 20)
    {
        let _ = conn.execute_batch(
            "INSERT OR IGNORE INTO media_types (id, label) VALUES ('audio', 'Audio');
             INSERT OR IGNORE INTO media_types (id, label) VALUES ('image', 'Image');
             INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('audio-uncategorized', 'audio', 'Uncategorized');
             INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('image-uncategorized', 'image', 'Uncategorized');",
        );
    }

    // Migration: add llm_conversations table (db_version 21)
    if !has_table(conn, "llm_conversations") {
        let _ = conn.execute_batch(
            "CREATE TABLE llm_conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                system_prompt TEXT,
                messages TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TRIGGER IF NOT EXISTS llm_conversations_updated_at
                AFTER UPDATE ON llm_conversations FOR EACH ROW
            BEGIN UPDATE llm_conversations SET updated_at = datetime('now') WHERE id = OLD.id; END;",
        );
    }

    // Migration: migrate legacy columns to library_item_links (db_version 14 data)
    if has_table(conn, "library_items") && has_column(conn, "library_items", "tmdb_id") {
        let _ = conn.execute_batch(
            "INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id, season_number, episode_number)
             SELECT lower(hex(randomblob(16))), id, 'tmdb', CAST(tmdb_id AS TEXT), tmdb_season_number, tmdb_episode_number
             FROM library_items WHERE tmdb_id IS NOT NULL;

             INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id)
             SELECT lower(hex(randomblob(16))), id, 'youtube', youtube_id
             FROM library_items WHERE youtube_id IS NOT NULL;

             INSERT OR IGNORE INTO library_item_links (id, library_item_id, service, service_id)
             SELECT lower(hex(randomblob(16))), id, 'musicbrainz', musicbrainz_id
             FROM library_items WHERE musicbrainz_id IS NOT NULL;

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
             END;",
        );
    }

    // Migration: add tmdb_api_cache table
    if !has_table(conn, "tmdb_api_cache") {
        let _ = conn.execute_batch(
            "CREATE TABLE tmdb_api_cache (
                cache_key TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        );
    }

    // Migration: add torrent_fetch_cache table
    if !has_table(conn, "torrent_fetch_cache") {
        let _ = conn.execute_batch(
            "CREATE TABLE torrent_fetch_cache (
                tmdb_id INTEGER PRIMARY KEY,
                media_type TEXT NOT NULL,
                candidate_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        );
    }
}

/// App identifiers used to select which schema features to include.
pub fn app_id() -> Option<String> {
    std::env::var("APP_ID").ok()
}

/// Initialize the database schema, run migrations, and seed data.
/// Uses APP_ID env var to determine which feature tables to create.
pub fn initialize_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(CORE_SCHEMA_SQL)?;

    let app = app_id();
    let is_flix = app.as_deref() == Some("flix");

    if !is_flix {
        conn.execute_batch(MEDIA_LISTS_SQL)?;
        conn.execute_batch(YOUTUBE_TABLES_SQL)?;
        conn.execute_batch(IMAGE_TAGS_SQL)?;
    }

    run_migrations(conn);
    conn.execute_batch(SEED_SQL)?;
    Ok(())
}

/// Apply module schemas (addon tables).
/// Uses APP_ID env var to determine which module schemas to create.
pub fn initialize_module_schemas(conn: &Connection) -> Result<(), rusqlite::Error> {
    let app = app_id();
    let is_flix = app.as_deref() == Some("flix");

    // TMDB is used by flix and other apps
    conn.execute_batch(TMDB_SCHEMA_SQL)?;

    if !is_flix {
        conn.execute_batch(YOUTUBE_SCHEMA_SQL)?;
        conn.execute_batch(MUSICBRAINZ_SCHEMA_SQL)?;
        conn.execute_batch(LYRICS_SCHEMA_SQL)?;
        mhaol_cloud::initialize_cloud_schema(conn)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_schema() {
        let conn = Connection::open_in_memory().unwrap();
        // Test with no APP_ID (all tables)
        initialize_schema(&conn).unwrap();

        // Verify core tables exist
        assert!(has_table(&conn, "settings"));
        assert!(has_table(&conn, "metadata"));
        assert!(has_table(&conn, "media_types"));
        assert!(has_table(&conn, "categories"));
        assert!(has_table(&conn, "libraries"));
        assert!(has_table(&conn, "library_items"));
        assert!(has_table(&conn, "library_item_links"));
        assert!(has_table(&conn, "link_sources"));
        assert!(has_table(&conn, "youtube_content"));
        assert!(has_table(&conn, "youtube_downloads"));
        assert!(has_table(&conn, "youtube_channels"));
        assert!(has_table(&conn, "torrent_downloads"));
        assert!(has_table(&conn, "image_tags"));
        assert!(has_table(&conn, "media_lists"));
        assert!(has_table(&conn, "media_list_items"));
        assert!(has_table(&conn, "media_list_links"));
        assert!(has_table(&conn, "signaling_servers"));
        assert!(has_table(&conn, "llm_conversations"));
        assert!(has_table(&conn, "torrent_fetch_cache"));

        // Verify seed data
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM media_types", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_initialize_module_schemas() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        initialize_module_schemas(&conn).unwrap();

        assert!(has_table(&conn, "youtube_videos"));
        assert!(has_table(&conn, "tmdb_movies"));
        assert!(has_table(&conn, "tmdb_tv_shows"));
        assert!(has_table(&conn, "tmdb_seasons"));
    }
}

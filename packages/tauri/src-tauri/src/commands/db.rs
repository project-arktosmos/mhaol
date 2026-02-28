use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppDb(pub Mutex<Connection>);

// --- Types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_types: String,
    pub date_added: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub library_id: String,
    pub path: String,
    pub extension: String,
    pub media_type: String,
    pub category_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaType {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub media_type_id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

// --- Commands ---

#[tauri::command]
pub fn get_libraries(db: State<'_, AppDb>) -> Result<Vec<Library>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, path, media_types, date_added FROM libraries")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Library {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                media_types: row.get(3)?,
                date_added: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_library(
    db: State<'_, AppDb>,
    id: String,
    name: String,
    path: String,
    media_types: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO libraries (id, name, path, media_types, date_added) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, path, media_types, chrono::Utc::now().timestamp_millis()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_library_items(
    db: State<'_, AppDb>,
    library_id: String,
) -> Result<Vec<LibraryItem>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, library_id, path, extension, media_type, category_id, created_at \
             FROM library_items WHERE library_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![library_id], |row| {
            Ok(LibraryItem {
                id: row.get(0)?,
                library_id: row.get(1)?,
                path: row.get(2)?,
                extension: row.get(3)?,
                media_type: row.get(4)?,
                category_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_media_types(db: State<'_, AppDb>) -> Result<Vec<MediaType>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, label FROM media_types")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MediaType {
                id: row.get(0)?,
                label: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_categories(db: State<'_, AppDb>) -> Result<Vec<Category>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, media_type_id, label FROM categories")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                media_type_id: row.get(1)?,
                label: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(db: State<'_, AppDb>) -> Result<Vec<Setting>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(db: State<'_, AppDb>, key: String, value: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_item_category(
    db: State<'_, AppDb>,
    item_id: String,
    category_id: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE library_items SET category_id = ?1 WHERE id = ?2",
        params![category_id, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn browse_directory(path: String) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    dirs.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    dirs.sort();
    Ok(dirs)
}

#[tauri::command]
pub fn scan_library(
    db: State<'_, AppDb>,
    library_id: String,
) -> Result<u32, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get library path and media types
    let (lib_path, media_types_json): (String, String) = conn
        .query_row(
            "SELECT path, media_types FROM libraries WHERE id = ?1",
            params![library_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let media_types: Vec<String> =
        serde_json::from_str(&media_types_json).map_err(|e| e.to_string())?;

    // Build extension-to-media-type map
    let ext_map = build_extension_map(&media_types);
    if ext_map.is_empty() {
        return Ok(0);
    }

    // Recursively scan directory
    let mut count = 0u32;
    scan_dir(&conn, &lib_path, &library_id, &ext_map, &mut count)?;
    Ok(count)
}

fn build_extension_map(media_types: &[String]) -> Vec<(&'static str, &'static str)> {
    let mut map = Vec::new();
    for mt in media_types {
        match mt.as_str() {
            "video" => {
                for ext in &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"] {
                    map.push((*ext, "video"));
                }
            }
            "audio" => {
                for ext in &["mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "opus"] {
                    map.push((*ext, "audio"));
                }
            }
            "image" => {
                for ext in &["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "tiff"] {
                    map.push((*ext, "image"));
                }
            }
            _ => {}
        }
    }
    map
}

fn scan_dir(
    conn: &Connection,
    dir: &str,
    library_id: &str,
    ext_map: &[(&str, &str)],
    count: &mut u32,
) -> Result<(), String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let path = entry.path();

        if file_type.is_dir() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with('.') {
                scan_dir(conn, &path.to_string_lossy(), library_id, ext_map, count)?;
            }
        } else if file_type.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if let Some((_, media_type)) = ext_map.iter().find(|(e, _)| *e == ext_lower.as_str())
                {
                    let path_str = path.to_string_lossy().to_string();
                    let id = uuid::Uuid::new_v4().to_string();

                    let result = conn.execute(
                        "INSERT OR IGNORE INTO library_items (id, library_id, path, extension, media_type) \
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![id, library_id, path_str, ext_lower, media_type],
                    );

                    if let Ok(rows) = result {
                        if rows > 0 {
                            *count += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Initialize the database schema and seed data.
pub fn initialize_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'string' CHECK (type IN ('string', 'number', 'boolean', 'json')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

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

        INSERT OR IGNORE INTO media_types (id, label) VALUES ('video', 'Video');
        INSERT OR IGNORE INTO media_types (id, label) VALUES ('image', 'Image');
        INSERT OR IGNORE INTO media_types (id, label) VALUES ('audio', 'Audio');

        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('tv', 'video', 'TV');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('movies', 'video', 'Movies');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('youtube', 'video', 'YouTube');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('video-uncategorized', 'video', 'Uncategorized');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('music', 'audio', 'Music');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('podcast', 'audio', 'Podcast');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('audio-uncategorized', 'audio', 'Uncategorized');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('photos', 'image', 'Photos');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('memes', 'image', 'Memes');
        INSERT OR IGNORE INTO categories (id, media_type_id, label) VALUES ('image-uncategorized', 'image', 'Uncategorized');
        ",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

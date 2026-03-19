use crate::types::InsertCloudItem;

pub fn mime_from_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "mp4" => Some("video/mp4"),
        "mkv" => Some("video/x-matroska"),
        "webm" => Some("video/webm"),
        "avi" => Some("video/x-msvideo"),
        "mov" => Some("video/quicktime"),
        "flv" => Some("video/x-flv"),
        "m4v" => Some("video/x-m4v"),
        "mp3" => Some("audio/mpeg"),
        "flac" => Some("audio/flac"),
        "wav" => Some("audio/wav"),
        "ogg" => Some("audio/ogg"),
        "m4a" => Some("audio/mp4"),
        "opus" => Some("audio/opus"),
        "aac" => Some("audio/aac"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        "pdf" => Some("application/pdf"),
        "txt" => Some("text/plain"),
        "json" => Some("application/json"),
        _ => None,
    }
}

pub fn scan_dir_recursive(dir: &str, library_id: &str, files: &mut Vec<InsertCloudItem>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let path = entry.path();

        if file_type.is_dir() {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with('.') {
                scan_dir_recursive(&path.to_string_lossy(), library_id, files);
            }
        } else if file_type.is_file() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with('.') {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let mime_type = mime_from_extension(&ext).map(|s| s.to_string());
            let size_bytes = std::fs::metadata(&path).ok().map(|m| m.len() as i64);

            files.push(InsertCloudItem {
                id: uuid::Uuid::new_v4().to_string(),
                library_id: library_id.to_string(),
                path: path.to_string_lossy().to_string(),
                filename,
                extension: ext,
                size_bytes,
                mime_type,
            });
        }
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Total size in bytes of a file or directory tree on disk.
pub fn path_size_bytes(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total: u64 = 0;
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        total = total.saturating_add(path_size_bytes(&p));
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn timestamp_is_positive() {
        assert!(get_unix_timestamp() > 0);
    }

    #[test]
    fn path_size_for_missing_path_is_zero() {
        assert_eq!(path_size_bytes(std::path::Path::new("/__missing__/x")), 0);
    }

    #[test]
    fn path_size_for_file() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("a.bin");
        fs::write(&f, b"hello").unwrap();
        assert_eq!(path_size_bytes(&f), 5);
    }

    #[test]
    fn path_size_for_directory() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.bin"), b"hello").unwrap();
        fs::write(tmp.path().join("b.bin"), b"world!").unwrap();
        assert_eq!(path_size_bytes(tmp.path()), 11);
    }
}

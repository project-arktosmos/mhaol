use anyhow::{anyhow, Result};
use libp2p::pnet::PreSharedKey;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Generate a fresh swarm key in the standard go-ipfs `swarm.key` format:
///
/// ```text
/// /key/swarm/psk/1.0.0/
/// /base16/
/// <64 hex chars>
/// ```
///
/// Share the resulting string out-of-band with every other node that should
/// belong to the same private network.
pub fn generate_swarm_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    PreSharedKey::new(bytes).to_string()
}

/// Parse a `swarm.key` payload, returning the hex fingerprint go-libp2p uses
/// to compare keys without exposing them.
pub fn swarm_key_fingerprint(swarm_key: &str) -> Result<String> {
    let psk = PreSharedKey::from_str(swarm_key)
        .map_err(|e| anyhow!("Invalid swarm key: {}", e))?;
    Ok(format!("{}", psk.fingerprint()))
}

/// Read a `swarm.key` file off disk if it exists, or return `Ok(None)` if not.
pub fn load_swarm_key(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read swarm key at {}: {}", path.display(), e))?;
    Ok(Some(contents))
}

/// Write a `swarm.key` payload to disk with restrictive (0600 on Unix) perms.
pub fn save_swarm_key(path: &Path, swarm_key: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(path, swarm_key)
        .map_err(|e| anyhow!("Failed to write swarm key at {}: {}", path.display(), e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Default location of the shared `swarm.key` file. Resolved as:
/// `$DATA_DIR/swarm.key` when `DATA_DIR` is set, otherwise
/// `<home>/mhaol/swarm.key`, falling back to `./swarm.key` if the home
/// directory cannot be determined. Every Mhaol process that should join the
/// same private swarm reads this path by default — keeping it shared is what
/// lets the cloud and the rendezvous bootstrap node converge on the same PSK
/// without per-app configuration.
pub fn default_swarm_key_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("DATA_DIR") {
        return PathBuf::from(data_dir).join("swarm.key");
    }
    if let Some(home) = dirs_home() {
        return home.join("mhaol").join("swarm.key");
    }
    PathBuf::from("swarm.key")
}

#[cfg(unix)]
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(windows)]
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

#[cfg(not(any(unix, windows)))]
fn dirs_home() -> Option<PathBuf> {
    None
}

/// Returns an existing swarm key on disk, or generates and persists a new one.
/// Use this when you want the manager to own its swarm key lifecycle.
pub fn ensure_swarm_key(path: &Path) -> Result<String> {
    if let Some(existing) = load_swarm_key(path)? {
        return Ok(existing);
    }
    let key = generate_swarm_key();
    save_swarm_key(path, &key)?;
    Ok(key)
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

    #[test]
    fn generate_swarm_key_round_trips_through_psk_parser() {
        let key = generate_swarm_key();
        assert!(key.starts_with("/key/swarm/psk/1.0.0/"));
        // Must be parseable as a PreSharedKey and produce a stable fingerprint.
        let fp = swarm_key_fingerprint(&key).unwrap();
        assert!(!fp.is_empty());
    }

    #[test]
    fn generate_swarm_key_yields_different_keys() {
        // Vanishingly small probability of collision with 256 bits of entropy.
        let a = generate_swarm_key();
        let b = generate_swarm_key();
        assert_ne!(a, b);
    }

    #[test]
    fn fingerprint_rejects_invalid_key() {
        assert!(swarm_key_fingerprint("not a key").is_err());
    }

    #[test]
    fn ensure_swarm_key_creates_then_reuses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("swarm.key");
        assert!(!path.exists());
        let first = ensure_swarm_key(&path).unwrap();
        assert!(path.exists());
        let second = ensure_swarm_key(&path).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn load_swarm_key_returns_none_for_missing_path() {
        assert!(load_swarm_key(std::path::Path::new("/__missing__/swarm.key"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn save_swarm_key_writes_payload() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("swarm.key");
        let key = generate_swarm_key();
        save_swarm_key(&path, &key).unwrap();
        let loaded = load_swarm_key(&path).unwrap().unwrap();
        assert_eq!(loaded, key);
    }
}

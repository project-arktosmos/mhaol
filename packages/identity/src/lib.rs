pub mod passport;

pub use passport::{eip191_recover, eip191_sign, Passport};

use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages Ethereum-like secp256k1 identities stored as individual files in a directory.
/// Each file is named after the identity (e.g. `SIGNALING_WALLET`) and contains the private key hex.
#[derive(Clone)]
pub struct IdentityManager {
    dir_path: PathBuf,
}

impl IdentityManager {
    pub fn new(dir_path: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir_path);
        Self { dir_path }
    }

    /// Validate that a name is safe for use as a filename.
    fn validate_name(name: &str) -> bool {
        !name.is_empty()
            && !name.starts_with('.')
            && !name.contains('/')
            && !name.contains('\\')
            && !name.contains("..")
    }

    /// Load all identities from the directory. Returns name -> private_key_hex map.
    fn load_all(&self) -> BTreeMap<String, String> {
        let mut entries = BTreeMap::new();
        let read_dir = match fs::read_dir(&self.dir_path) {
            Ok(rd) => rd,
            Err(_) => return entries,
        };
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                let pk = content.trim().to_string();
                if !pk.is_empty() {
                    entries.insert(name, pk);
                }
            }
        }
        entries
    }

    /// Load a single identity's private key by name.
    fn load_one(&self, name: &str) -> Option<String> {
        if !Self::validate_name(name) {
            return None;
        }
        let path = self.dir_path.join(name);
        fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Write a single identity file. Uses atomic write (tmp + rename).
    fn write_one(&self, name: &str, private_key: &str) {
        let _ = fs::create_dir_all(&self.dir_path);
        let target = self.dir_path.join(name);
        let tmp = self.dir_path.join(format!(".{}.tmp", name));
        fs::write(&tmp, format!("{}\n", private_key)).unwrap();
        fs::rename(&tmp, &target).unwrap();
    }

    /// Derive the Ethereum address from a hex private key.
    fn private_key_to_address(private_key_hex: &str) -> String {
        let hex_str = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);
        let key_bytes = hex::decode(hex_str).unwrap();
        let signing_key = SigningKey::from_bytes((&key_bytes[..]).into()).unwrap();
        let verifying_key = signing_key.verifying_key();

        // Uncompressed public key (65 bytes), skip first byte (0x04)
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let pub_bytes = &public_key_bytes.as_bytes()[1..]; // skip 0x04 prefix

        // Keccak-256 hash, take last 20 bytes
        use sha3::{Digest, Keccak256};
        let hash = Keccak256::digest(pub_bytes);
        let address_bytes = &hash[12..];

        format!("0x{}", hex::encode(address_bytes))
    }

    /// Generate a new random private key as 0x-prefixed hex string.
    fn generate_private_key() -> String {
        let signing_key = SigningKey::random(&mut OsRng);
        format!("0x{}", hex::encode(signing_key.to_bytes()))
    }

    /// Get all identities as name -> address pairs.
    pub fn get_all(&self) -> BTreeMap<String, String> {
        let entries = self.load_all();
        let mut result = BTreeMap::new();
        for (name, pk) in &entries {
            result.insert(name.clone(), Self::private_key_to_address(pk));
        }
        result
    }

    /// Get the address for a named identity.
    pub fn get_address(&self, name: &str) -> Option<String> {
        self.load_one(name)
            .map(|pk| Self::private_key_to_address(&pk))
    }

    /// Get the raw private key hex for a named identity.
    pub fn get_private_key(&self, name: &str) -> Option<String> {
        self.load_one(name)
    }

    /// Store a private key under the given name.
    pub fn set(&self, name: &str, private_key: &str) -> String {
        self.write_one(name, private_key);
        Self::private_key_to_address(private_key)
    }

    /// Generate a new keypair and store it under the given name.
    pub fn regenerate(&self, name: &str) -> String {
        let pk = Self::generate_private_key();
        self.set(name, &pk)
    }

    /// Remove an identity.
    pub fn remove(&self, name: &str) -> bool {
        if !Self::validate_name(name) {
            return false;
        }
        let path = self.dir_path.join(name);
        fs::remove_file(&path).is_ok()
    }

    /// Get a signed passport for the named identity.
    pub fn get_passport(&self, name: &str) -> Option<passport::Passport> {
        let private_key_hex = self.load_one(name)?;
        let address = Self::private_key_to_address(&private_key_hex);
        Some(passport::sign_passport(name, &address, &private_key_hex))
    }

    /// Ensure an identity exists; create one if missing.
    pub fn ensure_identity(&self, name: &str) -> String {
        if let Some(addr) = self.get_address(name) {
            addr
        } else {
            self.regenerate(name)
        }
    }

    /// Get the address of the first identity, if any.
    pub fn get_default_address(&self) -> Option<String> {
        let entries = self.load_all();
        entries
            .values()
            .next()
            .map(|pk| Self::private_key_to_address(pk))
    }
}

/// Return the default identity directory: `~/.mhaol-identities/`.
pub fn default_identities_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".mhaol-identities")
}

/// Migrate identities from a legacy `.env.identities` file into the directory-based store.
/// Only imports identities that don't already exist in the directory.
/// Returns the number of identities migrated.
pub fn migrate_from_env_file(env_file: &Path, manager: &IdentityManager) -> usize {
    let content = match fs::read_to_string(env_file) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let mut count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let value = trimmed[eq_idx + 1..].trim();
            if !key.is_empty() && !value.is_empty() && manager.load_one(key).is_none() {
                manager.write_one(key, value);
                count += 1;
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_lifecycle() {
        let tmp_dir = std::env::temp_dir().join(format!("test_identities_{}", uuid::Uuid::new_v4()));
        let mgr = IdentityManager::new(tmp_dir.clone());

        // Start empty
        assert!(mgr.get_all().is_empty());

        // Create identity
        let addr = mgr.regenerate("TEST_WALLET");
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);

        // Verify file exists on disk
        assert!(tmp_dir.join("TEST_WALLET").exists());

        // Retrieve
        assert_eq!(mgr.get_address("TEST_WALLET"), Some(addr.clone()));
        assert!(mgr.get_private_key("TEST_WALLET").is_some());

        // Passport
        let passport = mgr.get_passport("TEST_WALLET").unwrap();
        assert!(!passport.raw.is_empty());
        assert!(!passport.hash.is_empty());
        assert!(!passport.signature.is_empty());

        // Ensure is idempotent
        let addr2 = mgr.ensure_identity("TEST_WALLET");
        assert_eq!(addr, addr2);

        // Remove
        assert!(mgr.remove("TEST_WALLET"));
        assert!(mgr.get_address("TEST_WALLET").is_none());
        assert!(!tmp_dir.join("TEST_WALLET").exists());

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_migration_from_env_file() {
        let tmp_dir = std::env::temp_dir().join(format!("test_migrate_{}", uuid::Uuid::new_v4()));
        let mgr = IdentityManager::new(tmp_dir.clone());

        // Create a legacy .env.identities file
        let env_file = std::env::temp_dir().join(format!("test_env_{}", uuid::Uuid::new_v4()));
        let pk = IdentityManager::generate_private_key();
        fs::write(&env_file, format!("WALLET_A={}\nWALLET_B={}\n", pk, pk)).unwrap();

        let count = migrate_from_env_file(&env_file, &mgr);
        assert_eq!(count, 2);
        assert!(mgr.get_address("WALLET_A").is_some());
        assert!(mgr.get_address("WALLET_B").is_some());

        // Second run should migrate 0 (already exist)
        let count2 = migrate_from_env_file(&env_file, &mgr);
        assert_eq!(count2, 0);

        let _ = fs::remove_dir_all(&tmp_dir);
        let _ = fs::remove_file(&env_file);
    }

    #[test]
    fn test_name_validation() {
        let tmp_dir = std::env::temp_dir().join(format!("test_validate_{}", uuid::Uuid::new_v4()));
        let mgr = IdentityManager::new(tmp_dir.clone());

        // Valid names work
        let addr = mgr.regenerate("GOOD_NAME");
        assert!(addr.starts_with("0x"));

        // Invalid names are rejected
        assert!(mgr.get_address("../escape").is_none());
        assert!(mgr.get_address(".hidden").is_none());
        assert!(mgr.get_address("path/traversal").is_none());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}

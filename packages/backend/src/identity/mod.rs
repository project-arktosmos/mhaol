pub mod passport;

use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages Ethereum-like secp256k1 identities stored in a .env.identities file.
#[derive(Clone)]
pub struct IdentityManager {
    file_path: PathBuf,
}

impl IdentityManager {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    /// Parse the .env.identities file into name -> private_key_hex map.
    fn parse(&self) -> BTreeMap<String, String> {
        let mut entries = BTreeMap::new();
        if !self.file_path.exists() {
            return entries;
        }

        let content = fs::read_to_string(&self.file_path).unwrap_or_default();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(eq_idx) = trimmed.find('=') {
                let key = trimmed[..eq_idx].trim().to_string();
                let value = trimmed[eq_idx + 1..].trim().to_string();
                if !key.is_empty() && !value.is_empty() {
                    entries.insert(key, value);
                }
            }
        }
        entries
    }

    /// Write entries back to the file.
    fn write(&self, entries: &BTreeMap<String, String>) {
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content: String = entries
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(&self.file_path, content).unwrap();
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
        let entries = self.parse();
        let mut result = BTreeMap::new();
        for (name, pk) in &entries {
            result.insert(name.clone(), Self::private_key_to_address(pk));
        }
        result
    }

    /// Get the address for a named identity.
    pub fn get_address(&self, name: &str) -> Option<String> {
        let entries = self.parse();
        entries
            .get(name)
            .map(|pk| Self::private_key_to_address(pk))
    }

    /// Get the raw private key hex for a named identity.
    pub fn get_private_key(&self, name: &str) -> Option<String> {
        let entries = self.parse();
        entries.get(name).cloned()
    }

    /// Store a private key under the given name.
    pub fn set(&self, name: &str, private_key: &str) -> String {
        let mut entries = self.parse();
        entries.insert(name.to_string(), private_key.to_string());
        self.write(&entries);
        Self::private_key_to_address(private_key)
    }

    /// Generate a new keypair and store it under the given name.
    pub fn regenerate(&self, name: &str) -> String {
        let pk = Self::generate_private_key();
        self.set(name, &pk)
    }

    /// Remove an identity.
    pub fn remove(&self, name: &str) -> bool {
        let mut entries = self.parse();
        if entries.remove(name).is_some() {
            self.write(&entries);
            true
        } else {
            false
        }
    }

    /// Get a signed passport for the named identity.
    pub fn get_passport(&self, name: &str) -> Option<passport::Passport> {
        let entries = self.parse();
        let private_key_hex = entries.get(name)?;
        let address = Self::private_key_to_address(private_key_hex);

        Some(passport::sign_passport(name, &address, private_key_hex))
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
        let entries = self.parse();
        entries
            .values()
            .next()
            .map(|pk| Self::private_key_to_address(pk))
    }
}

/// Find the .env.identities file path, searching upward for the repo root.
pub fn default_identities_path() -> PathBuf {
    // Try to find pnpm-workspace.yaml to locate repo root
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir.join(".env.identities");
        }
        if !dir.pop() {
            break;
        }
    }
    // Fallback: current directory
    Path::new(".env.identities").to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_identity_lifecycle() {
        let tmp = std::env::temp_dir().join(format!("test_identities_{}", uuid::Uuid::new_v4()));
        let mgr = IdentityManager::new(tmp.clone());

        // Start empty
        assert!(mgr.get_all().is_empty());

        // Create identity
        let addr = mgr.regenerate("TEST_WALLET");
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);

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

        let _ = fs::remove_file(&tmp);
    }
}

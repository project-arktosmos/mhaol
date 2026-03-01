use k256::ecdsa::{Signature, SigningKey};
use serde::Serialize;
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone, Serialize)]
pub struct Passport {
    pub raw: String,
    pub hash: String,
    pub signature: String,
}

/// Sign a passport message using EIP-191 personal_sign.
pub fn sign_passport(name: &str, address: &str, private_key_hex: &str) -> Passport {
    let raw = serde_json::json!({ "name": name, "address": address }).to_string();

    // EIP-191: "\x19Ethereum Signed Message:\n" + len + message
    let prefix = format!("\x19Ethereum Signed Message:\n{}", raw.len());
    let mut prefixed = Vec::new();
    prefixed.extend_from_slice(prefix.as_bytes());
    prefixed.extend_from_slice(raw.as_bytes());

    let hash = Keccak256::digest(&prefixed);
    let hash_hex = format!("0x{}", hex::encode(&hash));

    // Sign
    let hex_str = private_key_hex
        .strip_prefix("0x")
        .unwrap_or(private_key_hex);
    let key_bytes = hex::decode(hex_str).unwrap();
    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into()).unwrap();
    let (signature, recovery_id): (Signature, _) = signing_key.sign_prehash_recoverable(&hash).unwrap();

    // Encode as 65-byte signature (r || s || v)
    let mut sig_bytes = signature.to_bytes().to_vec();
    sig_bytes.push(recovery_id.to_byte() + 27); // v = recovery_id + 27

    let signature_hex = format!("0x{}", hex::encode(&sig_bytes));

    Passport {
        raw,
        hash: hash_hex,
        signature: signature_hex,
    }
}

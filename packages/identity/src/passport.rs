use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use serde::Serialize;
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone, Serialize)]
pub struct Passport {
    pub raw: String,
    pub hash: String,
    pub signature: String,
}

/// Internal struct for passport payload serialization.
/// Uses `skip_serializing_if` to omit optional profile fields from the signed JSON.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PassportPayloadData<'a> {
    name: &'a str,
    address: &'a str,
    instance_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_picture_url: Option<&'a str>,
}

/// Sign an arbitrary message using EIP-191 personal_sign, returning the 0x-prefixed hex signature.
pub fn eip191_sign(message: &str, private_key_hex: &str) -> String {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut prefixed = Vec::new();
    prefixed.extend_from_slice(prefix.as_bytes());
    prefixed.extend_from_slice(message.as_bytes());

    let hash = Keccak256::digest(&prefixed);

    let hex_str = private_key_hex
        .strip_prefix("0x")
        .unwrap_or(private_key_hex);
    let key_bytes = hex::decode(hex_str).unwrap();
    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into()).unwrap();
    let (signature, recovery_id): (Signature, _) = signing_key.sign_prehash_recoverable(&hash).unwrap();

    let mut sig_bytes = signature.to_bytes().to_vec();
    sig_bytes.push(recovery_id.to_byte() + 27);

    format!("0x{}", hex::encode(&sig_bytes))
}

/// Recover the Ethereum address from an EIP-191 signed message.
/// Returns the lowercase 0x-prefixed address.
pub fn eip191_recover(message: &str, signature_hex: &str) -> Result<String, String> {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut prefixed = Vec::new();
    prefixed.extend_from_slice(prefix.as_bytes());
    prefixed.extend_from_slice(message.as_bytes());
    let hash = Keccak256::digest(&prefixed);

    let sig_hex = signature_hex
        .strip_prefix("0x")
        .unwrap_or(signature_hex);
    let sig_bytes = hex::decode(sig_hex).map_err(|e| format!("Invalid hex: {e}"))?;
    if sig_bytes.len() != 65 {
        return Err(format!("Signature must be 65 bytes, got {}", sig_bytes.len()));
    }

    let v = sig_bytes[64];
    let recovery_byte = if v >= 27 { v - 27 } else { v };
    let recid = k256::ecdsa::RecoveryId::try_from(recovery_byte)
        .map_err(|e| format!("Invalid recovery id: {e}"))?;

    let signature = Signature::from_bytes((&sig_bytes[..64]).into())
        .map_err(|e| format!("Invalid signature: {e}"))?;

    let verifying_key = VerifyingKey::recover_from_prehash(&hash, &signature, recid)
        .map_err(|e| format!("Recovery failed: {e}"))?;

    let encoded = verifying_key.to_encoded_point(false);
    let pub_bytes = &encoded.as_bytes()[1..];
    let addr_hash = Keccak256::digest(pub_bytes);
    Ok(format!("0x{}", hex::encode(&addr_hash[12..])))
}

/// Sign a passport message using EIP-191 personal_sign.
pub fn sign_passport(
    name: &str,
    address: &str,
    instance_type: &str,
    private_key_hex: &str,
    username: Option<&str>,
    profile_picture_url: Option<&str>,
) -> Passport {
    let payload = PassportPayloadData {
        name,
        address,
        instance_type,
        username,
        profile_picture_url,
    };
    let raw = serde_json::to_string(&payload).unwrap();

    // EIP-191: "\x19Ethereum Signed Message:\n" + len + message
    let prefix = format!("\x19Ethereum Signed Message:\n{}", raw.len());
    let mut prefixed = Vec::new();
    prefixed.extend_from_slice(prefix.as_bytes());
    prefixed.extend_from_slice(raw.as_bytes());

    let hash = Keccak256::digest(&prefixed);
    let hash_hex = format!("0x{}", hex::encode(hash));

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

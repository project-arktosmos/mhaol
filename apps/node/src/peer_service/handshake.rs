use super::types::{
    ContactMessage, DataChannelEnvelope, Endorsement, PassportData, PassportPayload,
};
use mhaol_identity::Passport;
use tracing::info;

/// Verify a passport by recovering the signer address and comparing it to the
/// address field in the payload.
pub fn verify_passport(passport: &PassportData) -> Result<PassportPayload, String> {
    let payload: PassportPayload =
        serde_json::from_str(&passport.raw).map_err(|e| format!("Invalid passport JSON: {e}"))?;

    let recovered = mhaol_identity::eip191_recover(&passport.raw, &passport.signature)?;

    if recovered.to_lowercase() != payload.address.to_lowercase() {
        return Err(format!(
            "Passport signature mismatch: recovered {recovered}, expected {}",
            payload.address
        ));
    }

    Ok(payload)
}

/// Create an endorsement for a client's passport using the server's identity.
pub fn create_endorsement(
    passport_raw: &str,
    server_private_key: &str,
    server_address: &str,
) -> Endorsement {
    let signature = mhaol_identity::eip191_sign(passport_raw, server_private_key);
    let checksummed = mhaol_identity::to_eip55_checksum(server_address);

    Endorsement {
        passport_raw: passport_raw.to_string(),
        endorser_signature: signature,
        endorser_address: checksummed,
        endorsed_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Build a `contact-accept` envelope to send back to the requesting peer.
pub fn build_accept_envelope(
    server_passport: &Passport,
    endorsement: Option<Endorsement>,
) -> DataChannelEnvelope {
    let passport_data = PassportData {
        raw: server_passport.raw.clone(),
        hash: server_passport.hash.clone(),
        signature: server_passport.signature.clone(),
    };

    let msg = ContactMessage::ContactAccept {
        passport: passport_data,
        endorsement,
    };

    DataChannelEnvelope {
        channel: "contact".to_string(),
        payload: serde_json::to_value(msg).unwrap(),
    }
}

/// Handle an incoming contact request. The server auto-accepts all requests.
///
/// Returns the accept envelope to send back, and the verified payload of the
/// requesting peer.
pub fn handle_contact_request(
    request_passport: &PassportData,
    server_passport: &Passport,
    server_private_key: &str,
    server_address: &str,
) -> Result<(DataChannelEnvelope, PassportPayload), String> {
    let payload = verify_passport(request_passport)?;

    info!(
        name = %payload.name,
        address = %payload.address,
        instance_type = %payload.instance_type,
        "Auto-accepting contact request"
    );

    let endorsement = create_endorsement(&request_passport.raw, server_private_key, server_address);

    let envelope = build_accept_envelope(server_passport, Some(endorsement));

    Ok((envelope, payload))
}

/// Parse a data channel envelope as a contact message.
pub fn parse_contact_message(envelope: &DataChannelEnvelope) -> Option<ContactMessage> {
    if envelope.channel != "contact" {
        return None;
    }
    serde_json::from_value(envelope.payload.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_passport_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!("test_handshake_{}", uuid::Uuid::new_v4()));
        let mgr = mhaol_identity::IdentityManager::new(
            tmp_dir.clone(),
            "client".to_string(),
            "https://test.example.com".to_string(),
        );
        let _addr = mgr.regenerate("TEST");
        let passport = mgr.get_passport("TEST").unwrap();

        let passport_data = PassportData {
            raw: passport.raw.clone(),
            hash: passport.hash.clone(),
            signature: passport.signature.clone(),
        };

        let payload = verify_passport(&passport_data).unwrap();
        assert_eq!(payload.name, "TEST");
        assert_eq!(payload.instance_type, "client");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_verify_passport_bad_signature() {
        let bad = PassportData {
            raw: r#"{"name":"test","address":"0x0000000000000000000000000000000000000000","instanceType":"client","signalingUrl":"https://example.com"}"#.to_string(),
            hash: String::new(),
            signature: "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        };
        assert!(verify_passport(&bad).is_err());
    }
}

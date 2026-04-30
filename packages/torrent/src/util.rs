use std::time::{SystemTime, UNIX_EPOCH};

/// Map a filename's extension to a streaming mime type.
/// Returns `None` for files we don't consider streamable in a `<video>` element.
pub fn streamable_mime_type(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next()?.to_ascii_lowercase();
    match ext.as_str() {
        "mp4" | "m4v" => Some("video/mp4"),
        "webm" => Some("video/webm"),
        "mkv" => Some("video/x-matroska"),
        "mov" => Some("video/quicktime"),
        "avi" => Some("video/x-msvideo"),
        "ogv" => Some("video/ogg"),
        "ts" => Some("video/mp2t"),
        _ => None,
    }
}

pub fn get_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse info_hash and display name from a magnet URI.
/// Returns `(info_hash, display_name)` or `None` if the URI is not a valid magnet link.
pub fn parse_magnet_uri(magnet: &str) -> Option<(String, String)> {
    if !magnet.starts_with("magnet:") {
        return None;
    }

    let info_hash = magnet
        .split("btih:")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .map(|s| s.to_lowercase())?;

    let display_name = magnet
        .split("dn=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .and_then(|s| url::form_urlencoded::parse(s.as_bytes()).next())
        .map(|(k, v)| if v.is_empty() { k.to_string() } else { v.to_string() })
        .unwrap_or_else(|| format!("Torrent {}", &info_hash[..8.min(info_hash.len())]));

    Some((info_hash, display_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── get_unix_timestamp ──────────────────────────────────────────

    #[test]
    fn timestamp_is_positive() {
        let ts = get_unix_timestamp();
        assert!(ts > 0, "Timestamp should be positive, got {}", ts);
    }

    #[test]
    fn timestamp_is_reasonable() {
        // Should be after 2020-01-01 (1577836800) and before 2100-01-01 (4102444800)
        let ts = get_unix_timestamp();
        assert!(ts > 1_577_836_800, "Timestamp too old: {}", ts);
        assert!(ts < 4_102_444_800, "Timestamp too far in future: {}", ts);
    }

    #[test]
    fn timestamp_is_monotonic() {
        let ts1 = get_unix_timestamp();
        let ts2 = get_unix_timestamp();
        assert!(ts2 >= ts1);
    }

    // ── parse_magnet_uri ────────────────────────────────────────────

    #[test]
    fn parse_magnet_full_uri() {
        let magnet = "magnet:?xt=urn:btih:ABC123DEF456&dn=My+Torrent+File&tr=udp://tracker:1337";
        let result = parse_magnet_uri(magnet);
        assert!(result.is_some());
        let (hash, name) = result.unwrap();
        assert_eq!(hash, "abc123def456");
        assert_eq!(name, "My Torrent File");
    }

    #[test]
    fn parse_magnet_info_hash_lowercased() {
        let magnet = "magnet:?xt=urn:btih:AABBCCDD11223344&dn=Test";
        let (hash, _) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "aabbccdd11223344");
    }

    #[test]
    fn parse_magnet_no_display_name_uses_hash_prefix() {
        let magnet = "magnet:?xt=urn:btih:abcdef1234567890&tr=udp://tracker:1337";
        let (hash, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "abcdef1234567890");
        assert_eq!(name, "Torrent abcdef12");
    }

    #[test]
    fn parse_magnet_short_hash_no_display_name() {
        let magnet = "magnet:?xt=urn:btih:abc";
        let (hash, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "abc");
        assert_eq!(name, "Torrent abc");
    }

    #[test]
    fn parse_magnet_url_encoded_display_name() {
        let magnet = "magnet:?xt=urn:btih:1234567890abcdef&dn=Hello%20World%21";
        let (_, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(name, "Hello World!");
    }

    #[test]
    fn parse_magnet_display_name_with_plus_encoding() {
        let magnet = "magnet:?xt=urn:btih:1234567890abcdef&dn=My+Cool+File";
        let (_, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(name, "My Cool File");
    }

    #[test]
    fn parse_magnet_display_name_before_other_params() {
        let magnet = "magnet:?xt=urn:btih:aabb&dn=FileName&xl=12345&tr=udp://t:80";
        let (hash, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "aabb");
        assert_eq!(name, "FileName");
    }

    #[test]
    fn parse_magnet_not_a_magnet_link() {
        assert!(parse_magnet_uri("http://example.com").is_none());
        assert!(parse_magnet_uri("https://example.com").is_none());
        assert!(parse_magnet_uri("/path/to/file.torrent").is_none());
        assert!(parse_magnet_uri("").is_none());
        assert!(parse_magnet_uri("random text").is_none());
    }

    #[test]
    fn parse_magnet_no_btih() {
        let magnet = "magnet:?xt=urn:sha1:abc123&dn=Test";
        assert!(parse_magnet_uri(magnet).is_none());
    }

    #[test]
    fn parse_magnet_empty_after_magnet_prefix() {
        // "magnet:" with nothing after — no btih, so None
        assert!(parse_magnet_uri("magnet:").is_none());
        assert!(parse_magnet_uri("magnet:?").is_none());
    }

    #[test]
    fn parse_magnet_hash_with_trailing_params() {
        let magnet = "magnet:?xt=urn:btih:DEADBEEF&tr=udp://a:80&tr=udp://b:80";
        let (hash, name) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "deadbeef");
        // No dn= so falls back to hash prefix
        assert_eq!(name, "Torrent deadbeef");
    }

    #[test]
    fn parse_magnet_mixed_case_btih_prefix() {
        // The parser splits on "btih:" so the case of urn:btih matters
        let magnet = "magnet:?xt=urn:btih:AaBbCcDd";
        let (hash, _) = parse_magnet_uri(magnet).unwrap();
        assert_eq!(hash, "aabbccdd");
    }
}

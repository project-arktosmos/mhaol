use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parsed components of an `ed2k://|file|name|size|hash|...|/` URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEd2kFile {
    pub name: String,
    pub size: u64,
    pub file_hash: String,
}

/// Parse an `ed2k://|file|<name>|<size>|<hash>|/` URI. The name is URL-decoded
/// (percent and `+` rules), size is parsed as decimal, hash is lowercased.
pub fn parse_ed2k_file_uri(uri: &str) -> Option<ParsedEd2kFile> {
    let rest = uri.strip_prefix("ed2k://|file|")?;
    let mut parts = rest.split('|');
    let name_raw = parts.next()?;
    let size_raw = parts.next()?;
    let hash_raw = parts.next()?;

    if name_raw.is_empty() || hash_raw.is_empty() {
        return None;
    }

    let size: u64 = size_raw.parse().ok()?;

    let name = urlencoding::decode(name_raw)
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| name_raw.to_string());

    let file_hash = hash_raw.to_lowercase();
    if file_hash.len() != 32 || !file_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(ParsedEd2kFile {
        name,
        size,
        file_hash,
    })
}

/// Build a canonical `ed2k://|file|name|size|hash|/` URI from parts.
pub fn build_ed2k_file_uri(name: &str, size: u64, file_hash: &str) -> String {
    format!(
        "ed2k://|file|{}|{}|{}|/",
        urlencoding::encode(name),
        size,
        file_hash.to_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_is_positive() {
        assert!(get_unix_timestamp() > 0);
    }

    #[test]
    fn parse_basic_ed2k_uri() {
        let uri =
            "ed2k://|file|MyFile.mkv|1048576|aabbccdd11223344aabbccdd11223344|/";
        let parsed = parse_ed2k_file_uri(uri).unwrap();
        assert_eq!(parsed.name, "MyFile.mkv");
        assert_eq!(parsed.size, 1048576);
        assert_eq!(parsed.file_hash, "aabbccdd11223344aabbccdd11223344");
    }

    #[test]
    fn parse_with_extra_segments() {
        let uri =
            "ed2k://|file|Foo.iso|9999|0011223344556677889900AABBCCDDEE|h=ABC|p=123|/";
        let parsed = parse_ed2k_file_uri(uri).unwrap();
        assert_eq!(parsed.name, "Foo.iso");
        assert_eq!(parsed.size, 9999);
        assert_eq!(parsed.file_hash, "0011223344556677889900aabbccddee");
    }

    #[test]
    fn parse_url_encoded_name() {
        let uri =
            "ed2k://|file|Hello%20World.txt|10|aabbccdd11223344aabbccdd11223344|/";
        let parsed = parse_ed2k_file_uri(uri).unwrap();
        assert_eq!(parsed.name, "Hello World.txt");
    }

    #[test]
    fn parse_invalid_prefix() {
        assert!(parse_ed2k_file_uri("magnet:?xt=urn:btih:abc").is_none());
        assert!(parse_ed2k_file_uri("").is_none());
        assert!(parse_ed2k_file_uri("ed2k://|server|host|port|/").is_none());
    }

    #[test]
    fn parse_invalid_hash_length() {
        let uri = "ed2k://|file|x|10|deadbeef|/";
        assert!(parse_ed2k_file_uri(uri).is_none());
    }

    #[test]
    fn parse_invalid_hash_non_hex() {
        let uri = "ed2k://|file|x|10|zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz|/";
        assert!(parse_ed2k_file_uri(uri).is_none());
    }

    #[test]
    fn parse_invalid_size() {
        let uri =
            "ed2k://|file|x|notanumber|aabbccdd11223344aabbccdd11223344|/";
        assert!(parse_ed2k_file_uri(uri).is_none());
    }

    #[test]
    fn parse_empty_name_rejected() {
        let uri = "ed2k://|file||10|aabbccdd11223344aabbccdd11223344|/";
        assert!(parse_ed2k_file_uri(uri).is_none());
    }

    #[test]
    fn build_uri_round_trip() {
        let name = "My File.mkv";
        let size = 1024_u64;
        let hash = "AABBCCDD11223344AABBCCDD11223344";
        let uri = build_ed2k_file_uri(name, size, hash);
        let parsed = parse_ed2k_file_uri(&uri).unwrap();
        assert_eq!(parsed.name, name);
        assert_eq!(parsed.size, size);
        assert_eq!(parsed.file_hash, hash.to_lowercase());
    }
}

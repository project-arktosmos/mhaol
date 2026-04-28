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

/// Decode a 32-char hex string into a 16-byte MD4 hash array.
pub fn hex_to_md4(s: &str) -> anyhow::Result<[u8; 16]> {
    if s.len() != 32 {
        anyhow::bail!("ed2k hex hash must be 32 chars, got {}", s.len());
    }
    let bytes = hex::decode(s).map_err(|e| anyhow::anyhow!("invalid hex hash: {}", e))?;
    let mut out = [0u8; 16];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// One ed2k file part (chunk) is exactly 9.28 × 10^6 bytes.
pub const ED2K_PART_SIZE: u64 = 9_728_000;

/// One block (sub-range requested in a single REQUESTPARTS frame) is at most
/// 180 KB.
pub const ED2K_BLOCK_SIZE: u64 = 180 * 1024;

/// Number of parts a file of `size` bytes is divided into. A file of size 0
/// would have 0 parts; we always return at least 1 for non-empty files.
pub fn part_count(size: u64) -> u32 {
    if size == 0 {
        0
    } else {
        ((size + ED2K_PART_SIZE - 1) / ED2K_PART_SIZE) as u32
    }
}

/// Byte range `[start, end)` for the part at `index`, given `size`.
pub fn part_range(size: u64, index: u32) -> (u64, u64) {
    let start = (index as u64) * ED2K_PART_SIZE;
    let end = (start + ED2K_PART_SIZE).min(size);
    (start, end)
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
    fn hex_to_md4_round_trip() {
        let h = "00112233445566778899aabbccddeeff";
        let arr = hex_to_md4(h).unwrap();
        assert_eq!(arr[0], 0x00);
        assert_eq!(arr[1], 0x11);
        assert_eq!(arr[15], 0xff);
        assert_eq!(hex::encode(arr), h);
    }

    #[test]
    fn hex_to_md4_rejects_bad_input() {
        assert!(hex_to_md4("deadbeef").is_err());
        assert!(hex_to_md4("zz112233445566778899aabbccddeeff").is_err());
    }

    #[test]
    fn part_count_examples() {
        assert_eq!(part_count(0), 0);
        assert_eq!(part_count(1), 1);
        assert_eq!(part_count(ED2K_PART_SIZE), 1);
        assert_eq!(part_count(ED2K_PART_SIZE + 1), 2);
        assert_eq!(part_count(2 * ED2K_PART_SIZE), 2);
        assert_eq!(part_count(2 * ED2K_PART_SIZE + 7), 3);
    }

    #[test]
    fn part_range_examples() {
        let size = ED2K_PART_SIZE * 2 + 100;
        assert_eq!(part_range(size, 0), (0, ED2K_PART_SIZE));
        assert_eq!(
            part_range(size, 1),
            (ED2K_PART_SIZE, 2 * ED2K_PART_SIZE)
        );
        assert_eq!(
            part_range(size, 2),
            (2 * ED2K_PART_SIZE, 2 * ED2K_PART_SIZE + 100)
        );
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

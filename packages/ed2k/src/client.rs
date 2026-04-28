//! Minimal eDonkey TCP server protocol client.
//!
//! Implements just enough of the protocol to (a) connect & login to a server,
//! (b) send a basic string search and (c) parse server-status / search-result
//! frames. The network is best-effort: most public ed2k servers come and go,
//! so all calls have short timeouts and surface errors cleanly to the caller.

use anyhow::{anyhow, bail, Context, Result};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::config::Ed2kServer;
use crate::types::Ed2kSearchResult;
use crate::util::build_ed2k_file_uri;

/// eDonkey "standard" protocol byte.
const PROTO_EDONKEY: u8 = 0xE3;

/// Login opcode.
const OP_LOGIN: u8 = 0x01;
/// Server message (text broadcast).
const OP_SERVER_MESSAGE: u8 = 0x38;
/// Server status (active users / files).
const OP_SERVER_STATUS: u8 = 0x34;
/// Server identification.
const OP_SERVER_IDENT: u8 = 0x41;
/// Set ID (login response with assigned client ID).
const OP_ID_CHANGE: u8 = 0x40;
/// Search request.
const OP_SEARCH_REQUEST: u8 = 0x16;
/// Search result.
const OP_SEARCH_RESULT: u8 = 0x33;
/// Reject (server refused us).
const OP_REJECT: u8 = 0x05;
/// Offer files (announce we are a source).
const OP_OFFER_FILES: u8 = 0x15;
/// Get sources for a known file (32-bit size).
const OP_GETSOURCES: u8 = 0x9A;
/// Get sources for a known file (64-bit size, files > 4 GB).
const OP_GETSOURCES_OBFU: u8 = 0x9E;
/// Server reply with a source list.
const OP_FOUNDSOURCES: u8 = 0x42;

/// Tag types.
const TAG_HASH: u8 = 0x01;
const TAG_STRING: u8 = 0x02;
const TAG_UINT32: u8 = 0x03;
const TAG_UINT16: u8 = 0x08;
const TAG_UINT8: u8 = 0x09;
const TAG_UINT64: u8 = 0x0B;

/// Common tag names.
const CT_NAME: u8 = 0x01;
const CT_PORT: u8 = 0x0F;
const CT_VERSION: u8 = 0x11;
const CT_SERVER_FLAGS: u8 = 0x20;
const FT_FILENAME: u8 = 0x01;
const FT_FILESIZE: u8 = 0x02;
const FT_FILETYPE: u8 = 0x03;
const FT_SOURCES: u8 = 0x15;
const FT_COMPLETE_SOURCES: u8 = 0x30;

/// eMule client version tag value (eMule 0.50a == 60 == 0x3C).
const ED2K_VERSION: u32 = 0x3C;

/// Information returned by the server right after a successful login.
#[derive(Debug, Default, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub message: String,
    pub user_count: u32,
    pub file_count: u32,
    pub assigned_id: Option<u32>,
}

pub struct Ed2kClient {
    stream: TcpStream,
    info: ServerInfo,
}

impl Ed2kClient {
    /// Open a TCP connection and complete the login handshake. Returns the
    /// connected client plus any server status fields gleaned from the
    /// initial frames.
    pub async fn connect_and_login(
        server: &Ed2kServer,
        listen_port: u16,
        user_name: &str,
        connect_timeout: Duration,
    ) -> Result<Self> {
        let addr = format!("{}:{}", server.host, server.port);
        let stream = timeout(connect_timeout, TcpStream::connect(&addr))
            .await
            .with_context(|| format!("ed2k connect timeout to {}", addr))?
            .with_context(|| format!("ed2k connect failed to {}", addr))?;

        let mut client = Self {
            stream,
            info: ServerInfo {
                name: server.name.to_string(),
                ..Default::default()
            },
        };

        client.send_login(listen_port, user_name).await?;

        // Drain the initial server frames for ~2 seconds — we don't need a
        // strict handshake, just whatever metadata the server volunteers.
        let _ = timeout(Duration::from_secs(2), client.read_until_id_change())
            .await;

        Ok(client)
    }

    pub fn server_info(&self) -> &ServerInfo {
        &self.info
    }

    /// Issue a string search. Public ed2k servers vary in how strict they are
    /// about the encoding of the search expression — we send the simplest
    /// form (a single string node) which all known servers accept.
    pub async fn search(
        &mut self,
        query: &str,
        max_wait: Duration,
    ) -> Result<Vec<Ed2kSearchResult>> {
        let mut payload = Vec::with_capacity(8 + query.len());
        // Search node: 0x01 (string), 2-byte LE length, raw UTF-8 string.
        payload.push(0x01);
        write_u16_le(&mut payload, query.len() as u16);
        payload.extend_from_slice(query.as_bytes());
        self.write_frame(OP_SEARCH_REQUEST, &payload).await?;

        let mut results = Vec::new();
        let deadline = tokio::time::Instant::now() + max_wait;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            let frame = match timeout(remaining, self.read_frame()).await {
                Ok(Ok(f)) => f,
                Ok(Err(e)) => {
                    // An unknown protocol byte (e.g. compressed/extended
                    // frames we don't support) shows up as a read error.
                    // Skip it and keep waiting for a frame we can parse,
                    // rather than abandoning the whole search.
                    log::debug!("ed2k search: skipping unreadable frame: {}", e);
                    continue;
                }
                Err(_) => break,
            };
            match frame.opcode {
                OP_SEARCH_RESULT => {
                    if let Ok(parsed) = parse_search_results(&frame.payload) {
                        results.extend(parsed);
                    }
                    break;
                }
                OP_SERVER_MESSAGE => {
                    if let Some(msg) = parse_server_message(&frame.payload) {
                        self.info.message = msg;
                    }
                }
                OP_REJECT => {
                    bail!("ed2k server rejected the search");
                }
                _ => {}
            }
        }
        Ok(results)
    }

    /// Ask the server for sources of a given file. Returns the address list
    /// from the first FoundSources frame for the matching hash. Real-world
    /// public servers may legitimately reply with zero sources for unpopular
    /// files; that is not an error.
    pub async fn get_sources(
        &mut self,
        hash: &[u8; 16],
        size: u64,
        max_wait: Duration,
    ) -> Result<Vec<SocketAddrV4>> {
        let payload = encode_get_sources(hash, size);
        let opcode = if size > u32::MAX as u64 {
            OP_GETSOURCES_OBFU
        } else {
            OP_GETSOURCES
        };
        self.write_frame(opcode, &payload).await?;

        let deadline = tokio::time::Instant::now() + max_wait;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(Vec::new());
            }
            let frame = match timeout(remaining, self.read_frame()).await {
                Ok(Ok(f)) => f,
                Ok(Err(e)) => {
                    log::debug!("ed2k get_sources: skipping unreadable frame: {}", e);
                    continue;
                }
                Err(_) => return Ok(Vec::new()),
            };
            match frame.opcode {
                OP_FOUNDSOURCES => {
                    if let Ok((reply_hash, sources)) = parse_found_sources(&frame.payload) {
                        if &reply_hash == hash {
                            return Ok(sources);
                        }
                    }
                }
                OP_SERVER_MESSAGE => {
                    if let Some(msg) = parse_server_message(&frame.payload) {
                        self.info.message = msg;
                    }
                }
                OP_REJECT => bail!("ed2k server rejected the GetSources request"),
                _ => {}
            }
        }
    }

    /// Announce ourselves as a source for a list of files. Servers use this
    /// when they are asked for sources by other clients. Frame layout: 4-byte
    /// file count + per file (16-byte hash + 4-byte client ID + 2-byte port +
    /// tag count + tags).
    pub async fn offer_files(
        &mut self,
        files: &[OfferedFile],
        listen_port: u16,
    ) -> Result<()> {
        let payload = encode_offer_files(files, listen_port);
        self.write_frame(OP_OFFER_FILES, &payload).await?;
        Ok(())
    }

    async fn send_login(&mut self, listen_port: u16, user_name: &str) -> Result<()> {
        let mut payload = Vec::with_capacity(64);
        // 16-byte user hash. Public servers don't validate it, so we use a
        // stable per-process random hash.
        payload.extend_from_slice(&user_hash_bytes(user_name));
        // 4-byte client ID (0 — server assigns one back).
        payload.extend_from_slice(&0u32.to_le_bytes());
        // 2-byte advertised TCP port.
        payload.extend_from_slice(&listen_port.to_le_bytes());

        // Tag block.
        let mut tags = Vec::new();
        let mut tag_count = 0u32;

        // Name tag (string).
        write_tag_u8name_string(&mut tags, CT_NAME, user_name);
        tag_count += 1;
        // Version tag (uint32).
        write_tag_u8name_uint32(&mut tags, CT_VERSION, ED2K_VERSION);
        tag_count += 1;
        // Port tag (uint32).
        write_tag_u8name_uint32(&mut tags, CT_PORT, listen_port as u32);
        tag_count += 1;
        // Server flags tag (uint32). We deliberately do NOT advertise
        // SRVCAP_ZLIB (0x0001): we cannot decode zlib-compressed (proto 0xD4)
        // frames, and many servers compress search results when zlib is
        // negotiated, which would silently swallow every result. Leaving this
        // at 0 keeps responses in the plain 0xE3 protocol we can read.
        write_tag_u8name_uint32(&mut tags, CT_SERVER_FLAGS, 0x0000);
        tag_count += 1;

        payload.extend_from_slice(&tag_count.to_le_bytes());
        payload.extend_from_slice(&tags);

        self.write_frame(OP_LOGIN, &payload).await?;
        Ok(())
    }

    async fn read_until_id_change(&mut self) -> Result<()> {
        loop {
            let frame = self.read_frame().await?;
            match frame.opcode {
                OP_SERVER_MESSAGE => {
                    if let Some(msg) = parse_server_message(&frame.payload) {
                        self.info.message = msg;
                    }
                }
                OP_SERVER_STATUS => {
                    if frame.payload.len() >= 8 {
                        self.info.user_count =
                            u32::from_le_bytes(frame.payload[0..4].try_into().unwrap());
                        self.info.file_count =
                            u32::from_le_bytes(frame.payload[4..8].try_into().unwrap());
                    }
                }
                OP_ID_CHANGE => {
                    if frame.payload.len() >= 4 {
                        self.info.assigned_id = Some(u32::from_le_bytes(
                            frame.payload[0..4].try_into().unwrap(),
                        ));
                    }
                    return Ok(());
                }
                OP_SERVER_IDENT => {
                    // Optional — read but don't block on it.
                }
                OP_REJECT => bail!("ed2k server rejected the login"),
                _ => {}
            }
        }
    }

    async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<()> {
        let total = (payload.len() + 1) as u32;
        let mut hdr = [0u8; 6];
        hdr[0] = PROTO_EDONKEY;
        hdr[1..5].copy_from_slice(&total.to_le_bytes());
        hdr[5] = opcode;
        self.stream.write_all(&hdr).await?;
        if !payload.is_empty() {
            self.stream.write_all(payload).await?;
        }
        Ok(())
    }

    async fn read_frame(&mut self) -> Result<Frame> {
        let mut hdr = [0u8; 6];
        self.stream.read_exact(&mut hdr).await?;
        if hdr[0] != PROTO_EDONKEY {
            // Compressed (0xC5) and emule-extended (0xD4) frames exist; we
            // don't support them and skip silently by reading the announced
            // payload, then surface an error so the caller can stop.
            let total = u32::from_le_bytes(hdr[1..5].try_into().unwrap()) as usize;
            let body_len = total.saturating_sub(1);
            let mut sink = vec![0u8; body_len];
            if body_len > 0 {
                let _ = self.stream.read_exact(&mut sink).await;
            }
            return Err(anyhow!("unsupported ed2k protocol byte {:#x}", hdr[0]));
        }
        let total = u32::from_le_bytes(hdr[1..5].try_into().unwrap()) as usize;
        if total == 0 {
            return Err(anyhow!("empty ed2k frame"));
        }
        let opcode = hdr[5];
        let body_len = total - 1;
        let mut payload = vec![0u8; body_len];
        if body_len > 0 {
            self.stream.read_exact(&mut payload).await?;
        }
        Ok(Frame { opcode, payload })
    }
}

struct Frame {
    opcode: u8,
    payload: Vec<u8>,
}

// ───── encoding helpers ──────────────────────────────────────────────────

fn write_u16_le(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Write a "special" tag (1-byte name) with a string value.
fn write_tag_u8name_string(buf: &mut Vec<u8>, name: u8, value: &str) {
    // Set the high bit on the type byte to signal a 1-byte tag name.
    buf.push(TAG_STRING | 0x80);
    buf.push(name);
    write_u16_le(buf, value.len() as u16);
    buf.extend_from_slice(value.as_bytes());
}

fn write_tag_u8name_uint32(buf: &mut Vec<u8>, name: u8, value: u32) {
    buf.push(TAG_UINT32 | 0x80);
    buf.push(name);
    write_u32_le(buf, value);
}

fn write_tag_u8name_uint64(buf: &mut Vec<u8>, name: u8, value: u64) {
    buf.push(TAG_UINT64 | 0x80);
    buf.push(name);
    buf.extend_from_slice(&value.to_le_bytes());
}

/// A file we are announcing as a source.
#[derive(Debug, Clone)]
pub struct OfferedFile {
    pub hash: [u8; 16],
    pub name: String,
    pub size: u64,
    /// Best-effort file-type label (e.g. `"Video"`); when `None` we omit the
    /// tag entirely.
    pub file_type: Option<String>,
}

/// Encode a GetSources request body. Layout: 16-byte hash + 4 or 8 byte size.
fn encode_get_sources(hash: &[u8; 16], size: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(24);
    out.extend_from_slice(hash);
    if size > u32::MAX as u64 {
        out.extend_from_slice(&size.to_le_bytes());
    } else {
        out.extend_from_slice(&(size as u32).to_le_bytes());
    }
    out
}

/// Encode an OfferFiles request body.
fn encode_offer_files(files: &[OfferedFile], listen_port: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + files.len() * 64);
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for f in files {
        out.extend_from_slice(&f.hash);
        // Client ID 0 = LowID / not assigned; servers fill this in if they
        // care. Port is our advertised TCP listen port.
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&listen_port.to_le_bytes());

        let mut tags: Vec<u8> = Vec::with_capacity(64);
        let mut tag_count = 0u32;

        write_tag_u8name_string(&mut tags, FT_FILENAME, &f.name);
        tag_count += 1;
        if f.size > u32::MAX as u64 {
            write_tag_u8name_uint64(&mut tags, FT_FILESIZE, f.size);
        } else {
            write_tag_u8name_uint32(&mut tags, FT_FILESIZE, f.size as u32);
        }
        tag_count += 1;
        if let Some(ref t) = f.file_type {
            write_tag_u8name_string(&mut tags, FT_FILETYPE, t);
            tag_count += 1;
        }

        out.extend_from_slice(&tag_count.to_le_bytes());
        out.extend_from_slice(&tags);
    }
    out
}

/// Decode a FoundSources reply. Layout: 16-byte file hash + 1-byte count +
/// count × (4-byte IPv4 + 2-byte port). The IP is on the wire in big-endian
/// network order; the port is little-endian.
fn parse_found_sources(payload: &[u8]) -> Result<([u8; 16], Vec<SocketAddrV4>)> {
    let mut cur = Cursor::new(payload);
    let mut hash = [0u8; 16];
    let bytes = cur.read_bytes(16)?;
    hash.copy_from_slice(&bytes);
    let count = cur.read_u8()? as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let ip_bytes = cur.read_bytes(4)?;
        // Wire order on real ed2k servers is little-endian (reversed network
        // order) — client_id in the protocol is a 32-bit LE int interpreted
        // as ip-octet-1..4. We treat the four bytes as IP octets in the same
        // order they appear; both sides do the same and it round-trips.
        let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
        let port = cur.read_u16_le()?;
        out.push(SocketAddrV4::new(ip, port));
    }
    Ok((hash, out))
}

/// Stable 16-byte MD4 hash derived from the user name. Public servers do not
/// authenticate this hash; consistency just makes us look like a single
/// client across reconnects.
fn user_hash_bytes(user_name: &str) -> [u8; 16] {
    use md4::{Digest, Md4};
    let mut hasher = Md4::new();
    hasher.update(user_name.as_bytes());
    hasher.update(b"@mhaol/ed2k");
    let digest = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest);
    // The 6th and 15th bytes are reserved as eMule markers.
    out[5] = 14;
    out[14] = 111;
    out
}

// ───── frame body parsing ────────────────────────────────────────────────

fn parse_server_message(payload: &[u8]) -> Option<String> {
    if payload.len() < 2 {
        return None;
    }
    let len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + len {
        return None;
    }
    Some(String::from_utf8_lossy(&payload[2..2 + len]).into_owned())
}

fn parse_search_results(payload: &[u8]) -> Result<Vec<Ed2kSearchResult>> {
    let mut cursor = Cursor::new(payload);
    let count = cursor.read_u32_le()? as usize;
    let mut out = Vec::with_capacity(count.min(256));
    for _ in 0..count {
        // file hash (16 bytes)
        let hash = cursor.read_bytes(16)?;
        // client ID (4 bytes) + port (2 bytes)
        cursor.skip(4 + 2)?;
        // tag count
        let tag_count = cursor.read_u32_le()? as usize;

        let mut name = String::new();
        let mut size: u64 = 0;
        let mut sources: u32 = 0;
        let mut complete: u32 = 0;
        let mut media_type: Option<String> = None;

        for _ in 0..tag_count {
            let tag = read_tag(&mut cursor)?;
            match tag.name_id() {
                Some(FT_FILENAME) => {
                    if let TagValue::String(s) = tag.value {
                        name = s;
                    }
                }
                Some(FT_FILESIZE) => match tag.value {
                    TagValue::U32(v) => size = v as u64,
                    TagValue::U64(v) => size = v,
                    _ => {}
                },
                Some(FT_FILETYPE) => {
                    if let TagValue::String(s) = tag.value {
                        media_type = Some(s);
                    }
                }
                Some(FT_SOURCES) => {
                    if let TagValue::U32(v) = tag.value {
                        sources = v;
                    }
                }
                Some(FT_COMPLETE_SOURCES) => {
                    if let TagValue::U32(v) = tag.value {
                        complete = v;
                    }
                }
                _ => {}
            }
        }

        if name.is_empty() {
            continue;
        }
        let file_hash = hex::encode(&hash);
        let ed2k_link = build_ed2k_file_uri(&name, size, &file_hash);
        out.push(Ed2kSearchResult {
            name,
            file_hash,
            size,
            sources,
            complete_sources: complete,
            ed2k_link,
            media_type,
        });
    }
    Ok(out)
}

// ───── tiny binary cursor ────────────────────────────────────────────────

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        if self.remaining() < n {
            bail!("ed2k decode: short read");
        }
        let out = self.buf[self.pos..self.pos + n].to_vec();
        self.pos += n;
        Ok(out)
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        if self.remaining() < n {
            bail!("ed2k decode: short skip");
        }
        self.pos += n;
        Ok(())
    }
    fn read_u8(&mut self) -> Result<u8> {
        if self.remaining() < 1 {
            bail!("ed2k decode: short u8");
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }
    fn read_u16_le(&mut self) -> Result<u16> {
        if self.remaining() < 2 {
            bail!("ed2k decode: short u16");
        }
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }
    fn read_u32_le(&mut self) -> Result<u32> {
        if self.remaining() < 4 {
            bail!("ed2k decode: short u32");
        }
        let v = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }
    fn read_u64_le(&mut self) -> Result<u64> {
        if self.remaining() < 8 {
            bail!("ed2k decode: short u64");
        }
        let v = u64::from_le_bytes(self.buf[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }
    fn read_string_with_u16_len(&mut self) -> Result<String> {
        let len = self.read_u16_le()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

#[derive(Debug)]
struct Tag {
    /// Either a 1-byte name (eMule "special" form) or a longer string name.
    name_id_byte: Option<u8>,
    #[allow(dead_code)]
    name_str: Option<String>,
    value: TagValue,
}

impl Tag {
    fn name_id(&self) -> Option<u8> {
        self.name_id_byte
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum TagValue {
    Hash([u8; 16]),
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Other,
}

fn read_tag(cur: &mut Cursor<'_>) -> Result<Tag> {
    let raw_type = cur.read_u8()?;
    let short_name = raw_type & 0x80 != 0;
    let ttype = raw_type & 0x7F;

    let (name_id_byte, name_str) = if short_name {
        (Some(cur.read_u8()?), None)
    } else {
        let n = cur.read_string_with_u16_len()?;
        let id = if n.len() == 1 { Some(n.as_bytes()[0]) } else { None };
        (id, Some(n))
    };

    let value = match ttype {
        TAG_HASH => {
            let bytes = cur.read_bytes(16)?;
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            TagValue::Hash(arr)
        }
        TAG_STRING => TagValue::String(cur.read_string_with_u16_len()?),
        TAG_UINT8 => TagValue::U8(cur.read_u8()?),
        TAG_UINT16 => TagValue::U16(cur.read_u16_le()?),
        TAG_UINT32 => TagValue::U32(cur.read_u32_le()?),
        TAG_UINT64 => TagValue::U64(cur.read_u64_le()?),
        // eMule fixed-length string optimization: tag types 0x11..0x20 mean
        // a string of length (type - 0x10). The bytes follow immediately
        // with no length prefix.
        n if (0x11..=0x20).contains(&n) => {
            let len = (n - 0x10) as usize;
            let bytes = cur.read_bytes(len)?;
            TagValue::String(String::from_utf8_lossy(&bytes).into_owned())
        }
        // Anything else is opaque — propagate an error so the caller can
        // drop the frame; alternatives would risk de-syncing the stream.
        _ => return Err(anyhow!("unsupported ed2k tag type {:#x}", ttype)),
    };

    Ok(Tag {
        name_id_byte,
        name_str,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_hash_is_deterministic_per_name() {
        let a = user_hash_bytes("alice");
        let b = user_hash_bytes("alice");
        let c = user_hash_bytes("bob");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a[5], 14);
        assert_eq!(a[14], 111);
    }

    #[test]
    fn parse_server_message_basic() {
        let mut payload = Vec::new();
        let msg = "hello world";
        payload.extend_from_slice(&(msg.len() as u16).to_le_bytes());
        payload.extend_from_slice(msg.as_bytes());
        assert_eq!(parse_server_message(&payload), Some("hello world".into()));
    }

    #[test]
    fn parse_server_message_empty_too_short() {
        assert_eq!(parse_server_message(&[]), None);
        assert_eq!(parse_server_message(&[0x00]), None);
    }

    #[test]
    fn parse_search_results_zero_count() {
        let payload = 0u32.to_le_bytes();
        let parsed = parse_search_results(&payload).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_search_results_with_one_entry() {
        // Build a single-entry result manually.
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_le_bytes()); // count

        // hash (16 bytes 0x11..)
        let mut hash = [0u8; 16];
        for (i, b) in hash.iter_mut().enumerate() {
            *b = 0x10 + i as u8;
        }
        payload.extend_from_slice(&hash);
        // client id (4) + port (2)
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&0u16.to_le_bytes());
        // 3 tags: filename (string), filesize (uint32), sources (uint32)
        payload.extend_from_slice(&3u32.to_le_bytes());

        // filename
        payload.push(TAG_STRING | 0x80);
        payload.push(FT_FILENAME);
        let name = b"sample.iso";
        payload.extend_from_slice(&(name.len() as u16).to_le_bytes());
        payload.extend_from_slice(name);

        // filesize
        payload.push(TAG_UINT32 | 0x80);
        payload.push(FT_FILESIZE);
        payload.extend_from_slice(&12345u32.to_le_bytes());

        // sources
        payload.push(TAG_UINT32 | 0x80);
        payload.push(FT_SOURCES);
        payload.extend_from_slice(&42u32.to_le_bytes());

        let parsed = parse_search_results(&payload).unwrap();
        assert_eq!(parsed.len(), 1);
        let r = &parsed[0];
        assert_eq!(r.name, "sample.iso");
        assert_eq!(r.size, 12345);
        assert_eq!(r.sources, 42);
        assert_eq!(r.complete_sources, 0);
        assert_eq!(r.file_hash, hex::encode(hash));
        assert!(r.ed2k_link.contains("sample.iso"));
    }

    #[test]
    fn cursor_basic_reads() {
        let buf = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut c = Cursor::new(&buf);
        assert_eq!(c.read_u8().unwrap(), 0x01);
        assert_eq!(c.read_u16_le().unwrap(), 0x0302);
        assert_eq!(c.read_u8().unwrap(), 0x04);
        c.skip(1).unwrap();
        assert_eq!(c.remaining(), 1);
        assert_eq!(c.read_u8().unwrap(), 0x06);
    }

    #[test]
    fn cursor_short_read_errors() {
        let buf = [0x01];
        let mut c = Cursor::new(&buf);
        assert!(c.read_u32_le().is_err());
    }

    #[test]
    fn encode_get_sources_short_size() {
        let hash = [0xAA; 16];
        let body = encode_get_sources(&hash, 12345);
        assert_eq!(body.len(), 16 + 4);
        assert_eq!(&body[..16], &hash);
        assert_eq!(&body[16..20], &12345u32.to_le_bytes());
    }

    #[test]
    fn encode_get_sources_huge_size() {
        let hash = [0xBB; 16];
        let big = (u32::MAX as u64) + 17;
        let body = encode_get_sources(&hash, big);
        assert_eq!(body.len(), 16 + 8);
        assert_eq!(&body[16..24], &big.to_le_bytes());
    }

    #[test]
    fn parse_found_sources_round_trip() {
        let hash = [0x11; 16];
        let mut payload = Vec::new();
        payload.extend_from_slice(&hash);
        payload.push(2u8); // count
        // 1.2.3.4:5000
        payload.extend_from_slice(&[1, 2, 3, 4]);
        payload.extend_from_slice(&5000u16.to_le_bytes());
        // 9.8.7.6:6881
        payload.extend_from_slice(&[9, 8, 7, 6]);
        payload.extend_from_slice(&6881u16.to_le_bytes());

        let (got_hash, sources) = parse_found_sources(&payload).unwrap();
        assert_eq!(got_hash, hash);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].ip().octets(), [1, 2, 3, 4]);
        assert_eq!(sources[0].port(), 5000);
        assert_eq!(sources[1].ip().octets(), [9, 8, 7, 6]);
        assert_eq!(sources[1].port(), 6881);
    }

    #[test]
    fn parse_found_sources_zero_count() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xCC; 16]);
        payload.push(0u8);
        let (_, sources) = parse_found_sources(&payload).unwrap();
        assert!(sources.is_empty());
    }

    #[test]
    fn parse_found_sources_short_payload_errors() {
        assert!(parse_found_sources(&[0u8; 4]).is_err());
    }

    #[test]
    fn encode_offer_files_basic_layout() {
        let f = OfferedFile {
            hash: [0xAB; 16],
            name: "movie.mkv".to_string(),
            size: 1024,
            file_type: Some("Video".to_string()),
        };
        let body = encode_offer_files(&[f.clone()], 4662);
        // count
        assert_eq!(&body[0..4], &1u32.to_le_bytes());
        // hash
        assert_eq!(&body[4..20], &f.hash);
        // client id (zero) + port
        assert_eq!(&body[20..24], &0u32.to_le_bytes());
        assert_eq!(&body[24..26], &4662u16.to_le_bytes());
        // tag count == 3 (filename, filesize, filetype)
        assert_eq!(&body[26..30], &3u32.to_le_bytes());
        // Body should contain the filename bytes
        assert!(body.windows(b"movie.mkv".len())
            .any(|w| w == b"movie.mkv"));
    }

    #[test]
    fn fixed_length_string_tag_is_decoded() {
        // Build a tag block: type 0x12 means 1-byte name + fixed 2-char string.
        let mut payload = Vec::new();
        // count = 1
        payload.extend_from_slice(&1u32.to_le_bytes());
        // hash + cid + port
        payload.extend_from_slice(&[0x55; 16]);
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&0u16.to_le_bytes());
        // 1 tag: type = 0x12 (fixed string len = 2) | 0x80, name = filename, body = "ab"
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.push(0x12 | 0x80);
        payload.push(FT_FILENAME);
        payload.extend_from_slice(b"ab");

        let parsed = parse_search_results(&payload).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "ab");
    }
}

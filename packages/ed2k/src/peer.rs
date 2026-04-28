//! eDonkey client-to-client (CC) protocol.
//!
//! This module implements just enough of the peer protocol to let us pull
//! file blocks from another ed2k client. It does NOT implement upload —
//! incoming requests for our slots are answered with `OutOfPartReqs`.
//!
//! Wire format mirrors the server protocol: `proto byte (0xE3) | total len
//! u32 LE | opcode u8 | body`. Compressed (`0xC5`) and emule-extended
//! (`0xD4`) frames are received but currently dropped — we ask peers for
//! uncompressed parts and most respect that.
//!
//! TODO: support compressed parts (OP_COMPRESSEDPART = 0x40 / 0xA2). For
//! v1 we drop them; the peer reschedules the missing range on its own.

use anyhow::{bail, Context, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::util::ED2K_BLOCK_SIZE;

const PROTO_EDONKEY: u8 = 0xE3;

// ─── opcodes ─────────────────────────────────────────────────────────────
pub const OP_HELLO: u8 = 0x01;
pub const OP_HELLOANSWER: u8 = 0x4C;
pub const OP_REQUESTFILENAME: u8 = 0x58;
pub const OP_REQFILENAMEANSWER: u8 = 0x59;
pub const OP_FILEREQANSNOFIL: u8 = 0x48;
pub const OP_SETREQFILEID: u8 = 0x4F;
pub const OP_FILESTATUS: u8 = 0x50;
pub const OP_HASHSETREQUEST: u8 = 0x51;
pub const OP_HASHSETANSWER: u8 = 0x52;
pub const OP_STARTUPLOADREQ: u8 = 0x54;
pub const OP_ACCEPTUPLOADREQ: u8 = 0x55;
pub const OP_CANCELTRANSFER: u8 = 0x56;
pub const OP_OUTOFPARTREQS: u8 = 0x57;
pub const OP_QUEUERANK: u8 = 0x5C;
pub const OP_QUEUERANKING: u8 = 0x60;
pub const OP_REQUESTPARTS: u8 = 0x47;
pub const OP_REQUESTPARTS_I64: u8 = 0xA1;
pub const OP_SENDINGPART: u8 = 0x46;
pub const OP_SENDINGPART_I64: u8 = 0xA0;
pub const OP_END_OF_DOWNLOAD: u8 = 0x49;
pub const OP_COMPRESSEDPART: u8 = 0x40;
pub const OP_COMPRESSEDPART_I64: u8 = 0xA2;

// Tag types (mirrored from client.rs — duplicated here to keep this module
// self-contained for testing).
const TAG_STRING: u8 = 0x02;
const TAG_UINT32: u8 = 0x03;

// Common tag names used in HELLO bodies.
const CT_NAME: u8 = 0x01;
const CT_PORT: u8 = 0x0F;
const CT_VERSION: u8 = 0x11;
const CT_MULEVERSION: u8 = 0xFB;

const ED2K_VERSION: u32 = 0x3C;
const MULE_VERSION: u32 = 0x40;

/// One peer-to-peer frame.
#[derive(Debug, Clone)]
pub struct Frame {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

/// HELLO body fields (used both on send and on receive).
#[derive(Debug, Clone)]
pub struct HelloBody {
    pub user_hash: [u8; 16],
    pub client_id: u32,
    pub port: u16,
    pub server_ip: u32,
    pub server_port: u16,
}

/// One byte range to request from a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRange {
    pub start: u64,
    pub end: u64,
}

/// Decoded body of a SENDINGPART (or its 64-bit variant) frame.
#[derive(Debug, Clone)]
pub struct SendingPart {
    pub hash: [u8; 16],
    pub start: u64,
    pub end: u64,
    pub data: Vec<u8>,
}

/// File status reply: either "no file" or a part-availability bitmap.
#[derive(Debug, Clone)]
pub enum FileStatus {
    NoFile,
    Status {
        hash: [u8; 16],
        /// Number of parts according to the peer. Zero means the peer claims
        /// to have all parts (no bitmap follows on the wire).
        part_count: u16,
        /// Bitmap, LSB-first per byte. Empty if `part_count == 0`.
        bitmap: Vec<u8>,
    },
}

/// Peer-side connection wrapper. Owns the TcpStream and exposes typed
/// send/recv helpers. Construct via `connect`.
pub struct PeerConnection {
    pub addr: SocketAddr,
    stream: TcpStream,
    pub last_queue_rank: Option<u16>,
}

impl PeerConnection {
    pub async fn connect(addr: SocketAddr, connect_timeout: Duration) -> Result<Self> {
        let stream = timeout(connect_timeout, TcpStream::connect(addr))
            .await
            .with_context(|| format!("ed2k peer connect timeout to {}", addr))?
            .with_context(|| format!("ed2k peer connect failed to {}", addr))?;
        let _ = stream.set_nodelay(true);
        Ok(Self {
            addr,
            stream,
            last_queue_rank: None,
        })
    }

    pub async fn send_hello(&mut self, hello: &HelloBody) -> Result<()> {
        let body = encode_hello(hello, /*include_hash_marker=*/ true);
        self.write_frame(OP_HELLO, &body).await
    }

    pub async fn recv_frame(&mut self, idle_timeout: Duration) -> Result<Frame> {
        let frame = timeout(idle_timeout, read_frame(&mut self.stream))
            .await
            .with_context(|| "ed2k peer read timeout")??;
        if frame.opcode == OP_QUEUERANK || frame.opcode == OP_QUEUERANKING {
            if frame.payload.len() >= 2 {
                self.last_queue_rank =
                    Some(u16::from_le_bytes([frame.payload[0], frame.payload[1]]));
            }
        }
        Ok(frame)
    }

    pub async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<()> {
        write_frame(&mut self.stream, opcode, payload).await
    }

    pub async fn shutdown(&mut self) {
        let _ = self.stream.shutdown().await;
    }
}

// ─── frame I/O ───────────────────────────────────────────────────────────

async fn write_frame<W: AsyncWriteExt + Unpin>(
    w: &mut W,
    opcode: u8,
    payload: &[u8],
) -> Result<()> {
    let total = (payload.len() + 1) as u32;
    let mut hdr = [0u8; 6];
    hdr[0] = PROTO_EDONKEY;
    hdr[1..5].copy_from_slice(&total.to_le_bytes());
    hdr[5] = opcode;
    w.write_all(&hdr).await?;
    if !payload.is_empty() {
        w.write_all(payload).await?;
    }
    Ok(())
}

async fn read_frame<R: AsyncReadExt + Unpin>(r: &mut R) -> Result<Frame> {
    let mut hdr = [0u8; 6];
    r.read_exact(&mut hdr).await?;
    let total = u32::from_le_bytes(hdr[1..5].try_into().unwrap()) as usize;
    if total == 0 {
        bail!("ed2k peer: empty frame");
    }
    if total > 64 * 1024 * 1024 {
        bail!("ed2k peer: oversized frame {} bytes", total);
    }
    let opcode = hdr[5];
    let body_len = total - 1;
    let mut payload = vec![0u8; body_len];
    if body_len > 0 {
        r.read_exact(&mut payload).await?;
    }
    if hdr[0] != PROTO_EDONKEY {
        // Compressed / extended frame — surface as a special opcode the
        // caller can ignore. Returning an error would force the connection
        // closed even when the peer is just sending a chatty extension.
        return Ok(Frame {
            opcode: 0,
            payload: Vec::new(),
        });
    }
    Ok(Frame { opcode, payload })
}

// ─── encoders ────────────────────────────────────────────────────────────

/// Encode a HELLO or HELLOANSWER body. The HELLO frame has a leading
/// hash-length marker byte (0x10); HELLOANSWER skips it.
pub fn encode_hello(h: &HelloBody, include_hash_marker: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    if include_hash_marker {
        out.push(0x10);
    }
    out.extend_from_slice(&h.user_hash);
    out.extend_from_slice(&h.client_id.to_le_bytes());
    out.extend_from_slice(&h.port.to_le_bytes());

    // Tags: NAME (string), VERSION (uint32), PORT (uint32), MULEVERSION
    // (uint32). 4 tags total.
    let mut tags = Vec::new();
    let mut tag_count = 0u32;

    write_short_tag_string(&mut tags, CT_NAME, "mhaol");
    tag_count += 1;
    write_short_tag_u32(&mut tags, CT_VERSION, ED2K_VERSION);
    tag_count += 1;
    write_short_tag_u32(&mut tags, CT_PORT, h.port as u32);
    tag_count += 1;
    write_short_tag_u32(&mut tags, CT_MULEVERSION, MULE_VERSION);
    tag_count += 1;

    out.extend_from_slice(&tag_count.to_le_bytes());
    out.extend_from_slice(&tags);

    // Server IP + port.
    out.extend_from_slice(&h.server_ip.to_le_bytes());
    out.extend_from_slice(&h.server_port.to_le_bytes());
    out
}

fn write_short_tag_string(buf: &mut Vec<u8>, name: u8, value: &str) {
    buf.push(TAG_STRING | 0x80);
    buf.push(name);
    buf.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buf.extend_from_slice(value.as_bytes());
}

fn write_short_tag_u32(buf: &mut Vec<u8>, name: u8, value: u32) {
    buf.push(TAG_UINT32 | 0x80);
    buf.push(name);
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Body of REQUESTFILENAME / SETREQFILEID / HASHSETREQUEST / STARTUPLOADREQ:
/// just the 16-byte hash.
pub fn encode_hash_only(hash: &[u8; 16]) -> Vec<u8> {
    hash.to_vec()
}

/// Body of REQUESTPARTS (32-bit) — exactly 3 ranges. We pad with the same
/// range repeated when fewer are wanted (eMule does the same; treats the
/// duplicate as already covered).
pub fn encode_request_parts_32(hash: &[u8; 16], ranges: &[BlockRange; 3]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + 24);
    out.extend_from_slice(hash);
    for r in ranges {
        out.extend_from_slice(&(r.start as u32).to_le_bytes());
    }
    for r in ranges {
        out.extend_from_slice(&(r.end as u32).to_le_bytes());
    }
    out
}

/// Body of REQUESTPARTS (64-bit) — same shape with 8-byte offsets.
pub fn encode_request_parts_64(hash: &[u8; 16], ranges: &[BlockRange; 3]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + 48);
    out.extend_from_slice(hash);
    for r in ranges {
        out.extend_from_slice(&r.start.to_le_bytes());
    }
    for r in ranges {
        out.extend_from_slice(&r.end.to_le_bytes());
    }
    out
}

/// Pad an arbitrary slice of `BlockRange` up to 3 entries by repeating the
/// last entry. Returns `None` if `ranges` is empty.
pub fn pad_ranges(ranges: &[BlockRange]) -> Option<[BlockRange; 3]> {
    if ranges.is_empty() {
        return None;
    }
    let last = *ranges.last().unwrap();
    let r0 = ranges.first().copied().unwrap_or(last);
    let r1 = ranges.get(1).copied().unwrap_or(last);
    let r2 = ranges.get(2).copied().unwrap_or(last);
    Some([r0, r1, r2])
}

/// Compute the appropriate REQUESTPARTS flavour for the given file size.
pub fn want_64bit_offsets(file_size: u64) -> bool {
    file_size > u32::MAX as u64
}

// ─── decoders ────────────────────────────────────────────────────────────

pub fn decode_hello(payload: &[u8], expect_hash_marker: bool) -> Result<HelloBody> {
    let mut cur = Cursor::new(payload);
    if expect_hash_marker {
        let marker = cur.read_u8()?;
        if marker != 0x10 {
            // Some clients omit the marker even on HELLO. Don't bail, just
            // rewind so the hash byte is still there.
            cur.unread();
        }
    }
    let user_hash_bytes = cur.read_bytes(16)?;
    let mut user_hash = [0u8; 16];
    user_hash.copy_from_slice(&user_hash_bytes);
    let client_id = cur.read_u32_le()?;
    let port = cur.read_u16_le()?;
    let tag_count = cur.read_u32_le()?;
    for _ in 0..tag_count {
        if !skip_tag(&mut cur) {
            break;
        }
    }
    let server_ip = cur.read_u32_le().unwrap_or(0);
    let server_port = cur.read_u16_le().unwrap_or(0);
    Ok(HelloBody {
        user_hash,
        client_id,
        port,
        server_ip,
        server_port,
    })
}

/// Skip one tag in `cur`, returning false on malformed input. Used in HELLO
/// decoding where we don't actually care about the tag values.
fn skip_tag(cur: &mut Cursor<'_>) -> bool {
    let raw_type = match cur.read_u8() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let short_name = raw_type & 0x80 != 0;
    let ttype = raw_type & 0x7F;
    if short_name {
        if cur.read_u8().is_err() {
            return false;
        }
    } else {
        let len = match cur.read_u16_le() {
            Ok(v) => v,
            Err(_) => return false,
        };
        if cur.skip(len as usize).is_err() {
            return false;
        }
    }
    match ttype {
        0x01 => cur.skip(16).is_ok(),
        0x02 => {
            let len = match cur.read_u16_le() {
                Ok(v) => v,
                Err(_) => return false,
            };
            cur.skip(len as usize).is_ok()
        }
        0x03 => cur.skip(4).is_ok(),
        0x08 => cur.skip(2).is_ok(),
        0x09 => cur.skip(1).is_ok(),
        0x0B => cur.skip(8).is_ok(),
        n if (0x11..=0x20).contains(&n) => cur.skip((n - 0x10) as usize).is_ok(),
        _ => false,
    }
}

/// Decode a REQFILENAMEANSWER body: 16-byte hash + 4-byte name length + name.
pub fn decode_filename_answer(payload: &[u8]) -> Result<([u8; 16], String)> {
    let mut cur = Cursor::new(payload);
    let hash_bytes = cur.read_bytes(16)?;
    let mut hash = [0u8; 16];
    hash.copy_from_slice(&hash_bytes);
    let len = cur.read_u32_le()? as usize;
    let name_bytes = cur.read_bytes(len)?;
    let name = String::from_utf8_lossy(&name_bytes).into_owned();
    Ok((hash, name))
}

/// Decode a FILESTATUS body. Returns NoFile if the peer's part_count is
/// zero AND no bitmap follows; otherwise returns the bitmap.
pub fn decode_file_status(payload: &[u8]) -> Result<FileStatus> {
    let mut cur = Cursor::new(payload);
    let hash_bytes = cur.read_bytes(16)?;
    let mut hash = [0u8; 16];
    hash.copy_from_slice(&hash_bytes);
    let part_count = cur.read_u16_le()?;
    if part_count == 0 {
        // "Has all parts" — there is no bitmap.
        return Ok(FileStatus::Status {
            hash,
            part_count: 0,
            bitmap: Vec::new(),
        });
    }
    let bitmap_len = ((part_count as usize) + 7) / 8;
    let bitmap_bytes = cur.read_bytes(bitmap_len)?;
    Ok(FileStatus::Status {
        hash,
        part_count,
        bitmap: bitmap_bytes,
    })
}

/// Decode a HASHSETANSWER body: 16-byte file hash + 2-byte part count +
/// part_count × 16-byte MD4 part hashes.
pub fn decode_hashset_answer(payload: &[u8]) -> Result<([u8; 16], Vec<[u8; 16]>)> {
    let mut cur = Cursor::new(payload);
    let hash_bytes = cur.read_bytes(16)?;
    let mut file_hash = [0u8; 16];
    file_hash.copy_from_slice(&hash_bytes);
    let part_count = cur.read_u16_le()? as usize;
    let mut hashes = Vec::with_capacity(part_count);
    for _ in 0..part_count {
        let b = cur.read_bytes(16)?;
        let mut h = [0u8; 16];
        h.copy_from_slice(&b);
        hashes.push(h);
    }
    Ok((file_hash, hashes))
}

/// Decode a SENDINGPART body for a 32-bit-offset file.
pub fn decode_sending_part_32(payload: &[u8]) -> Result<SendingPart> {
    let mut cur = Cursor::new(payload);
    let hash_bytes = cur.read_bytes(16)?;
    let mut hash = [0u8; 16];
    hash.copy_from_slice(&hash_bytes);
    let start = cur.read_u32_le()? as u64;
    let end = cur.read_u32_le()? as u64;
    if end < start {
        bail!("ed2k sending part: end {} before start {}", end, start);
    }
    let len = (end - start) as usize;
    if cur.remaining() < len {
        bail!("ed2k sending part: short payload");
    }
    let data = cur.read_bytes(len)?;
    Ok(SendingPart {
        hash,
        start,
        end,
        data,
    })
}

/// Decode a SENDINGPART_I64 body for a 64-bit-offset file.
pub fn decode_sending_part_64(payload: &[u8]) -> Result<SendingPart> {
    let mut cur = Cursor::new(payload);
    let hash_bytes = cur.read_bytes(16)?;
    let mut hash = [0u8; 16];
    hash.copy_from_slice(&hash_bytes);
    let start = cur.read_u64_le()?;
    let end = cur.read_u64_le()?;
    if end < start {
        bail!("ed2k sending part 64: end before start");
    }
    let len = (end - start) as usize;
    if cur.remaining() < len {
        bail!("ed2k sending part 64: short payload");
    }
    let data = cur.read_bytes(len)?;
    Ok(SendingPart {
        hash,
        start,
        end,
        data,
    })
}

// ─── helpers ─────────────────────────────────────────────────────────────

/// Slice a single missing range into ≤ ED2K_BLOCK_SIZE chunks, then bundle
/// the first 3 into a REQUESTPARTS payload.
pub fn ranges_for_request(missing: &[(u64, u64)]) -> Vec<BlockRange> {
    let mut out = Vec::new();
    for &(s, e) in missing {
        let mut cur = s;
        while cur < e {
            let end = (cur + ED2K_BLOCK_SIZE).min(e);
            out.push(BlockRange { start: cur, end });
            cur = end;
            if out.len() >= 3 {
                return out;
            }
        }
        if out.len() >= 3 {
            return out;
        }
    }
    out
}

// ─── tiny binary cursor (private mirror of client.rs) ────────────────────

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
            bail!("ed2k peer decode: short read");
        }
        let out = self.buf[self.pos..self.pos + n].to_vec();
        self.pos += n;
        Ok(out)
    }
    fn skip(&mut self, n: usize) -> Result<()> {
        if self.remaining() < n {
            bail!("ed2k peer decode: short skip");
        }
        self.pos += n;
        Ok(())
    }
    fn read_u8(&mut self) -> Result<u8> {
        if self.remaining() < 1 {
            bail!("ed2k peer decode: short u8");
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }
    fn read_u16_le(&mut self) -> Result<u16> {
        if self.remaining() < 2 {
            bail!("ed2k peer decode: short u16");
        }
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }
    fn read_u32_le(&mut self) -> Result<u32> {
        if self.remaining() < 4 {
            bail!("ed2k peer decode: short u32");
        }
        let v = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }
    fn read_u64_le(&mut self) -> Result<u64> {
        if self.remaining() < 8 {
            bail!("ed2k peer decode: short u64");
        }
        let v = u64::from_le_bytes(self.buf[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }
    fn unread(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }
}

/// Helper used by tests and by callers who want to build a hello body
/// without recomputing it. Returns `(user_hash, advertised_port)` from a
/// stable seed string.
pub fn hello_body_for(user_seed: &str, port: u16) -> HelloBody {
    use md4::{Digest, Md4};
    let mut hasher = Md4::new();
    hasher.update(user_seed.as_bytes());
    hasher.update(b"@mhaol/ed2k/peer");
    let digest = hasher.finalize();
    let mut user_hash = [0u8; 16];
    user_hash.copy_from_slice(&digest);
    user_hash[5] = 14;
    user_hash[14] = 111;
    HelloBody {
        user_hash,
        client_id: 0,
        port,
        server_ip: 0,
        server_port: 0,
    }
}

#[allow(dead_code)]
const _BLOCK_SIZE_SANITY: u64 = ED2K_BLOCK_SIZE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_round_trip_with_marker() {
        let h = HelloBody {
            user_hash: [0x55; 16],
            client_id: 0xCAFEBABE,
            port: 4662,
            server_ip: 0x01020304,
            server_port: 4242,
        };
        let body = encode_hello(&h, true);
        let decoded = decode_hello(&body, true).unwrap();
        assert_eq!(decoded.user_hash, h.user_hash);
        assert_eq!(decoded.client_id, h.client_id);
        assert_eq!(decoded.port, h.port);
        assert_eq!(decoded.server_ip, h.server_ip);
        assert_eq!(decoded.server_port, h.server_port);
    }

    #[test]
    fn hello_answer_round_trip_no_marker() {
        let h = HelloBody {
            user_hash: [0xAB; 16],
            client_id: 1,
            port: 4663,
            server_ip: 0,
            server_port: 0,
        };
        let body = encode_hello(&h, false);
        let decoded = decode_hello(&body, false).unwrap();
        assert_eq!(decoded.user_hash, h.user_hash);
        assert_eq!(decoded.port, h.port);
    }

    #[test]
    fn encode_hash_only_basic() {
        let h = [0xEE; 16];
        let body = encode_hash_only(&h);
        assert_eq!(body, h.to_vec());
    }

    #[test]
    fn encode_request_parts_32_layout() {
        let h = [0x99; 16];
        let r = [
            BlockRange { start: 0, end: 100 },
            BlockRange { start: 100, end: 250 },
            BlockRange { start: 1000, end: 1500 },
        ];
        let body = encode_request_parts_32(&h, &r);
        assert_eq!(body.len(), 16 + 12 + 12);
        assert_eq!(&body[..16], &h);
        assert_eq!(&body[16..20], &0u32.to_le_bytes());
        assert_eq!(&body[20..24], &100u32.to_le_bytes());
        assert_eq!(&body[24..28], &1000u32.to_le_bytes());
        assert_eq!(&body[28..32], &100u32.to_le_bytes());
        assert_eq!(&body[32..36], &250u32.to_le_bytes());
        assert_eq!(&body[36..40], &1500u32.to_le_bytes());
    }

    #[test]
    fn encode_request_parts_64_layout() {
        let h = [0x77; 16];
        let r = [
            BlockRange { start: 0, end: 1 },
            BlockRange { start: 0, end: 1 },
            BlockRange {
                start: u32::MAX as u64 + 1,
                end: u32::MAX as u64 + 100,
            },
        ];
        let body = encode_request_parts_64(&h, &r);
        assert_eq!(body.len(), 16 + 24 + 24);
    }

    #[test]
    fn pad_ranges_repeats_last() {
        let r = pad_ranges(&[BlockRange { start: 0, end: 5 }]).unwrap();
        assert_eq!(r[0], r[1]);
        assert_eq!(r[1], r[2]);
        assert!(pad_ranges(&[]).is_none());
    }

    #[test]
    fn want_64bit_offsets_threshold() {
        assert!(!want_64bit_offsets(u32::MAX as u64));
        assert!(want_64bit_offsets(u32::MAX as u64 + 1));
    }

    #[test]
    fn decode_filename_answer_basic() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x33; 16]);
        let name = "movie.mkv";
        payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
        payload.extend_from_slice(name.as_bytes());

        let (hash, decoded) = decode_filename_answer(&payload).unwrap();
        assert_eq!(hash, [0x33; 16]);
        assert_eq!(decoded, name);
    }

    #[test]
    fn decode_file_status_with_bitmap() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xAA; 16]);
        payload.extend_from_slice(&5u16.to_le_bytes()); // 5 parts → 1 byte bitmap
        payload.push(0b00010110); // parts 1, 2, 4 present
        let st = decode_file_status(&payload).unwrap();
        match st {
            FileStatus::Status {
                hash,
                part_count,
                bitmap,
            } => {
                assert_eq!(hash, [0xAA; 16]);
                assert_eq!(part_count, 5);
                assert_eq!(bitmap, vec![0b00010110]);
            }
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn decode_file_status_part_count_zero_means_complete() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xBB; 16]);
        payload.extend_from_slice(&0u16.to_le_bytes());
        let st = decode_file_status(&payload).unwrap();
        match st {
            FileStatus::Status {
                part_count, bitmap, ..
            } => {
                assert_eq!(part_count, 0);
                assert!(bitmap.is_empty());
            }
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn decode_hashset_answer_basic() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xCC; 16]);
        payload.extend_from_slice(&3u16.to_le_bytes());
        for i in 0..3 {
            payload.extend_from_slice(&[i as u8; 16]);
        }
        let (hash, parts) = decode_hashset_answer(&payload).unwrap();
        assert_eq!(hash, [0xCC; 16]);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[2][0], 2);
    }

    #[test]
    fn decode_sending_part_32_basic() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xDD; 16]);
        payload.extend_from_slice(&100u32.to_le_bytes()); // start
        payload.extend_from_slice(&110u32.to_le_bytes()); // end
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let s = decode_sending_part_32(&payload).unwrap();
        assert_eq!(s.start, 100);
        assert_eq!(s.end, 110);
        assert_eq!(s.data.len(), 10);
        assert_eq!(s.data[9], 10);
    }

    #[test]
    fn decode_sending_part_32_short_data_errors() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0u8; 16]);
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&100u32.to_le_bytes());
        payload.extend_from_slice(&[1, 2, 3]);
        assert!(decode_sending_part_32(&payload).is_err());
    }

    #[test]
    fn decode_sending_part_64_basic() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xEE; 16]);
        let start: u64 = (u32::MAX as u64) + 1;
        let end = start + 4;
        payload.extend_from_slice(&start.to_le_bytes());
        payload.extend_from_slice(&end.to_le_bytes());
        payload.extend_from_slice(&[7, 7, 7, 7]);
        let s = decode_sending_part_64(&payload).unwrap();
        assert_eq!(s.start, start);
        assert_eq!(s.end, end);
        assert_eq!(s.data, vec![7, 7, 7, 7]);
    }

    #[test]
    fn ranges_for_request_caps_at_three() {
        let missing = vec![(0, ED2K_BLOCK_SIZE * 5)];
        let ranges = ranges_for_request(&missing);
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0].start, 0);
        assert_eq!(ranges[0].end, ED2K_BLOCK_SIZE);
    }

    #[test]
    fn ranges_for_request_handles_small_tail() {
        let missing = vec![(0, 100)];
        let ranges = ranges_for_request(&missing);
        assert_eq!(ranges, vec![BlockRange { start: 0, end: 100 }]);
    }

    #[test]
    fn ranges_for_request_walks_multiple_gaps() {
        let missing = vec![(0, 50), (200, 300)];
        let ranges = ranges_for_request(&missing);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].end - ranges[0].start, 50);
        assert_eq!(ranges[1].start, 200);
        assert_eq!(ranges[1].end, 300);
    }

    #[test]
    fn hello_body_for_is_deterministic() {
        let a = hello_body_for("alice", 4662);
        let b = hello_body_for("alice", 4662);
        let c = hello_body_for("bob", 4662);
        assert_eq!(a.user_hash, b.user_hash);
        assert_ne!(a.user_hash, c.user_hash);
        assert_eq!(a.user_hash[5], 14);
        assert_eq!(a.user_hash[14], 111);
    }

    #[test]
    fn decode_hello_recovers_from_missing_marker() {
        // Hello body without the 0x10 marker but flagged as "expect marker".
        let h = HelloBody {
            user_hash: [0x42; 16],
            client_id: 7,
            port: 1234,
            server_ip: 0,
            server_port: 0,
        };
        let body = encode_hello(&h, false);
        let decoded = decode_hello(&body, true).unwrap();
        assert_eq!(decoded.user_hash, h.user_hash);
        assert_eq!(decoded.port, h.port);
    }
}

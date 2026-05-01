//! Raw probe: open TCP, send a hand-crafted ed2k login, then dump the first
//! ~512 bytes the server replies with (hex + ASCII). Helps diagnose what
//! protocol byte / opcode stream the server actually sends so we can tell
//! "they're rejecting us" from "they're answering with frames we can't read".
//!
//! Run with: cargo run -p mhaol-ed2k --example probe_raw

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use mhaol_ed2k::config::DEFAULT_SERVERS;

const PROTO_EDONKEY: u8 = 0xE3;
const OP_LOGIN: u8 = 0x01;

fn build_login_frame(listen_port: u16, user_name: &str) -> Vec<u8> {
    let mut body = Vec::with_capacity(64);

    // user hash (16 bytes deterministic)
    let mut hash = [0u8; 16];
    for (i, b) in hash.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(17).wrapping_add(3);
    }
    hash[5] = 14;
    hash[14] = 111;
    body.extend_from_slice(&hash);

    // client id (4 bytes, 0 = ask for assignment)
    body.extend_from_slice(&0u32.to_le_bytes());
    // port (2 bytes LE)
    body.extend_from_slice(&listen_port.to_le_bytes());

    // tag block: name, version, port, server_flags
    let mut tags: Vec<u8> = Vec::new();
    let mut tag_count = 0u32;

    // CT_NAME (0x01) string
    tags.push(0x02 | 0x80);
    tags.push(0x01);
    tags.extend_from_slice(&(user_name.len() as u16).to_le_bytes());
    tags.extend_from_slice(user_name.as_bytes());
    tag_count += 1;

    // CT_VERSION (0x11) uint32 = 0x3C
    tags.push(0x03 | 0x80);
    tags.push(0x11);
    tags.extend_from_slice(&0x3Cu32.to_le_bytes());
    tag_count += 1;

    // CT_PORT (0x0F) uint32
    tags.push(0x03 | 0x80);
    tags.push(0x0F);
    tags.extend_from_slice(&(listen_port as u32).to_le_bytes());
    tag_count += 1;

    // CT_SERVER_FLAGS (0x20) uint32 = 0 (no zlib, no extensions)
    tags.push(0x03 | 0x80);
    tags.push(0x20);
    tags.extend_from_slice(&0u32.to_le_bytes());
    tag_count += 1;

    body.extend_from_slice(&tag_count.to_le_bytes());
    body.extend_from_slice(&tags);

    let total = (body.len() + 1) as u32;
    let mut frame = Vec::with_capacity(6 + body.len());
    frame.push(PROTO_EDONKEY);
    frame.extend_from_slice(&total.to_le_bytes());
    frame.push(OP_LOGIN);
    frame.extend_from_slice(&body);
    frame
}

fn dump_hex(label: &str, buf: &[u8]) {
    println!("  {} ({} bytes)", label, buf.len());
    for (i, chunk) in buf.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| if (32..127).contains(&b) { b as char } else { '.' })
            .collect();
        println!("    {:04x}  {:<48}  {}", i * 16, hex.join(" "), ascii);
    }
}

#[tokio::main]
async fn main() {
    for s in DEFAULT_SERVERS {
        println!("\n=== {} ({}:{}) ===", s.name, s.host, s.port);
        let addr = format!("{}:{}", s.host, s.port);

        let stream = match timeout(Duration::from_secs(8), TcpStream::connect(&addr)).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                println!("  connect FAILED: {}", e);
                continue;
            }
            Err(_) => {
                println!("  connect TIMEOUT");
                continue;
            }
        };
        println!("  TCP connected");
        let mut stream = stream;

        let frame = build_login_frame(4662, "mhaol");
        println!("  sending login ({} bytes)", frame.len());
        if let Err(e) = stream.write_all(&frame).await {
            println!("  write FAILED: {}", e);
            continue;
        }

        // Read up to 1024 bytes within 4 seconds.
        let mut buf = vec![0u8; 1024];
        let mut total = 0usize;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match timeout(remaining, stream.read(&mut buf[total..])).await {
                Ok(Ok(0)) => {
                    println!("  server closed connection after {} bytes", total);
                    break;
                }
                Ok(Ok(n)) => {
                    total += n;
                    if total >= buf.len() {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    println!("  read FAILED: {}", e);
                    break;
                }
                Err(_) => break,
            }
        }
        if total > 0 {
            dump_hex("first bytes from server", &buf[..total]);
        } else {
            println!("  no bytes received");
        }
    }
}

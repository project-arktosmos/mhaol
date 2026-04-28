//! Per-file download driver. Spawns one tokio task that:
//!  1. opens / creates the part file on disk,
//!  2. periodically asks the server for sources,
//!  3. maintains up to N peer worker tasks that handshake, request and
//!     write blocks,
//!  4. updates the shared `Ed2kFileInfo` so the API/SSE stream sees real
//!     progress numbers.
//!
//! Design note on concurrency: we keep all on-disk state behind a
//! `tokio::sync::Mutex<PartFile>` per download. That's fine because each
//! worker only acquires the mutex briefly to merge a written block. The
//! actual TCP I/O happens unsynchronised.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use crate::client::{Ed2kClient, OfferedFile};
use crate::config::{Ed2kConfig, DEFAULT_SERVERS};
use crate::partfile::PartFile;
use crate::peer::{
    decode_file_status, decode_filename_answer, decode_hashset_answer,
    decode_sending_part_32, decode_sending_part_64, encode_hash_only,
    encode_request_parts_32, encode_request_parts_64, hello_body_for, pad_ranges,
    ranges_for_request, want_64bit_offsets, FileStatus, PeerConnection,
    OP_ACCEPTUPLOADREQ, OP_CANCELTRANSFER, OP_END_OF_DOWNLOAD,
    OP_FILEREQANSNOFIL, OP_FILESTATUS, OP_HASHSETANSWER, OP_HASHSETREQUEST,
    OP_HELLO, OP_HELLOANSWER, OP_OUTOFPARTREQS, OP_QUEUERANK, OP_QUEUERANKING,
    OP_REQFILENAMEANSWER, OP_REQUESTFILENAME, OP_REQUESTPARTS,
    OP_REQUESTPARTS_I64, OP_SENDINGPART, OP_SENDINGPART_I64, OP_SETREQFILEID,
    OP_STARTUPLOADREQ, PROTO_EDONKEY, PROTO_EMULE,
};
use crate::types::Ed2kState;
use crate::util::{hex_to_md4, part_count, part_range, ED2K_PART_SIZE};
use crate::Ed2kManager;

const MAX_PEERS_PER_FILE: usize = 8;
const PEER_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const PEER_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const SOURCE_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);
const PROGRESS_TICK: Duration = Duration::from_secs(1);

/// Lightweight cancellation primitive — we don't need a full
/// `tokio_util::sync::CancellationToken` and keeping deps small is a
/// project goal.
#[derive(Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }
}

/// One running download. Held inside the manager so it can cancel / await.
pub struct DownloadHandle {
    pub task: JoinHandle<()>,
    pub cancel: CancelToken,
}

impl DownloadHandle {
    pub async fn cancel_and_wait(self) {
        self.cancel.cancel();
        let _ = self.task.await;
    }
}

/// Information needed to drive one download.
#[derive(Clone)]
pub struct DownloadSpec {
    pub file_hash_hex: String,
    pub file_hash: [u8; 16],
    pub name: String,
    pub size: u64,
    pub download_dir: PathBuf,
    pub final_path: PathBuf,
}

impl DownloadSpec {
    pub fn from_parts(
        file_hash_hex: &str,
        name: &str,
        size: u64,
        download_dir: PathBuf,
    ) -> Result<Self> {
        let file_hash = hex_to_md4(file_hash_hex)?;
        let final_path = download_dir.join(name);
        Ok(Self {
            file_hash_hex: file_hash_hex.to_string(),
            file_hash,
            name: name.to_string(),
            size,
            download_dir,
            final_path,
        })
    }
}

/// Spawn the download driver task and return the handle.
pub fn spawn_download(
    manager: Weak<Ed2kManager>,
    spec: DownloadSpec,
    config: Ed2kConfig,
) -> DownloadHandle {
    let cancel = CancelToken::new();
    let task_cancel = cancel.clone();
    let task = tokio::spawn(async move {
        if let Err(e) = run_download(manager.clone(), spec.clone(), config, task_cancel).await {
            log::warn!("ed2k download task for {} ended: {}", spec.name, e);
            if let Some(mgr) = manager.upgrade() {
                mgr.update_file(&spec.file_hash_hex, |f| {
                    if !matches!(f.state, Ed2kState::Paused | Ed2kState::Seeding) {
                        f.state = Ed2kState::Error;
                    }
                });
            }
        }
    });
    DownloadHandle { task, cancel }
}

async fn run_download(
    manager: Weak<Ed2kManager>,
    spec: DownloadSpec,
    config: Ed2kConfig,
    cancel: CancelToken,
) -> Result<()> {
    let pf = PartFile::open_or_create(
        &spec.download_dir,
        &spec.name,
        spec.size,
        &spec.file_hash_hex,
    )?;
    let pf = Arc::new(Mutex::new(pf));

    let candidate_pool: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));
    let active_addrs: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));
    let peer_count = Arc::new(AtomicU64::new(0));
    let bytes_in_window = Arc::new(AtomicU64::new(0));
    let part_hashes: Arc<Mutex<Option<Vec<[u8; 16]>>>> = Arc::new(Mutex::new(None));

    if let Some(mgr) = manager.upgrade() {
        mgr.update_file(&spec.file_hash_hex, |f| {
            f.state = Ed2kState::Downloading;
            f.peers = 0;
            f.download_speed = 0;
            f.eta = None;
        });
    }

    let mut last_source_refresh: Option<Instant> = None;
    let mut peer_workers: Vec<JoinHandle<()>> = Vec::new();
    let mut last_progress = Instant::now();
    let mut last_received = pf.lock().await.received_bytes();

    loop {
        if cancel.is_cancelled() {
            log::info!("ed2k download {} cancelled", spec.name);
            break;
        }

        // Re-acquire sources periodically.
        let need_refresh = match last_source_refresh {
            None => true,
            Some(t) => t.elapsed() >= SOURCE_REFRESH_INTERVAL,
        };
        let pool_empty = candidate_pool.lock().await.is_empty();
        if need_refresh || pool_empty {
            match refresh_sources(&spec, &config).await {
                Ok(addrs) => {
                    let mut pool = candidate_pool.lock().await;
                    for a in addrs {
                        pool.insert(a);
                    }
                    log::info!(
                        "ed2k {}: source pool size = {}",
                        spec.name,
                        pool.len()
                    );
                }
                Err(e) => {
                    log::warn!("ed2k source refresh for {} failed: {}", spec.name, e);
                }
            }
            last_source_refresh = Some(Instant::now());
        }

        // Reap finished peer worker tasks and respawn up to MAX_PEERS_PER_FILE.
        peer_workers.retain(|w| !w.is_finished());
        while peer_workers.len() < MAX_PEERS_PER_FILE {
            let next: Option<SocketAddr> = {
                let mut pool = candidate_pool.lock().await;
                let mut active = active_addrs.lock().await;
                let pick = pool
                    .iter()
                    .find(|a| !active.contains(*a))
                    .cloned();
                if let Some(a) = pick {
                    pool.remove(&a);
                    active.insert(a);
                    Some(a)
                } else {
                    None
                }
            };
            let Some(addr) = next else { break };

            let pf_c = pf.clone();
            let pool_c = candidate_pool.clone();
            let active_c = active_addrs.clone();
            let peer_c = peer_count.clone();
            let bytes_c = bytes_in_window.clone();
            let part_hashes_c = part_hashes.clone();
            let cancel_c = cancel.clone();
            let manager_c = manager.clone();
            let spec_c = spec.clone();
            let config_c = config.clone();
            let h = tokio::spawn(async move {
                peer_c.fetch_add(1, Ordering::SeqCst);
                let result = run_peer_worker(
                    addr,
                    spec_c.clone(),
                    config_c,
                    pf_c,
                    bytes_c,
                    part_hashes_c,
                    cancel_c,
                )
                .await;
                if let Err(e) = result {
                    log::debug!("ed2k peer {} for {}: {}", addr, spec_c.name, e);
                }
                peer_c.fetch_sub(1, Ordering::SeqCst);
                active_c.lock().await.remove(&addr);
                // Don't push a known-bad peer back into the pool — let the
                // next source refresh re-introduce it.
                let _ = pool_c;
                let _ = manager_c;
            });
            peer_workers.push(h);
        }

        // Periodically push progress numbers up to the manager.
        if last_progress.elapsed() >= PROGRESS_TICK {
            let now_received = pf.lock().await.received_bytes();
            let dt = last_progress.elapsed().as_secs_f64().max(0.001);
            let window = bytes_in_window.swap(0, Ordering::SeqCst);
            let speed = (window as f64 / dt) as u64;
            last_progress = Instant::now();

            let p = if spec.size > 0 {
                (now_received as f64) / (spec.size as f64)
            } else {
                0.0
            };
            let eta = if speed > 0 && now_received < spec.size {
                Some((spec.size - now_received) / speed.max(1))
            } else {
                None
            };
            let active = peer_count.load(Ordering::SeqCst) as u32;

            if let Some(mgr) = manager.upgrade() {
                mgr.update_file(&spec.file_hash_hex, |f| {
                    f.progress = p.clamp(0.0, 1.0);
                    f.download_speed = speed;
                    f.peers = active;
                    f.eta = eta;
                });
            }

            // Have we crossed the finish line?
            if now_received >= spec.size {
                if let Err(e) = finalize_file(
                    &manager,
                    &spec,
                    &pf,
                    &part_hashes,
                    &config,
                )
                .await
                {
                    log::warn!("ed2k finalize {} failed: {}", spec.name, e);
                }
                break;
            }

            let _ = last_received;
            last_received = now_received;
        }

        // Sleep briefly OR until we are cancelled.
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            _ = cancel.cancelled() => {}
        }
    }

    // Cancellation: wait for in-flight peer workers to wind down.
    for w in peer_workers {
        w.abort();
        let _ = w.await;
    }

    Ok(())
}

async fn refresh_sources(
    spec: &DownloadSpec,
    config: &Ed2kConfig,
) -> Result<Vec<SocketAddr>> {
    let mut servers: Vec<crate::config::Ed2kServer> = DEFAULT_SERVERS.iter().cloned().collect();
    servers.extend(config.extra_servers.iter().cloned());
    let connect_timeout = Duration::from_secs(config.connect_timeout_secs);

    for s in &servers {
        match Ed2kClient::connect_and_login(s, config.listen_port, &config.user_name, connect_timeout)
            .await
        {
            Ok(mut client) => {
                let addrs = client
                    .get_sources(&spec.file_hash, spec.size, Duration::from_secs(6))
                    .await
                    .unwrap_or_default();
                let out = addrs
                    .into_iter()
                    .map(SocketAddr::V4)
                    .filter(|a| !is_unroutable(a))
                    .collect::<Vec<_>>();
                return Ok(out);
            }
            Err(e) => {
                log::debug!("ed2k {}: server {} unreachable: {}", spec.name, s.host, e);
            }
        }
    }
    Ok(Vec::new())
}

fn is_unroutable(addr: &SocketAddr) -> bool {
    match addr {
        SocketAddr::V4(a) => {
            let o = a.ip().octets();
            // ed2k client IDs travel as 32-bit LE ints. HighID clients have
            // their public IP in those four bytes; LowID clients have ID
            // < 0x01000000, which on the wire is `[X, 0, 0, 0]` and decodes
            // here to `X.0.0.0`. We can't connect directly to LowIDs (they
            // need a server callback we don't perform), so drop them along
            // with the obvious garbage.
            let is_lowid = o[1] == 0 && o[2] == 0 && o[3] == 0;
            a.port() == 0
                || a.ip().is_unspecified()
                || a.ip().is_broadcast()
                || a.ip().is_loopback()
                || a.ip().is_private()
                || a.ip().is_link_local()
                || (o[0] == 0)
                || is_lowid
        }
        SocketAddr::V6(_) => true,
    }
}

async fn finalize_file(
    manager: &Weak<Ed2kManager>,
    spec: &DownloadSpec,
    pf: &Arc<Mutex<PartFile>>,
    part_hashes: &Arc<Mutex<Option<Vec<[u8; 16]>>>>,
    config: &Ed2kConfig,
) -> Result<()> {
    // Verify file hash. For multi-part files we need the hashset; for
    // single-part files we just MD4 the whole file.
    let hashes = part_hashes.lock().await.clone().unwrap_or_default();
    let mut guard = pf.lock().await;
    let ok = guard.verify_full_file(&spec.file_hash, &hashes)?;
    if !ok {
        // Best effort: nuke received ranges so the next attempt re-fetches
        // from scratch. Real eMule walks parts and finds the bad ones; for
        // v1 we just bail and let the user retry. TODO: smarter recovery.
        log::warn!("ed2k {}: full-file hash check failed", spec.name);
        return Err(anyhow::anyhow!("file hash mismatch"));
    }
    drop(guard);

    {
        let mut guard = pf.lock().await;
        guard.finalize_in_place(&spec.final_path)?;
    }

    if let Some(mgr) = manager.upgrade() {
        mgr.update_file(&spec.file_hash_hex, |f| {
            f.state = Ed2kState::Seeding;
            f.progress = 1.0;
            f.download_speed = 0;
            f.eta = Some(0);
            f.output_path = Some(spec.final_path.to_string_lossy().to_string());
        });
    }

    // Best-effort: announce ourselves as a source.
    let mut servers: Vec<crate::config::Ed2kServer> =
        DEFAULT_SERVERS.iter().cloned().collect();
    servers.extend(config.extra_servers.iter().cloned());
    let connect_timeout = Duration::from_secs(config.connect_timeout_secs);
    for s in &servers {
        if let Ok(mut client) = Ed2kClient::connect_and_login(
            s,
            config.listen_port,
            &config.user_name,
            connect_timeout,
        )
        .await
        {
            let _ = client
                .offer_files(
                    &[OfferedFile {
                        hash: spec.file_hash,
                        name: spec.name.clone(),
                        size: spec.size,
                        file_type: None,
                    }],
                    config.listen_port,
                )
                .await;
            break;
        }
    }
    Ok(())
}

async fn run_peer_worker(
    addr: SocketAddr,
    spec: DownloadSpec,
    config: Ed2kConfig,
    pf: Arc<Mutex<PartFile>>,
    bytes_in_window: Arc<AtomicU64>,
    part_hashes: Arc<Mutex<Option<Vec<[u8; 16]>>>>,
    cancel: CancelToken,
) -> Result<()> {
    if cancel.is_cancelled() {
        return Ok(());
    }
    let mut conn = PeerConnection::connect(addr, PEER_CONNECT_TIMEOUT).await?;

    let hello = hello_body_for(&config.user_name, config.listen_port);
    conn.send_hello(&hello).await?;

    // Drain frames until we get HELLOANSWER or until idle timeout.
    let mut got_hello_answer = false;
    let mut wants_64bit = want_64bit_offsets(spec.size);
    let mut accepted = false;
    let mut have_filename_ok = false;
    let mut have_status_ok = false;
    let mut have_hashset_ok = part_hashes.lock().await.is_some();

    // Send the initial probe sequence the moment HELLOANSWER arrives.
    let ph_cur = part_hashes.lock().await.clone();
    let mut ph_cached = ph_cur;

    loop {
        if cancel.is_cancelled() {
            break;
        }
        let frame = conn.recv_frame(PEER_IDLE_TIMEOUT).await?;
        // Same opcode value can mean different things on PROTO_EDONKEY vs.
        // PROTO_EMULE (e.g., 0x01 = HELLO on 0xE3 but EMULEINFO on 0xC5;
        // the I64 part-transfer variants live on 0xC5 only). Route on
        // (proto, opcode) so we can't misidentify an extended frame as a
        // standard one. Frames whose proto byte we don't recognise are
        // surfaced by `read_frame` with `opcode == 0` and dropped here.
        match (frame.proto, frame.opcode) {
            (PROTO_EDONKEY, OP_HELLO) => {
                // Peer initiated as well; reply with HELLOANSWER.
                let body =
                    crate::peer::encode_hello(&hello, /*include_marker=*/ false);
                conn.write_frame(OP_HELLOANSWER, &body).await?;
            }
            (PROTO_EDONKEY, OP_HELLOANSWER) => {
                got_hello_answer = true;
                // Send the filename and file-id queries so the peer puts us
                // in its slot logic.
                let hash = encode_hash_only(&spec.file_hash);
                conn.write_frame(OP_REQUESTFILENAME, &hash).await?;
                conn.write_frame(OP_SETREQFILEID, &hash).await?;
                if !have_hashset_ok {
                    conn.write_frame(OP_HASHSETREQUEST, &hash).await?;
                }
                conn.write_frame(OP_STARTUPLOADREQ, &hash).await?;
            }
            (PROTO_EDONKEY, OP_REQFILENAMEANSWER) => {
                if let Ok((h, _name)) = decode_filename_answer(&frame.payload) {
                    if h == spec.file_hash {
                        have_filename_ok = true;
                    }
                }
            }
            (PROTO_EDONKEY, OP_FILESTATUS) => {
                if let Ok(FileStatus::Status { hash, .. }) =
                    decode_file_status(&frame.payload)
                {
                    if hash == spec.file_hash {
                        have_status_ok = true;
                    }
                }
            }
            (PROTO_EDONKEY, OP_HASHSETANSWER) => {
                if let Ok((h, parts)) = decode_hashset_answer(&frame.payload) {
                    if h == spec.file_hash && parts.len() == part_count(spec.size) as usize {
                        // Cache hashset for later verifications and finalize.
                        *part_hashes.lock().await = Some(parts.clone());
                        ph_cached = Some(parts);
                        have_hashset_ok = true;
                    }
                }
            }
            (PROTO_EDONKEY, OP_FILEREQANSNOFIL) => {
                // Peer doesn't have the file (or we lost a race). Drop them.
                return Ok(());
            }
            (PROTO_EDONKEY, OP_QUEUERANK) | (PROTO_EMULE, OP_QUEUERANKING) => {
                // Just keep waiting; the receiver helper already updated
                // last_queue_rank.
            }
            (PROTO_EDONKEY, OP_ACCEPTUPLOADREQ) => {
                accepted = true;
            }
            (PROTO_EDONKEY, OP_OUTOFPARTREQS) => {
                // Peer says we asked for stuff they don't have. Wait — they
                // may send us back to the queue.
                accepted = false;
            }
            (PROTO_EDONKEY, OP_SENDINGPART) => {
                if let Ok(part) = decode_sending_part_32(&frame.payload) {
                    if part.hash == spec.file_hash {
                        let len = part.data.len() as u64;
                        let mut guard = pf.lock().await;
                        if let Err(e) = guard.write_block(part.start, &part.data) {
                            log::debug!("ed2k {} write fail: {}", spec.name, e);
                        }
                        drop(guard);
                        bytes_in_window.fetch_add(len, Ordering::SeqCst);
                    }
                }
            }
            (PROTO_EMULE, OP_SENDINGPART_I64) => {
                if let Ok(part) = decode_sending_part_64(&frame.payload) {
                    if part.hash == spec.file_hash {
                        let len = part.data.len() as u64;
                        let mut guard = pf.lock().await;
                        if let Err(e) = guard.write_block(part.start, &part.data) {
                            log::debug!("ed2k {} write fail: {}", spec.name, e);
                        }
                        drop(guard);
                        bytes_in_window.fetch_add(len, Ordering::SeqCst);
                    }
                }
            }
            (PROTO_EDONKEY, OP_END_OF_DOWNLOAD)
            | (PROTO_EDONKEY, OP_CANCELTRANSFER) => {
                // Peer closing this transfer. We may have been served a full
                // part. Loop back and let the driver pick a new peer.
                return Ok(());
            }
            _ => {
                // Ignore unknown opcodes / unsupported extensions
                // (compressed parts, source-exchange, etc.).
            }
        }

        // If we were accepted, drive the request pipeline forward whenever
        // we have nothing in flight. amule sends REQUESTPARTS on
        // PROTO_EDONKEY but the I64 variant on PROTO_EMULE
        // (`DownloadClient.cpp::SendBlockRequests`), so we must too.
        if accepted && have_filename_ok && have_status_ok {
            if let Some(req) = next_request(&spec, &pf).await {
                if wants_64bit {
                    let body = encode_request_parts_64(&spec.file_hash, &req);
                    conn.write_frame_ext(OP_REQUESTPARTS_I64, &body).await?;
                } else {
                    let body = encode_request_parts_32(&spec.file_hash, &req);
                    conn.write_frame(OP_REQUESTPARTS, &body).await?;
                }
            } else {
                // Nothing left to ask for: gracefully end.
                let _ = conn.write_frame(OP_END_OF_DOWNLOAD, &[]).await;
                return Ok(());
            }
        }

        let _ = (got_hello_answer, &ph_cached, &mut wants_64bit);
    }

    Ok(())
}

/// Build the next REQUESTPARTS payload by scanning the part file for missing
/// ranges. Returns `None` when the file is complete.
async fn next_request(
    spec: &DownloadSpec,
    pf: &Arc<Mutex<PartFile>>,
) -> Option<[crate::peer::BlockRange; 3]> {
    let guard = pf.lock().await;
    let total_parts = part_count(spec.size);
    for idx in 0..total_parts {
        if guard.is_part_complete(idx) {
            continue;
        }
        let (s, e) = part_range(spec.size, idx);
        if e <= s {
            continue;
        }
        // Limit per-part = ED2K_PART_SIZE; per request we ask up to 3 blocks.
        let missing = guard.missing_blocks_for_part(idx, ED2K_PART_SIZE);
        let ranges = ranges_for_request(&missing);
        if let Some(r) = pad_ranges(&ranges) {
            return Some(r);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn spec_in(dir: &TempDir, size: u64) -> DownloadSpec {
        DownloadSpec::from_parts(
            "00112233445566778899aabbccddeeff",
            "movie.mkv",
            size,
            dir.path().to_path_buf(),
        )
        .unwrap()
    }

    #[test]
    fn cancel_token_signals() {
        let t = CancelToken::new();
        assert!(!t.is_cancelled());
        t.cancel();
        assert!(t.is_cancelled());
    }

    #[tokio::test]
    async fn cancel_token_cancelled_returns_immediately_after_signal() {
        let t = CancelToken::new();
        t.cancel();
        // Should not block.
        tokio::time::timeout(Duration::from_millis(50), t.cancelled())
            .await
            .unwrap();
    }

    #[test]
    fn spec_decodes_hash() {
        let tmp = TempDir::new().unwrap();
        let s = spec_in(&tmp, 1000);
        assert_eq!(s.file_hash[0], 0x00);
        assert_eq!(s.file_hash[15], 0xff);
        assert_eq!(s.size, 1000);
    }

    #[tokio::test]
    async fn next_request_returns_blocks_for_empty_file() {
        let tmp = TempDir::new().unwrap();
        let spec = spec_in(&tmp, 5000);
        let pf = Arc::new(Mutex::new(
            PartFile::open_or_create(
                &spec.download_dir,
                &spec.name,
                spec.size,
                &spec.file_hash_hex,
            )
            .unwrap(),
        ));
        let req = next_request(&spec, &pf).await.unwrap();
        assert_eq!(req[0].start, 0);
        assert!(req[0].end <= 5000);
    }

    #[tokio::test]
    async fn next_request_none_when_complete() {
        let tmp = TempDir::new().unwrap();
        let spec = spec_in(&tmp, 100);
        let pf_arc = Arc::new(Mutex::new(
            PartFile::open_or_create(
                &spec.download_dir,
                &spec.name,
                spec.size,
                &spec.file_hash_hex,
            )
            .unwrap(),
        ));
        {
            let mut g = pf_arc.lock().await;
            g.write_block(0, &[1u8; 100]).unwrap();
        }
        assert!(next_request(&spec, &pf_arc).await.is_none());
    }

    #[test]
    fn is_unroutable_filters_invalid_addresses() {
        use std::net::{Ipv4Addr, SocketAddrV4};
        // Zero IP / zero port / loopback
        assert!(is_unroutable(&SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            4662
        ))));
        assert!(is_unroutable(&SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(1, 2, 3, 4),
            0
        ))));
        assert!(is_unroutable(&SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            4662
        ))));
        assert!(!is_unroutable(&SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(8, 8, 8, 8),
            4662
        ))));
    }

    #[tokio::test]
    async fn dedup_via_hashset() {
        // Just exercise the candidate-pool dedup logic. We don't need the
        // network to test this — building two spawn calls with same address
        // should never yield two active workers.
        use std::collections::HashSet;
        let pool: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));
        let active: Arc<Mutex<HashSet<SocketAddr>>> = Arc::new(Mutex::new(HashSet::new()));
        let addr: SocketAddr = "1.2.3.4:5".parse().unwrap();
        pool.lock().await.insert(addr);
        // First pick succeeds.
        let pick1 = {
            let mut p = pool.lock().await;
            let mut a = active.lock().await;
            let v = p.iter().find(|x| !a.contains(*x)).cloned();
            if let Some(v) = v {
                p.remove(&v);
                a.insert(v);
                Some(v)
            } else {
                None
            }
        };
        assert_eq!(pick1, Some(addr));
        // Re-add to pool: now active set blocks dedup picking it twice.
        pool.lock().await.insert(addr);
        let pick2 = {
            let p = pool.lock().await;
            let a = active.lock().await;
            p.iter().find(|x| !a.contains(*x)).cloned()
        };
        assert_eq!(pick2, None);
    }
}

//! Filestore-style block backing for the embedded IPFS node.
//!
//! The default `rust-ipfs` blockstore (`FsBlockStore`) writes every block's
//! bytes into `<repo>/blockstore/`. For library scans that's wasteful: a
//! 200 GB TV library would duplicate every byte, since the original files
//! are already on disk.
//!
//! The filestore decorator avoids that. For UnixFS *leaf* blocks (the
//! protobuf-wrapped chunks of file data — typically 256 KiB each), we
//! record `(cid, path, offset, length)` in a side index and SKIP the
//! `put_block` write. When bitswap or any other consumer asks for the
//! block via `BlockStore::get`, we read those bytes back from the source
//! file at the recorded offset, drive a fresh `FileAdder` over them to
//! reconstruct the exact same UnixFS-wrapped block, and return it. The
//! reconstructed block hashes back to the original CID by construction —
//! the same chunker + protobuf encoding always produce identical bytes.
//!
//! UnixFS *link* blocks (the small inner-tree blocks that reference leaf
//! CIDs) still go through the inner blockstore via normal `put_block`
//! writes, since they're tiny (a few hundred bytes per ~32 leaves) and
//! don't benefit from the filestore indirection.
//!
//! ## Index trait
//!
//! The index itself isn't bound to a particular database — `mhaol-backend`
//! supplies a SurrealDB-backed implementation, embedded test rigs can use
//! the in-memory `InMemoryFilestoreIndex`. The trait is async so a
//! persistent backing store can run real I/O in the read path; reads that
//! must stay fast can keep an in-process cache in front of disk.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use futures::stream::{self, BoxStream, StreamExt};
use parking_lot::RwLock;
use rust_ipfs::error::Error as IpfsError;
use rust_ipfs::repo::blockstore::flatfs::FsBlockStore;
use rust_ipfs::repo::{BlockPut, BlockStore};
use rust_ipfs::Block;
use rust_unixfs::file::adder::FileAdder;
use std::collections::HashMap;
use std::path::Path;

/// An entry in the filestore index. Identifies a UnixFS leaf block by the
/// `(path, offset, length)` triple of its source bytes on disk. The leaf's
/// own CID is the index key.
#[derive(Debug, Clone)]
pub struct FilestoreEntry {
    pub cid: Cid,
    pub path: PathBuf,
    pub offset: u64,
    pub length: u32,
}

/// Persistent index of `cid → (path, offset, length)` for filestore leaf
/// blocks. Implementations are responsible for any caching needed to keep
/// the read path fast — a SurrealDB-backed index, for example, would load
/// every entry into a `HashMap` at boot and write through on `record_leaf`.
///
/// `#[async_trait]` so the trait stays dyn-compatible — the BlockStore
/// decorator holds the index as `Arc<dyn FilestoreIndex>`.
#[async_trait]
pub trait FilestoreIndex: Send + Sync + std::fmt::Debug {
    /// Look up an entry by its leaf CID. Returns `None` when the CID isn't
    /// in the filestore — callers should fall through to the inner
    /// blockstore.
    async fn lookup(&self, cid: &Cid) -> Option<FilestoreEntry>;

    /// Record a new leaf entry. Callers should not have already
    /// `put_block`'d the leaf — the whole point is to skip that write.
    /// Idempotent: re-recording an entry with the same CID overwrites
    /// any prior entry (useful when the source file path changed but
    /// hashes the same).
    async fn record_leaf(&self, entry: FilestoreEntry) -> Result<()>;

    /// Drop a leaf entry. Called when the inner blockstore is told to
    /// remove a CID — keeps the two views in sync.
    async fn remove(&self, cid: &Cid) -> Result<()>;

    /// Stream every CID currently in the filestore. Used by
    /// `BlockStore::list` to surface filestore entries alongside the
    /// inner blockstore's entries.
    async fn list_cids(&self) -> Vec<Cid>;

    /// Total bytes referenced by the filestore (sum of every entry's
    /// `length`). Used by `BlockStore::total_size` so callers see a
    /// meaningful total, even though the bytes don't live in the
    /// blockstore directory.
    async fn total_size(&self) -> u64;
}

/// In-memory `FilestoreIndex` for tests and as a fallback when no
/// persistent backend is configured. Loses all entries on process exit.
#[derive(Debug, Default)]
pub struct InMemoryFilestoreIndex {
    inner: RwLock<HashMap<Cid, FilestoreEntry>>,
}

impl InMemoryFilestoreIndex {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl FilestoreIndex for InMemoryFilestoreIndex {
    async fn lookup(&self, cid: &Cid) -> Option<FilestoreEntry> {
        self.inner.read().get(cid).cloned()
    }

    async fn record_leaf(&self, entry: FilestoreEntry) -> Result<()> {
        self.inner.write().insert(entry.cid, entry);
        Ok(())
    }

    async fn remove(&self, cid: &Cid) -> Result<()> {
        self.inner.write().remove(cid);
        Ok(())
    }

    async fn list_cids(&self) -> Vec<Cid> {
        self.inner.read().keys().copied().collect()
    }

    async fn total_size(&self) -> u64 {
        self.inner
            .read()
            .values()
            .map(|e| e.length as u64)
            .sum()
    }
}

/// `BlockStore` decorator that combines an `FsBlockStore` (for materialised
/// blocks: link blocks, firkin metadata, torrent files, anything that came
/// in via `put_block`) with a `FilestoreIndex` (for leaf blocks of
/// scanned library files, where we hold only a `(path, offset, length)`
/// reference).
///
/// `get`/`contains`/`size` consult the filestore first; on miss they fall
/// through to the inner `FsBlockStore`. `put`/`remove`/`total_size`/`list`
/// span both views.
#[derive(Debug)]
pub struct FilestoreBlockStore {
    inner: FsBlockStore,
    index: Arc<dyn FilestoreIndex>,
}

impl FilestoreBlockStore {
    pub fn new(blockstore_path: PathBuf, index: Arc<dyn FilestoreIndex>) -> Self {
        Self {
            inner: FsBlockStore::new(blockstore_path),
            index,
        }
    }

    /// Read the leaf bytes for `entry` off disk and run them through a
    /// fresh `FileAdder` to reconstruct the UnixFS-wrapped block. The
    /// resulting CID matches `entry.cid` by construction (same chunker,
    /// same input bytes, same protobuf encoding) — when it doesn't, the
    /// source file has been edited and we surface that as `Err` so the
    /// caller (bitswap, etc.) doesn't ship corrupt data to peers.
    async fn reconstruct_leaf_block(entry: &FilestoreEntry) -> Result<Block> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

        if !entry.path.exists() {
            return Err(anyhow!(
                "filestore source file no longer exists: {}",
                entry.path.display()
            ));
        }
        let mut file = tokio::fs::File::open(&entry.path)
            .await
            .map_err(|e| anyhow!("open {}: {e}", entry.path.display()))?;
        file.seek(SeekFrom::Start(entry.offset))
            .await
            .map_err(|e| anyhow!("seek {} @ {}: {e}", entry.path.display(), entry.offset))?;

        let mut buf = vec![0u8; entry.length as usize];
        file.read_exact(&mut buf)
            .await
            .map_err(|e| anyhow!("read_exact {} @ {}: {e}", entry.path.display(), entry.offset))?;

        let mut adder = FileAdder::default();
        let mut blocks: Vec<(Cid, Vec<u8>)> = Vec::new();
        let mut consumed = 0;
        while consumed < buf.len() {
            let (iter, ate) = adder.push(&buf[consumed..]);
            blocks.extend(iter);
            if ate == 0 {
                break;
            }
            consumed += ate;
        }
        blocks.extend(adder.finish());

        // For a single-chunk push, FileAdder emits one leaf and possibly
        // one wrapping link block at finish() (the "root link" for a
        // 1-leaf tree). The leaf is the one whose CID matches the entry.
        let (cid, bytes) = blocks
            .into_iter()
            .find(|(cid, _)| *cid == entry.cid)
            .ok_or_else(|| {
                anyhow!(
                    "filestore CID mismatch: source file {} has changed since the scan",
                    entry.path.display()
                )
            })?;
        Block::new(cid, bytes).map_err(|e| anyhow!("block construct: {e}"))
    }
}

#[async_trait]
impl BlockStore for FilestoreBlockStore {
    async fn init(&self) -> Result<(), IpfsError> {
        self.inner.init().await
    }

    async fn contains(&self, cid: &Cid) -> Result<bool, IpfsError> {
        if self.index.lookup(cid).await.is_some() {
            return Ok(true);
        }
        self.inner.contains(cid).await
    }

    async fn get(&self, cid: &Cid) -> Result<Option<Block>, IpfsError> {
        if let Some(entry) = self.index.lookup(cid).await {
            return match Self::reconstruct_leaf_block(&entry).await {
                Ok(block) => Ok(Some(block)),
                Err(e) => {
                    log::warn!(
                        "[filestore] reconstruct failed for {cid} from {}: {e}",
                        entry.path.display()
                    );
                    Ok(None)
                }
            };
        }
        self.inner.get(cid).await
    }

    async fn size(&self, cids: &[Cid]) -> Result<Option<usize>, IpfsError> {
        let mut total: usize = 0;
        let mut any_filestore = false;
        let mut leftover: Vec<Cid> = Vec::new();
        for c in cids {
            if let Some(entry) = self.index.lookup(c).await {
                total += entry.length as usize;
                any_filestore = true;
            } else {
                leftover.push(*c);
            }
        }
        if leftover.is_empty() {
            return Ok(Some(total));
        }
        match self.inner.size(&leftover).await? {
            Some(inner_size) => Ok(Some(total + inner_size)),
            None if any_filestore => Ok(Some(total)),
            None => Ok(None),
        }
    }

    async fn total_size(&self) -> Result<usize, IpfsError> {
        let inner_size = self.inner.total_size().await?;
        let filestore_size = self.index.total_size().await as usize;
        Ok(inner_size + filestore_size)
    }

    async fn put(&self, block: &Block) -> Result<(Cid, BlockPut), IpfsError> {
        // Caller has bytes in hand — go straight to the inner blockstore.
        // Filestore entries are populated via the side channel
        // (`compute_and_index_file_cid`), not through `put`.
        self.inner.put(block).await
    }

    async fn remove(&self, cid: &Cid) -> Result<(), IpfsError> {
        let _ = self.index.remove(cid).await;
        self.inner.remove(cid).await
    }

    async fn remove_many(&self, blocks: BoxStream<'static, Cid>) -> BoxStream<'static, Cid> {
        let index = Arc::clone(&self.index);
        let inner_stream = self.inner.remove_many(blocks).await;
        async_stream::stream! {
            futures::pin_mut!(inner_stream);
            while let Some(cid) = inner_stream.next().await {
                let _ = index.remove(&cid).await;
                yield cid;
            }
        }
        .boxed()
    }

    async fn list(&self) -> BoxStream<'static, Cid> {
        let mut filestore_cids = self.index.list_cids().await;
        filestore_cids.sort();
        let filestore_stream = stream::iter(filestore_cids);
        let inner_stream = self.inner.list().await;
        // Filestore CIDs first, then anything in the inner blockstore.
        // Bitswap doesn't care about order; this just matches the
        // mental model that filestore-backed leaves are the "primary"
        // bulk of the library.
        Box::pin(filestore_stream.chain(inner_stream))
    }
}

/// Drive `FileAdder` over `path`, populate `index` with one entry per
/// emitted leaf block, and write every link block through `put_link`.
/// Used by the library scan path so a 200 GB library produces only the
/// link-block bytes (~megabytes total) in the blockstore, never the leaf
/// bytes themselves.
///
/// Returns the file's root CID — the same value `add()` would return.
///
/// `put_link` is called for every non-leaf block (the inner-tree link
/// blocks that reference leaf CIDs). Callers wire it to
/// `repo.put_block(...)` so link blocks live in the regular blockstore.
pub async fn compute_and_index_file_cid<F, Fut>(
    path: &Path,
    index: &Arc<dyn FilestoreIndex>,
    put_link: F,
) -> Result<(String, u64)>
where
    F: Fn(Block) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    use tokio::io::AsyncReadExt;

    if !path.exists() {
        return Err(anyhow!("Source path does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(anyhow!("compute_and_index_file_cid only supports files: {}", path.display()));
    }

    /// FileAdder's default chunker boundary. Feeding the adder exactly
    /// one CHUNK at a time triggers the "fast path" inside `push()`, so
    /// the iterator yields exactly one leaf per push (plus 0+ link
    /// blocks). That's what makes leaf attribution to a `(offset, length)`
    /// range deterministic.
    const CHUNK: usize = 256 * 1024;

    let total_size = tokio::fs::metadata(path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| anyhow!("open {}: {e}", path.display()))?;
    let mut buf = vec![0u8; CHUNK];

    let mut adder = FileAdder::default();
    let mut last_cid: Option<Cid> = None;

    let mut current_offset: u64 = 0;
    // When the file's last chunk is smaller than CHUNK, FileAdder buffers
    // it without emitting a leaf — that leaf comes out of `finish()`
    // instead. Track the chunk's range so we can attribute it correctly
    // when we drain finish() at the end.
    let mut pending_leaf: Option<(u64, u32)> = None;

    loop {
        // Refill the buffer up to CHUNK bytes (or EOF). Without this loop,
        // a short read leaves the FileAdder partial and breaks the
        // "one push per chunk" attribution scheme.
        let mut filled: usize = 0;
        while filled < CHUNK {
            let n = file
                .read(&mut buf[filled..])
                .await
                .map_err(|e| anyhow!("read {}: {e}", path.display()))?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        if filled == 0 {
            break;
        }
        let chunk_offset = current_offset;
        let chunk_length = filled as u32;
        current_offset += filled as u64;

        let mut emitted_leaf_for_chunk = false;
        let mut consumed = 0;
        while consumed < filled {
            let (iter, ate) = adder.push(&buf[consumed..filled]);
            consumed += ate;
            for (cid, block_bytes) in iter {
                last_cid = Some(cid);
                if !emitted_leaf_for_chunk {
                    // First block emitted for this chunk's push() is the
                    // leaf. Record it in the filestore index — its bytes
                    // do NOT go to the blockstore.
                    index
                        .record_leaf(FilestoreEntry {
                            cid,
                            path: path.to_path_buf(),
                            offset: chunk_offset,
                            length: chunk_length,
                        })
                        .await
                        .map_err(|e| anyhow!("filestore index record: {e}"))?;
                    emitted_leaf_for_chunk = true;
                    continue;
                }
                // Any subsequent blocks from this push are link blocks —
                // they're tiny and reference leaf CIDs, so they live in
                // the regular blockstore.
                let block = Block::new(cid, block_bytes)
                    .map_err(|e| anyhow!("block construct: {e}"))?;
                put_link(block).await?;
            }
            if ate == 0 {
                break;
            }
        }
        // `filled < CHUNK` means EOF in the middle of a chunk: FileAdder
        // buffered it without emitting (slow path, ready=false). The leaf
        // comes out of `finish()` later — track its range so we can
        // attribute it then.
        if !emitted_leaf_for_chunk {
            pending_leaf = Some((chunk_offset, chunk_length));
        } else {
            pending_leaf = None;
        }
    }

    let mut finish_iter = adder.finish().peekable();
    if let Some((chunk_offset, chunk_length)) = pending_leaf {
        if let Some((cid, _bytes)) = finish_iter.next() {
            last_cid = Some(cid);
            index
                .record_leaf(FilestoreEntry {
                    cid,
                    path: path.to_path_buf(),
                    offset: chunk_offset,
                    length: chunk_length,
                })
                .await
                .map_err(|e| anyhow!("filestore index record: {e}"))?;
        }
    }
    for (cid, block_bytes) in finish_iter {
        last_cid = Some(cid);
        let block = Block::new(cid, block_bytes)
            .map_err(|e| anyhow!("block construct: {e}"))?;
        put_link(block).await?;
    }

    let cid = last_cid
        .ok_or_else(|| anyhow!("FileAdder produced no blocks for {}", path.display()))?;
    Ok((cid.to_string(), total_size))
}

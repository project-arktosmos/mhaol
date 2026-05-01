//! Disk-backed sparse part file plus a JSON sidecar tracking which byte
//! ranges have been received. Real eMule uses a binary `.met` format; we use
//! JSON because we never need to interoperate with another client across the
//! disk format — the protocol on the wire is what matters.
//!
//! The file on disk is `<download_dir>/<name>.part`. The sidecar lives next
//! to it as `<download_dir>/<name>.part.met`. Sidecar writes go through a
//! `.tmp` + atomic-rename to survive crashes mid-write.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::util::{part_count, part_range, ED2K_PART_SIZE};

/// On-disk JSON sidecar shape.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SidecarV1 {
    /// Lowercase hex MD4.
    hash: String,
    /// Total file size in bytes.
    size: u64,
    /// Sorted, non-overlapping byte ranges `[start, end)` that have been
    /// written so far.
    received: Vec<RangeJson>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct RangeJson {
    start: u64,
    end: u64,
}

/// A part file open for read/write.
pub struct PartFile {
    path: PathBuf,
    sidecar_path: PathBuf,
    file: File,
    size: u64,
    hash_hex: String,
    /// Sorted, non-overlapping `[start, end)` ranges of bytes written. We
    /// merge on insert to keep this short and efficient even after many
    /// blocks.
    received: Vec<(u64, u64)>,
}

impl PartFile {
    /// Open or create a part file for `download_path/<name>.part` of the
    /// given total size. If a sidecar already exists, its received-ranges
    /// are loaded.
    pub fn open_or_create(
        download_dir: &Path,
        name: &str,
        size: u64,
        hash_hex: &str,
    ) -> Result<Self> {
        std::fs::create_dir_all(download_dir).with_context(|| {
            format!("ed2k: cannot create dir {}", download_dir.display())
        })?;

        let part_name = format!("{}.part", sanitize_filename(name));
        let path = download_dir.join(&part_name);
        let sidecar_path = download_dir.join(format!("{}.met", part_name));

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("ed2k: open part file {}", path.display()))?;

        let current_len = file.metadata()?.len();
        if current_len < size {
            // Pre-allocate a sparse file by seeking to size-1 and writing a
            // zero byte. On a copy-on-write fs this stays sparse; on others
            // it just grows the file but reads return zero either way.
            // SAFETY: we just opened the file in r/w mode.
            let mut f = &file;
            f.seek(SeekFrom::Start(size.saturating_sub(1)))?;
            f.write_all(&[0u8])?;
            f.flush()?;
        }

        let mut received: Vec<(u64, u64)> = Vec::new();
        if let Ok(s) = std::fs::read_to_string(&sidecar_path) {
            if let Ok(parsed) = serde_json::from_str::<SidecarV1>(&s) {
                if parsed.size == size && parsed.hash.eq_ignore_ascii_case(hash_hex) {
                    received =
                        parsed.received.into_iter().map(|r| (r.start, r.end)).collect();
                    received.sort_by_key(|r| r.0);
                }
            }
        }

        Ok(Self {
            path,
            sidecar_path,
            file,
            size,
            hash_hex: hash_hex.to_lowercase(),
            received,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    /// Total bytes written to disk so far (sum of received range lengths).
    pub fn received_bytes(&self) -> u64 {
        self.received.iter().map(|(a, b)| b - a).sum()
    }

    /// Write a block at `start`. Records the range and persists the sidecar.
    pub fn write_block(&mut self, start: u64, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let end = start
            .checked_add(data.len() as u64)
            .ok_or_else(|| anyhow!("ed2k: block end overflows"))?;
        if end > self.size {
            return Err(anyhow!(
                "ed2k: block [{}..{}) past file size {}",
                start,
                end,
                self.size
            ));
        }
        let f = &mut self.file;
        f.seek(SeekFrom::Start(start))?;
        f.write_all(data)?;
        // Don't fsync each block — too slow on real disks. Sidecar persist
        // gives us the durability story.
        self.merge_range(start, end);
        self.persist_sidecar()?;
        Ok(())
    }

    /// Whether all bytes belonging to the given part index are present.
    pub fn is_part_complete(&self, part_index: u32) -> bool {
        if part_index >= part_count(self.size) {
            return false;
        }
        let (s, e) = part_range(self.size, part_index);
        self.is_range_present(s, e)
    }

    /// Return the list of `(start, end)` byte ranges still missing from the
    /// part with `part_index`, capped by `block_size` per range so the caller
    /// can send a small set of REQUESTPARTS frames.
    pub fn missing_blocks_for_part(
        &self,
        part_index: u32,
        block_size: u64,
    ) -> Vec<(u64, u64)> {
        if part_index >= part_count(self.size) {
            return Vec::new();
        }
        let (s, e) = part_range(self.size, part_index);
        self.missing_in_range(s, e, block_size)
    }

    /// Read part `part_index` from disk and return the bytes (allocates).
    pub fn read_part(&mut self, part_index: u32) -> Result<Vec<u8>> {
        if part_index >= part_count(self.size) {
            return Err(anyhow!("ed2k: part index out of range"));
        }
        let (s, e) = part_range(self.size, part_index);
        let mut buf = vec![0u8; (e - s) as usize];
        self.file.seek(SeekFrom::Start(s))?;
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Verify a complete part by computing MD4 and comparing.
    pub fn verify_part(&mut self, part_index: u32, expected: &[u8; 16]) -> Result<bool> {
        let bytes = self.read_part(part_index)?;
        let actual = md4_bytes(&bytes);
        Ok(actual == *expected)
    }

    /// Drop a previously-recorded byte range — used when a part fails MD4
    /// verification and we want to re-fetch its bytes.
    pub fn invalidate_range(&mut self, start: u64, end: u64) -> Result<()> {
        if end <= start {
            return Ok(());
        }
        let mut new_received = Vec::with_capacity(self.received.len());
        for &(a, b) in &self.received {
            if b <= start || a >= end {
                new_received.push((a, b));
                continue;
            }
            if a < start {
                new_received.push((a, start));
            }
            if b > end {
                new_received.push((end, b));
            }
        }
        self.received = new_received;
        self.persist_sidecar()?;
        Ok(())
    }

    /// For files ≤ 9.28 MB the file hash is just MD4 of the file. For larger
    /// files it is MD4 of the concatenated 16-byte part hashes. Verify the
    /// full file matches `file_hash`.
    pub fn verify_full_file(
        &mut self,
        file_hash: &[u8; 16],
        part_hashes: &[[u8; 16]],
    ) -> Result<bool> {
        if self.size <= ED2K_PART_SIZE {
            // Single-part file. The "part hash" IS the file hash.
            let bytes = self.read_part(0)?;
            let actual = md4_bytes(&bytes);
            return Ok(&actual == file_hash);
        }
        if part_hashes.len() != part_count(self.size) as usize {
            return Ok(false);
        }
        let mut concat = Vec::with_capacity(part_hashes.len() * 16);
        for ph in part_hashes {
            concat.extend_from_slice(ph);
        }
        let actual = md4_bytes(&concat);
        Ok(&actual == file_hash)
    }

    /// Atomically rename the .part file to its final destination and remove
    /// the sidecar. Best-effort: missing sidecar is not an error. On Unix
    /// this is safe even with the file handle still open.
    pub fn finalize(self, final_path: &Path) -> Result<()> {
        let Self {
            path,
            sidecar_path,
            file,
            ..
        } = self;
        drop(file);

        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::rename(&path, final_path).with_context(|| {
            format!(
                "ed2k: finalize rename {} -> {}",
                path.display(),
                final_path.display()
            )
        })?;
        let _ = std::fs::remove_file(&sidecar_path);
        Ok(())
    }

    /// In-place finalize that doesn't consume `self`. Used when the part
    /// file lives behind a `Mutex`/`Arc` and we cannot easily own it. The
    /// underlying file handle keeps pointing at the renamed inode (Unix
    /// rename semantics) so reads continue to work.
    pub fn finalize_in_place(&mut self, final_path: &Path) -> Result<()> {
        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::rename(&self.path, final_path).with_context(|| {
            format!(
                "ed2k: finalize rename {} -> {}",
                self.path.display(),
                final_path.display()
            )
        })?;
        let _ = std::fs::remove_file(&self.sidecar_path);
        self.path = final_path.to_path_buf();
        // Sidecar at the new location won't actually exist — we just
        // update the path so any further `persist_sidecar` writes go to a
        // sensible spot. In practice no further writes happen post-finalize.
        let mut sidecar = final_path.as_os_str().to_owned();
        sidecar.push(".met");
        self.sidecar_path = PathBuf::from(sidecar);
        Ok(())
    }

    fn merge_range(&mut self, start: u64, end: u64) {
        if end <= start {
            return;
        }
        // Insert sorted, then sweep merging adjacent / overlapping ranges.
        let mut merged: Vec<(u64, u64)> = Vec::with_capacity(self.received.len() + 1);
        let mut inserted = false;
        let mut cur_start = start;
        let mut cur_end = end;

        for &(a, b) in &self.received {
            if b < cur_start {
                merged.push((a, b));
            } else if a > cur_end {
                if !inserted {
                    merged.push((cur_start, cur_end));
                    inserted = true;
                }
                merged.push((a, b));
            } else {
                cur_start = cur_start.min(a);
                cur_end = cur_end.max(b);
            }
        }
        if !inserted {
            merged.push((cur_start, cur_end));
        }
        self.received = merged;
    }

    fn is_range_present(&self, start: u64, end: u64) -> bool {
        if end <= start {
            return true;
        }
        for &(a, b) in &self.received {
            if a <= start && b >= end {
                return true;
            }
            if a > start {
                return false;
            }
        }
        false
    }

    fn missing_in_range(
        &self,
        start: u64,
        end: u64,
        block_size: u64,
    ) -> Vec<(u64, u64)> {
        let mut out = Vec::new();
        let mut cur = start;
        for &(a, b) in &self.received {
            if b <= cur {
                continue;
            }
            if a >= end {
                break;
            }
            if a > cur {
                let gap_end = a.min(end);
                split_into_blocks(&mut out, cur, gap_end, block_size);
            }
            cur = cur.max(b);
            if cur >= end {
                break;
            }
        }
        if cur < end {
            split_into_blocks(&mut out, cur, end, block_size);
        }
        out
    }

    fn persist_sidecar(&self) -> Result<()> {
        let sidecar = SidecarV1 {
            hash: self.hash_hex.clone(),
            size: self.size,
            received: self
                .received
                .iter()
                .map(|&(start, end)| RangeJson { start, end })
                .collect(),
        };
        let json = serde_json::to_vec(&sidecar)?;
        let tmp = self.sidecar_path.with_extension("met.tmp");
        std::fs::write(&tmp, &json)
            .with_context(|| format!("ed2k: write sidecar tmp {}", tmp.display()))?;
        std::fs::rename(&tmp, &self.sidecar_path).with_context(|| {
            format!(
                "ed2k: rename sidecar {} -> {}",
                tmp.display(),
                self.sidecar_path.display()
            )
        })?;
        Ok(())
    }
}

fn md4_bytes(data: &[u8]) -> [u8; 16] {
    use md4::{Digest, Md4};
    let mut h = Md4::new();
    h.update(data);
    let d = h.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&d);
    out
}

fn split_into_blocks(out: &mut Vec<(u64, u64)>, mut s: u64, e: u64, block: u64) {
    if block == 0 {
        out.push((s, e));
        return;
    }
    while s < e {
        let next = (s + block).min(e);
        out.push((s, next));
        s = next;
    }
}

/// Strip path separators and other characters that would let a hostile peer
/// or server filename escape the download directory.
fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '\0' => '_',
            _ => c,
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == ' ' || c == '.');
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const HASH_HEX: &str = "00112233445566778899aabbccddeeff";

    #[test]
    fn open_creates_sparse_file_of_given_size() {
        let tmp = TempDir::new().unwrap();
        let pf = PartFile::open_or_create(tmp.path(), "x.bin", 12345, HASH_HEX).unwrap();
        let meta = std::fs::metadata(pf.path()).unwrap();
        assert_eq!(meta.len(), 12345);
        assert_eq!(pf.received_bytes(), 0);
    }

    #[test]
    fn write_block_and_received_bytes() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "y.bin", 1000, HASH_HEX).unwrap();
        pf.write_block(0, &[1u8; 100]).unwrap();
        pf.write_block(200, &[2u8; 100]).unwrap();
        assert_eq!(pf.received_bytes(), 200);
        // Adjacent merge:
        pf.write_block(100, &[3u8; 100]).unwrap();
        assert_eq!(pf.received_bytes(), 300);
    }

    #[test]
    fn write_block_past_end_errors() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "z.bin", 100, HASH_HEX).unwrap();
        assert!(pf.write_block(99, &[0u8; 5]).is_err());
    }

    #[test]
    fn sidecar_round_trip_preserves_received_ranges() {
        let tmp = TempDir::new().unwrap();
        {
            let mut pf =
                PartFile::open_or_create(tmp.path(), "r.bin", 1000, HASH_HEX).unwrap();
            pf.write_block(10, &[7u8; 50]).unwrap();
            pf.write_block(200, &[8u8; 50]).unwrap();
        }
        let pf2 =
            PartFile::open_or_create(tmp.path(), "r.bin", 1000, HASH_HEX).unwrap();
        assert_eq!(pf2.received_bytes(), 100);
    }

    #[test]
    fn sidecar_mismatched_hash_is_ignored() {
        let tmp = TempDir::new().unwrap();
        {
            let mut pf =
                PartFile::open_or_create(tmp.path(), "m.bin", 1000, HASH_HEX).unwrap();
            pf.write_block(0, &[1u8; 100]).unwrap();
        }
        // Same path but different hash — we should reset.
        let pf2 =
            PartFile::open_or_create(tmp.path(), "m.bin", 1000, "ff".repeat(16).as_str())
                .unwrap();
        assert_eq!(pf2.received_bytes(), 0);
    }

    #[test]
    fn is_part_complete_within_single_part_file() {
        let tmp = TempDir::new().unwrap();
        let mut pf = PartFile::open_or_create(tmp.path(), "p.bin", 500, HASH_HEX).unwrap();
        assert!(!pf.is_part_complete(0));
        pf.write_block(0, &[0u8; 500]).unwrap();
        assert!(pf.is_part_complete(0));
    }

    #[test]
    fn is_part_complete_handles_multi_part_file() {
        let tmp = TempDir::new().unwrap();
        let size = ED2K_PART_SIZE * 2 + 17;
        let mut pf = PartFile::open_or_create(tmp.path(), "big.bin", size, HASH_HEX).unwrap();
        // Fill part 1 (the middle one) only.
        let (s, e) = part_range(size, 1);
        pf.write_block(s, &vec![1u8; (e - s) as usize]).unwrap();
        assert!(!pf.is_part_complete(0));
        assert!(pf.is_part_complete(1));
        assert!(!pf.is_part_complete(2));
    }

    #[test]
    fn missing_blocks_returns_correct_chunks() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "mb.bin", 1000, HASH_HEX).unwrap();
        // Initially empty: one block of size 1000.
        let blocks = pf.missing_blocks_for_part(0, 400);
        assert_eq!(blocks, vec![(0, 400), (400, 800), (800, 1000)]);

        pf.write_block(0, &[0u8; 600]).unwrap();
        let blocks = pf.missing_blocks_for_part(0, 400);
        assert_eq!(blocks, vec![(600, 1000)]);
    }

    #[test]
    fn verify_part_with_known_md4() {
        let tmp = TempDir::new().unwrap();
        let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let expected = md4_bytes(&data);

        let mut pf = PartFile::open_or_create(
            tmp.path(),
            "vp.bin",
            data.len() as u64,
            HASH_HEX,
        )
        .unwrap();
        pf.write_block(0, &data).unwrap();
        assert!(pf.verify_part(0, &expected).unwrap());
        let mut bad = expected;
        bad[0] ^= 0xFF;
        assert!(!pf.verify_part(0, &bad).unwrap());
    }

    #[test]
    fn invalidate_range_clears_received_bytes() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "iv.bin", 1000, HASH_HEX).unwrap();
        pf.write_block(0, &[0u8; 1000]).unwrap();
        assert_eq!(pf.received_bytes(), 1000);
        pf.invalidate_range(200, 500).unwrap();
        assert_eq!(pf.received_bytes(), 700);
    }

    #[test]
    fn finalize_renames_and_removes_sidecar() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "f.bin", 10, HASH_HEX).unwrap();
        pf.write_block(0, &[5u8; 10]).unwrap();
        let final_path = tmp.path().join("f.bin");
        let sidecar_path = pf.sidecar_path.clone();
        let part_path = pf.path().to_path_buf();
        pf.finalize(&final_path).unwrap();
        assert!(final_path.exists());
        assert!(!part_path.exists());
        assert!(!sidecar_path.exists());
    }

    #[test]
    fn finalize_in_place_renames_without_consuming() {
        let tmp = TempDir::new().unwrap();
        let mut pf =
            PartFile::open_or_create(tmp.path(), "fp.bin", 10, HASH_HEX).unwrap();
        pf.write_block(0, &[3u8; 10]).unwrap();
        let final_path = tmp.path().join("fp.bin");
        pf.finalize_in_place(&final_path).unwrap();
        assert!(final_path.exists());
        assert_eq!(pf.path(), final_path);
    }

    #[test]
    fn sanitize_filename_strips_path_separators() {
        // Leading/trailing dots and spaces are trimmed (Windows-hostile,
        // confusing on Unix). Inner dots stay.
        assert_eq!(sanitize_filename("../etc/passwd"), "_etc_passwd");
        assert_eq!(sanitize_filename("a/b\\c:d"), "a_b_c_d");
        assert_eq!(sanitize_filename("   "), "file");
        assert_eq!(sanitize_filename("normal.mkv"), "normal.mkv");
        assert_eq!(sanitize_filename("inner.dot.name.bin"), "inner.dot.name.bin");
    }
}

//! Library scan side-effects: compute a UnixFS CID for every scanned
//! file whose type matches the library's declared `local-*` addons and
//! record a **lite** `ipfs_pin` row for it (no blocks written into the
//! blockstore yet). The WebUI's libraries table can then show a CID per
//! file and firkins can reference these CIDs as `ipfs`-typed file
//! entries, without paying the per-byte duplication cost of materialising
//! every file into `<data_root>/downloads/ipfs/`.
//!
//! Materialisation — the actual `repo.put_block` writes that make the
//! file's blocks reachable via bitswap to other peers — happens lazily
//! via `POST /api/ipfs/pins/:cid/materialise` when a peer or the user
//! explicitly asks for it. Until then `serve_pin_file` still streams the
//! on-disk bytes directly (we hold the original path on the pin row), so
//! local playback works without materialisation.
//!
//! **No firkin records are created from a library scan** — the firkin
//! store is for explicit bookmarks and catalog flows, not auto-detection
//! of local files.
//!
//! A library only ever processes files whose type is relevant to its
//! declared addons (video for `local-movie`/`local-tv`, audio for
//! `local-album`, book exts for `local-book`, ROM exts for `local-game`).
//! Libraries with no addons declared are skipped entirely — the directory
//! walk still runs (so the WebUI's scan-results table populates), but no
//! pins are produced until the user picks at least one addon.
//!
//! Re-running the scan is idempotent because pin records are deduplicated
//! by `(cid, path)` in `ipfs_pins::record_lite_pin`.

#![cfg(not(target_os = "android"))]

use std::path::{Path, PathBuf};

use crate::ipfs_pins;
use crate::libraries::ScanEntry;
use crate::state::CloudState;

const ADDON_LOCAL_MOVIE: &str = "local-movie";
const ADDON_LOCAL_TV: &str = "local-tv";
const ADDON_LOCAL_ALBUM: &str = "local-album";
const ADDON_LOCAL_BOOK: &str = "local-book";
const ADDON_LOCAL_GAME: &str = "local-game";

const ALL_LOCAL_ADDONS: &[&str] = &[
    ADDON_LOCAL_MOVIE,
    ADDON_LOCAL_TV,
    ADDON_LOCAL_ALBUM,
    ADDON_LOCAL_BOOK,
    ADDON_LOCAL_GAME,
];

const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts", "mpg", "mpeg",
];
const AUDIO_EXTS: &[&str] =
    &["mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "wma", "alac", "aiff"];
const BOOK_EXTS: &[&str] = &[
    "epub", "pdf", "mobi", "azw3", "azw", "cbz", "cbr", "djvu", "fb2",
];
const GAME_EXTS: &[&str] = &[
    "iso", "rom", "smc", "sfc", "gba", "nes", "gb", "gbc", "n64", "z64", "v64", "md", "sms",
    "gg", "nds", "3ds", "wad", "cue", "chd", "gcm",
];

fn ext_lower(path: &str) -> Option<String> {
    PathBuf::from(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

fn is_video(entry: &ScanEntry) -> bool {
    if entry.mime.starts_with("video/") {
        return true;
    }
    ext_lower(&entry.relative_path)
        .map(|e| VIDEO_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

fn is_audio(entry: &ScanEntry) -> bool {
    if entry.mime.starts_with("audio/") {
        return true;
    }
    ext_lower(&entry.relative_path)
        .map(|e| AUDIO_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

fn is_book(entry: &ScanEntry) -> bool {
    ext_lower(&entry.relative_path)
        .map(|e| BOOK_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

fn is_game(entry: &ScanEntry) -> bool {
    ext_lower(&entry.relative_path)
        .map(|e| GAME_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

/// True when `entry`'s media type is relevant to at least one of the
/// library's declared addons. Used both to narrow the scan response (so a
/// movie library doesn't list mp3s or `.torrent` files) and to decide
/// which files actually get pinned.
pub(crate) fn entry_matches_addons(entry: &ScanEntry, addons: &[String]) -> bool {
    addons.iter().any(|a| match a.as_str() {
        ADDON_LOCAL_MOVIE | ADDON_LOCAL_TV => is_video(entry),
        ADDON_LOCAL_ALBUM => is_audio(entry),
        ADDON_LOCAL_BOOK => is_book(entry),
        ADDON_LOCAL_GAME => is_game(entry),
        _ => false,
    })
}

/// Spawn a fire-and-forget task that computes a UnixFS CID for every
/// entry matching the library's declared addons and records a **lite**
/// `ipfs_pin` row (no blockstore writes). Pin rows are deduplicated by
/// `(cid, path)` in `ipfs_pins::record_lite_pin`, so re-running the scan
/// is idempotent. An empty `addons` list falls back to "process anything
/// that matches any local-* addon" so a freshly-created library still
/// gets CIDs on the first scan.
///
/// The IPFS node does NOT need to be running for this path — `compute_file_cid`
/// is a pure FileAdder hash and writes nothing through the libp2p stack.
/// Materialisation (the per-block `put_block` writes that make CIDs
/// reachable via bitswap) is deferred until a peer asks via
/// `POST /api/ipfs/pins/:cid/materialise`.
pub fn schedule_pins(
    state: &CloudState,
    entries: &[ScanEntry],
    addons: Vec<String>,
    lib_root: String,
) {
    let effective_addons: Vec<String> = if addons.is_empty() {
        ALL_LOCAL_ADDONS.iter().map(|s| s.to_string()).collect()
    } else {
        addons
    };

    let to_process: Vec<(String, String, u64)> = entries
        .iter()
        .filter(|e| entry_matches_addons(e, &effective_addons))
        .map(|e| (e.path.clone(), e.mime.clone(), e.size))
        .collect();

    if to_process.is_empty() {
        tracing::info!("[library-scan] {lib_root}: nothing to hash");
        return;
    }

    let ipfs = state.ipfs_manager.clone();
    let state = state.clone();
    let total = to_process.len();
    tokio::spawn(async move {
        tracing::info!(
            "[library-scan] {lib_root}: hashing {} file(s) (lite mode — bytes are NOT copied to the blockstore)",
            total
        );
        for (path_str, mime, size) in to_process {
            let path = Path::new(&path_str);
            // Skip lite-pinning when a row already exists for this path
            // — `compute_file_cid` would otherwise re-hash the file just
            // to discover the row is a duplicate.
            if pin_exists_for_path(&state, &path_str).await {
                continue;
            }
            match ipfs.compute_file_cid(path).await {
                Ok((cid, _)) => {
                    if let Err(e) =
                        ipfs_pins::record_lite_pin(&state, cid, path_str.clone(), mime, size)
                            .await
                    {
                        tracing::warn!(
                            "[library-scan] failed to record lite pin for {path_str}: {e}"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("[library-scan] failed to hash {path_str}: {e}");
                }
            }
        }
    });
}

/// Cheap pre-flight check: does any `ipfs_pin` row already exist whose
/// `path` matches `path_str`? Used to short-circuit re-scans so we don't
/// re-hash a 3 GB file we've already CID'd.
async fn pin_exists_for_path(state: &CloudState, path_str: &str) -> bool {
    let pins: Vec<crate::ipfs_pins::IpfsPin> = match state.db.select(crate::ipfs_pins::TABLE).await
    {
        Ok(p) => p,
        Err(_) => return false,
    };
    pins.iter().any(|p| p.path == path_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(rel: &str, mime: &str) -> ScanEntry {
        ScanEntry {
            path: format!("/lib/{}", rel),
            relative_path: rel.to_string(),
            size: 100,
            mime: mime.to_string(),
            extracted_query: None,
            tmdb_match: None,
            extracted_tv_query: None,
        }
    }

    #[test]
    fn movie_addon_keeps_videos() {
        let e = entry("Movie.2020.1080p.mkv", "video/x-matroska");
        assert!(entry_matches_addons(&e, &[ADDON_LOCAL_MOVIE.to_string()]));
    }

    #[test]
    fn movie_addon_drops_audio() {
        let e = entry("track.mp3", "audio/mpeg");
        assert!(!entry_matches_addons(&e, &[ADDON_LOCAL_MOVIE.to_string()]));
    }

    #[test]
    fn album_addon_keeps_audio() {
        let e = entry("Artist/Album/01 track.flac", "audio/flac");
        assert!(entry_matches_addons(&e, &[ADDON_LOCAL_ALBUM.to_string()]));
    }

    #[test]
    fn book_addon_uses_extension() {
        let e = entry("book.epub", "application/octet-stream");
        assert!(entry_matches_addons(&e, &[ADDON_LOCAL_BOOK.to_string()]));
    }

    #[test]
    fn empty_addons_predicate_returns_false() {
        let e = entry("Movie.2020.mkv", "video/x-matroska");
        assert!(!entry_matches_addons(&e, &[]));
    }
}

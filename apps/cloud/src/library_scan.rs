//! Library scan side-effects: pin every scanned file whose type matches
//! the library's declared `local-*` addons to the embedded IPFS node, so
//! the WebUI's libraries table can show a CID per file. **No firkin
//! records are created from a library scan** — the firkin store is for
//! explicit bookmarks and catalog flows, not auto-detection of local
//! files.
//!
//! A library only ever pins files whose type is relevant to its declared
//! addons (video for `local-movie`/`local-tv`, audio for `local-album`,
//! book exts for `local-book`, ROM exts for `local-game`). Libraries with
//! no addons declared are skipped entirely — the directory walk still
//! runs (so the WebUI's scan-results table populates), but no pins are
//! produced until the user picks at least one addon.
//!
//! Re-running the scan is idempotent because pin records are deduplicated
//! by `(cid, path)` in `ipfs_pins::record_pin`.

#![cfg(not(target_os = "android"))]

use std::path::PathBuf;

use crate::ipfs_pins;
use crate::libraries::{wait_for_ipfs_ready, ScanEntry};
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

/// Spawn a fire-and-forget task that pins every entry matching the
/// library's declared addons to IPFS. Pin rows are deduplicated by
/// `(cid, path)` in `ipfs_pins::record_pin`, so re-running the scan is
/// idempotent. An empty `addons` list falls back to "pin anything that
/// matches any local-* addon" so a freshly-created library still gets its
/// files pinned on the first scan.
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

    let to_pin: Vec<(String, String, u64)> = entries
        .iter()
        .filter(|e| entry_matches_addons(e, &effective_addons))
        .map(|e| (e.path.clone(), e.mime.clone(), e.size))
        .collect();

    if to_pin.is_empty() {
        tracing::info!("[library-scan] {lib_root}: nothing to pin");
        return;
    }

    let ipfs = state.ipfs_manager.clone();
    let state = state.clone();
    let total = to_pin.len();
    tokio::spawn(async move {
        if !wait_for_ipfs_ready(&ipfs).await {
            tracing::warn!(
                "[library-scan] {lib_root}: ipfs node never reached running state — skipping {} file(s)",
                total
            );
            return;
        }

        tracing::info!("[library-scan] {lib_root}: pinning {} file(s)", total);
        for (path, mime, size) in to_pin {
            let req = mhaol_ipfs_core::AddIpfsRequest {
                source: path.clone(),
                pin: Some(true),
            };
            match ipfs.add(req).await {
                Ok(info) => {
                    if let Err(e) =
                        ipfs_pins::record_pin(&state, info.cid, path.clone(), mime, size).await
                    {
                        tracing::warn!("[library-scan] failed to record pin for {path}: {e}");
                    }
                }
                Err(e) => tracing::warn!("[library-scan] failed to pin {path}: {e}"),
            }
        }
    });
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

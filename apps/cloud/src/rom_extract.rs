//! ROM extraction for retroachievement console firkins.
//!
//! Walks the on-disk download directory of every completed torrent attached
//! to a firkin, extracts compressed archives in place (zip / 7z), and returns
//! the list of ROM files (plus archive status) so the WebUI can render them.

#![cfg(not(target_os = "android"))]

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::firkins::{Firkin, TABLE as FIRKIN_TABLE};
use crate::state::CloudState;

const ROM_EXTS: &[&str] = &[
    "iso", "rom", "smc", "sfc", "gba", "nes", "gb", "gbc", "n64", "z64", "v64", "md", "sms", "gg",
    "nds", "3ds", "wad", "cue", "chd", "gcm",
];

const ARCHIVE_EXTS: &[&str] = &["zip", "7z", "rar"];

#[derive(Serialize, Debug)]
pub struct ArchiveReport {
    pub name: String,
    pub relative_path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct RomEntry {
    pub name: String,
    pub relative_path: String,
    pub size: u64,
}

#[derive(Serialize, Debug, Default)]
pub struct RomsResponse {
    pub torrent_paths: Vec<String>,
    pub archives: Vec<ArchiveReport>,
    pub roms: Vec<RomEntry>,
}

/// Look up the firkin, find every completed torrent attached to it, extract
/// any archives in their download directories, and walk for ROM files.
pub async fn extract_roms_for_firkin(
    state: &CloudState,
    firkin_id: &str,
) -> anyhow::Result<RomsResponse> {
    let doc: Option<Firkin> = state.db.select((FIRKIN_TABLE, firkin_id)).await?;
    let firkin = match doc {
        Some(d) => d,
        None => anyhow::bail!("firkin not found"),
    };

    let mut response = RomsResponse::default();

    if !state.torrent_manager.is_initialized() {
        return Ok(response);
    }

    let hashes: Vec<String> = firkin
        .files
        .iter()
        .filter(|f| f.kind == "torrent magnet")
        .filter_map(|f| btih_from_magnet(&f.value))
        .collect();
    if hashes.is_empty() {
        return Ok(response);
    }

    let torrents = state.torrent_manager.list().await?;

    for hash in &hashes {
        let torrent = match torrents.iter().find(|t| t.info_hash.to_lowercase() == *hash) {
            Some(t) => t,
            None => continue,
        };
        let finished = matches!(torrent.state, mhaol_torrent::TorrentState::Seeding)
            || torrent.progress >= 1.0;
        if !finished {
            continue;
        }
        let output_path = match torrent.output_path.as_deref() {
            Some(p) => PathBuf::from(p),
            None => continue,
        };
        if !output_path.exists() {
            continue;
        }

        response.torrent_paths.push(output_path.to_string_lossy().into_owned());

        // Extract archives that haven't been extracted yet.
        for archive in find_archives(&output_path) {
            let rel = archive
                .strip_prefix(&output_path)
                .unwrap_or(&archive)
                .to_string_lossy()
                .into_owned();
            let name = archive
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let dest = extracted_dir_for(&archive);
            if dest.exists() {
                response.archives.push(ArchiveReport {
                    name,
                    relative_path: rel,
                    status: "already_extracted".to_string(),
                    error: None,
                });
                continue;
            }
            let ext = archive
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            let result = match ext.as_str() {
                "zip" => extract_zip(&archive, &dest),
                "7z" => extract_7z(&archive, &dest),
                "rar" => Err(anyhow::anyhow!("rar extraction is not supported yet")),
                _ => Err(anyhow::anyhow!("unsupported archive type: {ext}")),
            };
            match result {
                Ok(()) => {
                    response.archives.push(ArchiveReport {
                        name,
                        relative_path: rel,
                        status: "extracted".to_string(),
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&dest);
                    response.archives.push(ArchiveReport {
                        name,
                        relative_path: rel,
                        status: "failed".to_string(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // Walk every ROM file under the torrent's output directory (including
        // any directories produced by the extraction step above).
        for rom in find_roms(&output_path) {
            let relative_path = rom
                .strip_prefix(&output_path)
                .unwrap_or(&rom)
                .to_string_lossy()
                .into_owned();
            let name = rom
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let size = std::fs::metadata(&rom).map(|m| m.len()).unwrap_or(0);
            response.roms.push(RomEntry {
                name,
                relative_path,
                size,
            });
        }
    }

    response
        .roms
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(response)
}

fn btih_from_magnet(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    let idx = lower.find("btih:")?;
    let tail = &lower[idx + "btih:".len()..];
    let end = tail.find('&').unwrap_or(tail.len());
    let hash = tail[..end].trim().to_string();
    if hash.is_empty() {
        None
    } else {
        Some(hash)
    }
}

fn extracted_dir_for(archive: &Path) -> PathBuf {
    let stem = archive
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "extracted".to_string());
    let parent = archive.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}.extracted"))
}

fn find_archives(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        // Skip files that already live under an `*.extracted` directory.
        if path
            .components()
            .any(|c| match c {
                std::path::Component::Normal(s) => s
                    .to_str()
                    .map(|n| n.ends_with(".extracted"))
                    .unwrap_or(false),
                _ => false,
            })
        {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ARCHIVE_EXTS.contains(&ext.as_str()) {
            out.push(path);
        }
    }
    out
}

fn find_roms(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if ROM_EXTS.contains(&ext.as_str()) {
            out.push(path);
        }
    }
    out
}

fn extract_zip(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    std::fs::create_dir_all(dest)?;
    zip.extract(dest)?;
    Ok(())
}

fn extract_7z(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest)?;
    sevenz_rust2::decompress_file(archive, dest)
        .map_err(|e| anyhow::anyhow!("7z extract failed: {e}"))?;
    Ok(())
}

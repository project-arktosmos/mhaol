//! ROM extraction for retroachievement console firkins.
//!
//! Walks the on-disk download directory of every completed torrent attached
//! to a firkin, extracts compressed archives in place (zip / 7z), pins every
//! ROM file (whether already there or freshly extracted) to IPFS, appends
//! them as `ipfs` file entries, and rolls the firkin's CID forward — same
//! versioning contract as `torrent_completion::rollforward`.

#![cfg(not(target_os = "android"))]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;

use crate::firkins::{
    compute_firkin_cid, pin_firkin_body, serialize_firkin_payload, FileEntry, Firkin,
    TABLE as FIRKIN_TABLE,
};
use crate::ipfs_pins;
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
    pub cid: String,
}

#[derive(Serialize, Debug, Default)]
pub struct RomsResponse {
    pub firkin_id: String,
    pub torrent_paths: Vec<String>,
    pub archives: Vec<ArchiveReport>,
    pub roms: Vec<RomEntry>,
}

/// Look up the firkin, find every completed torrent attached to it, extract
/// any archives in their download directories, pin every ROM to IPFS, append
/// new ROMs as `ipfs` file entries on the firkin, and roll the firkin's CID
/// forward when there's anything new.
pub async fn extract_roms_for_firkin(
    state: &CloudState,
    firkin_id: &str,
) -> anyhow::Result<RomsResponse> {
    let (mut current, mut current_id) =
        resolve_head_firkin(state, firkin_id).await?;

    let mut response = RomsResponse {
        firkin_id: current_id.clone(),
        ..RomsResponse::default()
    };

    if !state.torrent_manager.is_initialized() {
        return Ok(response);
    }

    let hashes: Vec<String> = current
        .files
        .iter()
        .filter(|f| f.kind == "torrent magnet")
        .filter_map(|f| btih_from_magnet(&f.value))
        .collect();
    if hashes.is_empty() {
        response.firkin_id = current_id;
        return Ok(response);
    }

    let torrents = state.torrent_manager.list().await?;

    // Aggregate every ROM we discover across all completed torrents, then do
    // one rollforward at the end so the version chain doesn't grow by N when
    // a firkin happens to have multiple magnets.
    let mut new_entries: Vec<FileEntry> = Vec::new();
    let mut roms_acc: Vec<RomEntry> = Vec::new();

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

        response
            .torrent_paths
            .push(output_path.to_string_lossy().into_owned());

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

        // Rebuild the "already present in firkin" set on each torrent loop so
        // entries we just appended in this call also count as present.
        let existing_titles: HashSet<String> = current
            .files
            .iter()
            .chain(new_entries.iter())
            .filter(|f| f.kind == "ipfs")
            .filter_map(|f| f.title.clone())
            .collect();
        let existing_by_title: std::collections::HashMap<String, String> = current
            .files
            .iter()
            .chain(new_entries.iter())
            .filter(|f| f.kind == "ipfs")
            .filter_map(|f| f.title.clone().map(|t| (t, f.value.clone())))
            .collect();

        for rom_path in find_roms(&output_path) {
            let relative_path = rom_path
                .strip_prefix(&output_path)
                .unwrap_or(&rom_path)
                .to_string_lossy()
                .into_owned();
            let name = rom_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let size = std::fs::metadata(&rom_path).map(|m| m.len()).unwrap_or(0);

            // Reuse the CID from the firkin if this rom was already pinned in
            // a prior pass; otherwise pin it now and append a new entry. A
            // single failing file (e.g. rust-ipfs's add_unixfs returning
            // "invalid data" on a 0-byte or symlink-like entry) used to
            // bubble out via `?` and bail the entire scan — log + skip
            // instead so the rest of the ROMs still land in the firkin.
            let cid = if let Some(cid) = existing_by_title.get(&relative_path) {
                cid.clone()
            } else {
                let req = mhaol_ipfs_core::AddIpfsRequest {
                    source: rom_path.to_string_lossy().to_string(),
                    pin: Some(true),
                };
                let info = match state.ipfs_manager.add(req).await {
                    Ok(i) => i,
                    Err(e) => {
                        tracing::warn!(
                            "[rom-extract] skip {}: ipfs add failed: {e}",
                            rom_path.display()
                        );
                        continue;
                    }
                };
                let mime = mime_guess::from_path(&rom_path)
                    .first()
                    .map(|m| m.essence_str().to_string())
                    .unwrap_or_default();
                let _ = ipfs_pins::record_pin(
                    state,
                    info.cid.clone(),
                    rom_path.to_string_lossy().to_string(),
                    mime,
                    info.size,
                )
                .await;
                if !existing_titles.contains(&relative_path) {
                    new_entries.push(FileEntry {
                        kind: "ipfs".to_string(),
                        value: info.cid.clone(),
                        title: Some(relative_path.clone()),
                    });
                }
                info.cid
            };

            roms_acc.push(RomEntry {
                name,
                relative_path,
                size,
                cid,
            });
        }
    }

    if !new_entries.is_empty() {
        current_id = rollforward(state, &current, new_entries).await?;
        // Refresh the in-memory copy for callers that walk version_hashes.
        if let Some(refreshed) = state
            .db
            .select((FIRKIN_TABLE, current_id.as_str()))
            .await?
        {
            current = refreshed;
            let _ = current; // silence unused-assignment lint when no further reads
        }
    }

    roms_acc.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    response.firkin_id = current_id;
    response.roms = roms_acc;
    Ok(response)
}

/// Extract a single archive identified by its firkin-relative title
/// (e.g. `roms/Foo.7z`). Walks the firkin's torrents to find the archive
/// on disk, extracts it if not already, pins every resulting ROM and
/// rolls the firkin forward. The response only contains entries for that
/// one archive — callers can scan archives one at a time without
/// blocking on the full pack (a No-Intro torrent has hundreds of `.7z`
/// files; doing them all in one request takes ages).
pub async fn extract_single_archive(
    state: &CloudState,
    firkin_id: &str,
    archive_relative: &str,
) -> anyhow::Result<RomsResponse> {
    // If the user clicked Scan against an already-rolled-forward firkin
    // (e.g. `torrent_completion` rolled it while the page was open and
    // the auto-poller hasn't caught up yet), walk forward to the head
    // of the version chain instead of failing.
    let (mut current, mut current_id) =
        resolve_head_firkin(state, firkin_id).await?;

    let mut response = RomsResponse {
        firkin_id: current_id.clone(),
        ..RomsResponse::default()
    };

    if !state.torrent_manager.is_initialized() {
        return Ok(response);
    }

    let hashes: Vec<String> = current
        .files
        .iter()
        .filter(|f| f.kind == "torrent magnet")
        .filter_map(|f| btih_from_magnet(&f.value))
        .collect();
    if hashes.is_empty() {
        return Ok(response);
    }

    let torrents = state.torrent_manager.list().await?;

    let mut new_entries: Vec<FileEntry> = Vec::new();
    let mut roms_acc: Vec<RomEntry> = Vec::new();
    let mut found_archive = false;

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

        let archive_path = output_path.join(archive_relative);
        if !archive_path.is_file() {
            continue;
        }
        found_archive = true;
        response
            .torrent_paths
            .push(output_path.to_string_lossy().into_owned());

        let name = archive_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dest = extracted_dir_for(&archive_path);
        let ext = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if dest.exists() {
            response.archives.push(ArchiveReport {
                name,
                relative_path: archive_relative.to_string(),
                status: "already_extracted".to_string(),
                error: None,
            });
        } else {
            let result = match ext.as_str() {
                "zip" => extract_zip(&archive_path, &dest),
                "7z" => extract_7z(&archive_path, &dest),
                "rar" => Err(anyhow::anyhow!("rar extraction is not supported yet")),
                _ => Err(anyhow::anyhow!("unsupported archive type: {ext}")),
            };
            match result {
                Ok(()) => response.archives.push(ArchiveReport {
                    name,
                    relative_path: archive_relative.to_string(),
                    status: "extracted".to_string(),
                    error: None,
                }),
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&dest);
                    response.archives.push(ArchiveReport {
                        name,
                        relative_path: archive_relative.to_string(),
                        status: "failed".to_string(),
                        error: Some(e.to_string()),
                    });
                    response.firkin_id = current_id;
                    return Ok(response);
                }
            }
        }

        // Pin every ROM that landed in this archive's `.extracted/` dir.
        let existing_titles: HashSet<String> = current
            .files
            .iter()
            .filter(|f| f.kind == "ipfs")
            .filter_map(|f| f.title.clone())
            .collect();
        let existing_by_title: std::collections::HashMap<String, String> = current
            .files
            .iter()
            .filter(|f| f.kind == "ipfs")
            .filter_map(|f| f.title.clone().map(|t| (t, f.value.clone())))
            .collect();

        for rom_path in find_roms(&dest) {
            let relative_path = rom_path
                .strip_prefix(&output_path)
                .unwrap_or(&rom_path)
                .to_string_lossy()
                .into_owned();
            let rom_name = rom_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let size = std::fs::metadata(&rom_path).map(|m| m.len()).unwrap_or(0);

            let cid = if let Some(cid) = existing_by_title.get(&relative_path) {
                cid.clone()
            } else {
                let req = mhaol_ipfs_core::AddIpfsRequest {
                    source: rom_path.to_string_lossy().to_string(),
                    pin: Some(true),
                };
                let info = match state.ipfs_manager.add(req).await {
                    Ok(i) => i,
                    Err(e) => {
                        tracing::warn!(
                            "[rom-extract] skip {}: ipfs add failed: {e}",
                            rom_path.display()
                        );
                        continue;
                    }
                };
                let mime = mime_guess::from_path(&rom_path)
                    .first()
                    .map(|m| m.essence_str().to_string())
                    .unwrap_or_default();
                let _ = crate::ipfs_pins::record_pin(
                    state,
                    info.cid.clone(),
                    rom_path.to_string_lossy().to_string(),
                    mime,
                    info.size,
                )
                .await;
                if !existing_titles.contains(&relative_path) {
                    new_entries.push(FileEntry {
                        kind: "ipfs".to_string(),
                        value: info.cid.clone(),
                        title: Some(relative_path.clone()),
                    });
                }
                info.cid
            };

            roms_acc.push(RomEntry {
                name: rom_name,
                relative_path,
                size,
                cid,
            });
        }
        break;
    }

    if !found_archive {
        anyhow::bail!(
            "archive {} not found under any completed torrent for this firkin",
            archive_relative
        );
    }

    if !new_entries.is_empty() {
        current_id = rollforward(state, &current, new_entries).await?;
        if let Some(refreshed) = state
            .db
            .select((FIRKIN_TABLE, current_id.as_str()))
            .await?
        {
            current = refreshed;
            let _ = current;
        }
    }

    roms_acc.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    response.firkin_id = current_id;
    response.roms = roms_acc;
    Ok(response)
}

/// Roll the firkin forward with `new_entries` appended to `files`. Returns the
/// new firkin id (the new CID).
async fn rollforward(
    state: &CloudState,
    doc: &Firkin,
    new_entries: Vec<FileEntry>,
) -> anyhow::Result<String> {
    let old_id = doc.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
    if old_id.is_empty() {
        anyhow::bail!("firkin has no id");
    }

    let mut updated_files = doc.files.clone();
    updated_files.extend(new_entries);

    let new_version = doc.version.saturating_add(1);
    let mut new_hashes = doc.version_hashes.clone();
    new_hashes.push(old_id.clone());

    let new_id = compute_firkin_cid(
        &doc.title,
        &doc.description,
        &doc.artists,
        &doc.images,
        &updated_files,
        doc.year,
        &doc.addon,
        &doc.creator,
        new_version,
        &new_hashes,
        &doc.trailers,
    );

    if new_id == old_id {
        return Ok(old_id);
    }

    let new_body_json = serialize_firkin_payload(
        &doc.title,
        &doc.description,
        &doc.artists,
        &doc.images,
        &updated_files,
        doc.year,
        &doc.addon,
        &doc.creator,
        new_version,
        &new_hashes,
        &doc.trailers,
    );

    let new_record = Firkin {
        id: None,
        title: doc.title.clone(),
        artists: doc.artists.clone(),
        description: doc.description.clone(),
        images: doc.images.clone(),
        files: updated_files,
        year: doc.year,
        addon: doc.addon.clone(),
        creator: doc.creator.clone(),
        created_at: doc.created_at,
        updated_at: Utc::now(),
        version: new_version,
        version_hashes: new_hashes,
        trailers: doc.trailers.clone(),
    };

    let _: Option<Firkin> = state.db.delete((FIRKIN_TABLE, old_id.as_str())).await?;
    let create_result: Result<Option<Firkin>, _> = state
        .db
        .create((FIRKIN_TABLE, new_id.as_str()))
        .content(new_record)
        .await;
    match create_result {
        Ok(_) => {}
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                tracing::info!(
                    "[rom-extract] {} → {} already created by a concurrent attempt",
                    old_id,
                    new_id,
                );
                return Ok(new_id);
            }
            return Err(e.into());
        }
    }

    pin_firkin_body(state, &new_id, new_body_json).await;

    tracing::info!(
        "[rom-extract] {} → {} (v{})",
        old_id,
        new_id,
        new_version,
    );

    Ok(new_id)
}

/// Walk the version chain forward from `firkin_id` to the latest
/// existing record. Falls back to `firkin_id` if it's still the head.
/// Bails when the id can't be resolved at all (truly deleted firkin).
async fn resolve_head_firkin(
    state: &CloudState,
    firkin_id: &str,
) -> anyhow::Result<(Firkin, String)> {
    if let Some(doc) = state
        .db
        .select::<Option<Firkin>>((FIRKIN_TABLE, firkin_id))
        .await?
    {
        return Ok((doc, firkin_id.to_string()));
    }
    let docs: Vec<Firkin> = state.db.select(FIRKIN_TABLE).await?;
    let successor = docs
        .into_iter()
        .find(|d| d.version_hashes.iter().any(|h| h == firkin_id));
    match successor {
        Some(doc) => {
            let id = doc.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
            if id.is_empty() {
                anyhow::bail!("firkin not found");
            }
            Ok((doc, id))
        }
        None => anyhow::bail!("firkin not found"),
    }
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

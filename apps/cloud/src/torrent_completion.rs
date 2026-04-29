//! Background watcher that detects finished torrents and rolls the matching
//! document forward by pinning each downloaded file to IPFS, appending them as
//! `ipfs` file entries, and re-creating the SurrealDB record at its new CID.
//!
//! Versioning: each completion bumps `version` and pushes the prior CID onto
//! `version_hashes`. The chain integrity invariant is `version_hashes.len() ==
//! version` — verifiable by walking the chain in reverse.

#[cfg(not(target_os = "android"))]
use std::collections::HashSet;
#[cfg(not(target_os = "android"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "android"))]
use std::sync::Arc;
#[cfg(not(target_os = "android"))]
use std::time::Duration;

#[cfg(not(target_os = "android"))]
use parking_lot::Mutex;

#[cfg(not(target_os = "android"))]
use crate::documents::{compute_document_cid, Document, FileEntry, TABLE as DOCUMENT_TABLE};
#[cfg(not(target_os = "android"))]
use crate::ipfs_pins;
#[cfg(not(target_os = "android"))]
use crate::state::CloudState;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::{TorrentInfo, TorrentState};

#[cfg(not(target_os = "android"))]
const POLL_INTERVAL: Duration = Duration::from_secs(3);

#[cfg(not(target_os = "android"))]
pub async fn run(state: CloudState) {
    let processed: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        if !state.torrent_manager.is_initialized() {
            continue;
        }

        let torrents = match state.torrent_manager.list().await {
            Ok(t) => t,
            Err(e) => {
                tracing::debug!("[torrent-completion] list failed: {e}");
                continue;
            }
        };

        for t in torrents {
            let finished = matches!(t.state, TorrentState::Seeding) || t.progress >= 1.0;
            if !finished {
                continue;
            }
            if processed.lock().contains(&t.info_hash) {
                continue;
            }

            match handle_completion(&state, &t).await {
                Ok(true) => {
                    processed.lock().insert(t.info_hash.clone());
                }
                Ok(false) => {
                    // No matching doc, or already migrated — don't retry.
                    processed.lock().insert(t.info_hash.clone());
                }
                Err(e) => {
                    tracing::warn!(
                        "[torrent-completion] failed for {}: {e}",
                        t.info_hash
                    );
                    // Leave it unmarked so we retry on the next tick.
                }
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
async fn handle_completion(state: &CloudState, torrent: &TorrentInfo) -> anyhow::Result<bool> {
    let docs: Vec<Document> = state.db.select(DOCUMENT_TABLE).await?;
    let needle = format!("btih:{}", torrent.info_hash.to_lowercase());
    let target = match docs
        .into_iter()
        .find(|d| matches_magnet(&d.files, &needle))
    {
        Some(d) => d,
        None => return Ok(false),
    };

    let output_path = match torrent.output_path.as_deref() {
        Some(p) => PathBuf::from(p),
        None => return Ok(false),
    };
    if !output_path.exists() {
        // Path may not yet exist if the cached output_path is stale (e.g.
        // magnet dn vs resolved metadata name). Bubble up so the watcher
        // retries on the next tick instead of permanently marking processed.
        anyhow::bail!("output path does not exist yet: {}", output_path.display());
    }

    let walked = walk_torrent_files(&output_path);
    if walked.is_empty() {
        anyhow::bail!("no files yet under {}", output_path.display());
    }

    let existing_titles: HashSet<String> = target
        .files
        .iter()
        .filter(|f| f.kind == "ipfs")
        .filter_map(|f| f.title.clone())
        .collect();

    let mut new_entries: Vec<FileEntry> = Vec::new();
    for file in &walked {
        if existing_titles.contains(&file.relative_path) {
            continue;
        }
        let req = mhaol_ipfs::AddIpfsRequest {
            source: file.path.to_string_lossy().to_string(),
            pin: Some(true),
        };
        let info = state.ipfs_manager.add(req).await?;
        let mime = mime_guess::from_path(&file.path)
            .first()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_default();
        let _ = ipfs_pins::record_pin(
            state,
            info.cid.clone(),
            file.path.to_string_lossy().to_string(),
            mime,
            info.size,
        )
        .await;
        new_entries.push(FileEntry {
            kind: "ipfs".to_string(),
            value: info.cid,
            title: Some(file.relative_path.clone()),
        });
    }

    if new_entries.is_empty() {
        return Ok(false);
    }

    let old_id = target
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .unwrap_or_default();
    if old_id.is_empty() {
        return Ok(false);
    }

    let mut updated_files = target.files.clone();
    updated_files.extend(new_entries);

    let new_version = target.version.saturating_add(1);
    let mut new_hashes = target.version_hashes.clone();
    new_hashes.push(old_id.clone());

    let new_id = compute_document_cid(
        &target.title,
        &target.description,
        &target.artists,
        &target.images,
        &updated_files,
        target.year,
        &target.kind,
        &target.source,
        new_version,
        &new_hashes,
    );

    if new_id == old_id {
        // Defensive: nothing actually changed.
        return Ok(false);
    }

    let new_record = Document {
        id: None,
        title: target.title.clone(),
        artists: target.artists.clone(),
        description: target.description.clone(),
        images: target.images.clone(),
        files: updated_files,
        year: target.year,
        kind: target.kind.clone(),
        source: target.source.clone(),
        created_at: target.created_at,
        updated_at: chrono::Utc::now(),
        version: new_version,
        version_hashes: new_hashes,
    };

    let _: Option<Document> = state
        .db
        .delete((DOCUMENT_TABLE, old_id.as_str()))
        .await?;
    let _: Option<Document> = state
        .db
        .create((DOCUMENT_TABLE, new_id.as_str()))
        .content(new_record)
        .await?;

    tracing::info!(
        "[torrent-completion] {} → {} (v{}, +{} file(s))",
        old_id,
        new_id,
        new_version,
        walked.len()
    );

    Ok(true)
}

#[cfg(not(target_os = "android"))]
fn matches_magnet(files: &[FileEntry], needle: &str) -> bool {
    files
        .iter()
        .any(|f| f.kind == "torrent magnet" && f.value.to_lowercase().contains(needle))
}

#[cfg(not(target_os = "android"))]
struct WalkedFile {
    path: PathBuf,
    relative_path: String,
}

#[cfg(not(target_os = "android"))]
fn walk_torrent_files(root: &Path) -> Vec<WalkedFile> {
    let mut out = Vec::new();
    if root.is_file() {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(WalkedFile {
            path: root.to_path_buf(),
            relative_path: name,
        });
        return out;
    }
    if !root.is_dir() {
        return out;
    }
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        let rel = path
            .strip_prefix(root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            });
        out.push(WalkedFile {
            path,
            relative_path: rel,
        });
    }
    out
}

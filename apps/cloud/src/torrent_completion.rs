//! Background watcher that detects finished torrents and rolls the matching
//! firkin forward by pinning each downloaded file to IPFS, appending them as
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
use crate::firkins::{
    compute_firkin_cid, pin_firkin_body, serialize_firkin_payload, FileEntry, Firkin,
    TABLE as FIRKIN_TABLE,
};
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
    let docs: Vec<Firkin> = state.db.select(FIRKIN_TABLE).await?;
    let needle = format!("btih:{}", torrent.info_hash.to_lowercase());
    let target = match docs
        .into_iter()
        .find(|d| matches_magnet(&d.files, &needle))
    {
        Some(d) => d,
        None => return Ok(false),
    };

    rollforward(state, &target, torrent).await
}

/// Pin every downloaded file from `torrent` into IPFS, append them to `doc`'s
/// files, and roll the firkin forward to a new CID. Returns `Ok(true)` when
/// the firkin was rolled forward, `Ok(false)` when nothing changed.
#[cfg(not(target_os = "android"))]
async fn rollforward(
    state: &CloudState,
    doc: &Firkin,
    torrent: &TorrentInfo,
) -> anyhow::Result<bool> {
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

    let existing_titles: HashSet<String> = doc
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
        // A single bad file (e.g. rust-ipfs's add_unixfs returning
        // "invalid data" on a 0-byte or symlink entry inside the torrent)
        // used to bubble through `?` and abort the rollforward — which
        // meant the watcher retried the same torrent forever, the firkin
        // never gained any IPFS files, and the ROM scan fell back on a
        // half-pinned tree. Log + skip per file instead so the rest land.
        let info = match state.ipfs_manager.add(req).await {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!(
                    "[torrent-completion] skip {}: ipfs add failed: {e}",
                    file.path.display()
                );
                continue;
            }
        };
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

    let old_id = doc.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
    if old_id.is_empty() {
        return Ok(false);
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
    );

    if new_id == old_id {
        // Defensive: nothing actually changed.
        return Ok(false);
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
        updated_at: chrono::Utc::now(),
        version: new_version,
        version_hashes: new_hashes,
    };

    let _: Option<Firkin> = state
        .db
        .delete((FIRKIN_TABLE, old_id.as_str()))
        .await?;
    let create_result: Result<Option<Firkin>, _> = state
        .db
        .create((FIRKIN_TABLE, new_id.as_str()))
        .content(new_record)
        .await;
    match create_result {
        Ok(_) => {}
        Err(e) => {
            // Concurrent rollforward attempts (watcher + manual finalize) can
            // both compute the same `new_id`. Whichever runs second sees the
            // record already in place; treat that as success — the desired
            // state is already on disk.
            let msg = e.to_string();
            if msg.contains("already exists") {
                tracing::info!(
                    "[torrent-completion] {} → {} already created by a concurrent attempt",
                    old_id,
                    new_id,
                );
                return Ok(true);
            }
            return Err(e.into());
        }
    }

    pin_firkin_body(state, &new_id, new_body_json).await;

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

/// On-demand rollforward: pin every completed torrent attached to `doc_id`
/// and roll the firkin forward. Returns the latest firkin id (the new
/// CID after rollforward, or `doc_id` unchanged if nothing was finalized).
/// `Ok(None)` means the firkin does not exist.
#[cfg(not(target_os = "android"))]
pub async fn finalize_firkin(
    state: &CloudState,
    doc_id: &str,
) -> anyhow::Result<Option<String>> {
    let mut current: Firkin = match state.db.select((FIRKIN_TABLE, doc_id)).await? {
        Some(d) => d,
        None => return Ok(None),
    };
    let mut current_id = doc_id.to_string();

    if !state.torrent_manager.is_initialized() {
        return Ok(Some(current_id));
    }

    let torrents = state.torrent_manager.list().await?;

    let hashes: Vec<String> = current
        .files
        .iter()
        .filter(|f| f.kind == "torrent magnet")
        .filter_map(|f| {
            let lower = f.value.to_lowercase();
            let idx = lower.find("btih:")?;
            let tail = &lower[idx + "btih:".len()..];
            let end = tail.find('&').unwrap_or(tail.len());
            let hash = tail[..end].trim().to_string();
            if hash.is_empty() {
                None
            } else {
                Some(hash)
            }
        })
        .collect();

    for hash in hashes {
        let torrent = match torrents.iter().find(|t| t.info_hash.to_lowercase() == hash) {
            Some(t) => t,
            None => continue,
        };
        let finished = matches!(torrent.state, TorrentState::Seeding) || torrent.progress >= 1.0;
        if !finished {
            continue;
        }
        match rollforward(state, &current, torrent).await {
            Ok(true) => {
                let docs: Vec<Firkin> = state.db.select(FIRKIN_TABLE).await?;
                if let Some(next) = docs
                    .into_iter()
                    .find(|d| d.version_hashes.iter().any(|h| h == &current_id))
                {
                    current_id = next
                        .id
                        .as_ref()
                        .map(|t| t.id.to_raw())
                        .unwrap_or(current_id);
                    current = next;
                }
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(
                    "[torrent-completion] finalize failed for {} ({}): {e}",
                    current_id,
                    hash
                );
                return Err(e);
            }
        }
    }

    Ok(Some(current_id))
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

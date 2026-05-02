//! Background watcher that detects finished torrents and updates the
//! matching firkin in place by pinning each downloaded file to IPFS,
//! appending them as `ipfs` file entries, and recomputing the firkin's
//! body CID.
//!
//! Versioning: each completion bumps `version` and pushes the prior CID
//! onto `version_hashes`. The record id (a stable UUID) is left alone.

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
use crate::firkins::{rollforward_firkin, FileEntry, Firkin, TABLE as FIRKIN_TABLE};
#[cfg(not(target_os = "android"))]
use crate::ipfs_pins;
#[cfg(not(target_os = "android"))]
use crate::state::CloudState;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::{TorrentInfo, TorrentState};

#[cfg(not(target_os = "android"))]
const POLL_INTERVAL: Duration = Duration::from_secs(3);

#[cfg(not(target_os = "android"))]
#[derive(Debug)]
enum CompletionResult {
    /// At least one matching firkin was rolled forward this tick.
    RolledForward,
    /// One or more matching firkins existed but all were already up to date.
    AlreadyLinked,
    /// No firkin currently references this torrent's magnet — keep checking
    /// in case the user bookmarks it later.
    NoMatch,
}

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
                Ok(CompletionResult::RolledForward) | Ok(CompletionResult::AlreadyLinked) => {
                    processed.lock().insert(t.info_hash.clone());
                }
                Ok(CompletionResult::NoMatch) => {
                    // Leave unmarked: a firkin may be created later that
                    // references this torrent's magnet, and the watcher
                    // should pick it up on a subsequent tick.
                }
                Err(e) => {
                    tracing::warn!("[torrent-completion] failed for {}: {e}", t.info_hash);
                    // Leave it unmarked so we retry on the next tick.
                }
            }
        }
    }
}

#[cfg(not(target_os = "android"))]
async fn handle_completion(
    state: &CloudState,
    torrent: &TorrentInfo,
) -> anyhow::Result<CompletionResult> {
    let docs: Vec<Firkin> = state.db.select(FIRKIN_TABLE).await?;

    let needle = format!("btih:{}", torrent.info_hash.to_lowercase());
    let matches: Vec<Firkin> = docs
        .into_iter()
        .filter(|d| matches_magnet(&d.files, &needle))
        .collect();

    if matches.is_empty() {
        return Ok(CompletionResult::NoMatch);
    }

    // Multiple firkins can legitimately reference the same magnet (e.g. the
    // user bookmarked the same item twice, or two firkins that share a
    // torrent). Roll each one forward independently so none are stranded
    // without IPFS files.
    let mut any_rolled = false;
    let mut last_err: Option<anyhow::Error> = None;
    for doc in &matches {
        let id = doc.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        // Hold the per-firkin lock across the (heavy) IPFS pin loop and
        // the rollforward write so a concurrent `PUT /api/firkins/:id`
        // (or any other mutation) can't slip in between our re-read and
        // write and have its change silently overwritten.
        let _firkin_guard = state.firkin_lock(&id).lock_owned().await;
        match rollforward(state, doc, torrent).await {
            Ok(true) => any_rolled = true,
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(
                    "[torrent-completion] rollforward failed for firkin {id} ({}): {e}",
                    torrent.info_hash
                );
                last_err = Some(e);
            }
        }
    }

    if let Some(e) = last_err {
        return Err(e);
    }

    Ok(if any_rolled {
        CompletionResult::RolledForward
    } else {
        CompletionResult::AlreadyLinked
    })
}

/// Pin every downloaded file from `torrent` into IPFS, append them to
/// the firkin's files, and roll the firkin's body forward via the
/// canonical [`firkins::rollforward_firkin`]. Returns `Ok(true)` when
/// the firkin was updated, `Ok(false)` when nothing changed.
///
/// **Caller must hold `state.firkin_lock(<doc id>)`** for the entire
/// duration. The function intentionally re-reads the firkin from
/// SurrealDB *after* IPFS pinning so the merge is against the latest
/// committed state, not the stale `doc` snapshot the polling loop saw.
/// Without that re-read, a concurrent `PUT /api/firkins/:id` (e.g. the
/// catalog detail page persisting a freshly-resolved trailer) would be
/// silently clobbered when this rollforward writes its stale snapshot
/// back. The caller-held lock is what guarantees the re-read sees a
/// consistent view through the rollforward write.
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

    let id = doc.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
    if id.is_empty() {
        return Ok(false);
    }

    // Cheap pre-pin dedup against the snapshot we already have. The
    // authoritative re-check happens against the freshly-loaded record
    // below — this just avoids redundant IPFS work for files we already
    // pinned in a prior run.
    let snapshot_titles: HashSet<String> = doc
        .files
        .iter()
        .filter(|f| f.kind == "ipfs")
        .filter_map(|f| f.title.clone())
        .collect();

    let mut new_entries: Vec<FileEntry> = Vec::new();
    for file in &walked {
        let title = format_file_title(&doc.addon, &file.relative_path);
        if snapshot_titles.contains(&title) || snapshot_titles.contains(&file.relative_path) {
            continue;
        }
        let req = mhaol_ipfs_core::AddIpfsRequest {
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
            title: Some(title),
        });
    }

    if new_entries.is_empty() {
        return Ok(false);
    }

    // Re-read under the caller-held lock so the merge sees any concurrent
    // mutations (subtitle attach, trailer/youtube-preferred-client persist,
    // manual PUT, …) that landed while we were pinning.
    let fresh: Option<Firkin> = state.db.select((FIRKIN_TABLE, id.as_str())).await?;
    let mut fresh =
        fresh.ok_or_else(|| anyhow::anyhow!("firkin {id} disappeared before rollforward"))?;

    let fresh_titles: HashSet<String> = fresh
        .files
        .iter()
        .filter(|f| f.kind == "ipfs")
        .filter_map(|f| f.title.clone())
        .collect();
    let mut to_append: Vec<FileEntry> = Vec::new();
    for entry in new_entries {
        let title = entry.title.clone().unwrap_or_default();
        if !title.is_empty() && fresh_titles.contains(&title) {
            continue;
        }
        to_append.push(entry);
    }
    if to_append.is_empty() {
        return Ok(false);
    }

    let added = to_append.len();
    fresh.files.extend(to_append);
    fresh.id = None;
    fresh.updated_at = chrono::Utc::now();

    let updated = rollforward_firkin(state, &id, fresh)
        .await
        .map_err(|(s, j)| {
            let msg = j
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("rollforward failed")
                .to_string();
            anyhow::anyhow!("{s}: {msg}")
        })?;

    tracing::info!(
        "[torrent-completion] {id} → cid {} (v{}, +{added} file(s))",
        updated.cid,
        updated.version,
    );

    Ok(true)
}

#[cfg(not(target_os = "android"))]
fn matches_magnet(files: &[FileEntry], needle: &str) -> bool {
    files
        .iter()
        .any(|f| f.kind == "torrent magnet" && f.value.to_lowercase().contains(needle))
}

/// On-demand update: pin every completed torrent attached to `doc_id`
/// and update the firkin in place. Returns the firkin id (unchanged
/// across updates since the record id is a stable UUID), or `Ok(None)`
/// if the firkin does not exist.
///
/// **Caller must already hold `state.firkin_lock(doc_id)`** — this
/// function and the [`rollforward`] helper it calls expect that lock to
/// be held for the entire read-modify-write sequence. The handler at
/// [`crate::firkins::finalize`] takes the lock before delegating here.
#[cfg(not(target_os = "android"))]
pub async fn finalize_firkin(state: &CloudState, doc_id: &str) -> anyhow::Result<Option<String>> {
    let mut current: Firkin = match state.db.select((FIRKIN_TABLE, doc_id)).await? {
        Some(d) => d,
        None => return Ok(None),
    };
    let id = doc_id.to_string();

    if !state.torrent_manager.is_initialized() {
        return Ok(Some(id));
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
                if let Some(refreshed) = state.db.select((FIRKIN_TABLE, id.as_str())).await? {
                    current = refreshed;
                }
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(
                    "[torrent-completion] finalize failed for {} ({}): {e}",
                    id,
                    hash
                );
                return Err(e);
            }
        }
    }

    Ok(Some(id))
}

/// Format the FileEntry title for a downloaded torrent file. For
/// `tmdb-tv` firkins we lift the `SxxExx` token out of the relative
/// path (`Show.Name.S02E03.1080p.mkv` → `S02E03 — Show.Name.S02E03.1080p.mkv`)
/// so the firkin's files can be queried per-episode without re-parsing
/// the filename downstream. Other addons keep the raw relative path.
#[cfg(not(target_os = "android"))]
fn format_file_title(addon: &str, relative_path: &str) -> String {
    if addon != "tmdb-tv" {
        return relative_path.to_string();
    }
    use std::sync::OnceLock;
    static SE_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = SE_RE.get_or_init(|| {
        regex::Regex::new(r"(?i)\bs(\d{1,2})\s*e(\d{1,3})\b").expect("static regex compiles")
    });
    match re.captures(relative_path) {
        Some(caps) => {
            let s: u32 = caps[1].parse().unwrap_or(0);
            let e: u32 = caps[2].parse().unwrap_or(0);
            format!("S{s:02}E{e:02} — {relative_path}")
        }
        None => relative_path.to_string(),
    }
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

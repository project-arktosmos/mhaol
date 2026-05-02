//! `/api/disk` — host disk inventory plus a per-subdir breakdown of the
//! cloud's on-disk data root (`<data_root>` from `paths.rs`). Used by the
//! frontend's `/disk` page.

use crate::paths;
use crate::state::CloudState;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub kind: String,
    pub is_removable: bool,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub is_data_root_disk: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubdirInfo {
    /// Display name relative to data_root (e.g. "db", "downloads/torrents").
    pub name: String,
    /// Absolute path on disk.
    pub path: String,
    pub kind: SubdirKind,
    pub exists: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SubdirKind {
    Dir,
    File,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskResponse {
    pub data_root: String,
    pub data_root_total_bytes: u64,
    pub data_root_mount_point: Option<String>,
    pub disks: Vec<DiskInfo>,
    pub subdirs: Vec<SubdirInfo>,
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/", get(get_disks))
}

async fn get_disks() -> Json<DiskResponse> {
    let data_root = paths::data_root();
    let data_root_str = data_root.to_string_lossy().to_string();

    let disks_raw = Disks::new_with_refreshed_list();

    // The disk hosting the data root is the one whose mount point is the
    // longest prefix of the data root path.
    let data_root_mount: Option<PathBuf> = disks_raw
        .iter()
        .filter_map(|d| {
            let mp = d.mount_point().to_path_buf();
            if data_root.starts_with(&mp) {
                Some(mp)
            } else {
                None
            }
        })
        .max_by_key(|mp| mp.as_os_str().len());

    let mut disks: Vec<DiskInfo> = disks_raw
        .iter()
        .map(|d| {
            let mp = d.mount_point().to_path_buf();
            let total = d.total_space();
            let available = d.available_space();
            DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: mp.to_string_lossy().to_string(),
                file_system: d.file_system().to_string_lossy().to_string(),
                kind: format!("{:?}", d.kind()),
                is_removable: d.is_removable(),
                total_bytes: total,
                available_bytes: available,
                used_bytes: total.saturating_sub(available),
                is_data_root_disk: data_root_mount.as_ref().map(|m| m == &mp).unwrap_or(false),
            }
        })
        .collect();

    // Stable order: data-root disk first, then by mount point.
    disks.sort_by(|a, b| match (b.is_data_root_disk, a.is_data_root_disk) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _ => a.mount_point.cmp(&b.mount_point),
    });

    let subdirs = collect_subdirs(&data_root);
    let data_root_total_bytes = subdirs.iter().map(|s| s.size_bytes).sum();

    Json(DiskResponse {
        data_root: data_root_str,
        data_root_total_bytes,
        data_root_mount_point: data_root_mount.map(|p| p.to_string_lossy().to_string()),
        disks,
        subdirs,
    })
}

/// Reports a stable list of the data root's known subdirs (matches
/// `packages/backend/src/paths.rs`), plus any extra top-level entries the
/// user might have dropped in. Each dir is recursively summed; missing
/// entries are reported with `exists: false` and size 0.
fn collect_subdirs(data_root: &Path) -> Vec<SubdirInfo> {
    let known: &[(&str, &str)] = &[
        ("db", "db"),
        ("identities", "identities"),
        ("swarm.key", "swarm.key"),
        ("downloads/torrents", "downloads/torrents"),
        ("downloads/torrent-streams", "downloads/torrent-streams"),
        ("downloads/ipfs", "downloads/ipfs"),
        ("downloads/ipfs-stream", "downloads/ipfs-stream"),
        ("downloads/youtube", "downloads/youtube"),
    ];

    let mut accounted: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut out: Vec<SubdirInfo> = Vec::new();

    for (name, rel) in known {
        let path = data_root.join(rel);
        accounted.insert(path.clone());
        let info = describe_path(name, &path);
        out.push(info);
    }

    // Surface any extra top-level entries (or unexpected entries directly
    // under `downloads/`) the user may have placed in the data root, so the
    // page reflects reality instead of just our defaults.
    if let Ok(read) = std::fs::read_dir(data_root) {
        for entry in read.flatten() {
            let path = entry.path();
            if accounted.contains(&path) {
                continue;
            }
            let parent_known = matches!(path.file_name().and_then(|n| n.to_str()), Some("downloads"));
            if parent_known {
                if let Ok(children) = std::fs::read_dir(&path) {
                    for child in children.flatten() {
                        let cpath = child.path();
                        if accounted.contains(&cpath) {
                            continue;
                        }
                        let name = format!(
                            "downloads/{}",
                            child.file_name().to_string_lossy()
                        );
                        out.push(describe_path(&name, &cpath));
                    }
                }
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            out.push(describe_path(&name, &path));
        }
    }

    out
}

fn describe_path(name: &str, path: &Path) -> SubdirInfo {
    let display_path = path.to_string_lossy().to_string();
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => {
            return SubdirInfo {
                name: name.to_string(),
                path: display_path,
                kind: SubdirKind::Dir,
                exists: false,
                size_bytes: 0,
            };
        }
    };
    if meta.is_file() {
        return SubdirInfo {
            name: name.to_string(),
            path: display_path,
            kind: SubdirKind::File,
            exists: true,
            size_bytes: meta.len(),
        };
    }
    SubdirInfo {
        name: name.to_string(),
        path: display_path,
        kind: SubdirKind::Dir,
        exists: true,
        size_bytes: dir_size(path),
    }
}

fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

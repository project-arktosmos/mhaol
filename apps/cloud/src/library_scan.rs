//! Detects movies, TV shows, music albums, books, and games from a scanned
//! library directory and persists them as `firkin` records (one per movie,
//! show, album, book, or game).
//!
//! For each library, the scan task uses the library's `kinds` to decide which
//! detectors to run. Files matched by a detector are pinned to IPFS (so the
//! resulting firkins have content-addressed file entries) and grouped under
//! a single `firkin` whose `files` list mirrors the IPFS pins.
//!
//! Re-running the scan is idempotent: existing files (matched by relative
//! path under the library) are skipped, and existing firkins (matched by
//! `(title, kind, source="local")`) are version-rolled forward with the new
//! file entries.

#![cfg(not(target_os = "android"))]

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::firkins::{
    compute_firkin_cid, FileEntry, Firkin, TABLE as FIRKIN_TABLE,
};
use crate::ipfs_pins;
use crate::libraries::{is_pinnable_mime, wait_for_ipfs_ready, ScanEntry};
use crate::state::CloudState;

const FIRKIN_SOURCE_LOCAL: &str = "local";
const FIRKIN_KIND_MOVIE: &str = "movie";
const FIRKIN_KIND_TV_SHOW: &str = "tv show";
const FIRKIN_KIND_ALBUM: &str = "album";
const FIRKIN_KIND_BOOK: &str = "book";
const FIRKIN_KIND_GAME: &str = "game";

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

static EPISODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)s(\d{1,3})[\s._-]*e(\d{1,4})").unwrap());
static EPISODE_X_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(\d{1,3})x(\d{1,4})\b").unwrap());
static SEASON_DIR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(?:season[\s._-]*)?(\d{1,3})$|^s(\d{1,3})$").unwrap());
static YEAR_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((\d{4})\)").unwrap());
static TRACK_PREFIX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{1,3})\b").unwrap());

#[derive(Debug, Clone)]
struct GroupFile {
    absolute_path: String,
    mime: String,
    size: u64,
    /// Display title used as the IPFS file entry's `title` field.
    title: String,
}

#[derive(Debug, Clone)]
struct MediaGroup {
    kind: &'static str,
    title: String,
    description: String,
    year: Option<i32>,
    files: Vec<GroupFile>,
}

pub fn schedule_pins_and_firkins(
    state: &CloudState,
    entries: &[ScanEntry],
    kinds: Vec<String>,
    lib_root: String,
) {
    let groups = if kinds.is_empty() {
        Vec::new()
    } else {
        detect_media_groups(entries, &kinds)
    };

    // Files that fall outside any detected group still get pinned (matches
    // the previous schedule_audio_pins behavior for libraries with no kinds,
    // and keeps stragglers like loose images/audio reachable for the legacy
    // `/api/libraries/:id/pins` view).
    let in_group_paths: std::collections::HashSet<String> = groups
        .iter()
        .flat_map(|g| g.files.iter().map(|f| f.absolute_path.clone()))
        .collect();
    let stragglers: Vec<(String, String, u64)> = entries
        .iter()
        .filter(|e| is_pinnable_mime(&e.mime))
        .filter(|e| !in_group_paths.contains(&e.path))
        .map(|e| (e.path.clone(), e.mime.clone(), e.size))
        .collect();

    if groups.is_empty() && stragglers.is_empty() {
        tracing::info!("[library-scan] {lib_root}: nothing to pin");
        return;
    }

    let ipfs = state.ipfs_manager.clone();
    let state = state.clone();
    let total_groups = groups.len();
    let total_stragglers = stragglers.len();
    tokio::spawn(async move {
        if !wait_for_ipfs_ready(&ipfs).await {
            tracing::warn!(
                "[library-scan] {lib_root}: ipfs node never reached running state — skipping {} group(s) / {} file(s)",
                total_groups,
                total_stragglers
            );
            return;
        }

        if total_groups > 0 {
            tracing::info!(
                "[library-scan] {lib_root}: pinning {} media group(s)",
                total_groups
            );
        }
        for group in groups {
            if let Err(e) = pin_and_persist_group(&state, &ipfs, &group).await {
                tracing::warn!(
                    "[library-scan] failed to persist {:?} group {:?}: {e}",
                    group.kind,
                    group.title
                );
            }
        }

        for (path, mime, size) in stragglers {
            let req = mhaol_ipfs::AddIpfsRequest {
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

async fn pin_and_persist_group(
    state: &CloudState,
    ipfs: &std::sync::Arc<mhaol_ipfs::IpfsManager>,
    group: &MediaGroup,
) -> anyhow::Result<()> {
    let mut new_entries: Vec<FileEntry> = Vec::with_capacity(group.files.len());
    for f in &group.files {
        let req = mhaol_ipfs::AddIpfsRequest {
            source: f.absolute_path.clone(),
            pin: Some(true),
        };
        let info = ipfs.add(req).await?;
        let _ = ipfs_pins::record_pin(
            state,
            info.cid.clone(),
            f.absolute_path.clone(),
            f.mime.clone(),
            f.size,
        )
        .await;
        new_entries.push(FileEntry {
            kind: "ipfs".to_string(),
            value: info.cid,
            title: Some(f.title.clone()),
        });
    }

    if new_entries.is_empty() {
        return Ok(());
    }

    let docs: Vec<Firkin> = state.db.select(FIRKIN_TABLE).await?;
    let existing = docs.into_iter().find(|d| {
        d.kind == group.kind && d.source == FIRKIN_SOURCE_LOCAL && d.title == group.title
    });

    match existing {
        Some(existing) => upsert_firkin_with_files(state, existing, new_entries).await,
        None => create_firkin(state, group, new_entries).await,
    }
}

async fn create_firkin(
    state: &CloudState,
    group: &MediaGroup,
    files: Vec<FileEntry>,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let version: u32 = 0;
    let version_hashes: Vec<String> = Vec::new();
    let new_id = compute_firkin_cid(
        &group.title,
        &group.description,
        &[],
        &[],
        &files,
        group.year,
        group.kind,
        FIRKIN_SOURCE_LOCAL,
        version,
        &version_hashes,
    );

    let already: Option<Firkin> = state.db.select((FIRKIN_TABLE, new_id.as_str())).await?;
    if already.is_some() {
        return Ok(());
    }

    let record = Firkin {
        id: None,
        title: group.title.clone(),
        artists: Vec::new(),
        description: group.description.clone(),
        images: Vec::new(),
        files,
        year: group.year,
        kind: group.kind.to_string(),
        source: FIRKIN_SOURCE_LOCAL.to_string(),
        created_at: now,
        updated_at: now,
        version,
        version_hashes,
    };

    let _: Option<Firkin> = state
        .db
        .create((FIRKIN_TABLE, new_id.as_str()))
        .content(record)
        .await?;
    tracing::info!(
        "[library-scan] created firkin {} ({}, {} file(s))",
        new_id,
        group.kind,
        group.files.len()
    );
    Ok(())
}

async fn upsert_firkin_with_files(
    state: &CloudState,
    existing: Firkin,
    new_entries: Vec<FileEntry>,
) -> anyhow::Result<()> {
    let existing_titles: std::collections::HashSet<String> = existing
        .files
        .iter()
        .filter(|f| f.kind == "ipfs")
        .filter_map(|f| f.title.clone())
        .collect();
    let to_add: Vec<FileEntry> = new_entries
        .into_iter()
        .filter(|f| {
            f.title
                .as_ref()
                .map(|t| !existing_titles.contains(t))
                .unwrap_or(true)
        })
        .collect();
    if to_add.is_empty() {
        return Ok(());
    }

    let old_id = existing
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .unwrap_or_default();
    if old_id.is_empty() {
        return Ok(());
    }

    let mut updated_files = existing.files.clone();
    updated_files.extend(to_add);

    let new_version = existing.version.saturating_add(1);
    let mut new_hashes = existing.version_hashes.clone();
    new_hashes.push(old_id.clone());

    let new_id = compute_firkin_cid(
        &existing.title,
        &existing.description,
        &existing.artists,
        &existing.images,
        &updated_files,
        existing.year,
        &existing.kind,
        &existing.source,
        new_version,
        &new_hashes,
    );

    if new_id == old_id {
        return Ok(());
    }

    let new_record = Firkin {
        id: None,
        title: existing.title.clone(),
        artists: existing.artists.clone(),
        description: existing.description.clone(),
        images: existing.images.clone(),
        files: updated_files,
        year: existing.year,
        kind: existing.kind.clone(),
        source: existing.source.clone(),
        created_at: existing.created_at,
        updated_at: Utc::now(),
        version: new_version,
        version_hashes: new_hashes,
    };

    let _: Option<Firkin> = state
        .db
        .delete((FIRKIN_TABLE, old_id.as_str()))
        .await?;
    let _: Option<Firkin> = state
        .db
        .create((FIRKIN_TABLE, new_id.as_str()))
        .content(new_record)
        .await?;
    tracing::info!(
        "[library-scan] {} → {} ({} v{})",
        old_id,
        new_id,
        existing.kind,
        new_version,
    );
    Ok(())
}

// ---------- detection ----------

fn detect_media_groups(entries: &[ScanEntry], kinds: &[String]) -> Vec<MediaGroup> {
    let want_movie = kinds.iter().any(|k| k == "movie");
    let want_tv = kinds.iter().any(|k| k == "tv");
    let want_album = kinds.iter().any(|k| k == "album");
    let want_book = kinds.iter().any(|k| k == "book");
    let want_game = kinds.iter().any(|k| k == "game");

    let mut groups: Vec<MediaGroup> = Vec::new();
    let mut tv_video_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    if want_tv {
        let tv = detect_tv_shows(entries);
        for g in &tv {
            for f in &g.files {
                tv_video_paths.insert(f.absolute_path.clone());
            }
        }
        groups.extend(tv);
    }

    if want_movie {
        groups.extend(detect_movies(entries, &tv_video_paths));
    }

    if want_album {
        groups.extend(detect_albums(entries));
    }

    if want_book {
        groups.extend(detect_books(entries));
    }

    if want_game {
        groups.extend(detect_games(entries));
    }

    groups
}

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

/// Strip the `(YYYY)` tag (if any) and return `(title, year)`.
fn split_year(name: &str) -> (String, Option<i32>) {
    if let Some(c) = YEAR_TAG_RE.captures(name) {
        let year = c.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
        let stripped = YEAR_TAG_RE.replace(name, "").trim().to_string();
        let cleaned = stripped.trim_end_matches(|c: char| matches!(c, '.' | '-' | '_' | ' ')).to_string();
        return (cleaned, year);
    }
    (name.to_string(), None)
}

fn humanize(name: &str) -> String {
    name.replace(['.', '_'], " ").trim().to_string()
}

fn file_stem(path: &str) -> String {
    PathBuf::from(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

fn relative_components(rel: &str) -> Vec<String> {
    PathBuf::from(rel)
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str().map(|x| x.to_string()),
            _ => None,
        })
        .collect()
}

/// True when the directory name looks like `Season 01`, `season 1`, `S01`,
/// `S1`, or just a bare number that sits under another directory.
fn is_season_dir(name: &str) -> Option<u32> {
    let trimmed = name.trim();
    if let Some(c) = SEASON_DIR_RE.captures(trimmed) {
        let n = c
            .get(1)
            .or_else(|| c.get(2))
            .and_then(|m| m.as_str().parse::<u32>().ok());
        if n.is_some() {
            return n;
        }
    }
    None
}

fn parse_episode(name: &str) -> Option<(u32, u32)> {
    if let Some(c) = EPISODE_RE.captures(name) {
        let s = c.get(1)?.as_str().parse::<u32>().ok()?;
        let e = c.get(2)?.as_str().parse::<u32>().ok()?;
        return Some((s, e));
    }
    if let Some(c) = EPISODE_X_RE.captures(name) {
        let s = c.get(1)?.as_str().parse::<u32>().ok()?;
        let e = c.get(2)?.as_str().parse::<u32>().ok()?;
        return Some((s, e));
    }
    None
}

fn detect_tv_shows(entries: &[ScanEntry]) -> Vec<MediaGroup> {
    let mut by_show: BTreeMap<String, (Option<i32>, Vec<(u32, u32, GroupFile)>)> =
        BTreeMap::new();

    for e in entries.iter().filter(|e| is_video(e)) {
        let comps = relative_components(&e.relative_path);
        if comps.len() < 2 {
            // A loose video file at the library root has no folder context — we
            // can only treat it as TV if the filename itself carries SxxExx.
            let stem = file_stem(&e.relative_path);
            if let Some((season, episode)) = parse_episode(&stem) {
                let (show_title, year) = split_year(&humanize(&stem));
                let show_title = show_title
                    .split(|ch: char| matches!(ch, '.' | '-' | '_' | ' '))
                    .take_while(|p| {
                        !EPISODE_RE.is_match(p) && !EPISODE_X_RE.is_match(p)
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                let key = if show_title.is_empty() {
                    "Unknown Show".to_string()
                } else {
                    show_title
                };
                let title = format!("S{:02}E{:02} - {}", season, episode, file_stem(&e.relative_path));
                by_show
                    .entry(key)
                    .or_insert((year, Vec::new()))
                    .1
                    .push((
                        season,
                        episode,
                        GroupFile {
                            absolute_path: e.path.clone(),
                            mime: e.mime.clone(),
                            size: e.size,
                            title,
                        },
                    ));
            }
            continue;
        }

        // Walk the path components to find a season dir; the dir above it is
        // the show. If no season dir exists, the show is the top component
        // and we infer season from the filename.
        let leaf = comps.last().cloned().unwrap_or_default();
        let stem = PathBuf::from(&leaf)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or(leaf.clone());

        let mut show_idx: Option<usize> = None;
        let mut season_from_dir: Option<u32> = None;
        for (i, c) in comps.iter().enumerate() {
            if i == comps.len() - 1 {
                break;
            }
            if let Some(n) = is_season_dir(c) {
                show_idx = if i == 0 { Some(0) } else { Some(i - 1) };
                season_from_dir = Some(n);
                break;
            }
        }

        let (season, episode) = match parse_episode(&stem) {
            Some(p) => p,
            None => match season_from_dir {
                Some(s) => {
                    // Best-effort episode index from a leading number in the
                    // filename ("01 - Pilot.mkv").
                    let ep = TRACK_PREFIX_RE
                        .captures(&stem)
                        .and_then(|c| c.get(1))
                        .and_then(|m| m.as_str().parse::<u32>().ok())
                        .unwrap_or(0);
                    (s, ep)
                }
                None => continue,
            },
        };

        let raw_show = match show_idx {
            Some(i) => comps.get(i).cloned().unwrap_or_else(|| comps[0].clone()),
            None => comps[0].clone(),
        };
        let (show_title, year) = split_year(&humanize(&raw_show));
        let key = if show_title.is_empty() {
            raw_show
        } else {
            show_title
        };
        let display_title = format!("S{:02}E{:02} - {}", season, episode, stem);
        by_show
            .entry(key)
            .or_insert((year, Vec::new()))
            .1
            .push((
                season,
                episode,
                GroupFile {
                    absolute_path: e.path.clone(),
                    mime: e.mime.clone(),
                    size: e.size,
                    title: display_title,
                },
            ));
    }

    let mut out: Vec<MediaGroup> = Vec::with_capacity(by_show.len());
    for (show, (year, mut eps)) in by_show {
        eps.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        let files: Vec<GroupFile> = eps.into_iter().map(|(_, _, f)| f).collect();
        let description = format!("{} episode(s) detected from local files", files.len());
        out.push(MediaGroup {
            kind: FIRKIN_KIND_TV_SHOW,
            title: show,
            description,
            year,
            files,
        });
    }
    out
}

fn detect_movies(
    entries: &[ScanEntry],
    skip_paths: &std::collections::HashSet<String>,
) -> Vec<MediaGroup> {
    let mut out: Vec<MediaGroup> = Vec::new();
    for e in entries.iter().filter(|e| is_video(e)) {
        if skip_paths.contains(&e.path) {
            continue;
        }
        let stem = file_stem(&e.relative_path);
        // Guard against TV episodes that the TV detector skipped (e.g. when
        // the library only declares "movie") leaking in as movies. They
        // *would* be classified as movies otherwise, which is fine — but if
        // the filename clearly carries SxxExx without a folder context we
        // skip it from movies to keep results sensible.
        if parse_episode(&stem).is_some() {
            continue;
        }
        let comps = relative_components(&e.relative_path);
        let raw_name = if comps.len() >= 2 {
            comps[comps.len() - 2].clone()
        } else {
            stem.clone()
        };
        let (title, year) = split_year(&humanize(&raw_name));
        let title = if title.is_empty() {
            humanize(&stem)
        } else {
            title
        };
        out.push(MediaGroup {
            kind: FIRKIN_KIND_MOVIE,
            title,
            description: String::new(),
            year,
            files: vec![GroupFile {
                absolute_path: e.path.clone(),
                mime: e.mime.clone(),
                size: e.size,
                title: PathBuf::from(&e.relative_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or(stem),
            }],
        });
    }
    out
}

fn detect_albums(entries: &[ScanEntry]) -> Vec<MediaGroup> {
    // Group every audio file by its parent directory. An album lives in one
    // directory; multiple sibling audio files in the same directory form a
    // single album. Loose audio at the library root is grouped under
    // "Singles".
    let mut by_dir: BTreeMap<String, Vec<GroupFile>> = BTreeMap::new();
    for e in entries.iter().filter(|e| is_audio(e)) {
        let comps = relative_components(&e.relative_path);
        let dir_key = if comps.len() >= 2 {
            comps[..comps.len() - 1].join("/")
        } else {
            String::new()
        };
        let stem = file_stem(&e.relative_path);
        let track_title = humanize(&stem);
        by_dir
            .entry(dir_key)
            .or_default()
            .push(GroupFile {
                absolute_path: e.path.clone(),
                mime: e.mime.clone(),
                size: e.size,
                title: track_title,
            });
    }

    let mut out: Vec<MediaGroup> = Vec::with_capacity(by_dir.len());
    for (dir_key, mut files) in by_dir {
        files.sort_by(|a, b| {
            let an = TRACK_PREFIX_RE
                .captures(&a.title)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .unwrap_or(u32::MAX);
            let bn = TRACK_PREFIX_RE
                .captures(&b.title)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .unwrap_or(u32::MAX);
            an.cmp(&bn).then_with(|| a.title.cmp(&b.title))
        });
        let (title, year) = if dir_key.is_empty() {
            ("Singles".to_string(), None)
        } else {
            let leaf = Path::new(&dir_key)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(dir_key.as_str());
            split_year(&humanize(leaf))
        };
        let description = format!("{} track(s) detected from local files", files.len());
        out.push(MediaGroup {
            kind: FIRKIN_KIND_ALBUM,
            title,
            description,
            year,
            files,
        });
    }
    out
}

fn detect_books(entries: &[ScanEntry]) -> Vec<MediaGroup> {
    entries
        .iter()
        .filter(|e| is_book(e))
        .map(|e| {
            let stem = file_stem(&e.relative_path);
            let (title, year) = split_year(&humanize(&stem));
            let title = if title.is_empty() {
                humanize(&stem)
            } else {
                title
            };
            MediaGroup {
                kind: FIRKIN_KIND_BOOK,
                title,
                description: String::new(),
                year,
                files: vec![GroupFile {
                    absolute_path: e.path.clone(),
                    mime: e.mime.clone(),
                    size: e.size,
                    title: PathBuf::from(&e.relative_path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or(stem),
                }],
            }
        })
        .collect()
}

fn detect_games(entries: &[ScanEntry]) -> Vec<MediaGroup> {
    entries
        .iter()
        .filter(|e| is_game(e))
        .map(|e| {
            let stem = file_stem(&e.relative_path);
            let (title, year) = split_year(&humanize(&stem));
            let title = if title.is_empty() {
                humanize(&stem)
            } else {
                title
            };
            MediaGroup {
                kind: FIRKIN_KIND_GAME,
                title,
                description: String::new(),
                year,
                files: vec![GroupFile {
                    absolute_path: e.path.clone(),
                    mime: e.mime.clone(),
                    size: e.size,
                    title: PathBuf::from(&e.relative_path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                        .unwrap_or(stem),
                }],
            }
        })
        .collect()
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
        }
    }

    #[test]
    fn tv_show_detected_from_season_dir() {
        let entries = vec![
            entry("Breaking Bad/Season 01/Breaking.Bad.S01E01.mkv", "video/x-matroska"),
            entry("Breaking Bad/Season 01/Breaking.Bad.S01E02.mkv", "video/x-matroska"),
            entry("Breaking Bad/Season 02/Breaking.Bad.S02E01.mkv", "video/x-matroska"),
        ];
        let groups = detect_media_groups(&entries, &["tv".to_string()]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].kind, FIRKIN_KIND_TV_SHOW);
        assert_eq!(groups[0].title, "Breaking Bad");
        assert_eq!(groups[0].files.len(), 3);
    }

    #[test]
    fn tv_show_detected_from_flat_dir() {
        let entries = vec![
            entry("The Office/The.Office.S01E01.mkv", "video/x-matroska"),
            entry("The Office/The.Office.S01E02.mkv", "video/x-matroska"),
        ];
        let groups = detect_media_groups(&entries, &["tv".to_string()]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].title, "The Office");
        assert_eq!(groups[0].files.len(), 2);
    }

    #[test]
    fn movies_skip_tv_video_paths() {
        let entries = vec![
            entry("Breaking Bad/Season 01/Breaking.Bad.S01E01.mkv", "video/x-matroska"),
            entry("Inception (2010)/Inception.mkv", "video/x-matroska"),
        ];
        let groups = detect_media_groups(
            &entries,
            &["tv".to_string(), "movie".to_string()],
        );
        let movies: Vec<&MediaGroup> = groups.iter().filter(|g| g.kind == FIRKIN_KIND_MOVIE).collect();
        let shows: Vec<&MediaGroup> = groups.iter().filter(|g| g.kind == FIRKIN_KIND_TV_SHOW).collect();
        assert_eq!(shows.len(), 1);
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].title, "Inception");
        assert_eq!(movies[0].year, Some(2010));
    }

    #[test]
    fn albums_grouped_by_directory() {
        let entries = vec![
            entry("Pink Floyd/The Wall/01 - In the Flesh.flac", "audio/flac"),
            entry("Pink Floyd/The Wall/02 - The Thin Ice.flac", "audio/flac"),
            entry("Pink Floyd/Animals/01 - Pigs on the Wing 1.flac", "audio/flac"),
        ];
        let groups = detect_media_groups(&entries, &["album".to_string()]);
        let mut titles: Vec<&str> = groups.iter().map(|g| g.title.as_str()).collect();
        titles.sort();
        assert_eq!(titles, vec!["Animals", "The Wall"]);
        for g in &groups {
            assert_eq!(g.kind, FIRKIN_KIND_ALBUM);
        }
    }

    #[test]
    fn loose_audio_grouped_under_singles() {
        let entries = vec![entry("song.mp3", "audio/mpeg")];
        let groups = detect_media_groups(&entries, &["album".to_string()]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].title, "Singles");
    }

    #[test]
    fn books_one_per_file() {
        let entries = vec![
            entry("Dune (1965).epub", "application/epub+zip"),
            entry("library/1984.pdf", "application/pdf"),
        ];
        let groups = detect_media_groups(&entries, &["book".to_string()]);
        assert_eq!(groups.len(), 2);
        let mut titles: Vec<&str> = groups.iter().map(|g| g.title.as_str()).collect();
        titles.sort();
        assert_eq!(titles, vec!["1984", "Dune"]);
    }

    #[test]
    fn games_one_per_file() {
        let entries = vec![
            entry("Super Metroid.smc", "application/octet-stream"),
            entry("snes/Chrono Trigger.sfc", "application/octet-stream"),
        ];
        let groups = detect_media_groups(&entries, &["game".to_string()]);
        assert_eq!(groups.len(), 2);
    }
}

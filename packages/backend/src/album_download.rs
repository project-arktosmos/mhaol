//! Background album-download task.
//!
//! Walks the MusicBrainz tracklist of a bookmarked firkin, downloads each
//! track's persisted YouTube URL via the yt-dlp manager (audio-only),
//! pins the resulting file to IPFS, appends it to the firkin's `files`
//! as an `ipfs`-typed entry, and rolls the firkin forward. Independent
//! of any HTTP request — the spawned task survives page reloads and
//! tab closures. Per-track progress lives on `state.album_download_progress`
//! so the catalog detail page can render live status.

#![cfg(not(target_os = "android"))]

use crate::firkins::{rollforward_firkin, FileEntry, Firkin, TABLE as FIRKIN_TABLE};
use crate::ipfs_pins;
use crate::state::CloudState;
use crate::track_resolve;
use chrono::{DateTime, Utc};
use mhaol_yt_dlp::{AudioFormat, AudioQuality, DownloadMode, DownloadState, QueueDownloadRequest};
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlbumDownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Serialize)]
pub struct AlbumDownloadTrack {
    pub position: i64,
    pub title: String,
    pub status: AlbumDownloadStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub progress: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AlbumDownloadProgress {
    #[serde(rename = "firkinId")]
    pub firkin_id: String,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub tracks: Vec<AlbumDownloadTrack>,
}

#[derive(Clone, Default)]
pub struct AlbumDownloadProgressMap {
    inner: Arc<RwLock<HashMap<String, AlbumDownloadProgress>>>,
}

impl AlbumDownloadProgressMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, firkin_id: &str) -> Option<AlbumDownloadProgress> {
        self.inner.read().get(firkin_id).cloned()
    }

    pub fn is_running(&self, firkin_id: &str) -> bool {
        self.inner
            .read()
            .get(firkin_id)
            .map(|p| !p.completed)
            .unwrap_or(false)
    }

    pub fn insert(&self, firkin_id: String, progress: AlbumDownloadProgress) {
        self.inner.write().insert(firkin_id, progress);
    }

    pub fn update<F: FnOnce(&mut AlbumDownloadProgress)>(&self, firkin_id: &str, f: F) {
        let mut map = self.inner.write();
        if let Some(p) = map.get_mut(firkin_id) {
            f(p);
            p.updated_at = Utc::now();
        }
    }
}

/// Spawn `download_album` as a fire-and-forget background task. Returns
/// immediately. Subsequent calls for the same id while a task is in flight
/// are ignored — the in-memory progress map is the lock.
pub fn spawn_download_album(state: CloudState, id: String) -> bool {
    if state.album_download_progress.is_running(&id) {
        return false;
    }
    tracing::info!(firkin_id = %id, "spawning background album download");
    tokio::spawn(async move {
        if let Err(e) = download_album(&state, &id).await {
            tracing::warn!(
                firkin_id = %id,
                error = %e,
                "background album download failed"
            );
            state.album_download_progress.update(&id, |p| {
                p.completed = true;
                p.error = Some(e);
            });
        }
    });
    true
}

async fn download_album(state: &CloudState, id: &str) -> Result<(), String> {
    let existing: Option<Firkin> = state
        .db
        .select((FIRKIN_TABLE, id))
        .await
        .map_err(|e| format!("db select failed: {e}"))?;
    let initial = existing.ok_or_else(|| "firkin not found".to_string())?;

    if initial.addon != "musicbrainz" {
        return Err("download-album only supports musicbrainz firkins".to_string());
    }

    let release_group_id = initial
        .files
        .iter()
        .filter(|f| f.kind == "url")
        .find_map(|f| extract_mb_release_group_id(&f.value))
        .ok_or_else(|| {
            "firkin is missing a MusicBrainz release-group url in `files`".to_string()
        })?;

    let tracks = track_resolve::fetch_release_group_tracks(&release_group_id)
        .await
        .map_err(|e| format!("musicbrainz tracklist fetch failed: {e}"))?;

    let now = Utc::now();
    let initial_tracks: Vec<AlbumDownloadTrack> = tracks
        .iter()
        .map(|t| AlbumDownloadTrack {
            position: t.position,
            title: t.title.clone(),
            status: AlbumDownloadStatus::Pending,
            cid: None,
            progress: 0.0,
            error: None,
        })
        .collect();
    state.album_download_progress.insert(
        id.to_string(),
        AlbumDownloadProgress {
            firkin_id: id.to_string(),
            started_at: now,
            updated_at: now,
            completed: false,
            error: None,
            tracks: initial_tracks,
        },
    );

    let audio_dir = crate::paths::youtube_dir()
        .join("albums")
        .join(id)
        .to_string_lossy()
        .into_owned();
    std::fs::create_dir_all(&audio_dir).ok();

    for (idx, track) in tracks.iter().enumerate() {
        let track_title = track.title.trim();
        if track_title.is_empty() {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Skipped;
                }
            });
            continue;
        }

        let current = match load_firkin(state, id).await {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        // Skip when the track already has an `ipfs`-typed file entry — this
        // makes re-runs idempotent across crashes / restarts.
        let already_local = current.files.iter().any(|f| {
            f.kind == "ipfs"
                && f.title
                    .as_deref()
                    .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                    .unwrap_or(false)
        });
        if already_local {
            let existing_cid = current
                .files
                .iter()
                .find(|f| {
                    f.kind == "ipfs"
                        && f.title
                            .as_deref()
                            .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                            .unwrap_or(false)
                })
                .map(|f| f.value.clone());
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Skipped;
                    t.cid = existing_cid.clone();
                    t.progress = 1.0;
                }
            });
            continue;
        }

        let yt_url = current.files.iter().find_map(|f| {
            if f.kind == "url"
                && f.title
                    .as_deref()
                    .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                    .unwrap_or(false)
                && is_youtube_url(&f.value)
            {
                Some(f.value.clone())
            } else {
                None
            }
        });
        let Some(yt_url) = yt_url else {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Failed;
                    t.error = Some("no YouTube url for track".to_string());
                }
            });
            continue;
        };

        let video_id = extract_youtube_video_id(&yt_url).unwrap_or_default();
        if video_id.is_empty() {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Failed;
                    t.error = Some("could not extract video id".to_string());
                }
            });
            continue;
        }

        state.album_download_progress.update(id, |p| {
            if let Some(t) = p.tracks.get_mut(idx) {
                t.status = AlbumDownloadStatus::Downloading;
            }
        });

        // Max-quality audio. The source picker in
        // `mhaol_yt_dlp::download::format::select_audio_format` always
        // takes the highest-bitrate audio-only stream YouTube exposes
        // (the `quality` arg is unused there). `AudioFormat::Mp3 +
        // AudioQuality::Best` then transcodes at 320 kbps — the top of
        // the mp3 ladder; the alternative targets (`Aac`, `Opus`) are
        // pinned to 128 kbps in `muxer::convert_audio` regardless of
        // `AudioQuality`, so mp3@320 is the genuine max in this stack.
        let request = QueueDownloadRequest {
            url: yt_url.clone(),
            video_id: video_id.clone(),
            title: track_title.to_string(),
            mode: Some(DownloadMode::Audio),
            quality: Some(AudioQuality::Best),
            format: Some(AudioFormat::Mp3),
            video_quality: None,
            video_format: None,
            video_output_dir: None,
            audio_output_dir: Some(audio_dir.clone()),
            thumbnail_url: None,
            duration_seconds: track.length_ms.map(|ms| ms as f64 / 1000.0),
            channel_name: None,
            subtitle_mode: None,
            subtitle_langs: None,
        };

        let download_id = state.ytdl_manager.queue_download(request);

        let (output_path, dl_err) = wait_for_download(state, &download_id, id, idx).await;
        if let Some(e) = dl_err {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Failed;
                    t.error = Some(e);
                }
            });
            continue;
        }
        let Some(output_path) = output_path else {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Failed;
                    t.error = Some("download finished without output path".to_string());
                }
            });
            continue;
        };

        let add_req = mhaol_ipfs_core::AddIpfsRequest {
            source: output_path.clone(),
            pin: Some(true),
        };
        let info = match state.ipfs_manager.add(add_req).await {
            Ok(i) => i,
            Err(e) => {
                state.album_download_progress.update(id, |p| {
                    if let Some(t) = p.tracks.get_mut(idx) {
                        t.status = AlbumDownloadStatus::Failed;
                        t.error = Some(format!("ipfs add failed: {e}"));
                    }
                });
                continue;
            }
        };
        let mime = mime_guess::from_path(&output_path)
            .first()
            .map(|m| m.essence_str().to_string())
            .unwrap_or_else(|| "audio/mpeg".to_string());
        let _ = ipfs_pins::record_pin(
            state,
            info.cid.clone(),
            output_path.clone(),
            mime,
            info.size,
        )
        .await;

        if let Err(e) = append_ipfs_entry(state, id, track_title, &info.cid).await {
            state.album_download_progress.update(id, |p| {
                if let Some(t) = p.tracks.get_mut(idx) {
                    t.status = AlbumDownloadStatus::Failed;
                    t.error = Some(format!("firkin update failed: {e}"));
                }
            });
            continue;
        }

        state.album_download_progress.update(id, |p| {
            if let Some(t) = p.tracks.get_mut(idx) {
                t.status = AlbumDownloadStatus::Completed;
                t.cid = Some(info.cid.clone());
                t.progress = 1.0;
            }
        });
    }

    state.album_download_progress.update(id, |p| {
        p.completed = true;
    });

    Ok(())
}

async fn load_firkin(state: &CloudState, id: &str) -> Result<Firkin, String> {
    let existing: Option<Firkin> = state
        .db
        .select((FIRKIN_TABLE, id))
        .await
        .map_err(|e| format!("db select failed: {e}"))?;
    existing.ok_or_else(|| "firkin not found".to_string())
}

async fn append_ipfs_entry(
    state: &CloudState,
    id: &str,
    track_title: &str,
    cid: &str,
) -> Result<(), String> {
    // Hold the per-firkin lock across read-modify-write so a concurrent
    // mutation (subtitle attach, magnet pick, manual `PUT /api/firkins/:id`)
    // can't slip in between the load and the rollforward and have its
    // change silently overwritten.
    let _firkin_guard = state.firkin_lock(id).lock_owned().await;
    let mut current = load_firkin(state, id).await?;

    let already = current.files.iter().any(|f| {
        f.kind == "ipfs"
            && f.title
                .as_deref()
                .map(|t| t.trim().eq_ignore_ascii_case(track_title))
                .unwrap_or(false)
    });
    if already {
        return Ok(());
    }

    current.files.push(FileEntry {
        kind: "ipfs".to_string(),
        value: cid.to_string(),
        title: Some(track_title.to_string()),
    });
    current.updated_at = Utc::now();
    current.id = None;

    rollforward_firkin(state, id, current)
        .await
        .map_err(|(s, j)| {
            let msg = j
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("rollforward failed");
            format!("{}: {}", s, msg)
        })?;
    Ok(())
}

async fn wait_for_download(
    state: &CloudState,
    download_id: &str,
    firkin_id: &str,
    track_idx: usize,
) -> (Option<String>, Option<String>) {
    loop {
        tokio::time::sleep(Duration::from_millis(750)).await;
        let progress = state.ytdl_manager.get_progress(download_id);
        let Some(progress) = progress else {
            return (None, Some("download record disappeared".to_string()));
        };
        state.album_download_progress.update(firkin_id, |p| {
            if let Some(t) = p.tracks.get_mut(track_idx) {
                t.progress = progress.progress;
            }
        });
        match progress.state {
            DownloadState::Completed => {
                let path = progress
                    .audio_output_path
                    .or(progress.output_path)
                    .filter(|p| !p.is_empty());
                return (path, None);
            }
            DownloadState::Failed => {
                return (
                    None,
                    Some(
                        progress
                            .error
                            .unwrap_or_else(|| "download failed".to_string()),
                    ),
                );
            }
            DownloadState::Cancelled => {
                return (None, Some("download cancelled".to_string()));
            }
            _ => {}
        }
    }
}

fn extract_mb_release_group_id(value: &str) -> Option<String> {
    let url = url::Url::parse(value).ok()?;
    if !url
        .host_str()
        .map(|h| h.eq_ignore_ascii_case("musicbrainz.org"))
        .unwrap_or(false)
    {
        return None;
    }
    let mut segments = url.path_segments()?;
    if segments.next()? != "release-group" {
        return None;
    }
    let id = segments.next()?.to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn is_youtube_url(value: &str) -> bool {
    let host = match url::Url::parse(value)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
    {
        Some(h) => h,
        None => return false,
    };
    matches!(
        host.as_str(),
        "www.youtube.com" | "youtube.com" | "m.youtube.com" | "music.youtube.com" | "youtu.be"
    )
}

fn extract_youtube_video_id(value: &str) -> Option<String> {
    let url = url::Url::parse(value).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    match host.as_str() {
        "www.youtube.com" | "youtube.com" | "m.youtube.com" | "music.youtube.com" => url
            .query_pairs()
            .find(|(k, _)| k == "v")
            .map(|(_, v)| v.into_owned())
            .filter(|s| !s.is_empty()),
        "youtu.be" => url
            .path_segments()?
            .find(|s| !s.is_empty())
            .map(|s| s.to_string()),
        _ => None,
    }
}

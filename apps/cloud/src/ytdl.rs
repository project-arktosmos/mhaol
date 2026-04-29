//! `/api/ytdl/*` — thin wrapper around `mhaol_yt_dlp::build_router` so the
//! cloud exposes the same yt-dlp surface the player Tauri shell exposes via
//! its embedded yt-dlp server. The state-bearing router built by the package
//! is mounted directly on the cloud's outer `Router<CloudState>`.

#[cfg(not(target_os = "android"))]
use std::sync::Arc;

#[cfg(not(target_os = "android"))]
use axum::Router;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;

#[cfg(not(target_os = "android"))]
pub fn router(manager: Arc<DownloadManager>) -> Router {
    mhaol_yt_dlp::build_router(manager)
}

use serde::{Deserialize, Serialize};

// ===== Download States =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadState {
    Pending,
    Fetching,
    Downloading,
    Muxing,
    Completed,
    Failed,
    Cancelled,
}

// ===== Audio Quality =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AudioQuality {
    Best,
    High,
    Medium,
    Low,
}

// ===== Audio Format =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AudioFormat {
    Aac,
    Mp3,
    Opus,
}

// ===== Download Mode =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadMode {
    Audio,
    Video,
}

// ===== Video Quality =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoQuality {
    #[serde(rename = "best")]
    Best,
    #[serde(rename = "1080p")]
    P1080,
    #[serde(rename = "720p")]
    P720,
    #[serde(rename = "480p")]
    P480,
}

// ===== Video Format =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum VideoFormat {
    Mp4,
    Mkv,
    Webm,
}

// ===== API Request Types =====

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueDownloadRequest {
    pub url: String,
    pub video_id: String,
    pub title: String,
    pub mode: Option<DownloadMode>,
    pub quality: Option<AudioQuality>,
    pub format: Option<AudioFormat>,
    pub video_quality: Option<VideoQuality>,
    pub video_format: Option<VideoFormat>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuePlaylistRequest {
    pub videos: Vec<PlaylistVideoRequest>,
    pub mode: Option<DownloadMode>,
    pub quality: Option<AudioQuality>,
    pub format: Option<AudioFormat>,
    pub video_quality: Option<VideoQuality>,
    pub video_format: Option<VideoFormat>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideoRequest {
    pub url: String,
    pub video_id: String,
    pub title: String,
}

// ===== API Response Types =====

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub download_id: String,
    pub url: String,
    pub video_id: String,
    pub title: String,
    pub state: DownloadState,
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub mode: DownloadMode,
    pub quality: AudioQuality,
    pub format: AudioFormat,
    pub video_quality: Option<VideoQuality>,
    pub video_format: Option<VideoFormat>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub title: String,
    pub duration: f64,
    pub thumbnail_url: Option<String>,
    pub uploader: Option<String>,
    pub video_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideo {
    pub video_id: String,
    pub title: String,
    pub duration: f64,
    pub thumbnail_url: Option<String>,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub playlist_id: String,
    pub title: String,
    pub video_count: usize,
    pub videos: Vec<PlaylistVideo>,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerStats {
    pub active_downloads: u32,
    pub queued_downloads: u32,
    pub completed_downloads: u32,
    pub failed_downloads: u32,
    pub ytdlp_available: bool,
    pub ytdlp_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YtDlpStatus {
    pub available: bool,
    pub version: Option<String>,
    pub downloading: bool,
}

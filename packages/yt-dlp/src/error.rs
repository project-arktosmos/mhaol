use thiserror::Error;

#[derive(Error, Debug)]
pub enum YtDlpError {
    #[error("Video not found: {video_id}")]
    VideoNotFound { video_id: String },

    #[error("Video is unavailable: {reason}")]
    VideoUnavailable { reason: String },

    #[error("Age restricted content requires authentication")]
    AgeRestricted,

    #[error("No suitable format found for requested quality")]
    NoSuitableFormat,

    #[error("Signature decryption failed: {0}")]
    SignatureError(String),

    #[error("N-parameter transformation failed: {0}")]
    NParamError(String),

    #[error("Player.js could not be fetched or parsed")]
    PlayerJsError,

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("Download cancelled")]
    Cancelled,

    #[error("Muxing failed: {0}")]
    MuxError(String),

    #[error("FFmpeg not available")]
    FfmpegNotAvailable,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

use serde::{Deserialize, Serialize};

/// Video codec selection for encoding before WebRTC transport.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoCodec {
    Vp8,
    Vp9,
    H264,
}

impl VideoCodec {
    /// GStreamer encoder element name.
    pub fn encoder_element(&self) -> &'static str {
        match self {
            VideoCodec::Vp8 => "vp8enc",
            VideoCodec::Vp9 => "vp9enc",
            VideoCodec::H264 => "x264enc",
        }
    }

    /// GStreamer RTP payloader element name.
    pub fn rtp_payloader(&self) -> &'static str {
        match self {
            VideoCodec::Vp8 => "rtpvp8pay",
            VideoCodec::Vp9 => "rtpvp9pay",
            VideoCodec::H264 => "rtph264pay",
        }
    }

    /// RTP payload type number.
    pub fn payload_type(&self) -> u32 {
        match self {
            VideoCodec::Vp8 => 96,
            VideoCodec::Vp9 => 97,
            VideoCodec::H264 => 98,
        }
    }

    /// Encoding name for SDP.
    pub fn encoding_name(&self) -> &'static str {
        match self {
            VideoCodec::Vp8 => "VP8",
            VideoCodec::Vp9 => "VP9",
            VideoCodec::H264 => "H264",
        }
    }
}

/// Audio codec selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioCodec {
    Opus,
}

impl AudioCodec {
    pub fn encoder_element(&self) -> &'static str {
        match self {
            AudioCodec::Opus => "opusenc",
        }
    }

    pub fn rtp_payloader(&self) -> &'static str {
        match self {
            AudioCodec::Opus => "rtpopuspay",
        }
    }

    pub fn payload_type(&self) -> u32 {
        match self {
            AudioCodec::Opus => 111,
        }
    }
}

/// Video quality/resolution preset.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoQuality {
    #[default]
    Native,
    #[serde(rename = "1080p")]
    Q1080p,
    #[serde(rename = "720p")]
    Q720p,
    #[serde(rename = "480p")]
    Q480p,
    #[serde(rename = "360p")]
    Q360p,
}

impl VideoQuality {
    /// Returns the target height for scaling, or `None` for native (no scaling).
    pub fn target_height(&self) -> Option<i32> {
        match self {
            VideoQuality::Native => None,
            VideoQuality::Q1080p => Some(1080),
            VideoQuality::Q720p => Some(720),
            VideoQuality::Q480p => Some(480),
            VideoQuality::Q360p => Some(360),
        }
    }

    /// Target bitrate in bits/sec for a given video codec.
    pub fn target_bitrate(&self, codec: &VideoCodec) -> u32 {
        match (self, codec) {
            (VideoQuality::Native | VideoQuality::Q1080p, VideoCodec::Vp9) => 3_000_000,
            (VideoQuality::Native | VideoQuality::Q1080p, _) => 4_000_000,
            (VideoQuality::Q720p, VideoCodec::Vp9) => 1_800_000,
            (VideoQuality::Q720p, _) => 2_500_000,
            (VideoQuality::Q480p, VideoCodec::Vp9) => 900_000,
            (VideoQuality::Q480p, _) => 1_200_000,
            (VideoQuality::Q360p, VideoCodec::Vp9) => 500_000,
            (VideoQuality::Q360p, _) => 700_000,
        }
    }
}


/// Codec configuration for a streaming session.
#[derive(Debug, Clone)]
pub struct CodecConfig {
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
    pub video_quality: VideoQuality,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            video: Some(VideoCodec::Vp8),
            audio: Some(AudioCodec::Opus),
            video_quality: VideoQuality::default(),
        }
    }
}

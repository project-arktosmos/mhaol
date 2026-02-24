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

/// Codec configuration for a streaming session.
#[derive(Debug, Clone)]
pub struct CodecConfig {
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
}

impl Default for CodecConfig {
    fn default() -> Self {
        Self {
            video: Some(VideoCodec::Vp8),
            audio: Some(AudioCodec::Opus),
        }
    }
}

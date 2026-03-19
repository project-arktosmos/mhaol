use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Raw player response from the Innertube API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub streaming_data: Option<StreamingData>,
    pub video_details: Option<VideoDetails>,
    pub playability_status: Option<PlayabilityStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {
    pub formats: Option<Vec<StreamFormat>>,
    pub adaptive_formats: Option<Vec<StreamFormat>>,
    pub expires_in_seconds: Option<String>,
    pub hls_manifest_url: Option<String>,
    pub dash_manifest_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamFormat {
    pub itag: u32,
    pub url: Option<String>,
    pub signature_cipher: Option<String>,
    pub mime_type: String,
    pub bitrate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub content_length: Option<String>,
    pub quality: Option<String>,
    pub quality_label: Option<String>,
    pub audio_quality: Option<String>,
    pub audio_sample_rate: Option<String>,
    pub audio_channels: Option<u32>,
    pub fps: Option<u32>,
    pub approx_duration_ms: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub video_id: String,
    pub title: String,
    pub length_seconds: String,
    pub channel_id: Option<String>,
    pub short_description: Option<String>,
    pub thumbnail: Option<ThumbnailList>,
    pub author: Option<String>,
    pub view_count: Option<String>,
    pub is_live_content: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailList {
    pub thumbnails: Option<Vec<Thumbnail>>,
}

#[derive(Debug, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub status: String,
    pub reason: Option<String>,
    pub playable_in_embed: Option<bool>,
}

/// A fully resolved format with a usable download URL.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFormat {
    pub itag: u32,
    pub url: String,
    pub mime_type: String,
    pub bitrate: u64,
    pub content_length: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality_label: Option<String>,
    pub audio_quality: Option<String>,
    pub fps: Option<u32>,
    pub is_audio_only: bool,
    pub is_video_only: bool,
    pub codec: String,
    pub container: String,
}

impl PlayerResponse {
    /// Check if the video is playable.
    pub fn is_playable(&self) -> bool {
        self.playability_status
            .as_ref()
            .map(|s| s.status == "OK")
            .unwrap_or(false)
    }

    /// Get the reason for unplayability.
    pub fn unplayable_reason(&self) -> Option<String> {
        self.playability_status
            .as_ref()
            .and_then(|s| s.reason.clone())
    }

    /// Get the best thumbnail URL.
    pub fn thumbnail_url(&self) -> Option<String> {
        self.video_details
            .as_ref()
            .and_then(|vd| vd.thumbnail.as_ref())
            .and_then(|tl| tl.thumbnails.as_ref())
            .and_then(|thumbs| {
                thumbs
                    .iter()
                    .max_by_key(|t| t.width.unwrap_or(0) * t.height.unwrap_or(0))
                    .map(|t| t.url.clone())
            })
    }

    /// Get all raw formats (both muxed and adaptive).
    pub fn all_formats(&self) -> Vec<&StreamFormat> {
        let mut formats = Vec::new();
        if let Some(sd) = &self.streaming_data {
            if let Some(f) = &sd.formats {
                formats.extend(f.iter());
            }
            if let Some(af) = &sd.adaptive_formats {
                formats.extend(af.iter());
            }
        }
        formats
    }
}

impl StreamFormat {
    /// Parse the signature cipher into its components.
    pub fn parse_signature_cipher(&self) -> Option<SignatureCipherParts> {
        let cipher = self.signature_cipher.as_ref()?;
        let mut s = None;
        let mut sp = None;
        let mut url = None;

        for part in cipher.split('&') {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "s" => s = Some(urlencoding_decode(value)),
                    "sp" => sp = Some(value.to_string()),
                    "url" => url = Some(urlencoding_decode(value)),
                    _ => {}
                }
            }
        }

        Some(SignatureCipherParts {
            encrypted_sig: s?,
            sig_param: sp.unwrap_or_else(|| "sig".to_string()),
            base_url: url?,
        })
    }

    /// Determine if this format is audio-only.
    pub fn is_audio_only(&self) -> bool {
        self.mime_type.starts_with("audio/")
    }

    /// Determine if this format is video-only (no audio track).
    pub fn is_video_only(&self) -> bool {
        self.mime_type.starts_with("video/") && self.audio_quality.is_none()
    }

    /// Parse the codec from mime_type (e.g., "video/mp4; codecs=\"avc1.640028\"" → "avc1").
    pub fn codec(&self) -> String {
        if let Some(codecs_start) = self.mime_type.find("codecs=\"") {
            let rest = &self.mime_type[codecs_start + 8..];
            if let Some(end) = rest.find('"') {
                let codecs = &rest[..end];
                // Return the first codec name (before the dot for the profile)
                return codecs
                    .split(',')
                    .next()
                    .unwrap_or(codecs)
                    .trim()
                    .split('.')
                    .next()
                    .unwrap_or("")
                    .to_string();
            }
        }
        "unknown".to_string()
    }

    /// Parse the container from mime_type (e.g., "video/mp4" → "mp4").
    pub fn container(&self) -> String {
        self.mime_type
            .split(';')
            .next()
            .unwrap_or(&self.mime_type)
            .split('/')
            .nth(1)
            .unwrap_or("mp4")
            .trim()
            .to_string()
    }

    /// Convert to a ResolvedFormat given a resolved URL.
    pub fn to_resolved(&self, url: String) -> ResolvedFormat {
        ResolvedFormat {
            itag: self.itag,
            url,
            mime_type: self.mime_type.clone(),
            bitrate: self.bitrate.unwrap_or(0),
            content_length: self.content_length.as_ref().and_then(|s| s.parse().ok()),
            width: self.width,
            height: self.height,
            quality_label: self.quality_label.clone(),
            audio_quality: self.audio_quality.clone(),
            fps: self.fps,
            is_audio_only: self.is_audio_only(),
            is_video_only: self.is_video_only(),
            codec: self.codec(),
            container: self.container(),
        }
    }
}

/// Components of a parsed signature cipher.
#[derive(Debug, Clone)]
pub struct SignatureCipherParts {
    pub encrypted_sig: String,
    pub sig_param: String,
    pub base_url: String,
}

/// Extract the player.js URL from YouTube watch page HTML.
pub fn extract_player_js_url(html: &str) -> Result<String> {
    // Pattern: /s/player/{hash}/player_ias.vflset/en_US/base.js
    let re = Regex::new(r#"(/s/player/[a-zA-Z0-9]+/[^"]+?base\.js)"#)?;
    if let Some(cap) = re.captures(html) {
        return Ok(cap[1].to_string());
    }

    // Alternative pattern: "jsUrl":"/s/player/..."
    let re2 = Regex::new(r#""jsUrl"\s*:\s*"([^"]+)""#)?;
    if let Some(cap) = re2.captures(html) {
        return Ok(cap[1].to_string());
    }

    anyhow::bail!("Could not find player.js URL in watch page HTML")
}

/// Extract the signature timestamp (STS) from player.js source.
pub fn extract_sts(player_js: &str) -> Option<u64> {
    let re = Regex::new(r"signatureTimestamp[=:](\d+)").ok()?;
    re.captures(player_js)
        .and_then(|cap| cap[1].parse().ok())
}

fn urlencoding_decode(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(key, val)| {
            if val.is_empty() {
                key.to_string()
            } else {
                format!("{}={}", key, val)
            }
        })
        .collect::<Vec<_>>()
        .join("&")
        // Simple percent-decode for standalone values
        .replace("%3A", ":")
        .replace("%2F", "/")
        .replace("%3F", "?")
        .replace("%3D", "=")
        .replace("%26", "&")
        .replace("%25", "%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_format_codec_parsing() {
        let fmt = StreamFormat {
            itag: 137,
            url: None,
            signature_cipher: None,
            mime_type: "video/mp4; codecs=\"avc1.640028\"".to_string(),
            bitrate: Some(4500000),
            width: Some(1920),
            height: Some(1080),
            content_length: None,
            quality: Some("hd1080".to_string()),
            quality_label: Some("1080p".to_string()),
            audio_quality: None,
            audio_sample_rate: None,
            audio_channels: None,
            fps: Some(30),
            approx_duration_ms: None,
            last_modified: None,
        };

        assert_eq!(fmt.codec(), "avc1");
        assert_eq!(fmt.container(), "mp4");
        assert!(fmt.is_video_only());
        assert!(!fmt.is_audio_only());
    }

    #[test]
    fn test_stream_format_audio() {
        let fmt = StreamFormat {
            itag: 140,
            url: Some("https://example.com/audio".to_string()),
            signature_cipher: None,
            mime_type: "audio/mp4; codecs=\"mp4a.40.2\"".to_string(),
            bitrate: Some(128000),
            width: None,
            height: None,
            content_length: Some("3456789".to_string()),
            quality: None,
            quality_label: None,
            audio_quality: Some("AUDIO_QUALITY_MEDIUM".to_string()),
            audio_sample_rate: Some("44100".to_string()),
            audio_channels: Some(2),
            fps: None,
            approx_duration_ms: None,
            last_modified: None,
        };

        assert!(fmt.is_audio_only());
        assert!(!fmt.is_video_only());
        assert_eq!(fmt.codec(), "mp4a");
        assert_eq!(fmt.container(), "mp4");
    }
}

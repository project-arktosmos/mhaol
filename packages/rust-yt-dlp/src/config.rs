use serde::{Deserialize, Serialize};

use crate::types::{AudioFormat, AudioQuality};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YtDownloadConfig {
    pub output_path: String,
    pub default_quality: AudioQuality,
    pub default_format: AudioFormat,
    pub po_token: Option<String>,
    pub visitor_data: Option<String>,
    pub cookies: Option<String>,
}

impl Default for YtDownloadConfig {
    fn default() -> Self {
        Self {
            output_path: default_output_path(),
            default_quality: AudioQuality::High,
            default_format: AudioFormat::Aac,
            po_token: None,
            visitor_data: None,
            cookies: None,
        }
    }
}

impl YtDownloadConfig {
    pub fn from_env() -> Self {
        let output_path = std::env::var("YTDL_OUTPUT_DIR")
            .unwrap_or_else(|_| default_output_path());
        let po_token = std::env::var("YTDL_PO_TOKEN").ok();
        let visitor_data = std::env::var("YTDL_VISITOR_DATA").ok();
        let cookies = std::env::var("YTDL_COOKIES").ok();

        Self {
            output_path,
            po_token,
            visitor_data,
            cookies,
            ..Default::default()
        }
    }
}

fn default_output_path() -> String {
    std::env::var("HOME")
        .map(|home| format!("{}/Downloads/youtube", home))
        .unwrap_or_else(|_| "/tmp/youtube-downloads".to_string())
}

use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::error::YtDlpError;
use crate::types::{AudioFormat, AudioQuality, VideoFormat};

/// Muxer that uses ffmpeg subprocess for combining video+audio and format conversion.
pub struct FfmpegMuxer {
    ffmpeg_path: Option<String>,
}

impl FfmpegMuxer {
    pub fn new() -> Self {
        let path = detect_ffmpeg();
        if let Some(ref p) = path {
            log::info!("FFmpeg found at: {}", p);
        } else {
            log::warn!("FFmpeg not found; muxing and format conversion will be unavailable");
        }
        Self { ffmpeg_path: path }
    }

    pub fn is_available(&self) -> bool {
        self.ffmpeg_path.is_some()
    }

    /// Mux separate video and audio files into a single output file.
    pub async fn mux(
        &self,
        video_path: &Path,
        audio_path: &Path,
        output_path: &Path,
        _format: &VideoFormat,
    ) -> Result<()> {
        let ffmpeg = self
            .ffmpeg_path
            .as_ref()
            .ok_or(YtDlpError::FfmpegNotAvailable)?;

        let output = Command::new(ffmpeg)
            .args([
                "-y",
                "-i",
                video_path.to_str().unwrap_or(""),
                "-i",
                audio_path.to_str().unwrap_or(""),
                "-c:v",
                "copy",
                "-c:a",
                "copy",
                "-movflags",
                "+faststart",
                output_path.to_str().unwrap_or(""),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YtDlpError::MuxError(format!(
                "ffmpeg muxing failed: {}",
                stderr.chars().take(500).collect::<String>()
            ))
            .into());
        }

        Ok(())
    }

    /// Convert audio from one format to another (e.g., AAC → MP3).
    pub async fn convert_audio(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: &AudioFormat,
        quality: &AudioQuality,
    ) -> Result<()> {
        let ffmpeg = self
            .ffmpeg_path
            .as_ref()
            .ok_or(YtDlpError::FfmpegNotAvailable)?;

        let (codec, ext_args): (&str, Vec<&str>) = match target_format {
            AudioFormat::Mp3 => {
                let q = match quality {
                    AudioQuality::Best => "0",
                    AudioQuality::High => "2",
                    AudioQuality::Medium => "5",
                    AudioQuality::Low => "9",
                };
                ("libmp3lame", vec!["-q:a", q])
            }
            AudioFormat::Aac => ("aac", vec!["-b:a", "128k"]),
            AudioFormat::Opus => ("libopus", vec!["-b:a", "128k"]),
        };

        let mut args = vec![
            "-y",
            "-i",
            input_path.to_str().unwrap_or(""),
            "-c:a",
            codec,
        ];
        args.extend(ext_args);
        args.push(output_path.to_str().unwrap_or(""));

        let output = Command::new(ffmpeg)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YtDlpError::MuxError(format!(
                "ffmpeg audio conversion failed: {}",
                stderr.chars().take(500).collect::<String>()
            ))
            .into());
        }

        Ok(())
    }
}

/// Detect ffmpeg installation.
fn detect_ffmpeg() -> Option<String> {
    // Try `which ffmpeg` on Unix
    std::process::Command::new("which")
        .arg("ffmpeg")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback: just check if "ffmpeg" is in PATH
            std::process::Command::new("ffmpeg")
                .arg("-version")
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|_| "ffmpeg".to_string())
        })
}

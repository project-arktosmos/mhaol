use crate::error::YtDlpError;
use crate::extractor::player::ResolvedFormat;
use crate::types::{AudioFormat, AudioQuality, DownloadMode, VideoFormat, VideoQuality};

/// The result of format selection.
#[derive(Debug, Clone)]
pub struct SelectedFormats {
    pub video: Option<ResolvedFormat>,
    pub audio: ResolvedFormat,
    pub needs_muxing: bool,
    pub output_extension: String,
}

/// Known itag to quality mappings for audio formats.
struct AudioItag {
    itag: u32,
    codec: &'static str,
    bitrate: u32,
}

const AUDIO_ITAGS: &[AudioItag] = &[
    AudioItag { itag: 140, codec: "aac", bitrate: 128 },
    AudioItag { itag: 141, codec: "aac", bitrate: 256 },
    AudioItag { itag: 256, codec: "aac", bitrate: 192 },
    AudioItag { itag: 258, codec: "aac", bitrate: 384 },
    AudioItag { itag: 251, codec: "opus", bitrate: 160 },
    AudioItag { itag: 250, codec: "opus", bitrate: 70 },
    AudioItag { itag: 249, codec: "opus", bitrate: 50 },
];

/// Select the best format(s) for the given mode and quality preferences.
pub fn select_formats(
    formats: &[ResolvedFormat],
    mode: &DownloadMode,
    audio_quality: &AudioQuality,
    audio_format: &AudioFormat,
    video_quality: Option<&VideoQuality>,
    video_format: Option<&VideoFormat>,
) -> Result<SelectedFormats, YtDlpError> {
    match mode {
        DownloadMode::Audio => select_audio_format(formats, audio_quality, audio_format),
        DownloadMode::Video => select_video_formats(
            formats,
            video_quality.unwrap_or(&VideoQuality::Best),
            video_format.unwrap_or(&VideoFormat::Mp4),
            audio_quality,
        ),
    }
}

fn select_audio_format(
    formats: &[ResolvedFormat],
    quality: &AudioQuality,
    format: &AudioFormat,
) -> Result<SelectedFormats, YtDlpError> {
    let audio_formats: Vec<&ResolvedFormat> =
        formats.iter().filter(|f| f.is_audio_only).collect();

    if audio_formats.is_empty() {
        return Err(YtDlpError::NoSuitableFormat);
    }

    // Prefer the codec matching the requested format
    let preferred_codec = match format {
        AudioFormat::Aac => "mp4a",
        AudioFormat::Mp3 => "mp4a", // Download as AAC, convert to MP3 later
        AudioFormat::Opus => "opus",
    };

    let target_bitrate = match quality {
        AudioQuality::Best => u64::MAX,
        AudioQuality::High => 160_000,
        AudioQuality::Medium => 128_000,
        AudioQuality::Low => 64_000,
    };

    // Sort: prefer matching codec, then closest bitrate
    let mut candidates: Vec<&&ResolvedFormat> = audio_formats.iter().collect();
    candidates.sort_by(|a, b| {
        let a_codec_match = a.codec == preferred_codec;
        let b_codec_match = b.codec == preferred_codec;

        // Prefer matching codec
        if a_codec_match != b_codec_match {
            return b_codec_match.cmp(&a_codec_match);
        }

        // Then prefer closest bitrate (higher is better up to target)
        if *quality == AudioQuality::Best {
            b.bitrate.cmp(&a.bitrate)
        } else {
            let a_dist = (a.bitrate as i64 - target_bitrate as i64).unsigned_abs();
            let b_dist = (b.bitrate as i64 - target_bitrate as i64).unsigned_abs();
            a_dist.cmp(&b_dist)
        }
    });

    let selected = candidates
        .first()
        .ok_or(YtDlpError::NoSuitableFormat)?;

    let ext = match format {
        AudioFormat::Aac => "m4a",
        AudioFormat::Mp3 => "mp3",
        AudioFormat::Opus => "opus",
    };

    Ok(SelectedFormats {
        video: None,
        audio: (**selected).clone(),
        needs_muxing: false,
        output_extension: ext.to_string(),
    })
}

fn select_video_formats(
    formats: &[ResolvedFormat],
    quality: &VideoQuality,
    format: &VideoFormat,
    audio_quality: &AudioQuality,
) -> Result<SelectedFormats, YtDlpError> {
    let video_formats: Vec<&ResolvedFormat> =
        formats.iter().filter(|f| f.is_video_only).collect();
    let audio_formats: Vec<&ResolvedFormat> =
        formats.iter().filter(|f| f.is_audio_only).collect();

    // If we have separate video+audio adaptive formats, use those
    if !video_formats.is_empty() && !audio_formats.is_empty() {
        let target_height = match quality {
            VideoQuality::Best => u32::MAX,
            VideoQuality::P1080 => 1080,
            VideoQuality::P720 => 720,
            VideoQuality::P480 => 480,
        };

        let preferred_container = match format {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Mkv => "mp4", // Download as mp4, mux to mkv
            VideoFormat::Webm => "webm",
        };

        // Select video: prefer matching container, then best quality within target
        let mut video_candidates: Vec<&&ResolvedFormat> = video_formats.iter().collect();
        video_candidates.sort_by(|a, b| {
            let a_height = a.height.unwrap_or(0);
            let b_height = b.height.unwrap_or(0);

            // Filter out formats above target (unless target is Best)
            if *quality != VideoQuality::Best {
                let a_over = a_height > target_height;
                let b_over = b_height > target_height;
                if a_over != b_over {
                    return a_over.cmp(&b_over); // Prefer not-over
                }
            }

            // Prefer matching container
            let a_container_match = a.container == preferred_container;
            let b_container_match = b.container == preferred_container;
            if a_container_match != b_container_match {
                return b_container_match.cmp(&a_container_match);
            }

            // Prefer higher resolution, then higher bitrate
            if a_height != b_height {
                return b_height.cmp(&a_height);
            }
            b.bitrate.cmp(&a.bitrate)
        });

        let video = video_candidates
            .first()
            .ok_or(YtDlpError::NoSuitableFormat)?;

        // Select best audio
        let mut audio_candidates: Vec<&&ResolvedFormat> = audio_formats.iter().collect();
        audio_candidates.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));

        let audio = audio_candidates
            .first()
            .ok_or(YtDlpError::NoSuitableFormat)?;

        let ext = match format {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Mkv => "mkv",
            VideoFormat::Webm => "webm",
        };

        return Ok(SelectedFormats {
            video: Some((**video).clone()),
            audio: (**audio).clone(),
            needs_muxing: true,
            output_extension: ext.to_string(),
        });
    }

    // Fallback to muxed formats (video+audio combined)
    let muxed_formats: Vec<&ResolvedFormat> = formats
        .iter()
        .filter(|f| !f.is_audio_only && !f.is_video_only)
        .collect();

    if muxed_formats.is_empty() {
        return Err(YtDlpError::NoSuitableFormat);
    }

    let mut muxed_candidates: Vec<&&ResolvedFormat> = muxed_formats.iter().collect();
    muxed_candidates.sort_by(|a, b| {
        let a_height = a.height.unwrap_or(0);
        let b_height = b.height.unwrap_or(0);
        if a_height != b_height {
            b_height.cmp(&a_height)
        } else {
            b.bitrate.cmp(&a.bitrate)
        }
    });

    let best_muxed = muxed_candidates
        .first()
        .ok_or(YtDlpError::NoSuitableFormat)?;

    Ok(SelectedFormats {
        video: None,
        audio: (**best_muxed).clone(),
        needs_muxing: false,
        output_extension: best_muxed.container.clone(),
    })
}

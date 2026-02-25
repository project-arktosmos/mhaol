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
    /// True when the selected audio is actually a muxed (video+audio) stream because
    /// no suitable audio-only format was available (e.g. ANDROID CDN throttles adaptive
    /// streams). ffmpeg will extract the audio track after download.
    pub needs_audio_extraction: bool,
}

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
    _quality: &AudioQuality,
    _format: &AudioFormat,
) -> Result<SelectedFormats, YtDlpError> {
    // ANDROID client adaptive audio streams (itag=139/140/251) require a GVS PO Token for
    // full download access. Without that token, YouTube CDN returns HTTP 403 regardless of
    // whether the URL contains ratebypass=yes. Muxed formats (itag=18, etc.) are explicitly
    // exempt from the GVS PO Token requirement and download without restriction.
    //
    // To avoid 403 failures we always use a muxed format and extract the audio track via
    // ffmpeg afterward. When GVS PO Token support is added in the future, the adaptive path
    // can be restored here.
    let mut muxed: Vec<&ResolvedFormat> = formats
        .iter()
        .filter(|f| !f.is_audio_only && !f.is_video_only)
        .collect();

    if muxed.is_empty() {
        return Err(YtDlpError::NoSuitableFormat);
    }

    muxed.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));
    let selected = muxed.first().ok_or(YtDlpError::NoSuitableFormat)?;

    log::info!(
        "Audio download: using muxed itag={} ({} container, {} kbps) for audio extraction",
        selected.itag,
        selected.container,
        selected.bitrate / 1000,
    );

    Ok(SelectedFormats {
        video: None,
        audio: (*selected).clone(),
        needs_muxing: false,
        output_extension: selected.container.clone(),
        needs_audio_extraction: true,
    })
}

fn select_video_formats(
    formats: &[ResolvedFormat],
    quality: &VideoQuality,
    format: &VideoFormat,
    _audio_quality: &AudioQuality,
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
            needs_audio_extraction: false,
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
        needs_audio_extraction: false,
    })
}

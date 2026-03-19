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
    has_po_token: bool,
) -> Result<SelectedFormats, YtDlpError> {
    match mode {
        DownloadMode::Audio => select_audio_format(formats, audio_quality, audio_format, has_po_token),
        DownloadMode::Video | DownloadMode::Both => select_video_formats(
            formats,
            video_quality.unwrap_or(&VideoQuality::Best),
            video_format.unwrap_or(&VideoFormat::Mp4),
            audio_quality,
            has_po_token,
        ),
    }
}

fn select_audio_format(
    formats: &[ResolvedFormat],
    _quality: &AudioQuality,
    _format: &AudioFormat,
    _has_po_token: bool,
) -> Result<SelectedFormats, YtDlpError> {
    // Always prefer adaptive audio-only streams (available with or without PO token).
    let mut audio_only: Vec<&ResolvedFormat> = formats
        .iter()
        .filter(|f| f.is_audio_only)
        .collect();

    if !audio_only.is_empty() {
        audio_only.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));
        let selected = audio_only[0];

        log::info!(
            "Audio download: using adaptive audio itag={} ({} codec, {} container, {} kbps)",
            selected.itag,
            selected.codec,
            selected.container,
            selected.bitrate / 1000,
        );

        return Ok(SelectedFormats {
            video: None,
            audio: selected.clone(),
            needs_muxing: false,
            output_extension: selected.container.clone(),
            needs_audio_extraction: false,
        });
    }

    // Fallback: use muxed format and extract audio via ffmpeg
    log::warn!("No adaptive audio-only formats found, falling back to muxed stream (requires ffmpeg for extraction)");

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
    _quality: &VideoQuality,
    format: &VideoFormat,
    _audio_quality: &AudioQuality,
    has_po_token: bool,
) -> Result<SelectedFormats, YtDlpError> {
    let ext = match format {
        VideoFormat::Mp4 => "mp4",
        VideoFormat::Mkv => "mkv",
        VideoFormat::Webm => "webm",
    };

    // When a PO token is available, use adaptive video-only + audio-only for best quality.
    if has_po_token {
        let mut video_only: Vec<&ResolvedFormat> = formats
            .iter()
            .filter(|f| f.is_video_only)
            .collect();

        let mut audio_only: Vec<&ResolvedFormat> = formats
            .iter()
            .filter(|f| f.is_audio_only)
            .collect();

        if !video_only.is_empty() && !audio_only.is_empty() {
            // Sort video by height (desc), then bitrate (desc)
            video_only.sort_by(|a, b| {
                let a_height = a.height.unwrap_or(0);
                let b_height = b.height.unwrap_or(0);
                if a_height != b_height {
                    b_height.cmp(&a_height)
                } else {
                    b.bitrate.cmp(&a.bitrate)
                }
            });

            // Sort audio by bitrate (desc)
            audio_only.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));

            let video = video_only[0];
            let audio = audio_only[0];

            log::info!(
                "Video download: adaptive video itag={} ({}x{}, {} kbps) + audio itag={} ({} kbps), will mux to {}",
                video.itag,
                video.width.unwrap_or(0),
                video.height.unwrap_or(0),
                video.bitrate / 1000,
                audio.itag,
                audio.bitrate / 1000,
                ext,
            );

            return Ok(SelectedFormats {
                video: Some(video.clone()),
                audio: audio.clone(),
                needs_muxing: true,
                output_extension: ext.to_string(),
                needs_audio_extraction: false,
            });
        }
        log::warn!("No adaptive video+audio formats found despite having PO token, falling back to muxed");
    }

    // Fallback: use muxed formats (max ~720p but no token needed)
    let mut muxed: Vec<&ResolvedFormat> = formats
        .iter()
        .filter(|f| !f.is_audio_only && !f.is_video_only)
        .collect();

    if muxed.is_empty() {
        return Err(YtDlpError::NoSuitableFormat);
    }

    muxed.sort_by(|a, b| {
        let a_height = a.height.unwrap_or(0);
        let b_height = b.height.unwrap_or(0);
        if a_height != b_height {
            b_height.cmp(&a_height)
        } else {
            b.bitrate.cmp(&a.bitrate)
        }
    });

    let selected = muxed.first().ok_or(YtDlpError::NoSuitableFormat)?;
    let needs_remux = selected.container != ext;

    log::info!(
        "Video download: using muxed itag={} ({}x{}, {} container, {} kbps){}",
        selected.itag,
        selected.width.unwrap_or(0),
        selected.height.unwrap_or(0),
        selected.container,
        selected.bitrate / 1000,
        if needs_remux {
            format!(", will remux to {}", ext)
        } else {
            String::new()
        },
    );

    Ok(SelectedFormats {
        video: None,
        audio: (*selected).clone(),
        needs_muxing: false,
        output_extension: ext.to_string(),
        needs_audio_extraction: false,
    })
}

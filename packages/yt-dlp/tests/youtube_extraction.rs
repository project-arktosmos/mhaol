//! Integration tests for the yt-dlp browser-streaming extraction flow.
//!
//! These exercise the same code path the cloud and player webuis hit when a
//! user clicks "Stream video"/"Stream audio" on a search result:
//!
//!   `frontend → /api/ytdl/info/stream-urls-browser?url=...`
//!     → `DownloadManager::extract_stream_urls_for_browser`
//!       → `DownloadPipeline::extract_stream_urls_inner(prefer_browser=true)`
//!         → `fetch_and_resolve_with_clients(WEB → WEB_EMBEDDED → TV → ANDROID → IOS)`
//!
//! Each test:
//!  1. Calls the manager against a real YouTube URL (env-overridable).
//!  2. Asserts at least one resolved format of the expected kind exists.
//!  3. Picks the format the frontend would pick (highest-bitrate muxed for
//!     video, highest-bitrate mp4 audio for audio).
//!  4. Issues a HEAD request to the resolved URL with browser-realistic
//!     headers and asserts a 2xx + a non-zero `content-length` so we catch
//!     "extracted but the CDN 403s the URL" — the failure mode that originally
//!     motivated the browser-priority signing path.
//!
//! What we are protecting against: the WEB innertube client occasionally
//! returns `playabilityStatus: OK` but every format has an unresolvable
//! `signatureCipher`. Before the per-client fallback in
//! `fetch_and_resolve_with_clients`, that resolved to zero formats and the
//! whole request 503'd with `NoSuitableFormat`. These tests assert that the
//! extraction now lands a usable URL even when the first client misses.
//!
//! Run with `cargo test -p mhaol-yt-dlp --test youtube_extraction -- --nocapture`.
//! Tests reach the public internet; in air-gapped CI, set
//! `MHAOL_SKIP_YOUTUBE_TESTS=1` to skip them.

use mhaol_yt_dlp::{DownloadManager, StreamUrlResult, YtDownloadConfig};
use std::time::Duration;

/// Default video used when `MHAOL_TEST_YOUTUBE_URL` is unset. This needs to be
/// a long-lived public video; "Me at the zoo" (the first YouTube upload) is
/// about as stable as anything on the platform.
const DEFAULT_TEST_URL: &str = "https://www.youtube.com/watch?v=jNQXAC9IVRw";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

fn test_url() -> String {
    std::env::var("MHAOL_TEST_YOUTUBE_URL").unwrap_or_else(|_| DEFAULT_TEST_URL.to_string())
}

fn should_skip() -> bool {
    if std::env::var("MHAOL_SKIP_YOUTUBE_TESTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        eprintln!("test skipped: MHAOL_SKIP_YOUTUBE_TESTS=1");
        return true;
    }
    false
}

fn ensure_logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        "mhaol_yt_dlp=debug,reqwest=info",
    ))
    .is_test(true)
    .try_init();
}

fn make_manager() -> DownloadManager {
    let config = YtDownloadConfig::from_env();
    DownloadManager::new(config)
}

/// Pick the format the cloud/player frontend's `pickMuxed` picks: highest
/// resolution muxed (video+audio) format, ties broken by bitrate. Mirrors
/// `apps/cloud/web/src/routes/youtube/+page.svelte::pickMuxed`.
fn pick_muxed(result: &StreamUrlResult) -> Option<&mhaol_yt_dlp::ResolvedFormat> {
    let mut muxed: Vec<&mhaol_yt_dlp::ResolvedFormat> = result
        .formats
        .iter()
        .filter(|f| !f.is_audio_only && !f.is_video_only)
        .collect();
    muxed.sort_by(|a, b| {
        b.height
            .unwrap_or(0)
            .cmp(&a.height.unwrap_or(0))
            .then_with(|| b.bitrate.cmp(&a.bitrate))
    });
    muxed.first().copied()
}

/// Pick the format `pickAudio` picks: muxed first (so the browser can
/// actually play it), then audio-only as a last-resort fallback. Mirrors
/// `apps/cloud/web/src/routes/youtube/+page.svelte::pickAudio`.
fn pick_audio(result: &StreamUrlResult) -> Option<&mhaol_yt_dlp::ResolvedFormat> {
    if let Some(muxed) = pick_muxed(result) {
        return Some(muxed);
    }
    let audio_only: Vec<&mhaol_yt_dlp::ResolvedFormat> = result
        .formats
        .iter()
        .filter(|f| f.is_audio_only)
        .collect();
    let mut mp4_audio: Vec<&mhaol_yt_dlp::ResolvedFormat> = audio_only
        .iter()
        .copied()
        .filter(|f| f.container == "mp4")
        .collect();
    mp4_audio.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));
    if let Some(f) = mp4_audio.first().copied() {
        return Some(f);
    }
    let mut any_audio = audio_only;
    any_audio.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));
    any_audio.first().copied()
}

/// Range-fetch the resolved format URL and confirm the CDN actually serves it.
/// Returns `(status, content_length, content_type)`.
///
/// `as_media_element=true` simulates how a `<video src=...>` / `<audio src=...>`
/// element actually fetches in a browser tab loaded from `http://localhost:9898`:
/// no `Origin`, the `Referer` is the page URL (downgraded to origin), and the
/// `User-Agent` is the browser's default. Crucially, this is **not** the same
/// as a `fetch()` from JS — which is what a lot of yt-dlp ports get wrong.
///
/// `as_media_element=false` mimics youtube.com itself fetching the URL
/// (`Origin` + `Referer` set to youtube.com). This proves the URL is signed
/// correctly even if it gets rejected when fetched from a localhost page.
async fn fetch_resolved_url(
    url: &str,
    as_media_element: bool,
) -> anyhow::Result<(u16, Option<u64>, String)> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;
    let mut req = client
        .get(url)
        .header(reqwest::header::USER_AGENT, BROWSER_USER_AGENT)
        .header(reqwest::header::RANGE, "bytes=0-0");
    if as_media_element {
        // Mirror what a media element on a localhost cloud webui sends. No
        // Origin (HTML media elements don't send Origin without `crossorigin`),
        // Referer is the page origin.
        req = req.header(reqwest::header::REFERER, "http://localhost:9898/");
    } else {
        req = req
            .header(reqwest::header::REFERER, "https://www.youtube.com/")
            .header(reqwest::header::ORIGIN, "https://www.youtube.com");
    }
    let resp = req.send().await?;
    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let content_length = resp
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cr| cr.rsplit('/').next())
        .and_then(|total| total.parse::<u64>().ok())
        .or_else(|| {
            resp.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
        });
    Ok((status, content_length, content_type))
}

/// `extract_stream_urls_for_browser` must return at least one muxed format,
/// and that format's URL must actually serve bytes to a browser User-Agent.
/// This is the "Stream video" button path on `/youtube`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn browser_extraction_yields_playable_video_url() {
    if should_skip() {
        return;
    }
    ensure_logging();

    let url = test_url();
    eprintln!("video extraction test using: {url}");

    let manager = make_manager();
    let result = manager
        .extract_stream_urls_for_browser(&url)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "extract_stream_urls_for_browser failed for {url}: {e}\n\
                 If this is the per-client fallback regressing, check \
                 fetch_and_resolve_with_clients in pipeline.rs."
            )
        });

    assert!(
        !result.formats.is_empty(),
        "expected >= 1 resolved format, got 0; \
         the prefer-browser client fallback isn't catching empty resolves anymore"
    );

    let format = pick_muxed(&result).unwrap_or_else(|| {
        let summary = result
            .formats
            .iter()
            .map(|f| {
                format!(
                    "itag={} container={} audioOnly={} videoOnly={} mime={}",
                    f.itag, f.container, f.is_audio_only, f.is_video_only, f.mime_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n  ");
        panic!(
            "no muxed (video+audio) format in extraction result for {url}.\n  {summary}"
        )
    });

    eprintln!(
        "picked muxed: itag={} container={} {}x{} {}kbps",
        format.itag,
        format.container,
        format.width.unwrap_or(0),
        format.height.unwrap_or(0),
        format.bitrate / 1000
    );

    let (status, content_length, content_type) = fetch_resolved_url(&format.url, true)
        .await
        .unwrap_or_else(|e| panic!("range-fetch as <video> failed for muxed itag {}: {e}", format.itag));

    assert!(
        (200..300).contains(&status),
        "muxed URL returned non-2xx as <video> for itag={}: status={}, content-type={:?}, url={}",
        format.itag,
        status,
        content_type,
        format.url
    );
    assert!(
        content_type.starts_with("video/") || content_type.starts_with("audio/"),
        "muxed URL returned unexpected content-type {:?} (expected video/* or audio/*)",
        content_type
    );
    assert!(
        content_length.unwrap_or(0) > 0,
        "muxed URL returned 2xx but content-length is 0 — CDN handed us an empty stream"
    );
}

/// `extract_stream_urls_for_browser` must produce at least one mp4 audio-only
/// format and that format's URL must serve bytes. This is the "Stream audio"
/// button path on `/youtube`. The mp4 (AAC) preference matters for Safari —
/// Safari won't decode the webm/opus audio-only adaptive streams YouTube
/// serves to non-WEB clients.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn browser_extraction_yields_playable_audio_url() {
    if should_skip() {
        return;
    }
    ensure_logging();

    let url = test_url();
    eprintln!("audio extraction test using: {url}");

    let manager = make_manager();
    let result = manager
        .extract_stream_urls_for_browser(&url)
        .await
        .unwrap_or_else(|e| panic!("extract_stream_urls_for_browser failed for {url}: {e}"));

    let format = pick_audio(&result).unwrap_or_else(|| {
        panic!("no playable format (muxed nor audio-only) in extraction result for {url}")
    });

    eprintln!(
        "picked audio-mode format: itag={} container={} {}kbps audioOnly={} videoOnly={}",
        format.itag,
        format.container,
        format.bitrate / 1000,
        format.is_audio_only,
        format.is_video_only,
    );

    // Audio-mode playback in the cloud webui must NOT pick a fragmented-MP4
    // audio-only stream — those need MediaSource Extensions to play in a
    // plain `<video src=...>` element and the browser surfaces them as
    // MEDIA_ERR_SRC_NOT_SUPPORTED. The picker is meant to prefer the muxed
    // (self-contained mp4) format for that reason.
    assert!(
        !format.is_audio_only || pick_muxed(&result).is_none(),
        "audio mode picked an audio-only fragmented MP4 (itag={}) when a muxed format was \
         available — this regresses the browser-playable contract",
        format.itag
    );

    // Browser <video> simulation. The cloud's PlayerVideo always uses a
    // single <video> element regardless of mode, so the audio test fetches
    // with the same shape the browser would.
    let (status, content_length, content_type) = fetch_resolved_url(&format.url, true)
        .await
        .unwrap_or_else(|e| panic!("range-fetch failed for itag {}: {e}", format.itag));

    if !(200..300).contains(&status) {
        // Useful diagnostics: try the same URL with the youtube-origin headers
        // — if that succeeds, we know the URL is signed correctly and the
        // CDN is rejecting the localhost-page Referer specifically.
        let yt = fetch_resolved_url(&format.url, false).await.ok();
        panic!(
            "audio URL returned non-2xx as <audio> for itag={}: status={}, content-type={:?}\n  \
             with youtube origin: {:?}\n  url={}",
            format.itag, status, content_type, yt, format.url
        );
    }
    assert!(
        content_type.starts_with("audio/") || content_type.starts_with("video/"),
        "audio URL returned unexpected content-type {:?}",
        content_type
    );
    assert!(
        content_length.unwrap_or(0) > 0,
        "audio URL returned 2xx but content-length is 0"
    );
}

/// The non-browser `extract_stream_urls` path is exercised by the player's
/// download flow. Cover it too so a regression in the shared
/// `extract_stream_urls_inner` isn't masked by the browser-only test passing.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn non_browser_extraction_yields_resolved_formats() {
    if should_skip() {
        return;
    }
    ensure_logging();

    let url = test_url();
    eprintln!("non-browser extraction test using: {url}");

    let manager = make_manager();
    let result = manager
        .extract_stream_urls(&url)
        .await
        .unwrap_or_else(|e| panic!("extract_stream_urls failed for {url}: {e}"));

    assert!(
        !result.formats.is_empty(),
        "extract_stream_urls returned 0 formats"
    );
    assert!(
        result.formats.iter().any(|f| !f.url.is_empty()),
        "every resolved format has an empty URL"
    );
}

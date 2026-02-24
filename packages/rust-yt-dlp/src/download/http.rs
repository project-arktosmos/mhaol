use anyhow::Result;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

/// 1 MB chunk size for range requests.
/// YouTube's CDN rejects range requests larger than ~1 MB for some videos.
const CHUNK_SIZE: u64 = 1024 * 1024;

/// Progress update for a download.
#[derive(Debug, Clone)]
pub struct DownloadProgressUpdate {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Download a file using HTTP range requests with progress reporting.
pub async fn download_with_progress(
    client: &reqwest::Client,
    url: &str,
    output_path: &Path,
    total_bytes: Option<u64>,
    progress_tx: watch::Sender<DownloadProgressUpdate>,
    cancel_rx: watch::Receiver<bool>,
) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Check if we can resume
    let resume_from = if output_path.exists() {
        tokio::fs::metadata(output_path).await?.len()
    } else {
        0
    };

    // If total_bytes is known and we've already downloaded everything
    if let Some(total) = total_bytes {
        if resume_from >= total {
            let _ = progress_tx.send(DownloadProgressUpdate {
                downloaded_bytes: total,
                total_bytes: total,
            });
            return Ok(());
        }
    }

    // Determine total size if not provided
    let total = if let Some(t) = total_bytes {
        t
    } else {
        // HEAD request to get content length
        let resp = client
            .head(url)
            .header("Accept-Encoding", "identity")
            .send()
            .await?;
        resp.content_length().unwrap_or(0)
    };

    let mut downloaded = resume_from;

    // Open file in append mode for resume support
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)
        .await?;

    // Download in chunks using range requests
    while downloaded < total || total == 0 {
        // Check for cancellation
        if *cancel_rx.borrow() {
            anyhow::bail!("Download cancelled");
        }

        let range_start = downloaded;
        let range_end = if total > 0 {
            std::cmp::min(downloaded + CHUNK_SIZE - 1, total - 1)
        } else {
            downloaded + CHUNK_SIZE - 1
        };

        let response = client
            .get(url)
            .header("Range", format!("bytes={}-{}", range_start, range_end))
            .header("Accept-Encoding", "identity")
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
            // We've downloaded everything
            break;
        }

        if status == reqwest::StatusCode::OK && downloaded > 0 {
            // Server doesn't support range requests; need to re-download from start
            log::warn!("Server doesn't support range requests, restarting download");
            downloaded = 0;
            file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output_path)
                .await?;
        }

        if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
            anyhow::bail!("HTTP error: {}", status);
        }

        // If the server returned 200 (full content) instead of 206
        let is_full_response = status == reqwest::StatusCode::OK;

        // Update total from content-length if we didn't know it
        let actual_total = if is_full_response {
            response.content_length().unwrap_or(total)
        } else {
            total
        };

        let bytes = response.bytes().await?;
        let chunk_len = bytes.len() as u64;

        if chunk_len == 0 {
            break;
        }

        file.write_all(&bytes).await?;
        downloaded += chunk_len;

        let _ = progress_tx.send(DownloadProgressUpdate {
            downloaded_bytes: downloaded,
            total_bytes: if actual_total > 0 { actual_total } else { downloaded },
        });

        // If we got the full file in one response, we're done
        if is_full_response {
            break;
        }
    }

    file.flush().await?;

    let _ = progress_tx.send(DownloadProgressUpdate {
        downloaded_bytes: downloaded,
        total_bytes: if total > 0 { total } else { downloaded },
    });

    Ok(())
}

/// Simple download without range requests (for smaller files or when range isn't supported).
pub async fn download_simple(
    client: &reqwest::Client,
    url: &str,
    output_path: &Path,
) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let response = client
        .get(url)
        .header("Accept-Encoding", "identity")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }

    let bytes = response.bytes().await?;
    tokio::fs::write(output_path, &bytes).await?;

    Ok(())
}

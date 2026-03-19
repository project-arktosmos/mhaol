use anyhow::Result;
use reqwest::header::HeaderMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

/// Progress update for a download.
#[derive(Debug, Clone)]
pub struct DownloadProgressUpdate {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Download a file by streaming the full response in one connection.
///
/// YouTube's CDN returns 403 for arbitrary mid-file range requests (it only accepts
/// requests that start from byte 0). Chunked range downloads therefore break on
/// any resume scenario. Streaming the whole file avoids this entirely.
pub async fn download_with_progress(
    client: &reqwest::Client,
    url: &str,
    output_path: &Path,
    total_bytes: Option<u64>,
    extra_headers: &HeaderMap,
    progress_tx: watch::Sender<DownloadProgressUpdate>,
    cancel_rx: watch::Receiver<bool>,
) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Remove any stale partial file — YouTube CDN 403s on range requests that do
    // not start at byte 0, so resume is not supported for these stream URLs.
    if output_path.exists() {
        tokio::fs::remove_file(output_path).await?;
    }

    if *cancel_rx.borrow() {
        anyhow::bail!("Download cancelled");
    }

    log::debug!("Starting download from host: {}", url::Url::parse(url).map(|u| u.host_str().unwrap_or("?").to_string()).unwrap_or_default());

    let mut response = client
        .get(url)
        .headers(extra_headers.clone())
        .header("Accept-Encoding", "identity")
        .send()
        .await?;

    let status = response.status();
    log::debug!("Download response status: {}", status);
    if !status.is_success() {
        anyhow::bail!("HTTP error: {}", status);
    }

    let total = total_bytes
        .or_else(|| response.content_length())
        .unwrap_or(0);

    let mut file = File::create(output_path).await?;
    let mut downloaded = 0u64;

    while let Some(chunk) = response.chunk().await? {
        if *cancel_rx.borrow() {
            drop(file);
            let _ = tokio::fs::remove_file(output_path).await;
            anyhow::bail!("Download cancelled");
        }

        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx.send(DownloadProgressUpdate {
            downloaded_bytes: downloaded,
            total_bytes: if total > 0 { total } else { downloaded },
        });
    }

    file.flush().await?;

    let _ = progress_tx.send(DownloadProgressUpdate {
        downloaded_bytes: downloaded,
        total_bytes: if total > 0 { total } else { downloaded },
    });

    Ok(())
}

/// Simple download without progress reporting.
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

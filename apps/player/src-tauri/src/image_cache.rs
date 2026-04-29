use std::path::PathBuf;

use sha3::{Digest, Sha3_256};
use tauri::{ipc::Response, AppHandle, Manager};

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let docs = app
        .path()
        .document_dir()
        .map_err(|e| format!("document_dir: {}", e))?;
    let dir = docs.join("mhaol-player").join("image-cache");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn filename_for(url_str: &str) -> String {
    let hash = Sha3_256::digest(url_str.as_bytes());
    let hex = hex::encode(hash);
    let ext = ::url::Url::parse(url_str)
        .ok()
        .and_then(|u| {
            std::path::Path::new(u.path())
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase())
        })
        .filter(|s| !s.is_empty() && s.len() <= 8 && s.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "bin".to_string());
    format!("{}.{}", hex, ext)
}

#[tauri::command]
pub async fn image_cache_resolve(app: AppHandle, url: String) -> Result<Response, String> {
    let dir = cache_dir(&app)?;
    let path = dir.join(filename_for(&url));

    if let Ok(bytes) = tokio::fs::read(&path).await {
        return Ok(Response::new(bytes));
    }

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?.to_vec();

    if let Err(e) = tokio::fs::write(&path, &bytes).await {
        log::warn!("image cache write failed for {}: {}", path.display(), e);
    }
    Ok(Response::new(bytes))
}

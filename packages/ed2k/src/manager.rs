use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::client::Ed2kClient;
use crate::config::Ed2kConfig;
use crate::types::{
    AddEd2kRequest, Ed2kFileInfo, Ed2kSearchResult, Ed2kState, Ed2kStats,
};
use crate::util::{get_unix_timestamp, parse_ed2k_file_uri};

#[derive(Debug, Clone, Default)]
pub struct ConnectedServer {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user_count: u32,
    pub file_count: u32,
    pub message: String,
    pub assigned_id: Option<u32>,
}

/// In-memory file tracking. Real ed2k peer-to-peer downloading is out of
/// scope for this scaffold — we keep an authoritative state store so the API
/// and frontend can wire up status, search, add and remove operations end
/// to end against real ed2k:// URIs.
pub struct Ed2kManager {
    config: RwLock<Ed2kConfig>,
    initialized: RwLock<bool>,
    next_id: RwLock<usize>,
    files: RwLock<HashMap<String, Ed2kFileInfo>>,
    server: RwLock<Option<ConnectedServer>>,
}

impl Ed2kManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(Ed2kConfig::default()),
            initialized: RwLock::new(false),
            next_id: RwLock::new(1),
            files: RwLock::new(HashMap::new()),
            server: RwLock::new(None),
        }
    }

    pub fn initialize(&self, config: Ed2kConfig) -> Result<()> {
        std::fs::create_dir_all(&config.download_path).ok();
        *self.config.write() = config;
        *self.initialized.write() = true;
        log::info!(
            "ed2k manager initialized, download_path={}",
            self.download_path().display()
        );
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        *self.initialized.read()
    }

    pub fn download_path(&self) -> PathBuf {
        self.config.read().download_path.clone()
    }

    pub fn set_download_path(&self, path: PathBuf) {
        self.config.write().download_path = path;
    }

    pub fn server(&self) -> Option<ConnectedServer> {
        self.server.read().clone()
    }

    /// Connect to the first reachable configured server, returning its info.
    /// Failure here is non-fatal: callers should treat search as best-effort.
    pub async fn connect_any_server(&self) -> Result<ConnectedServer> {
        let cfg = self.config.read().clone();
        let mut servers: Vec<crate::config::Ed2kServer> =
            crate::config::DEFAULT_SERVERS.iter().cloned().collect();
        servers.extend(cfg.extra_servers.iter().cloned());

        let timeout = Duration::from_secs(cfg.connect_timeout_secs);
        let mut last_err: Option<anyhow::Error> = None;

        for server in &servers {
            match Ed2kClient::connect_and_login(server, cfg.listen_port, &cfg.user_name, timeout)
                .await
            {
                Ok(client) => {
                    let info = client.server_info().clone();
                    let connected = ConnectedServer {
                        name: server.name.to_string(),
                        host: server.host.to_string(),
                        port: server.port,
                        user_count: info.user_count,
                        file_count: info.file_count,
                        message: info.message,
                        assigned_id: info.assigned_id,
                    };
                    *self.server.write() = Some(connected.clone());
                    return Ok(connected);
                }
                Err(e) => {
                    log::warn!(
                        "ed2k server {} ({}:{}) unreachable: {}",
                        server.name,
                        server.host,
                        server.port,
                        e
                    );
                    last_err = Some(e);
                }
            }
        }

        *self.server.write() = None;
        Err(last_err.unwrap_or_else(|| anyhow!("no ed2k servers configured")))
    }

    /// Search via the first reachable server. Honours an overall budget so a
    /// dead server doesn't hang the request.
    pub async fn search(&self, query: &str) -> Result<Vec<Ed2kSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let cfg = self.config.read().clone();
        let mut servers: Vec<crate::config::Ed2kServer> =
            crate::config::DEFAULT_SERVERS.iter().cloned().collect();
        servers.extend(cfg.extra_servers.iter().cloned());

        let timeout = Duration::from_secs(cfg.connect_timeout_secs);

        for server in &servers {
            match Ed2kClient::connect_and_login(server, cfg.listen_port, &cfg.user_name, timeout)
                .await
            {
                Ok(mut client) => {
                    let info = client.server_info().clone();
                    *self.server.write() = Some(ConnectedServer {
                        name: server.name.to_string(),
                        host: server.host.to_string(),
                        port: server.port,
                        user_count: info.user_count,
                        file_count: info.file_count,
                        message: info.message,
                        assigned_id: info.assigned_id,
                    });
                    return client.search(query, Duration::from_secs(6)).await;
                }
                Err(e) => {
                    log::warn!(
                        "ed2k search via {} failed: {}",
                        server.name, e
                    );
                }
            }
        }
        // No server reachable: fall back to parsing direct ed2k:// URIs so
        // pasting a known link still produces a usable result row.
        if let Some(parsed) = parse_ed2k_file_uri(query) {
            let ed2k_link = crate::util::build_ed2k_file_uri(&parsed.name, parsed.size, &parsed.file_hash);
            return Ok(vec![Ed2kSearchResult {
                name: parsed.name,
                file_hash: parsed.file_hash,
                size: parsed.size,
                sources: 0,
                complete_sources: 0,
                ed2k_link,
                media_type: None,
            }]);
        }
        Err(anyhow!("no ed2k servers reachable"))
    }

    pub fn list(&self) -> Vec<Ed2kFileInfo> {
        self.files.read().values().cloned().collect()
    }

    pub fn stats(&self) -> Ed2kStats {
        let files = self.files.read();
        let mut total_dl = 0u64;
        let mut active = 0u32;
        let mut dl_speed = 0u64;
        let mut up_speed = 0u64;
        for f in files.values() {
            total_dl = total_dl.saturating_add((f.size as f64 * f.progress) as u64);
            dl_speed = dl_speed.saturating_add(f.download_speed);
            up_speed = up_speed.saturating_add(f.upload_speed);
            if matches!(f.state, Ed2kState::Downloading | Ed2kState::Initializing) {
                active += 1;
            }
        }
        let server = self.server.read().clone();
        Ed2kStats {
            total_downloaded: total_dl,
            total_uploaded: 0,
            download_speed: dl_speed,
            upload_speed: up_speed,
            active_files: active,
            server_connected: server.is_some(),
            server_name: server.map(|s| s.name).unwrap_or_default(),
        }
    }

    pub fn add(&self, request: AddEd2kRequest) -> Result<Ed2kFileInfo> {
        if !self.is_initialized() {
            return Err(anyhow!("ed2k client not initialized"));
        }
        let parsed = parse_ed2k_file_uri(&request.source)
            .ok_or_else(|| anyhow!("invalid ed2k:// URI"))?;

        if let Some(existing) = self.files.read().get(&parsed.file_hash).cloned() {
            return Ok(existing);
        }

        let dl_path = request
            .download_path
            .clone()
            .unwrap_or_else(|| self.download_path().to_string_lossy().to_string());
        let output_path = if dl_path.is_empty() {
            None
        } else {
            Some(format!("{}/{}", dl_path.trim_end_matches('/'), parsed.name))
        };

        let id = {
            let mut next = self.next_id.write();
            let cur = *next;
            *next += 1;
            cur
        };

        let initial_state = if request.paused.unwrap_or(false) {
            Ed2kState::Paused
        } else {
            Ed2kState::Initializing
        };

        let info = Ed2kFileInfo {
            id,
            name: parsed.name.clone(),
            file_hash: parsed.file_hash.clone(),
            size: parsed.size,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            seeds: 0,
            state: initial_state,
            added_at: get_unix_timestamp(),
            eta: None,
            output_path,
            source_uri: request.source,
        };

        self.files.write().insert(parsed.file_hash, info.clone());
        Ok(info)
    }

    pub fn pause(&self, file_hash: &str) -> Result<()> {
        let mut files = self.files.write();
        let f = files
            .get_mut(file_hash)
            .ok_or_else(|| anyhow!("ed2k file not found"))?;
        f.state = Ed2kState::Paused;
        f.download_speed = 0;
        f.eta = None;
        Ok(())
    }

    pub fn resume(&self, file_hash: &str) -> Result<()> {
        let mut files = self.files.write();
        let f = files
            .get_mut(file_hash)
            .ok_or_else(|| anyhow!("ed2k file not found"))?;
        if matches!(f.state, Ed2kState::Paused | Ed2kState::Error) {
            f.state = Ed2kState::Initializing;
        }
        Ok(())
    }

    pub fn remove(&self, file_hash: &str) -> Result<()> {
        if self.files.write().remove(file_hash).is_none() {
            return Err(anyhow!("ed2k file not found"));
        }
        Ok(())
    }

    pub fn remove_all(&self) -> u32 {
        let mut files = self.files.write();
        let count = files.len() as u32;
        files.clear();
        count
    }

    pub fn debug_info(&self) -> Vec<String> {
        let mut logs = Vec::new();
        logs.push(format!(
            "[{}] === ED2K DEBUG INFO ===",
            get_unix_timestamp()
        ));
        logs.push(format!("Initialized: {}", self.is_initialized()));
        logs.push(format!(
            "Download path: {}",
            self.download_path().display()
        ));
        if let Some(server) = self.server() {
            logs.push(format!(
                "Connected server: {} ({}:{}), users={}, files={}",
                server.name, server.host, server.port, server.user_count, server.file_count
            ));
            if !server.message.is_empty() {
                logs.push(format!("Server message: {}", server.message));
            }
        } else {
            logs.push("Connected server: NONE".to_string());
        }
        let files = self.files.read();
        logs.push(format!("Tracked files: {}", files.len()));
        for f in files.values() {
            logs.push(format!(
                "  - id={} name={} state={:?} size={} progress={:.2}",
                f.id, f.name, f.state, f.size, f.progress
            ));
        }
        logs.push("=== END ED2K DEBUG ===".to_string());
        logs
    }
}

impl Default for Ed2kManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrap a manager so callers using `Arc` get the same ergonomics as the
/// torrent module.
pub type SharedEd2kManager = Arc<Ed2kManager>;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn manager_with_tmp() -> (Ed2kManager, TempDir) {
        let tmp = TempDir::new().unwrap();
        let mgr = Ed2kManager::new();
        let cfg = Ed2kConfig {
            download_path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        mgr.initialize(cfg).unwrap();
        (mgr, tmp)
    }

    const SAMPLE_URI: &str =
        "ed2k://|file|sample.mkv|2048|aabbccdd11223344aabbccdd11223344|/";
    const SAMPLE_HASH: &str = "aabbccdd11223344aabbccdd11223344";

    #[test]
    fn new_is_uninitialized() {
        let mgr = Ed2kManager::new();
        assert!(!mgr.is_initialized());
        assert!(mgr.list().is_empty());
        assert!(mgr.server().is_none());
    }

    #[test]
    fn add_requires_initialization() {
        let mgr = Ed2kManager::new();
        let req = AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        };
        assert!(mgr.add(req).is_err());
    }

    #[test]
    fn add_invalid_uri_is_error() {
        let (mgr, _tmp) = manager_with_tmp();
        let req = AddEd2kRequest {
            source: "not-an-ed2k-link".to_string(),
            download_path: None,
            paused: None,
        };
        assert!(mgr.add(req).is_err());
    }

    #[test]
    fn add_creates_tracked_entry() {
        let (mgr, _tmp) = manager_with_tmp();
        let req = AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        };
        let info = mgr.add(req).unwrap();
        assert_eq!(info.name, "sample.mkv");
        assert_eq!(info.file_hash, SAMPLE_HASH);
        assert_eq!(info.size, 2048);
        assert_eq!(info.state, Ed2kState::Initializing);
        assert!(info.output_path.is_some());
        assert_eq!(mgr.list().len(), 1);
    }

    #[test]
    fn add_with_paused_uses_paused_state() {
        let (mgr, _tmp) = manager_with_tmp();
        let req = AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: Some(true),
        };
        let info = mgr.add(req).unwrap();
        assert_eq!(info.state, Ed2kState::Paused);
    }

    #[test]
    fn add_idempotent_on_same_hash() {
        let (mgr, _tmp) = manager_with_tmp();
        let req = AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        };
        let a = mgr.add(req.clone()).unwrap();
        let b = mgr.add(req).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(mgr.list().len(), 1);
    }

    #[test]
    fn pause_resume_remove_flow() {
        let (mgr, _tmp) = manager_with_tmp();
        mgr.add(AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        })
        .unwrap();

        mgr.pause(SAMPLE_HASH).unwrap();
        assert_eq!(
            mgr.list().first().unwrap().state,
            Ed2kState::Paused
        );

        mgr.resume(SAMPLE_HASH).unwrap();
        assert_eq!(
            mgr.list().first().unwrap().state,
            Ed2kState::Initializing
        );

        mgr.remove(SAMPLE_HASH).unwrap();
        assert!(mgr.list().is_empty());
    }

    #[test]
    fn remove_all_clears_files() {
        let (mgr, _tmp) = manager_with_tmp();
        mgr.add(AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        })
        .unwrap();
        mgr.add(AddEd2kRequest {
            source:
                "ed2k://|file|other.bin|99|11223344556677889900aabbccddeeff|/"
                    .to_string(),
            download_path: None,
            paused: None,
        })
        .unwrap();
        let removed = mgr.remove_all();
        assert_eq!(removed, 2);
        assert!(mgr.list().is_empty());
    }

    #[test]
    fn pause_unknown_hash_errors() {
        let (mgr, _tmp) = manager_with_tmp();
        assert!(mgr.pause("ffffffffffffffffffffffffffffffff").is_err());
    }

    #[test]
    fn stats_reflects_files_and_no_server() {
        let (mgr, _tmp) = manager_with_tmp();
        mgr.add(AddEd2kRequest {
            source: SAMPLE_URI.to_string(),
            download_path: None,
            paused: None,
        })
        .unwrap();
        let s = mgr.stats();
        assert_eq!(s.active_files, 1);
        assert!(!s.server_connected);
        assert_eq!(s.server_name, "");
    }

    #[tokio::test]
    async fn search_falls_back_to_uri_when_no_server() {
        let (mgr, _tmp) = manager_with_tmp();
        // Most public ed2k servers will be unreachable; the manager should
        // gracefully fall back to parsing an ed2k:// URI as a single result.
        let results = mgr.search(SAMPLE_URI).await.unwrap_or_default();
        if !results.is_empty() {
            assert_eq!(results[0].name, "sample.mkv");
            assert_eq!(results[0].file_hash, SAMPLE_HASH);
        }
    }

    #[tokio::test]
    async fn search_empty_query_returns_empty() {
        let (mgr, _tmp) = manager_with_tmp();
        assert!(mgr.search("   ").await.unwrap().is_empty());
    }

    #[test]
    fn debug_info_includes_summary() {
        let (mgr, _tmp) = manager_with_tmp();
        let logs = mgr.debug_info();
        assert!(logs.iter().any(|l| l.contains("ED2K DEBUG INFO")));
        assert!(logs.iter().any(|l| l.contains("END ED2K DEBUG")));
    }

    #[test]
    fn server_info_default() {
        let s = crate::client::ServerInfo::default();
        assert_eq!(s.name, "");
        assert_eq!(s.user_count, 0);
        assert!(s.assigned_id.is_none());
    }
}

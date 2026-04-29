use crate::state::CloudState;
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

const STARTED_AT: once_cell::sync::Lazy<u64> = once_cell::sync::Lazy::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
});

#[derive(Serialize)]
struct CloudStatus {
    status: &'static str,
    version: &'static str,
    started_at: u64,
    now: u64,
    uptime_seconds: u64,
    host: String,
    port: u16,
    local_ip: Option<String>,
    signaling_address: Option<String>,
    client_address: Option<String>,
    db: DbStatus,
    packages: PackagesHealth,
}

#[derive(Serialize)]
struct DbStatus {
    engine: &'static str,
    namespace: &'static str,
    database: &'static str,
    connected: bool,
    version: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PackagesHealth {
    p2p_stream: PackageHealth,
    queue: PackageHealth,
    yt_dlp: PackageHealth,
    torrent: PackageHealth,
    ed2k: PackageHealth,
    ipfs: PackageHealth,
}

#[derive(Serialize)]
struct PackageHealth {
    name: &'static str,
    status: &'static str,
    available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    details: serde_json::Value,
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/status", get(status))
}

async fn status(State(state): State<CloudState>) -> Json<CloudStatus> {
    let _ = *STARTED_AT;
    let started_at = *STARTED_AT;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(started_at);
    let uptime_seconds = if now > started_at {
        (now - started_at) / 1000
    } else {
        0
    };

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9898);
    let local_ip = get_local_ip();

    let signaling_address = state
        .identity_manager
        .get_address("SIGNALING_WALLET")
        .map(|a| mhaol_identity::to_eip55_checksum(&a));
    let client_address = state
        .identity_manager
        .get_address("CLIENT_WALLET")
        .map(|a| mhaol_identity::to_eip55_checksum(&a));

    let db_version = state.db.version().await.ok().map(|v| v.to_string());
    let db = DbStatus {
        engine: "surrealkv",
        namespace: crate::db::NAMESPACE,
        database: crate::db::DATABASE,
        connected: db_version.is_some(),
        version: db_version,
    };

    let packages = PackagesHealth {
        p2p_stream: p2p_stream_health(),
        queue: queue_health(&state),
        yt_dlp: yt_dlp_health(&state),
        torrent: torrent_health(&state).await,
        ed2k: ed2k_health(&state),
        ipfs: ipfs_health(&state).await,
    };

    Json(CloudStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        started_at,
        now,
        uptime_seconds,
        host,
        port,
        local_ip,
        signaling_address,
        client_address,
        db,
        packages,
    })
}

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

fn p2p_stream_health() -> PackageHealth {
    #[cfg(not(target_os = "android"))]
    {
        let initialized = mhaol_p2p_stream::init().is_ok();
        let missing = if initialized {
            mhaol_p2p_stream::check_required_elements()
        } else {
            Vec::new()
        };
        let (status, message) = if !initialized {
            (
                "error",
                Some("GStreamer failed to initialize".to_string()),
            )
        } else if !missing.is_empty() {
            (
                "warning",
                Some(format!("Missing GStreamer elements: {}", missing.join(", "))),
            )
        } else {
            ("ok", None)
        };
        return PackageHealth {
            name: "p2p-stream",
            status,
            available: initialized && missing.is_empty(),
            message,
            details: serde_json::json!({
                "gstreamerInitialized": initialized,
                "missingElements": missing,
            }),
        };
    }
    #[cfg(target_os = "android")]
    {
        PackageHealth {
            name: "p2p-stream",
            status: "unavailable",
            available: false,
            message: Some("Not built for this target".to_string()),
            details: serde_json::json!({}),
        }
    }
}

fn queue_health(state: &CloudState) -> PackageHealth {
    let pending = state.queue.list(Some("pending"), None).len();
    let running = state.queue.list(Some("running"), None).len();
    let completed = state.queue.list(Some("completed"), None).len();
    let failed = state.queue.list(Some("failed"), None).len();
    let cancelled = state.queue.list(Some("cancelled"), None).len();
    let total = pending + running + completed + failed + cancelled;

    let status = if failed > 0 { "warning" } else { "ok" };

    PackageHealth {
        name: "queue",
        status,
        available: true,
        message: None,
        details: serde_json::json!({
            "total": total,
            "pending": pending,
            "running": running,
            "completed": completed,
            "failed": failed,
            "cancelled": cancelled,
        }),
    }
}

#[cfg_attr(target_os = "android", allow(unused_variables))]
fn yt_dlp_health(state: &CloudState) -> PackageHealth {
    #[cfg(not(target_os = "android"))]
    {
        let stats = state.ytdl_manager.get_stats();
        let (status, message) = if !stats.ytdlp_available {
            (
                "warning",
                Some("yt-dlp binary not detected".to_string()),
            )
        } else if stats.failed_downloads > 0 {
            ("warning", None)
        } else {
            ("ok", None)
        };
        return PackageHealth {
            name: "yt-dlp",
            status,
            available: stats.ytdlp_available,
            message,
            details: serde_json::json!({
                "ytdlpAvailable": stats.ytdlp_available,
                "ytdlpVersion": stats.ytdlp_version,
                "active": stats.active_downloads,
                "queued": stats.queued_downloads,
                "completed": stats.completed_downloads,
                "failed": stats.failed_downloads,
            }),
        };
    }
    #[cfg(target_os = "android")]
    {
        PackageHealth {
            name: "yt-dlp",
            status: "unavailable",
            available: false,
            message: Some("Not built for this target".to_string()),
            details: serde_json::json!({}),
        }
    }
}

#[cfg_attr(target_os = "android", allow(unused_variables))]
async fn torrent_health(state: &CloudState) -> PackageHealth {
    #[cfg(not(target_os = "android"))]
    {
        let initialized = state.torrent_manager.is_initialized();
        if !initialized {
            return PackageHealth {
                name: "torrent",
                status: "warning",
                available: false,
                message: Some("Torrent session warming up".to_string()),
                details: serde_json::json!({ "initialized": false }),
            };
        }
        match state.torrent_manager.stats().await {
            Ok(stats) => PackageHealth {
                name: "torrent",
                status: "ok",
                available: true,
                message: None,
                details: serde_json::json!({
                    "initialized": true,
                    "activeTorrents": stats.active_torrents,
                    "downloadSpeed": stats.download_speed,
                    "uploadSpeed": stats.upload_speed,
                    "totalDownloaded": stats.total_downloaded,
                    "totalUploaded": stats.total_uploaded,
                }),
            },
            Err(e) => PackageHealth {
                name: "torrent",
                status: "error",
                available: false,
                message: Some(e.to_string()),
                details: serde_json::json!({ "initialized": true }),
            },
        }
    }
    #[cfg(target_os = "android")]
    {
        PackageHealth {
            name: "torrent",
            status: "unavailable",
            available: false,
            message: Some("Not built for this target".to_string()),
            details: serde_json::json!({}),
        }
    }
}

#[cfg_attr(target_os = "android", allow(unused_variables))]
async fn ipfs_health(state: &CloudState) -> PackageHealth {
    #[cfg(not(target_os = "android"))]
    {
        let stats = state.ipfs_manager.stats().await;
        let initialized = stats.state == mhaol_ipfs::IpfsState::Running;
        let (status, message) = match stats.state {
            mhaol_ipfs::IpfsState::Running => ("ok", None),
            mhaol_ipfs::IpfsState::Starting => (
                "warning",
                Some("IPFS node starting".to_string()),
            ),
            mhaol_ipfs::IpfsState::Stopped => (
                "warning",
                Some("IPFS node not initialized".to_string()),
            ),
            mhaol_ipfs::IpfsState::Error => (
                "error",
                Some("IPFS node failed to start".to_string()),
            ),
        };
        return PackageHealth {
            name: "ipfs",
            status,
            available: initialized,
            message,
            details: serde_json::json!({
                "state": stats.state,
                "peerId": stats.peer_id,
                "agentVersion": stats.agent_version,
                "connectedPeers": stats.connected_peers,
                "pinnedCount": stats.pinned_count,
                "repoSizeBytes": stats.repo_size_bytes,
                "listenAddrs": stats.listen_addrs,
            }),
        };
    }
    #[cfg(target_os = "android")]
    {
        PackageHealth {
            name: "ipfs",
            status: "unavailable",
            available: false,
            message: Some("Not built for this target".to_string()),
            details: serde_json::json!({}),
        }
    }
}

#[cfg_attr(target_os = "android", allow(unused_variables))]
fn ed2k_health(state: &CloudState) -> PackageHealth {
    #[cfg(not(target_os = "android"))]
    {
        let initialized = state.ed2k_manager.is_initialized();
        let stats = state.ed2k_manager.stats();
        let (status, message) = if !initialized {
            (
                "warning",
                Some("ed2k client not initialized".to_string()),
            )
        } else if !stats.server_connected {
            (
                "warning",
                Some("Not connected to an ed2k server".to_string()),
            )
        } else {
            ("ok", None)
        };
        return PackageHealth {
            name: "ed2k",
            status,
            available: initialized && stats.server_connected,
            message,
            details: serde_json::json!({
                "initialized": initialized,
                "serverConnected": stats.server_connected,
                "serverName": stats.server_name,
                "activeFiles": stats.active_files,
                "downloadSpeed": stats.download_speed,
                "uploadSpeed": stats.upload_speed,
            }),
        };
    }
    #[cfg(target_os = "android")]
    {
        PackageHealth {
            name: "ed2k",
            status: "unavailable",
            available: false,
            message: Some("Not built for this target".to_string()),
            details: serde_json::json!({}),
        }
    }
}

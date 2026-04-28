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
        .unwrap_or(1540);
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

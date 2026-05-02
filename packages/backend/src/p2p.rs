//! Browser-side libp2p bootstrap endpoint.
//!
//! Returns the cloud's libp2p peer id, swarm key, and browser-dialable
//! multiaddrs as JSON so any browser-resident peer can join the same
//! private swarm without out-of-band configuration. Browsers cannot dial
//! raw TCP libp2p and cannot read the cloud's swarm-key off disk, so
//! they fetch both via HTTP from this endpoint on mount and hand the
//! values to Helia.
//!
//! Trust boundary: anyone who can reach `/api/p2p/bootstrap` is presumed
//! to be on the LAN and is granted full membership in the private swarm.
//! That is the same trust boundary the cloud's HTTP API already operates
//! under.

use crate::state::CloudState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::net::UdpSocket;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct P2pBootstrap {
    peer_id: String,
    swarm_key: String,
    multiaddrs: Vec<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/bootstrap", get(bootstrap))
}

async fn bootstrap(State(state): State<CloudState>) -> Response {
    let stats = state.ipfs_manager.stats().await;
    if stats.state != mhaol_ipfs_core::IpfsState::Running {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER, "1")],
            "ipfs node not ready",
        )
            .into_response();
    }

    let Some(peer_id) = stats.peer_id else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER, "1")],
            "ipfs peer id unavailable",
        )
            .into_response();
    };

    let swarm_key = match std::fs::read_to_string(crate::paths::swarm_key_path()) {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!("[p2p] swarm key read failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "swarm key unavailable")
                .into_response();
        }
    };

    let local_ip = get_local_ip();
    let multiaddrs = browser_dialable_addrs(&stats.listen_addrs, &peer_id, local_ip.as_deref());

    Json(P2pBootstrap {
        peer_id,
        swarm_key,
        multiaddrs,
    })
    .into_response()
}

/// Filter the IPFS node's listen addresses down to ones a browser can dial,
/// substitute `0.0.0.0` for loopback + LAN IP, and append `/p2p/<peer_id>`
/// when missing so Helia knows whom it's connecting to.
fn browser_dialable_addrs(
    listen_addrs: &[String],
    peer_id: &str,
    local_ip: Option<&str>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for addr in listen_addrs {
        if !is_browser_dialable(addr) {
            continue;
        }
        for resolved in expand_wildcard_host(addr, local_ip) {
            let with_peer = if resolved.contains("/p2p/") {
                resolved
            } else {
                format!("{}/p2p/{}", resolved, peer_id)
            };
            if !out.contains(&with_peer) {
                out.push(with_peer);
            }
        }
    }
    out
}

fn is_browser_dialable(addr: &str) -> bool {
    addr.contains("/ws") || addr.contains("/wss") || addr.contains("/webtransport")
}

fn expand_wildcard_host(addr: &str, local_ip: Option<&str>) -> Vec<String> {
    if addr.starts_with("/ip4/0.0.0.0/") {
        let mut variants = vec![addr.replacen("/ip4/0.0.0.0/", "/ip4/127.0.0.1/", 1)];
        if let Some(ip) = local_ip {
            if ip != "127.0.0.1" {
                variants.push(addr.replacen("/ip4/0.0.0.0/", &format!("/ip4/{}/", ip), 1));
            }
        }
        variants
    } else if addr.starts_with("/ip6/::/") {
        vec![addr.replacen("/ip6/::/", "/ip6/::1/", 1)]
    } else {
        vec![addr.to_string()]
    }
}

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_to_browser_dialable() {
        let addrs = vec![
            "/ip4/0.0.0.0/tcp/9900".to_string(),
            "/ip4/0.0.0.0/tcp/9901/ws".to_string(),
            "/ip4/0.0.0.0/udp/4001/quic".to_string(),
        ];
        let out = browser_dialable_addrs(&addrs, "12D3PEER", None);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], "/ip4/127.0.0.1/tcp/9901/ws/p2p/12D3PEER");
    }

    #[test]
    fn expands_wildcard_to_loopback_and_lan() {
        let addrs = vec!["/ip4/0.0.0.0/tcp/9901/ws".to_string()];
        let out = browser_dialable_addrs(&addrs, "12D3PEER", Some("192.168.1.42"));
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|a| a.contains("/ip4/127.0.0.1/")));
        assert!(out.iter().any(|a| a.contains("/ip4/192.168.1.42/")));
    }

    #[test]
    fn skips_lan_ip_when_loopback() {
        let addrs = vec!["/ip4/0.0.0.0/tcp/9901/ws".to_string()];
        let out = browser_dialable_addrs(&addrs, "12D3PEER", Some("127.0.0.1"));
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn preserves_existing_peer_id() {
        let addrs = vec!["/ip4/0.0.0.0/tcp/9901/ws/p2p/12D3OTHER".to_string()];
        let out = browser_dialable_addrs(&addrs, "12D3PEER", None);
        assert_eq!(out.len(), 1);
        assert!(out[0].ends_with("/p2p/12D3OTHER"));
    }
}

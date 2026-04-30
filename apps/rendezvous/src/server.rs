mod config;
mod signaling;
mod state;
mod status;

use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::Router;
use config::RendezvousConfig;
use mhaol_ipfs::{ensure_swarm_key, IpfsConfig, IpfsManager};
use state::RendezvousState;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_rendezvous=debug".into()),
        )
        .init();

    let cfg = RendezvousConfig::from_env();

    std::fs::create_dir_all(&cfg.repo_path).ok();
    if let Some(parent) = cfg.swarm_key_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // The rendezvous node only operates on the private swarm. If we cannot
    // load or generate a swarm key we refuse to start — running it on the
    // public DHT would defeat the whole point.
    let swarm_key = ensure_swarm_key(&cfg.swarm_key_path).map_err(|e| {
        anyhow!(
            "Failed to load or generate swarm key at {}: {}",
            cfg.swarm_key_path.display(),
            e
        )
    })?;

    let manager = Arc::new(IpfsManager::new());
    let ipfs_config = IpfsConfig {
        repo_path: cfg.repo_path.clone(),
        listen_port: cfg.ipfs_listen_port,
        // mDNS discovery is unnecessary for a server-mode bootstrap node and
        // any peers it would find would still need our PSK; off by default.
        enable_mdns: false,
        enable_kad_dht: true,
        // Standalone bootstrap node: nothing else to dial on startup.
        bootstrap_on_start: false,
        extra_bootstrap: vec![],
        agent_version: format!("mhaol-rendezvous/{}", env!("CARGO_PKG_VERSION")),
        swarm_key: Some(swarm_key),
        // Bootstrap nodes must serve DHT queries from other peers.
        dht_server_mode: true,
    };

    manager
        .initialize(ipfs_config)
        .await
        .map_err(|e| anyhow!("Failed to initialize IPFS: {}", e))?;

    let peer_id = manager
        .peer_id()
        .ok_or_else(|| anyhow!("IPFS node started without a peer id"))?;
    let listen_addrs = manager.listen_addrs();
    let advertised = build_advertised_multiaddrs(&listen_addrs, &peer_id);
    write_bootstrap_file(&cfg.bootstrap_file, &advertised);

    tracing::info!("rendezvous peer id: {}", peer_id);
    for addr in &advertised {
        tracing::info!("rendezvous bootstrap addr: {}", addr);
    }
    if let Some(fp) = manager.swarm_key_fingerprint() {
        tracing::info!("private swarm fingerprint: {}", fp);
    }

    let state = RendezvousState {
        ipfs: Arc::clone(&manager),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api/status", status::router())
        .nest("/signal", signaling::router())
        .with_state(state)
        .layer(cors);

    let addr = format!("{}:{}", cfg.host, cfg.http_port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow!("Failed to bind {}: {}", addr, e))?;

    tracing::info!(
        "rendezvous signaling http listening on {} (libp2p on tcp/{})",
        addr,
        cfg.ipfs_listen_port
    );

    let shutdown_manager = Arc::clone(&manager);
    let serve = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("shutting down rendezvous");
        shutdown_manager.shutdown().await;
    });

    serve.await.map_err(|e| anyhow!("server error: {}", e))?;
    Ok(())
}

fn build_advertised_multiaddrs(listen_addrs: &[String], peer_id: &str) -> Vec<String> {
    listen_addrs
        .iter()
        .map(|addr| {
            if addr.contains("/p2p/") {
                addr.clone()
            } else {
                format!("{addr}/p2p/{peer_id}")
            }
        })
        .collect()
}

fn write_bootstrap_file(path: &std::path::Path, addrs: &[String]) {
    if addrs.is_empty() {
        return;
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let body = addrs.join("\n");
    if let Err(e) = std::fs::write(path, body) {
        tracing::warn!("failed to write bootstrap file at {}: {}", path.display(), e);
    }
}

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use mhaol_ipfs::{ensure_swarm_key, IpfsConfig, IpfsManager};
use mhaol_rendezvous::{
    build_router,
    config::RendezvousConfig,
    health_check, rooms::RoomManager,
    setup,
    state::RendezvousState,
};

#[derive(Parser)]
#[command(
    name = "mhaol-rendezvous",
    about = "Private-swarm IPFS bootstrap node + DHT/WebSocket WebRTC signaling"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the rendezvous server (default).
    Serve {
        /// Optional TOML config file. Env vars override file values.
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Interactive Linux deployment wizard (coturn + Let's Encrypt + systemd).
    Setup,
    /// Health-check a running rendezvous instance.
    Status {
        #[arg(short, long, default_value = "http://localhost:14080")]
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_rendezvous=debug".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Serve { config: None }) {
        Commands::Serve { config } => serve(config).await,
        Commands::Setup => setup::run_wizard().map_err(|e| anyhow!(e)),
        Commands::Status { url } => health_check::check(&url).await.map_err(|e| anyhow!(e)),
    }
}

async fn serve(config_path: Option<PathBuf>) -> Result<()> {
    let cfg = RendezvousConfig::from_env_with_file(config_path.as_deref());

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
    if cfg.turn.is_configured() {
        tracing::info!("turn configured for domain {}", cfg.turn.domain);
    }

    let state = RendezvousState {
        ipfs: Arc::clone(&manager),
        rooms: Arc::new(RoomManager::new()),
        turn: Arc::new(cfg.turn.clone()),
    };

    let app = build_router(state);

    let addr = format!("{}:{}", cfg.host, cfg.http_port);
    tracing::info!(
        "rendezvous http listening on {} (libp2p on tcp/{})",
        addr,
        cfg.ipfs_listen_port
    );

    let shutdown_manager = Arc::clone(&manager);

    match (&cfg.tls_cert, &cfg.tls_key) {
        (Some(cert), Some(key)) => {
            tracing::info!("tls enabled (cert={}, key={})", cert, key);
            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
                .await
                .map_err(|e| anyhow!("TLS config error: {}", e))?;
            let listener = std::net::TcpListener::bind(&addr)
                .map_err(|e| anyhow!("Bind error: {}", e))?;
            // axum_server doesn't expose a graceful-shutdown hook that fits
            // our need to drain the IPFS node, so we install a ctrl_c watcher
            // that tears down the IPFS node before exiting the process.
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("shutting down rendezvous");
                shutdown_manager.shutdown().await;
                std::process::exit(0);
            });
            axum_server::from_tcp_rustls(listener, tls_config)
                .serve(app.into_make_service())
                .await
                .map_err(|e| anyhow!("Server error: {}", e))?;
        }
        _ => {
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(|e| anyhow!("Failed to bind {}: {}", addr, e))?;
            let serve = axum::serve(listener, app).with_graceful_shutdown(async move {
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("shutting down rendezvous");
                shutdown_manager.shutdown().await;
            });
            serve.await.map_err(|e| anyhow!("server error: {}", e))?;
        }
    }
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

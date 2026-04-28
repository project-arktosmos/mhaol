mod cloud_status;
mod db;
mod frontend;
mod state;

use axum::Router;
use mhaol_identity::IdentityManager;
use state::CloudState;
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    #[cfg(not(target_os = "android"))]
    if std::env::args().nth(1).as_deref() == Some("worker") {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,mhaol_p2p_stream=debug".into()),
            )
            .init();
        mhaol_p2p_stream::worker::run().await;
        return;
    }

    load_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_cloud=debug,surrealdb=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1540);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_path = std::env::var("DB_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("DATA_DIR")
                .ok()
                .map(|d| PathBuf::from(d).join("cloud-surrealkv"))
        })
        .unwrap_or_else(|| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            manifest_dir.join("cloud-surrealkv")
        });

    tracing::info!("Opening SurrealDB store at {}", db_path.display());
    let surreal = db::open(&db_path)
        .await
        .expect("Failed to initialize SurrealDB");

    let identities_dir = std::env::var("DATA_DIR")
        .ok()
        .map(|d| PathBuf::from(d).join("identities"))
        .unwrap_or_else(mhaol_identity::default_identities_dir);
    let signaling_url = std::env::var("SIGNALING_URL").unwrap_or_else(|_| {
        "https://mhaol-signaling.project-arktosmos.partykit.dev".to_string()
    });
    let identity_manager =
        IdentityManager::new(identities_dir, "cloud".to_string(), signaling_url);

    identity_manager.ensure_identity("SIGNALING_WALLET");
    identity_manager.ensure_identity("CLIENT_WALLET");

    if identity_manager.get_profile("SIGNALING_WALLET").is_none() {
        identity_manager.set_profile(
            "SIGNALING_WALLET",
            &mhaol_identity::Profile {
                username: "Cloud".to_string(),
                profile_picture_url: None,
            },
        );
    }

    let state = CloudState::new(surreal, identity_manager);

    let app = Router::new()
        .nest("/api/cloud", cloud_status::router())
        .fallback(frontend::serve_frontend)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Cloud server (SurrealDB + web UI) listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}

/// Load .env from the workspace root into process environment variables.
/// Only sets variables that are not already present in the environment.
fn load_env() {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env");
        }
        if !dir.pop() {
            break PathBuf::from(".env");
        }
    };

    let content = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let value = trimmed[eq_idx + 1..].trim();
            if !key.is_empty() && std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}

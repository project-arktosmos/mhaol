use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mhaol-signaling", about = "Self-hosted signaling + TURN server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the signaling server
    Serve {
        /// Path to config file
        #[arg(short, long, default_value = "signaling.toml")]
        config: PathBuf,
    },
    /// Interactive setup wizard for Linux deployment
    Setup,
    /// Check health of signaling server and coturn
    Status {
        /// Signaling server URL to check
        #[arg(short, long, default_value = "http://localhost:8443")]
        url: String,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mhaol_signaling=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { config } => {
            let cfg = match mhaol_signaling::config::Config::load(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to load config from {}: {e}", config.display());
                    std::process::exit(1);
                }
            };
            if let Err(e) = mhaol_signaling::server::run(cfg).await {
                eprintln!("Server error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Setup => {
            if let Err(e) = mhaol_signaling::setup::run_wizard() {
                eprintln!("Setup failed: {e}");
                std::process::exit(1);
            }
        }
        Commands::Status { url } => {
            if let Err(e) = mhaol_signaling::status::check(&url).await {
                eprintln!("Health check failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

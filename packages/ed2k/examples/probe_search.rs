//! One-shot probe: connect to each DEFAULT_SERVERS entry, run a known
//! popular search, and print what comes back. Used to verify the search
//! path actually works end-to-end against live servers.
//!
//! Run with: cargo run -p mhaol-ed2k --example probe_search -- queen

use std::time::Duration;

use mhaol_ed2k::client::Ed2kClient;
use mhaol_ed2k::config::{Ed2kConfig, DEFAULT_SERVERS};

#[tokio::main]
async fn main() {
    let query = std::env::args().nth(1).unwrap_or_else(|| "queen".to_string());
    let cfg = Ed2kConfig::default();

    println!("== probe_search query={:?}", query);
    println!("== {} server(s) configured", DEFAULT_SERVERS.len());

    for s in DEFAULT_SERVERS {
        println!("\n--- {} ({}:{}) ---", s.name, s.host, s.port);
        let connect_timeout = Duration::from_secs(cfg.connect_timeout_secs);

        let mut client = match Ed2kClient::connect_and_login(
            s,
            cfg.listen_port,
            &cfg.user_name,
            connect_timeout,
        )
        .await
        {
            Ok(c) => {
                let info = c.server_info();
                println!(
                    "  login OK | users={} files={} assigned_id={:?} msg={:?}",
                    info.user_count, info.file_count, info.assigned_id, info.message
                );
                c
            }
            Err(e) => {
                println!("  login FAILED: {}", e);
                continue;
            }
        };

        match client.search(&query, Duration::from_secs(8)).await {
            Ok(results) => {
                println!("  search returned {} result(s)", results.len());
                for (i, r) in results.iter().take(5).enumerate() {
                    println!(
                        "    [{}] {} | size={} sources={} complete={} hash={}",
                        i, r.name, r.size, r.sources, r.complete_sources, r.file_hash
                    );
                }
                if results.len() > 5 {
                    println!("    ... and {} more", results.len() - 5);
                }
            }
            Err(e) => {
                println!("  search FAILED: {}", e);
            }
        }
    }
}

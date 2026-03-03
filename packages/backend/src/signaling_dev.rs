use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const DEV_PORT: u16 = 1999;

pub struct SignalingDevServer {
    available: Arc<AtomicBool>,
}

impl SignalingDevServer {
    pub fn new() -> Self {
        Self {
            available: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    pub fn dev_url(&self) -> String {
        format!("http://127.0.0.1:{}", DEV_PORT)
    }

    /// Spawn the local partykit dev server in the background.
    pub async fn start(&self) {
        if self.is_available() {
            return;
        }

        let signaling_dir = match find_signaling_dir() {
            Some(dir) => dir,
            None => {
                tracing::warn!("[signaling-dev] Could not find packages/signaling directory");
                return;
            }
        };

        if !signaling_dir.join("partykit.json").exists() {
            tracing::warn!("[signaling-dev] No partykit.json found in {:?}", signaling_dir);
            return;
        }

        tracing::info!("[signaling-dev] Starting local signaling server on port {}", DEV_PORT);

        let result = Command::new("npx")
            .arg("partykit")
            .arg("dev")
            .arg("--port")
            .arg(DEV_PORT.to_string())
            .arg("--host")
            .arg("0.0.0.0")
            .current_dir(&signaling_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        let mut child = match result {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("[signaling-dev] Failed to spawn partykit dev: {}", e);
                return;
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let available = Arc::clone(&self.available);

        // Pipe stdout in background
        if let Some(out) = stdout {
            let avail = Arc::clone(&available);
            tokio::spawn(async move {
                let mut lines = BufReader::new(out).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("[signaling-dev] {}", line);
                    if line.contains("listening") || line.contains("Ready") || line.contains("ready") {
                        avail.store(true, Ordering::SeqCst);
                    }
                }
            });
        }

        // Pipe stderr in background
        if let Some(err) = stderr {
            let avail = Arc::clone(&available);
            tokio::spawn(async move {
                let mut lines = BufReader::new(err).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("[signaling-dev] {}", line);
                    if line.contains("listening") || line.contains("Ready") || line.contains("ready") {
                        avail.store(true, Ordering::SeqCst);
                    }
                }
            });
        }

        // Wait for the child process in the background so it doesn't become a zombie
        let avail_done = Arc::clone(&available);
        tokio::spawn(async move {
            let _ = child.wait().await;
            avail_done.store(false, Ordering::SeqCst);
            tracing::info!("[signaling-dev] Local signaling server exited");
        });

        // Give it a moment to start, then mark available if port is responding
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if !self.is_available() {
            let url = self.dev_url();
            if check_port_open(&url).await {
                self.available.store(true, Ordering::SeqCst);
            }
        }

        if self.is_available() {
            tracing::info!("[signaling-dev] Local signaling server is ready at {}", self.dev_url());
        }
    }
}

fn find_signaling_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            let signaling = dir.join("packages/signaling");
            if signaling.exists() {
                return Some(signaling);
            }
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

async fn check_port_open(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    match client {
        Ok(c) => c.get(url).send().await.is_ok(),
        Err(_) => false,
    }
}

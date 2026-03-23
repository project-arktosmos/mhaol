use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use std::collections::HashMap;
use std::sync::Arc;

/// An ICE server entry passed to the p2p-stream worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServerEntry {
    pub urls: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Commands sent to the p2p-stream-worker via stdin.
#[derive(Debug, Serialize)]
#[serde(tag = "command")]
enum WorkerCommand {
    #[serde(rename = "create_session")]
    CreateSession {
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stream_url: Option<String>,
        mode: Option<String>,
        video_codec: Option<String>,
        video_quality: Option<String>,
        signaling_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ice_servers: Option<Vec<IceServerEntry>>,
    },
    #[serde(rename = "delete_session")]
    DeleteSession {
        session_id: String,
    },
}

/// Events received from the p2p-stream-worker via stdout.
#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub enum WorkerEvent {
    #[serde(rename = "session_created")]
    SessionCreated { session_id: String, room_id: String },
    #[serde(rename = "session_deleted")]
    SessionDeleted { session_id: String },
    #[serde(rename = "error")]
    Error { session_id: Option<String>, error: String },
}

type PendingRequests = Arc<Mutex<HashMap<String, oneshot::Sender<WorkerEvent>>>>;

pub struct WorkerBridge {
    stdin_tx: Mutex<Option<tokio::sync::mpsc::Sender<String>>>,
    pending: PendingRequests,
    ready: std::sync::atomic::AtomicBool,
}

impl Default for WorkerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerBridge {
    pub fn new() -> Self {
        Self {
            stdin_tx: Mutex::new(None),
            pending: Arc::new(Mutex::new(HashMap::new())),
            ready: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Spawn the p2p-stream worker subprocess and set up communication channels.
    ///
    /// The worker runs as `mhaol-server worker` — the same binary with a
    /// subcommand — so only a single binary needs to be distributed.
    pub async fn start(&self) {
        if self.is_ready() {
            return;
        }

        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("[worker-bridge] Could not determine current executable: {}", e);
                return;
            }
        };

        tracing::info!("[worker-bridge] Spawning worker: {:?} worker", exe);

        let mut child: Child = match Command::new(&exe)
            .arg("worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("[worker-bridge] Failed to spawn worker: {}", e);
                return;
            }
        };

        let child_stdin = child.stdin.take().expect("stdin must be piped");
        let child_stdout = child.stdout.take().expect("stdout must be piped");
        let child_stderr = child.stderr.take().expect("stderr must be piped");

        // Channel for sending lines to the worker's stdin
        let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);
        *self.stdin_tx.lock() = Some(tx);

        // Task: write commands to worker stdin
        tokio::spawn(async move {
            let mut writer = child_stdin;
            while let Some(line) = rx.recv().await {
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
                if writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Task: read events from worker stdout and dispatch to pending requests
        let pending = Arc::clone(&self.pending);
        tokio::spawn(async move {
            let mut lines = BufReader::new(child_stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<WorkerEvent>(trimmed) {
                    Ok(event) => {
                        let sid = match &event {
                            WorkerEvent::SessionCreated { session_id, .. } => Some(session_id.clone()),
                            WorkerEvent::SessionDeleted { session_id } => Some(session_id.clone()),
                            WorkerEvent::Error { session_id, .. } => session_id.clone(),
                        };
                        if let Some(sid) = sid {
                            if let Some(tx) = pending.lock().remove(&sid) {
                                let _ = tx.send(event);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("[worker-bridge] Failed to parse worker event: {} (line: {})", e, trimmed);
                    }
                }
            }
        });

        // Task: pipe worker stderr to tracing
        tokio::spawn(async move {
            let mut lines = BufReader::new(child_stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!("[p2p-stream-worker] {}", line);
            }
        });

        // Task: wait for child to exit
        let ready = &self.ready as *const std::sync::atomic::AtomicBool as usize;
        tokio::spawn(async move {
            let _ = child.wait().await;
            // SAFETY: AtomicBool is Send+Sync and lives as long as AppState
            unsafe {
                let ready = &*(ready as *const std::sync::atomic::AtomicBool);
                ready.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            tracing::info!("[worker-bridge] Worker process exited");
        });

        self.ready.store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("[worker-bridge] Worker is ready");
    }

    /// Send a create_session command to the worker and wait for the response.
    ///
    /// Exactly one of `file_path` or `stream_url` must be `Some`.
    pub async fn create_session(
        &self,
        session_id: &str,
        file_path: Option<&str>,
        stream_url: Option<&str>,
        signaling_url: &str,
        mode: Option<String>,
        video_codec: Option<String>,
        video_quality: Option<String>,
        ice_servers: Option<Vec<IceServerEntry>>,
    ) -> Result<WorkerEvent, String> {
        if !self.is_ready() {
            return Err("Worker is not running".to_string());
        }

        let cmd = WorkerCommand::CreateSession {
            session_id: session_id.to_string(),
            file_path: file_path.map(|s| s.to_string()),
            stream_url: stream_url.map(|s| s.to_string()),
            mode,
            video_codec,
            video_quality,
            signaling_url: signaling_url.to_string(),
            ice_servers,
        };

        self.send_command(session_id, &cmd).await
    }

    /// Send a delete_session command to the worker.
    pub async fn delete_session(&self, session_id: &str) -> Result<WorkerEvent, String> {
        if !self.is_ready() {
            return Err("Worker is not running".to_string());
        }

        let cmd = WorkerCommand::DeleteSession {
            session_id: session_id.to_string(),
        };

        self.send_command(session_id, &cmd).await
    }

    async fn send_command(&self, session_id: &str, cmd: &WorkerCommand) -> Result<WorkerEvent, String> {
        let mut json = serde_json::to_string(cmd).map_err(|e| e.to_string())?;
        json.push('\n');

        let (tx, rx) = oneshot::channel();
        self.pending.lock().insert(session_id.to_string(), tx);

        let stdin_tx = self.stdin_tx.lock().clone();
        if let Some(stdin_tx) = stdin_tx {
            stdin_tx.send(json).await.map_err(|_| "Failed to send command to worker".to_string())?;
        } else {
            self.pending.lock().remove(session_id);
            return Err("Worker stdin not available".to_string());
        }

        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(event)) => Ok(event),
            Ok(Err(_)) => Err("Worker response channel closed".to_string()),
            Err(_) => {
                self.pending.lock().remove(session_id);
                Err("Timed out waiting for worker response".to_string())
            }
        }
    }
}


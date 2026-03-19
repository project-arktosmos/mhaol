use crate::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

struct AppEntry {
    name: &'static str,
    port: u16,
    filter_name: &'static str,
    headless_script: &'static str,
}

const APPS: &[AppEntry] = &[
    AppEntry {
        name: "storybook",
        port: 1405,
        filter_name: "storybook",
        headless_script: "apps/storybook/scripts/headless.js",
    },
    AppEntry {
        name: "identity",
        port: 1410,
        filter_name: "identity",
        headless_script: "apps/identity/scripts/headless.js",
    },
    AppEntry {
        name: "signaling",
        port: 1420,
        filter_name: "signaling-app",
        headless_script: "apps/signaling/scripts/headless.js",
    },
    AppEntry {
        name: "cloud",
        port: 1510,
        filter_name: "cloud",
        headless_script: "apps/cloud/scripts/headless.js",
    },
    AppEntry {
        name: "torrent",
        port: 1520,
        filter_name: "torrent",
        headless_script: "apps/torrent/scripts/headless.js",
    },
    AppEntry {
        name: "flix",
        port: 1530,
        filter_name: "flix",
        headless_script: "apps/flix/scripts/headless.js",
    },
];

const MAX_LOG_LINES: usize = 500;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AppStatus {
    Building,
    Starting,
    Running,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    port: u16,
    has_headless: bool,
    status: AppStatus,
    frontend_built: bool,
    backend_built: bool,
}

#[derive(Serialize)]
struct HealthResult {
    name: String,
    status: AppStatus,
}

#[derive(Serialize)]
struct ActionResult {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct LogsResult {
    name: String,
    build_logs: Vec<String>,
    runtime_logs: Vec<String>,
}

struct ManagedProcess {
    child: Child,
    build_logs: std::sync::Arc<Mutex<VecDeque<String>>>,
    runtime_logs: std::sync::Arc<Mutex<VecDeque<String>>>,
    status: std::sync::Arc<Mutex<AppStatus>>,
}

pub struct HubManager {
    processes: Mutex<HashMap<String, ManagedProcess>>,
    workspace_root: PathBuf,
}

impl HubManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            workspace_root: find_workspace_root(),
        }
    }

    pub async fn shutdown(&self) {
        let mut procs = self.processes.lock().await;
        for (name, proc) in procs.iter_mut() {
            tracing::info!("Stopping app: {}", name);
            let _ = proc.child.kill().await;
        }
        procs.clear();
        // Kill all known app ports
        for entry in APPS {
            kill_port(entry.port).await;
        }
    }
}

fn find_workspace_root() -> PathBuf {
    let compile_time_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if let Ok(root) = compile_time_root.canonicalize() {
        if root.join("pnpm-workspace.yaml").exists() {
            return root;
        }
    }

    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn push_log(logs: &mut VecDeque<String>, line: String) {
    if logs.len() >= MAX_LOG_LINES {
        logs.pop_front();
    }
    logs.push_back(line);
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_apps))
        .route("/{name}/health", get(check_health))
        .route("/{name}/build", post(build_app))
        .route("/{name}/start", post(start_app))
        .route("/{name}/stop", post(stop_app))
        .route("/{name}/logs", get(get_logs))
}

async fn list_apps(State(state): State<AppState>) -> Json<Vec<AppInfo>> {
    let procs = state.hub.processes.lock().await;
    let root = &state.hub.workspace_root;
    let backend_built = root.join("target/debug/mhaol-server").exists()
        || root.join("target/release/mhaol-server").exists();
    let client = reqwest::Client::new();

    let mut apps = Vec::new();
    for entry in APPS {
        let status = match procs.get(entry.name) {
            Some(proc) => proc.status.lock().await.clone(),
            None => {
                // No tracked process — check if something is listening on the port
                match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", entry.port)).await {
                    Ok(_) => AppStatus::Running,
                    Err(_) => AppStatus::Stopped,
                }
            }
        };
        let frontend_dist = root.join(format!("apps/{}/dist-static/index.html", entry.name));
        apps.push(AppInfo {
            name: entry.name.to_string(),
            port: entry.port,
            has_headless: true,
            status,
            frontend_built: frontend_dist.exists(),
            backend_built,
        });
    }
    Json(apps)
}

async fn check_health(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<HealthResult> {
    let entry = APPS.iter().find(|e| e.name == name);

    // If we have a tracked status, use it for building/starting/failed
    let procs = state.hub.processes.lock().await;
    if let Some(proc) = procs.get(name.as_str()) {
        let tracked = proc.status.lock().await.clone();
        match tracked {
            AppStatus::Building | AppStatus::Starting | AppStatus::Failed => {
                return Json(HealthResult {
                    name,
                    status: tracked,
                });
            }
            _ => {}
        }
    }
    drop(procs);

    let status = match entry {
        Some(app) => {
            match tokio::net::TcpStream::connect(format!("127.0.0.1:{}", app.port)).await {
                Ok(_) => AppStatus::Running,
                Err(_) => AppStatus::Stopped,
            }
        }
        None => AppStatus::Unknown,
    };

    // Update tracked status if we have a process
    let procs = state.hub.processes.lock().await;
    if let Some(proc) = procs.get(name.as_str()) {
        *proc.status.lock().await = status.clone();
    }

    Json(HealthResult { name, status })
}

async fn build_app(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<ActionResult> {
    let entry = match APPS.iter().find(|e| e.name == name) {
        Some(e) => e,
        None => {
            return Json(ActionResult {
                success: false,
                message: format!("Unknown app: {}", name),
            })
        }
    };

    // Don't build if already building or running
    {
        let procs = state.hub.processes.lock().await;
        if let Some(proc) = procs.get(entry.name) {
            let s = proc.status.lock().await.clone();
            if s == AppStatus::Building || s == AppStatus::Starting || s == AppStatus::Running {
                return Json(ActionResult {
                    success: false,
                    message: format!("{} is already {}", name, match s {
                        AppStatus::Building => "building",
                        AppStatus::Starting => "starting",
                        AppStatus::Running => "running",
                        _ => "busy",
                    }),
                });
            }
        }
    }

    let build_logs = std::sync::Arc::new(Mutex::new(VecDeque::<String>::with_capacity(MAX_LOG_LINES)));
    let runtime_logs = std::sync::Arc::new(Mutex::new(VecDeque::<String>::with_capacity(MAX_LOG_LINES)));
    let status = std::sync::Arc::new(Mutex::new(AppStatus::Building));

    let placeholder = Command::new("sleep").arg("999999")
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn placeholder");

    {
        let mut procs = state.hub.processes.lock().await;
        // Remove any old failed entry
        if let Some(mut old) = procs.remove(entry.name) {
            let _ = old.child.kill().await;
        }
        procs.insert(
            entry.name.to_string(),
            ManagedProcess {
                child: placeholder,
                build_logs: std::sync::Arc::clone(&build_logs),
                runtime_logs: std::sync::Arc::clone(&runtime_logs),
                status: std::sync::Arc::clone(&status),
            },
        );
    }

    let workspace = state.hub.workspace_root.clone();
    let filter = entry.filter_name.to_string();
    let app_name = name.clone();
    let hub = std::sync::Arc::clone(&state.hub);
    let entry_name = entry.name.to_string();

    tokio::spawn(async move {
        {
            let msg = format!("Building {} (pnpm --filter {} build)...", app_name, filter);
            tracing::info!("[{}] {}", app_name, msg);
            let mut buf = build_logs.lock().await;
            push_log(&mut buf, msg);
        }

        let result = Command::new("pnpm")
            .args(["--filter", &filter, "build"])
            .current_dir(&workspace)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                {
                    let mut buf = build_logs.lock().await;
                    for line in String::from_utf8_lossy(&output.stdout).lines() {
                        println!("[{}] {}", app_name, line);
                        push_log(&mut buf, line.to_string());
                    }
                    for line in String::from_utf8_lossy(&output.stderr).lines() {
                        eprintln!("[{}] {}", app_name, line);
                        push_log(&mut buf, line.to_string());
                    }
                }
                if output.status.success() {
                    let msg = format!("{} built successfully", app_name);
                    let mut buf = build_logs.lock().await;
                    push_log(&mut buf, msg);
                    *status.lock().await = AppStatus::Stopped;
                } else {
                    let msg = format!("Build failed (exit {})", output.status);
                    let mut buf = build_logs.lock().await;
                    push_log(&mut buf, msg);
                    *status.lock().await = AppStatus::Failed;
                }
            }
            Err(e) => {
                let msg = format!("Failed to run build: {}", e);
                let mut buf = build_logs.lock().await;
                push_log(&mut buf, msg);
                *status.lock().await = AppStatus::Failed;
            }
        }

        // Kill the placeholder
        let mut procs = hub.processes.lock().await;
        if let Some(proc) = procs.get_mut(&entry_name) {
            let _ = proc.child.kill().await;
        }
    });

    Json(ActionResult {
        success: true,
        message: format!("{} build started", name),
    })
}

async fn start_app(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<ActionResult> {
    let entry = match APPS.iter().find(|e| e.name == name) {
        Some(e) => e,
        None => {
            return Json(ActionResult {
                success: false,
                message: format!("Unknown app: {}", name),
            })
        }
    };

    {
        let mut procs = state.hub.processes.lock().await;
        if let Some(proc) = procs.get(entry.name) {
            let s = proc.status.lock().await.clone();
            match s {
                AppStatus::Building | AppStatus::Starting | AppStatus::Running => {
                    return Json(ActionResult {
                        success: false,
                        message: format!("{} is already {}", name, match s {
                            AppStatus::Building => "building",
                            AppStatus::Starting => "starting",
                            AppStatus::Running => "running",
                            _ => "busy",
                        }),
                    });
                }
                _ => {
                    // Remove stale entry so we can start fresh
                    if let Some(mut old) = procs.remove(entry.name) {
                        let _ = old.child.kill().await;
                    }
                }
            }
        }
    }

    let script_path = state.hub.workspace_root.join(entry.headless_script);
    if !script_path.exists() {
        return Json(ActionResult {
            success: false,
            message: format!("Headless script not found: {}", script_path.display()),
        });
    }

    let build_logs = std::sync::Arc::new(Mutex::new(VecDeque::<String>::with_capacity(MAX_LOG_LINES)));
    let runtime_logs = std::sync::Arc::new(Mutex::new(VecDeque::<String>::with_capacity(MAX_LOG_LINES)));
    let status = std::sync::Arc::new(Mutex::new(AppStatus::Starting));

    // Insert a placeholder immediately so status/logs are visible
    let placeholder = Command::new("sleep").arg("999999")
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn placeholder");

    {
        let mut procs = state.hub.processes.lock().await;
        procs.insert(
            entry.name.to_string(),
            ManagedProcess {
                child: placeholder,
                build_logs: std::sync::Arc::clone(&build_logs),
                runtime_logs: std::sync::Arc::clone(&runtime_logs),
                status: std::sync::Arc::clone(&status),
            },
        );
    }

    // Spawn the headless script in the background.
    // We run from the script's directory so its relative `cwd: '../..'` resolves correctly.
    // Pass --skip-build if the app is already built to avoid redundant rebuilds.
    let script_dir = script_path.parent().unwrap().to_path_buf();
    let app_name = name.clone();
    let entry_name = entry.name.to_string();
    let app_port = entry.port;
    let hub = std::sync::Arc::clone(&state.hub);

    let root = &state.hub.workspace_root;
    let frontend_built = root.join(format!("apps/{}/dist-static/index.html", entry.name)).exists();
    let backend_built = root.join("target/debug/mhaol-server").exists()
        || root.join("target/release/mhaol-server").exists();
    let skip_build = frontend_built && backend_built;

    tokio::spawn(async move {
        {
            let msg = if skip_build {
                format!("Starting {} (builds cached)...", app_name)
            } else {
                format!("Starting {} (will build first)...", app_name)
            };
            tracing::info!("[{}] {}", app_name, msg);
            let mut buf = build_logs.lock().await;
            push_log(&mut buf, msg);
        }

        let mut cmd = Command::new("node");
        cmd.arg(&script_path);
        if skip_build {
            cmd.arg("--skip-build");
        }
        cmd.current_dir(&script_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Create a new process group so we can kill the entire tree
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }

        match cmd.spawn()
        {
            Ok(mut child) => {
                let ready_pattern = format!("http://localhost:{}", app_port);
                let health_url = format!("http://localhost:{}/api/health", app_port);

                if let Some(stdout) = child.stdout.take() {
                    let build_clone = std::sync::Arc::clone(&build_logs);
                    let runtime_clone = std::sync::Arc::clone(&runtime_logs);
                    let status_clone = std::sync::Arc::clone(&status);
                    let name_clone = app_name.clone();
                    let pattern = ready_pattern.clone();
                    let url = health_url.clone();
                    tokio::spawn(async move {
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();
                        let mut is_running = false;
                        while let Ok(Some(line)) = lines.next_line().await {
                            println!("[{}] {}", name_clone, line);
                            if !is_running && line.contains(&pattern) {
                                if verify_health(&url).await {
                                    *status_clone.lock().await = AppStatus::Running;
                                    is_running = true;
                                }
                            }
                            if is_running {
                                let mut buf = runtime_clone.lock().await;
                                push_log(&mut buf, line);
                            } else {
                                let mut buf = build_clone.lock().await;
                                push_log(&mut buf, line);
                            }
                        }
                    });
                }

                if let Some(stderr) = child.stderr.take() {
                    let build_clone = std::sync::Arc::clone(&build_logs);
                    let runtime_clone = std::sync::Arc::clone(&runtime_logs);
                    let name_clone = app_name.clone();
                    let status_clone = std::sync::Arc::clone(&status);
                    let pattern = ready_pattern;
                    let url = health_url;
                    tokio::spawn(async move {
                        let reader = BufReader::new(stderr);
                        let mut lines = reader.lines();
                        let mut is_running = false;
                        while let Ok(Some(line)) = lines.next_line().await {
                            eprintln!("[{}] {}", name_clone, line);
                            if !is_running && line.contains(&pattern) {
                                if verify_health(&url).await {
                                    *status_clone.lock().await = AppStatus::Running;
                                    is_running = true;
                                }
                            }
                            if is_running {
                                let mut buf = runtime_clone.lock().await;
                                push_log(&mut buf, line);
                            } else {
                                let mut buf = build_clone.lock().await;
                                push_log(&mut buf, line);
                            }
                        }
                        let mut s = status_clone.lock().await;
                        if *s == AppStatus::Starting {
                            *s = AppStatus::Failed;
                        }
                    });
                }

                // Replace the placeholder with the real child process
                let mut procs = hub.processes.lock().await;
                if let Some(proc) = procs.get_mut(&entry_name) {
                    let _ = proc.child.kill().await;
                    proc.child = child;
                }
            }
            Err(e) => {
                let msg = format!("Failed to start {}: {}", app_name, e);
                let mut buf = build_logs.lock().await;
                push_log(&mut buf, msg);
                *status.lock().await = AppStatus::Failed;
            }
        }
    });

    Json(ActionResult {
        success: true,
        message: format!("{} build started", name),
    })
}

async fn stop_app(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<ActionResult> {
    let entry = match APPS.iter().find(|e| e.name == name) {
        Some(e) => e,
        None => {
            return Json(ActionResult {
                success: false,
                message: format!("Unknown app: {}", name),
            })
        }
    };

    // Remove tracked process if any
    {
        let mut procs = state.hub.processes.lock().await;
        if let Some(mut proc) = procs.remove(name.as_str()) {
            let _ = proc.child.kill().await;
        }
    }

    // Kill whatever is actually listening on the app's port
    let killed = kill_port(entry.port).await;

    if killed {
        Json(ActionResult {
            success: true,
            message: format!("{} stopped (port {})", name, entry.port),
        })
    } else {
        Json(ActionResult {
            success: true,
            message: format!("{} stopped", name),
        })
    }
}

/// Confirm an app is actually responding before marking it as running.
async fn verify_health(url: &str) -> bool {
    let client = reqwest::Client::new();
    for _ in 0..5 {
        match client
            .get(url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => return true,
            _ => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
    false
}

/// Find and kill all processes listening on a given port.
async fn kill_port(port: u16) -> bool {
    #[cfg(unix)]
    {
        // lsof to find PIDs listening on this port
        let output = Command::new("lsof")
            .args(["-ti", &format!("tcp:{}", port)])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .await;

        if let Ok(output) = output {
            let pids_str = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<i32> = pids_str
                .split_whitespace()
                .filter_map(|s| s.parse::<i32>().ok())
                .collect();

            if pids.is_empty() {
                return false;
            }

            for pid in &pids {
                tracing::info!("Killing PID {} on port {}", pid, port);
                unsafe { libc::kill(*pid, libc::SIGTERM); }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Force kill any survivors
            for pid in &pids {
                unsafe { libc::kill(*pid, libc::SIGKILL); }
            }

            return true;
        }
        false
    }

    #[cfg(windows)]
    {
        // netstat + taskkill on Windows
        let output = Command::new("cmd")
            .args(["/C", &format!("for /f \"tokens=5\" %a in ('netstat -aon ^| findstr :{} ^| findstr LISTENING') do taskkill /PID %a /F", port)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .await;
        output.is_ok()
    }
}

async fn get_logs(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<LogsResult> {
    let procs = state.hub.processes.lock().await;
    let (build_logs, runtime_logs) = match procs.get(name.as_str()) {
        Some(proc) => {
            let build = proc.build_logs.lock().await.iter().cloned().collect();
            let runtime = proc.runtime_logs.lock().await.iter().cloned().collect();
            (build, runtime)
        }
        None => (vec![], vec![]),
    };
    Json(LogsResult { name, build_logs, runtime_logs })
}

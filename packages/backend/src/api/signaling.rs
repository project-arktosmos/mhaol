use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{get, head, post},
    Json, Router,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

static DEPLOY_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/servers", get(list_servers).post(create_server))
        .route(
            "/servers/{id}",
            get(get_server).put(update_server).delete(delete_server),
        )
        .route("/servers/{id}/check", get(check_server))
        .route("/wallet", get(get_wallet).delete(regenerate_wallet))
        .route("/wallet/sign", post(sign_message))
        .route("/deploy", head(deploy_status).get(deploy))
}

/// GET /api/signaling/status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let identity_address = state.identity_manager.get_default_address();

    let dev_available = state.signaling_rooms.is_available();
    let dev_url = if dev_available {
        state.signaling_rooms.dev_url()
    } else {
        String::new()
    };

    let servers = state.signaling_servers.get_all();

    // Check availability of each enabled server concurrently
    let checks: Vec<_> = servers
        .iter()
        .map(|s| {
            let url = s.url.clone();
            let enabled = s.enabled;
            async move {
                if enabled {
                    check_url_available(&url).await
                } else {
                    false
                }
            }
        })
        .collect();
    let results = futures_util::future::join_all(checks).await;

    let server_statuses: Vec<_> = servers
        .iter()
        .zip(results)
        .map(|(server, available)| {
            serde_json::json!({
                "id": server.id,
                "name": server.name,
                "url": server.url,
                "enabled": server.enabled,
                "available": available,
                "created_at": server.created_at,
                "updated_at": server.updated_at,
            })
        })
        .collect();

    Json(serde_json::json!({
        "devAvailable": dev_available,
        "devUrl": dev_url,
        "identityAddress": identity_address,
        "servers": server_statuses,
    }))
}

/// GET /api/signaling/servers
async fn list_servers(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.signaling_servers.get_all())
}

/// GET /api/signaling/servers/{id}
async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.signaling_servers.get(&id) {
        Some(server) => Json(serde_json::json!(server)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Server not found"})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct CreateServerBody {
    name: String,
    url: String,
}

/// POST /api/signaling/servers
async fn create_server(
    State(state): State<AppState>,
    Json(body): Json<CreateServerBody>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    state
        .signaling_servers
        .insert(&id, &body.name, &body.url, true);
    (StatusCode::CREATED, Json(state.signaling_servers.get(&id))).into_response()
}

#[derive(Deserialize)]
struct UpdateServerBody {
    name: String,
    url: String,
    enabled: bool,
}

/// PUT /api/signaling/servers/{id}
async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateServerBody>,
) -> impl IntoResponse {
    if state.signaling_servers.get(&id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Server not found"})),
        )
            .into_response();
    }
    state
        .signaling_servers
        .update(&id, &body.name, &body.url, body.enabled);
    Json(state.signaling_servers.get(&id)).into_response()
}

/// DELETE /api/signaling/servers/{id}
async fn delete_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.signaling_servers.delete(&id);
    StatusCode::NO_CONTENT
}

/// GET /api/signaling/servers/{id}/check
async fn check_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.signaling_servers.get(&id) {
        Some(server) => {
            let available = check_url_available(&server.url).await;
            Json(serde_json::json!({"id": id, "available": available})).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Server not found"})),
        )
            .into_response(),
    }
}

/// GET /api/signaling/wallet
async fn get_wallet(State(state): State<AppState>) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let address = state.identity_manager.ensure_identity(name);
    Json(serde_json::json!({
        "name": name,
        "address": address,
    }))
}

/// DELETE /api/signaling/wallet
async fn regenerate_wallet(State(state): State<AppState>) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let address = state.identity_manager.regenerate(name);
    Json(serde_json::json!({
        "name": name,
        "address": address,
    }))
}

#[derive(Deserialize)]
struct SignMessageBody {
    message: String,
}

/// POST /api/signaling/wallet/sign
async fn sign_message(
    State(state): State<AppState>,
    Json(body): Json<SignMessageBody>,
) -> impl IntoResponse {
    let name = "SIGNALING_WALLET";
    let pk = match state.identity_manager.get_private_key(name) {
        Some(pk) => pk,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Wallet not found" })),
            )
                .into_response()
        }
    };

    let signature = mhaol_identity::eip191_sign(&body.message, &pk);
    Json(serde_json::json!({ "signature": signature })).into_response()
}

/// HEAD /api/signaling/deploy — check if a deploy is in progress
async fn deploy_status() -> impl IntoResponse {
    if DEPLOY_IN_PROGRESS.load(Ordering::SeqCst) {
        StatusCode::CONFLICT
    } else {
        StatusCode::NO_CONTENT
    }
}

/// Find the repo root by walking up from cwd looking for pnpm-workspace.yaml
fn find_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// GET /api/signaling/deploy — spawn `npx partykit deploy` and stream SSE
async fn deploy(State(state): State<AppState>) -> impl IntoResponse {
    if DEPLOY_IN_PROGRESS.swap(true, Ordering::SeqCst) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "A deploy is already in progress" })),
        )
            .into_response();
    }

    let signaling_dir = match find_repo_root() {
        Some(root) => root.join("packages/signaling"),
        None => {
            DEPLOY_IN_PROGRESS.store(false, Ordering::SeqCst);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Could not find repository root" })),
            )
                .into_response();
        }
    };

    if !signaling_dir.join("partykit.json").exists() {
        DEPLOY_IN_PROGRESS.store(false, Ordering::SeqCst);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Signaling package not found" })),
        )
            .into_response();
    }

    // Build deploy name from identity address
    let address = state
        .identity_manager
        .get_default_address()
        .unwrap_or_default();
    let short_addr = if address.starts_with("0x") && address.len() >= 10 {
        &address[2..10]
    } else {
        "unknown"
    };
    let deploy_name = format!("{}-mhaol-signaling", short_addr);

    let signaling_servers = state.signaling_servers.clone();
    let deploy_name_clone = deploy_name.clone();

    let stream = async_stream::stream! {
        let result = Command::new("npx")
            .arg("partykit")
            .arg("deploy")
            .arg("--name")
            .arg(&deploy_name_clone)
            .current_dir(&signaling_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(e) => {
                let msg = format!("Failed to spawn partykit deploy: {}", e);
                yield Ok::<_, Infallible>(Event::default().event("error").data(
                    serde_json::json!({ "message": msg }).to_string()
                ));
                DEPLOY_IN_PROGRESS.store(false, Ordering::SeqCst);
                return;
            }
        };

        let mut stdout_reader = child.stdout.take().map(|out| BufReader::new(out).lines());
        let mut stderr_reader = child.stderr.take().map(|err| BufReader::new(err).lines());

        let mut all_output = String::new();
        let mut stdout_done = stdout_reader.is_none();
        let mut stderr_done = stderr_reader.is_none();

        // Read stdout and stderr concurrently
        while !stdout_done || !stderr_done {
            tokio::select! {
                result = async { stdout_reader.as_mut().unwrap().next_line().await }, if !stdout_done => {
                    match result {
                        Ok(Some(line)) => {
                            all_output.push_str(&line);
                            all_output.push('\n');
                            yield Ok(Event::default().event("log").data(
                                serde_json::json!({ "text": line }).to_string()
                            ));
                        }
                        _ => { stdout_done = true; }
                    }
                }
                result = async { stderr_reader.as_mut().unwrap().next_line().await }, if !stderr_done => {
                    match result {
                        Ok(Some(line)) => {
                            all_output.push_str(&line);
                            all_output.push('\n');
                            yield Ok(Event::default().event("log").data(
                                serde_json::json!({ "text": line }).to_string()
                            ));
                        }
                        _ => { stderr_done = true; }
                    }
                }
            }
        }

        let status = child.wait().await;
        let exit_code = status.as_ref().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
        let success = status.map(|s| s.success()).unwrap_or(false);

        // Extract deployed URL from output
        let url_re_match = all_output
            .lines()
            .find_map(|line| {
                line.split_whitespace()
                    .find(|word| word.contains("partykit.dev"))
                    .and_then(|word| {
                        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '/' && c != '.' && c != '-');
                        if trimmed.starts_with("https://") {
                            Some(trimmed.to_string())
                        } else {
                            None
                        }
                    })
            });

        if success {
            if let Some(ref url) = url_re_match {
                // Upsert into signaling_servers table
                let existing = signaling_servers.get_all().into_iter().find(|s| s.url == *url);
                match existing {
                    Some(s) => signaling_servers.update(&s.id, &deploy_name_clone, url, true),
                    None => {
                        let id = uuid::Uuid::new_v4().to_string();
                        signaling_servers.insert(&id, &deploy_name_clone, url, true);
                    }
                }
            }
        }

        yield Ok(Event::default().event("done").data(
            serde_json::json!({
                "success": success,
                "code": exit_code,
                "url": url_re_match,
            }).to_string()
        ));

        DEPLOY_IN_PROGRESS.store(false, Ordering::SeqCst);
    };

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

async fn check_url_available(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    match client {
        Ok(c) => c
            .get(url)
            .send()
            .await
            .map(|r| {
                let s = r.status();
                // Any HTTP response means the server is reachable.
                // PartyKit returns 405 for non-WebSocket requests, which is fine.
                !s.is_server_error()
            })
            .unwrap_or(false),
        Err(_) => false,
    }
}

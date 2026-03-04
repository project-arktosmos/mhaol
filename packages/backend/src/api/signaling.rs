use crate::AppState;
use axum::{
    extract::State,
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
        .route("/wallet", get(get_wallet).delete(regenerate_wallet))
        .route("/wallet/sign", post(sign_message))
        .route("/deploy", head(deploy_status).get(deploy))
}

/// GET /api/signaling/status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let identity_address = state.identity_manager.get_default_address();
    let party_url = state
        .settings
        .get("signaling.partyUrl")
        .unwrap_or_default();
    let deploy_name = state
        .settings
        .get("signaling.deployName")
        .unwrap_or_default();

    // Check if the deployed URL is reachable
    let deployed_available = if !party_url.is_empty() {
        check_url_available(&party_url).await
    } else {
        false
    };

    let dev_available = state.signaling_rooms.is_available();
    let dev_url = if dev_available {
        state.signaling_rooms.dev_url()
    } else {
        String::new()
    };

    Json(serde_json::json!({
        "devAvailable": dev_available,
        "deployedAvailable": deployed_available,
        "devUrl": dev_url,
        "partyUrl": party_url,
        "deployName": deploy_name,
        "identityAddress": identity_address,
    }))
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

    let signature = crate::identity::passport::eip191_sign(&body.message, &pk);
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
async fn deploy(
    State(state): State<AppState>,
) -> impl IntoResponse {
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
    let address = state.identity_manager.get_default_address().unwrap_or_default();
    let short_addr = if address.starts_with("0x") && address.len() >= 10 {
        &address[2..10]
    } else {
        "unknown"
    };
    let deploy_name = format!("{}-mhaol-signaling", short_addr);

    // Save deploy name to settings
    state.settings.set("signaling.deployName", &deploy_name);

    let settings = state.settings.clone();
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

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let mut all_output = String::new();

        // Read stdout
        if let Some(out) = stdout {
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                all_output.push_str(&line);
                all_output.push('\n');
                yield Ok(Event::default().event("log").data(
                    serde_json::json!({ "text": line }).to_string()
                ));
            }
        }

        // Read stderr
        if let Some(err) = stderr {
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                all_output.push_str(&line);
                all_output.push('\n');
                yield Ok(Event::default().event("log").data(
                    serde_json::json!({ "text": line }).to_string()
                ));
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
                settings.set("signaling.partyUrl", url);
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

    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}

async fn check_url_available(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    match client {
        Ok(c) => c.head(url).send().await.map(|r| r.status().is_success()).unwrap_or(false),
        Err(_) => false,
    }
}

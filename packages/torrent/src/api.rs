use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{AddTorrentRequest, TorrentManager, TorrentStats};

type AppState = Arc<TorrentManager>;

/// Build a router with the WebSocket endpoint.
pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_upgrade))
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(mgr): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, mgr))
}

async fn handle_ws(socket: WebSocket, mgr: AppState) {
    let (sink, mut stream) = socket.split();
    let sender = Arc::new(Mutex::new(sink));

    // Spawn periodic push task (torrents + stats every 1s)
    let push_sender = Arc::clone(&sender);
    let push_mgr = Arc::clone(&mgr);
    let push_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;

            let torrents = push_mgr.list().await.unwrap_or_default();
            let stats = push_mgr.stats().await.ok();

            let mut tx = push_sender.lock().await;

            if let Ok(data) = serde_json::to_string(&ServerMsg::Torrents { torrents }) {
                if tx.send(Message::Text(data.into())).await.is_err() {
                    break;
                }
            }

            if let Some(stats) = stats {
                if let Ok(data) = serde_json::to_string(&ServerMsg::Stats { stats }) {
                    if tx.send(Message::Text(data.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Read client messages
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => {
                let response = handle_message(&text, &mgr).await;
                if let Some(resp) = response {
                    if let Ok(data) = serde_json::to_string(&resp) {
                        let mut tx = sender.lock().await;
                        if tx.send(Message::Text(data.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    push_task.abort();
}

async fn handle_message(text: &str, mgr: &AppState) -> Option<ServerMsg> {
    let msg: ClientMsg = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => return Some(ServerMsg::Error { error: format!("Invalid message: {}", e) }),
    };

    match msg {
        ClientMsg::GetStatus => {
            let stats = mgr.stats().await.ok();
            Some(ServerMsg::Status {
                initialized: mgr.is_initialized(),
                download_path: mgr.download_path().to_string_lossy().to_string(),
                stats,
            })
        }

        ClientMsg::GetConfig => Some(ServerMsg::Config {
            download_path: mgr.download_path().to_string_lossy().to_string(),
        }),

        ClientMsg::SetConfig { download_path } => {
            if let Some(path) = download_path {
                mgr.set_download_path(std::path::PathBuf::from(&path));
            }
            Some(ServerMsg::Config {
                download_path: mgr.download_path().to_string_lossy().to_string(),
            })
        }

        ClientMsg::AddTorrent { source, download_path } => {
            let req = AddTorrentRequest {
                source,
                download_path,
                paused: None,
            };
            match mgr.add(req).await {
                Ok(info) => Some(ServerMsg::TorrentAdded {
                    torrent: serde_json::to_value(info).unwrap_or_default(),
                }),
                Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
            }
        }

        ClientMsg::PauseTorrent { id } => match mgr.pause(id).await {
            Ok(()) => Some(ServerMsg::Ok),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::ResumeTorrent { id } => match mgr.resume(id).await {
            Ok(()) => Some(ServerMsg::Ok),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::RemoveTorrent { id } => match mgr.remove(id).await {
            Ok(()) => Some(ServerMsg::Ok),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::RemoveAll => match mgr.remove_all().await {
            Ok(count) => Some(ServerMsg::Removed { count: count as usize }),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::ClearStorage => match mgr.clear_storage().await {
            Ok(()) => Some(ServerMsg::Ok),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::GetDebug => match mgr.debug_info().await {
            Ok(logs) => Some(ServerMsg::Debug { logs }),
            Err(e) => Some(ServerMsg::Error { error: e.to_string() }),
        },

        ClientMsg::Search { query, category } => {
            Some(search_torrents(&query, category.as_deref().unwrap_or("0")).await)
        }
    }
}

// ── Client → Server messages ────────────────────────────────────────

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "getStatus")]
    GetStatus,
    #[serde(rename = "getConfig")]
    GetConfig,
    #[serde(rename = "setConfig")]
    SetConfig {
        #[serde(rename = "downloadPath")]
        download_path: Option<String>,
    },
    #[serde(rename = "addTorrent")]
    AddTorrent {
        source: String,
        #[serde(rename = "downloadPath")]
        download_path: Option<String>,
    },
    #[serde(rename = "pauseTorrent")]
    PauseTorrent { id: usize },
    #[serde(rename = "resumeTorrent")]
    ResumeTorrent { id: usize },
    #[serde(rename = "removeTorrent")]
    RemoveTorrent { id: usize },
    #[serde(rename = "removeAll")]
    RemoveAll,
    #[serde(rename = "clearStorage")]
    ClearStorage,
    #[serde(rename = "getDebug")]
    GetDebug,
    #[serde(rename = "search")]
    Search {
        query: String,
        category: Option<String>,
    },
}

// ── Server → Client messages ────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMsg {
    #[serde(rename = "status")]
    Status {
        initialized: bool,
        #[serde(rename = "downloadPath")]
        download_path: String,
        stats: Option<TorrentStats>,
    },
    #[serde(rename = "config")]
    Config {
        #[serde(rename = "downloadPath")]
        download_path: String,
    },
    #[serde(rename = "torrents")]
    Torrents {
        torrents: Vec<crate::TorrentInfo>,
    },
    #[serde(rename = "stats")]
    Stats {
        stats: TorrentStats,
    },
    #[serde(rename = "torrentAdded")]
    TorrentAdded {
        torrent: serde_json::Value,
    },
    #[serde(rename = "searchResults")]
    SearchResults {
        results: Vec<SearchResult>,
    },
    #[serde(rename = "debug")]
    Debug {
        logs: Vec<String>,
    },
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "removed")]
    Removed {
        count: usize,
    },
    #[serde(rename = "error")]
    Error {
        error: String,
    },
}

// ── Torrent Search (PirateBay API proxy) ────────────────────────────

const SEARCH_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://opentracker.i2p.rocks:6969/announce",
];

#[derive(Serialize)]
struct SearchResult {
    id: String,
    name: String,
    #[serde(rename = "infoHash")]
    info_hash: String,
    seeders: i64,
    leechers: i64,
    size: i64,
    #[serde(rename = "fileCount")]
    file_count: i64,
    #[serde(rename = "uploadedBy")]
    uploaded_by: String,
    #[serde(rename = "uploadedAt")]
    uploaded_at: i64,
    category: String,
    #[serde(rename = "magnetLink")]
    magnet_uri: String,
}

async fn search_torrents(query: &str, cat: &str) -> ServerMsg {
    let url = format!(
        "https://apibay.org/q.php?q={}&cat={}",
        urlencoding::encode(query),
        cat
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(results) => {
                    if results.len() == 1 {
                        if let Some(name) = results[0]["name"].as_str() {
                            if name == "No results returned" {
                                return ServerMsg::SearchResults { results: vec![] };
                            }
                        }
                    }

                    let search_results: Vec<SearchResult> = results
                        .iter()
                        .filter_map(|r| {
                            let info_hash = r["info_hash"].as_str()?.to_string();
                            let name = r["name"].as_str()?.to_string();
                            let trackers: String = SEARCH_TRACKERS
                                .iter()
                                .map(|t| format!("&tr={}", urlencoding::encode(t)))
                                .collect();
                            let magnet_uri = format!(
                                "magnet:?xt=urn:btih:{}&dn={}{}",
                                info_hash,
                                urlencoding::encode(&name),
                                trackers
                            );
                            Some(SearchResult {
                                id: r["id"].as_str().unwrap_or("0").to_string(),
                                name,
                                info_hash,
                                seeders: r["seeders"].as_str()?.parse().ok()?,
                                leechers: r["leechers"].as_str()?.parse().ok()?,
                                size: r["size"].as_str()?.parse().ok()?,
                                file_count: r["num_files"].as_str()?.parse().unwrap_or(0),
                                uploaded_by: r["username"].as_str().unwrap_or("").to_string(),
                                uploaded_at: r["added"].as_str()?.parse().ok()?,
                                category: r["category"].as_str().unwrap_or("0").to_string(),
                                magnet_uri,
                            })
                        })
                        .collect();

                    ServerMsg::SearchResults { results: search_results }
                }
                Err(e) => ServerMsg::Error { error: e.to_string() },
            }
        }
        _ => ServerMsg::Error {
            error: "PirateBay API unavailable".to_string(),
        },
    }
}

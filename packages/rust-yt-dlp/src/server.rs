use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json,
    },
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use mhaol_yt_dlp::{
    DownloadManager, QueueDownloadRequest, QueuePlaylistRequest, YtDlpStatus, YtDownloadConfig,
};

type AppState = Arc<DownloadManager>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = std::env::var("YTDL_PORT").unwrap_or_else(|_| "3040".to_string());
    let cors_origin =
        std::env::var("YTDL_CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:1530".to_string());

    let config = YtDownloadConfig::from_env();
    log::info!("Output directory: {}", config.output_path);

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&config.output_path) {
        log::error!("Failed to create output directory: {}", e);
    }

    let manager = Arc::new(DownloadManager::new(config));

    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:1530")),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        // Status
        .route("/api/status", get(get_status))
        // Config
        .route("/api/config", get(get_config))
        .route("/api/config", put(update_config))
        // yt-dlp status (native)
        .route("/api/ytdlp/status", get(get_ytdlp_status))
        // Video/Playlist info
        .route("/api/info/video", get(get_video_info))
        .route("/api/info/playlist", get(get_playlist_info))
        // Downloads CRUD
        .route("/api/downloads", get(list_downloads))
        .route("/api/downloads", post(queue_download))
        .route("/api/downloads/playlist", post(queue_playlist))
        .route("/api/downloads/events", get(download_events))
        .route("/api/downloads/queue", delete(clear_queue))
        .route("/api/downloads/completed", delete(clear_completed))
        .route("/api/downloads/{id}", get(get_download))
        .route("/api/downloads/{id}", delete(cancel_download))
        .layer(cors)
        .with_state(manager);

    let bind_addr = format!("0.0.0.0:{}", port);
    log::info!("Starting YouTube download server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to bind to {}: {}", bind_addr, e);
            std::process::exit(1);
        });

    println!("YouTube download server listening on {}", bind_addr);

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        log::error!("Server error: {}", e);
    });
}

// ── Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> impl IntoResponse {
    (status, Json(ErrorResponse { error: msg.into() }))
}

#[derive(Deserialize)]
struct UrlQuery {
    url: String,
}

// ── Handlers ────────────────────────────────────────────────────────

async fn get_status(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_stats())
}

async fn get_config(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_config())
}

async fn update_config(
    State(mgr): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    mgr.update_config(body);
    Json(mgr.get_config())
}

async fn get_ytdlp_status(State(_mgr): State<AppState>) -> impl IntoResponse {
    Json(YtDlpStatus {
        available: true,
        version: Some(format!("native-rust-{}", env!("CARGO_PKG_VERSION"))),
        downloading: false,
    })
}

async fn get_video_info(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.fetch_video_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn get_playlist_info(
    State(mgr): State<AppState>,
    Query(query): Query<UrlQuery>,
) -> impl IntoResponse {
    match mgr.fetch_playlist_info(&query.url).await {
        Ok(info) => (StatusCode::OK, Json(serde_json::to_value(info).unwrap())).into_response(),
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn list_downloads(State(mgr): State<AppState>) -> impl IntoResponse {
    Json(mgr.get_all_progress())
}

async fn queue_download(
    State(mgr): State<AppState>,
    Json(body): Json<QueueDownloadRequest>,
) -> impl IntoResponse {
    let download_id = mgr.queue_download(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadId": download_id })),
    )
}

async fn queue_playlist(
    State(mgr): State<AppState>,
    Json(body): Json<QueuePlaylistRequest>,
) -> impl IntoResponse {
    let download_ids = mgr.queue_playlist(body);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "downloadIds": download_ids })),
    )
}

async fn get_download(
    State(mgr): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match mgr.get_progress(&id) {
        Some(progress) => {
            (StatusCode::OK, Json(serde_json::to_value(progress).unwrap())).into_response()
        }
        None => error_response(StatusCode::NOT_FOUND, "Download not found").into_response(),
    }
}

async fn cancel_download(
    State(mgr): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if mgr.cancel_download(&id) {
        Json(serde_json::json!({ "ok": true })).into_response()
    } else {
        error_response(StatusCode::NOT_FOUND, "Download not found").into_response()
    }
}

async fn clear_queue(State(mgr): State<AppState>) -> impl IntoResponse {
    mgr.clear_queue();
    Json(serde_json::json!({ "ok": true }))
}

async fn clear_completed(State(mgr): State<AppState>) -> impl IntoResponse {
    mgr.clear_completed();
    Json(serde_json::json!({ "ok": true }))
}

async fn download_events(
    State(mgr): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = mgr.subscribe_events();

    let stream = async_stream::stream! {
        // Send initial connected event
        yield Ok(Event::default()
            .event("connected")
            .data(serde_json::json!({ "message": "Connected to download events" }).to_string()));

        // Send initial stats
        let stats = mgr.get_stats();
        if let Ok(data) = serde_json::to_string(&stats) {
            yield Ok(Event::default().event("stats").data(data));
        }

        // Send current downloads
        for progress in mgr.get_all_progress() {
            if let Ok(data) = serde_json::to_string(&progress) {
                yield Ok(Event::default().event("progress").data(data));
            }
        }

        loop {
            match rx.recv().await {
                Ok(mhaol_yt_dlp::manager::SseEvent::Progress(progress)) => {
                    if let Ok(data) = serde_json::to_string(&progress) {
                        yield Ok(Event::default().event("progress").data(data));
                    }
                }
                Ok(mhaol_yt_dlp::manager::SseEvent::Stats(stats)) => {
                    if let Ok(data) = serde_json::to_string(&stats) {
                        yield Ok(Event::default().event("stats").data(data));
                    }
                }
                Ok(mhaol_yt_dlp::manager::SseEvent::Connected) => {
                    yield Ok(Event::default()
                        .event("connected")
                        .data(serde_json::json!({ "message": "Connected" }).to_string()));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("SSE client lagged by {} messages", n);
                    continue;
                }
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

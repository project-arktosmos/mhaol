mod protocol;
mod signaling_client;

use mhaol_p2p_stream::prelude::*;
use protocol::{Command, Event};
use signaling_client::SignalingClient;
// Note: `mhaol_p2p_stream` is this crate's own lib — Cargo resolves `[[bin]]`
// targets' `extern crate` references to the same package's `[lib]`.
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Tracing goes to stderr (stdout is reserved for the JSON protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_p2p_stream=debug".into()),
        )
        .init();

    mhaol_p2p_stream::init().expect("Failed to initialize GStreamer");
    info!("p2p-stream-worker started (GStreamer initialized)");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    let mut sessions: HashMap<String, SignalingClient> = HashMap::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF — parent process closed stdin
                info!("stdin closed, shutting down");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("Failed to read stdin: {e}");
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command: Command = match serde_json::from_str(trimmed) {
            Ok(cmd) => cmd,
            Err(e) => {
                let event = Event::Error {
                    session_id: None,
                    error: format!("Invalid command: {e}"),
                };
                let _ = stdout.write_all(event.to_json_line().as_bytes()).await;
                let _ = stdout.flush().await;
                continue;
            }
        };

        match command {
            Command::CreateSession {
                session_id,
                file_path,
                mode,
                video_codec,
                video_quality,
                signaling_url,
            } => {
                let event = handle_create_session(
                    &mut sessions,
                    session_id,
                    file_path,
                    mode,
                    video_codec,
                    video_quality,
                    signaling_url,
                )
                .await;
                let _ = stdout.write_all(event.to_json_line().as_bytes()).await;
                let _ = stdout.flush().await;
            }

            Command::DeleteSession { session_id } => {
                let event = handle_delete_session(&mut sessions, &session_id).await;
                let _ = stdout.write_all(event.to_json_line().as_bytes()).await;
                let _ = stdout.flush().await;
            }
        }
    }

    // Clean up all sessions on shutdown
    for (id, mut client) in sessions.drain() {
        info!(session_id = %id, "Cleaning up session on shutdown");
        client.disconnect().await;
    }

    info!("p2p-stream-worker stopped");
}

async fn handle_create_session(
    sessions: &mut HashMap<String, SignalingClient>,
    session_id: String,
    file_path: String,
    mode: Option<String>,
    video_codec: Option<String>,
    video_quality: Option<String>,
    signaling_url: String,
) -> Event {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Event::Error {
            session_id: Some(session_id),
            error: format!("File not found: {file_path}"),
        };
    }

    let is_audio_only = mode.as_deref() == Some("audio");

    let codec = video_codec
        .as_deref()
        .and_then(parse_video_codec)
        .unwrap_or(VideoCodec::Vp8);

    let quality = video_quality
        .as_deref()
        .and_then(parse_video_quality)
        .unwrap_or(VideoQuality::Native);

    let file_source = if is_audio_only {
        FileSource::new(&path).audio_only()
    } else {
        FileSource::new(&path)
    };

    let manager = SessionManager::new(
        move || {
            let builder = PipelineBuilder::new()
                .video_codec(codec)
                .video_quality(quality);
            if is_audio_only {
                builder.no_video()
            } else {
                builder
            }
        },
        file_source,
    );

    let room_id = session_id.clone();

    match SignalingClient::connect(session_id.clone(), manager, &signaling_url).await {
        Ok(client) => {
            info!(session_id = %session_id, "Session created, connected to signaling");
            sessions.insert(session_id.clone(), client);
            Event::SessionCreated {
                session_id,
                room_id,
            }
        }
        Err(e) => {
            error!(session_id = %session_id, "Failed to connect to signaling: {e}");
            Event::Error {
                session_id: Some(session_id),
                error: e,
            }
        }
    }
}

async fn handle_delete_session(
    sessions: &mut HashMap<String, SignalingClient>,
    session_id: &str,
) -> Event {
    if let Some(mut client) = sessions.remove(session_id) {
        client.disconnect().await;
        info!(session_id = %session_id, "Session deleted");
        Event::SessionDeleted {
            session_id: session_id.to_string(),
        }
    } else {
        Event::Error {
            session_id: Some(session_id.to_string()),
            error: "Session not found".to_string(),
        }
    }
}

fn parse_video_codec(s: &str) -> Option<VideoCodec> {
    match s {
        "vp8" => Some(VideoCodec::Vp8),
        "vp9" => Some(VideoCodec::Vp9),
        "h264" => Some(VideoCodec::H264),
        _ => None,
    }
}

fn parse_video_quality(s: &str) -> Option<VideoQuality> {
    match s {
        "native" => Some(VideoQuality::Native),
        "1080p" => Some(VideoQuality::Q1080p),
        "720p" => Some(VideoQuality::Q720p),
        "480p" => Some(VideoQuality::Q480p),
        "360p" => Some(VideoQuality::Q360p),
        _ => None,
    }
}

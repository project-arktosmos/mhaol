use crate::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_downloads))
}

#[derive(Serialize)]
struct UnifiedDownload {
    id: String,
    #[serde(rename = "type")]
    download_type: String,
    name: String,
    state: String,
    progress: f64,
    size: i64,
    #[serde(rename = "outputPath")]
    output_path: Option<String>,
    error: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "downloadSpeed", skip_serializing_if = "Option::is_none")]
    download_speed: Option<i64>,
    #[serde(rename = "uploadSpeed", skip_serializing_if = "Option::is_none")]
    upload_speed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peers: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seeds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eta: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(rename = "videoId", skip_serializing_if = "Option::is_none")]
    video_id: Option<String>,
    #[serde(rename = "thumbnailUrl", skip_serializing_if = "Option::is_none")]
    thumbnail_url: Option<String>,
    #[serde(rename = "durationSeconds", skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<i64>,
}

async fn get_downloads(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.downloads.get_all();

    let downloads: Vec<UnifiedDownload> = rows
        .into_iter()
        .map(|row| {
            let is_torrent = row.download_type == "torrent";

            let output_path = if is_torrent {
                match (&row.output_path, &row.name) {
                    (Some(p), name) if !name.is_empty() => Some(format!("{}/{}", p, name)),
                    (Some(p), _) => Some(p.clone()),
                    _ => None,
                }
            } else {
                row.output_path
            };

            UnifiedDownload {
                id: row.id,
                download_type: row.download_type.clone(),
                name: row.name,
                state: row.state,
                progress: row.progress,
                size: row.size,
                output_path,
                error: row.error,
                created_at: row.created_at,
                updated_at: row.updated_at,
                download_speed: if is_torrent { Some(row.download_speed) } else { None },
                upload_speed: if is_torrent { Some(row.upload_speed) } else { None },
                peers: if is_torrent { Some(row.peers) } else { None },
                seeds: if is_torrent { Some(row.seeds) } else { None },
                eta: if is_torrent { row.eta } else { None },
                url: row.url,
                video_id: row.video_id,
                thumbnail_url: row.thumbnail_url,
                duration_seconds: row.duration_seconds,
            }
        })
        .collect();

    Json(downloads)
}

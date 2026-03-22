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
}

async fn get_downloads(State(state): State<AppState>) -> impl IntoResponse {
    let torrent_rows = state.torrent_downloads.get_all();

    let mut downloads: Vec<UnifiedDownload> = Vec::new();

    for row in torrent_rows {
        let output_path = match (&row.output_path, &row.name) {
            (Some(p), name) if !name.is_empty() => Some(format!("{}/{}", p, name)),
            (Some(p), _) => Some(p.clone()),
            _ => None,
        };

        downloads.push(UnifiedDownload {
            id: row.info_hash,
            download_type: "torrent".to_string(),
            name: row.name,
            state: row.state,
            progress: row.progress,
            size: row.size,
            output_path,
            error: None,
            created_at: row.created_at,
            updated_at: row.updated_at,
            download_speed: Some(row.download_speed),
            upload_speed: Some(row.upload_speed),
            peers: Some(row.peers),
            seeds: Some(row.seeds),
            eta: row.eta,
        });
    }

    // Sort by updated_at descending
    downloads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Json(downloads)
}

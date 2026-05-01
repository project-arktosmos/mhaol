use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::state::RendezvousState;

/// HTTP routes that store/look up WebRTC signaling payloads as Kademlia DHT
/// records on the shared private swarm. Every record key is namespaced with
/// `mhaol-sig` so they don't collide with anything else on the swarm.
///
/// Routes:
/// - `POST /signal/:room/offer`       — body `{ "sdp": "..." }`
/// - `GET  /signal/:room/offer`       — returns `{ "sdp": "..." }` or 404
/// - `POST /signal/:room/answer`      — body `{ "sdp": "..." }`
/// - `GET  /signal/:room/answer`
/// - `POST /signal/:room/candidates`  — body `{ "candidates": ["..."] }`
/// - `GET  /signal/:room/candidates`
pub fn router() -> Router<RendezvousState> {
    Router::new()
        .route("/{room}/offer", post(put_offer).get(get_offer))
        .route("/{room}/answer", post(put_answer).get(get_answer))
        .route("/{room}/candidates", post(put_candidates).get(get_candidates))
}

#[derive(Debug, Deserialize, Serialize)]
struct SdpPayload {
    sdp: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CandidatesPayload {
    candidates: Vec<String>,
}

fn key_for(room: &str, slot: &str) -> Vec<u8> {
    format!("/mhaol-sig/{room}/{slot}").into_bytes()
}

async fn put_offer(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
    Json(body): Json<SdpPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    put_record(&state, &room, "offer", body.sdp.into_bytes()).await
}

async fn get_offer(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
) -> Result<Json<SdpPayload>, (StatusCode, String)> {
    let bytes = read_record(&state, &room, "offer").await?;
    let sdp = String::from_utf8(bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("invalid utf-8 in record: {e}")))?;
    Ok(Json(SdpPayload { sdp }))
}

async fn put_answer(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
    Json(body): Json<SdpPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    put_record(&state, &room, "answer", body.sdp.into_bytes()).await
}

async fn get_answer(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
) -> Result<Json<SdpPayload>, (StatusCode, String)> {
    let bytes = read_record(&state, &room, "answer").await?;
    let sdp = String::from_utf8(bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("invalid utf-8 in record: {e}")))?;
    Ok(Json(SdpPayload { sdp }))
}

async fn put_candidates(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
    Json(body): Json<CandidatesPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let value = serde_json::to_vec(&body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("encode failed: {e}")))?;
    put_record(&state, &room, "candidates", value).await
}

async fn get_candidates(
    State(state): State<RendezvousState>,
    Path(room): Path<String>,
) -> Result<Json<CandidatesPayload>, (StatusCode, String)> {
    let bytes = read_record(&state, &room, "candidates").await?;
    let payload: CandidatesPayload = serde_json::from_slice(&bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("decode failed: {e}")))?;
    Ok(Json(payload))
}

async fn put_record(
    state: &RendezvousState,
    room: &str,
    slot: &str,
    value: Vec<u8>,
) -> Result<StatusCode, (StatusCode, String)> {
    if !state.ipfs.is_initialized() {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "ipfs node not ready".into()));
    }
    let key = key_for(room, slot);
    state
        .ipfs
        .dht_put(&key, value)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("dht put failed: {e}")))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn read_record(
    state: &RendezvousState,
    room: &str,
    slot: &str,
) -> Result<Vec<u8>, (StatusCode, String)> {
    if !state.ipfs.is_initialized() {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "ipfs node not ready".into()));
    }
    let key = key_for(room, slot);
    match state.ipfs.dht_get(&key).await {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err((StatusCode::NOT_FOUND, "no record".into())),
        Err(e) => Err((StatusCode::BAD_GATEWAY, format!("dht get failed: {e}"))),
    }
}

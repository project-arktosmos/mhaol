use crate::AppState;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::UdpSocket;

#[derive(Serialize)]
struct NetworkInfo {
    local_ip: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/info", get(network_info))
}

async fn network_info() -> Json<NetworkInfo> {
    let local_ip = get_local_ip();
    Json(NetworkInfo { local_ip })
}

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

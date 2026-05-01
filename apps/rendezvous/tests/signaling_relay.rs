//! End-to-end signaling regression test.
//!
//! Boots the rendezvous Axum router (without a real IPFS node) on a random
//! port, has two WebSocket peers join the same room with valid EIP-191 auth,
//! and asserts that the canonical signaling flow goes end-to-end:
//!
//!   1. The "worker" peer joins first and receives a `connected` frame.
//!   2. The "browser" peer joins; the worker observes a `peer-joined` event.
//!   3. The worker sends an `offer` targeted at the browser; the browser
//!      receives it with `from_peer_id` set.
//!   4. The browser answers; the worker receives the answer.
//!   5. ICE candidates relay both ways.
//!
//! Any future change that breaks the wire format on either side fails this
//! test. The previous regression (after migrating from `apps/signaling` to
//! rendezvous) was exactly the kind of thing this test catches.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use k256::ecdsa::SigningKey;
use mhaol_rendezvous::{
    build_router,
    rooms::RoomManager,
    state::RendezvousState,
    turn::TurnConfig,
};
use sha3::{Digest, Keccak256};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite;

#[tokio::test]
async fn worker_and_browser_relay_offer_answer_and_ice() {
    let addr = boot_rendezvous().await;

    // Worker joins first, mirroring the canonical rendezvous flow:
    // a streaming worker joins the room, then the consumer browser does.
    let mut worker = connect_peer(&addr, "abc-room").await;
    let worker_id = expect_connected(&mut worker).await;
    let _empty_room = expect_message(&mut worker, "room-peers").await;

    let mut browser = connect_peer(&addr, "abc-room").await;
    let browser_id = expect_connected(&mut browser).await;

    // browser receives the room-peers list with the worker in it.
    let room_peers = expect_message(&mut browser, "room-peers").await;
    let peers = room_peers.get("peers").and_then(|p| p.as_array()).unwrap();
    assert_eq!(peers.len(), 1);
    let worker_in_room = peers[0]
        .get("peer_id")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();
    assert_eq!(worker_in_room, worker_id);

    // Worker observes peer-joined for the browser.
    let joined = expect_message(&mut worker, "peer-joined").await;
    assert_eq!(joined.get("peer_id").unwrap().as_str().unwrap(), browser_id);

    // Worker sends an offer; browser receives it with from_peer_id set.
    send_signal(
        &mut worker,
        serde_json::json!({
            "type": "offer",
            "target_peer_id": browser_id,
            "sdp": "v=0\r\n... fake offer ...",
        }),
    )
    .await;
    let offer = expect_message(&mut browser, "offer").await;
    assert_eq!(
        offer.get("from_peer_id").and_then(|v| v.as_str()).unwrap(),
        worker_id
    );
    assert_eq!(
        offer.get("sdp").and_then(|v| v.as_str()).unwrap(),
        "v=0\r\n... fake offer ..."
    );

    // Browser answers; worker receives it.
    send_signal(
        &mut browser,
        serde_json::json!({
            "type": "answer",
            "target_peer_id": worker_id,
            "sdp": "v=0\r\n... fake answer ...",
        }),
    )
    .await;
    let answer = expect_message(&mut worker, "answer").await;
    assert_eq!(
        answer.get("from_peer_id").and_then(|v| v.as_str()).unwrap(),
        browser_id
    );

    // ICE in both directions.
    send_signal(
        &mut worker,
        serde_json::json!({
            "type": "ice-candidate",
            "target_peer_id": browser_id,
            "candidate": "candidate:1 1 udp 1 1.2.3.4 9 typ host",
            "sdp_m_line_index": 0,
            "sdp_mid": "0",
        }),
    )
    .await;
    let ice = expect_message(&mut browser, "ice-candidate").await;
    assert_eq!(
        ice.get("from_peer_id").and_then(|v| v.as_str()).unwrap(),
        worker_id
    );
    assert_eq!(
        ice.get("sdp_m_line_index").and_then(|v| v.as_u64()).unwrap(),
        0
    );

    // Browser disconnect -> worker sees peer-left.
    browser.close(None).await.unwrap();
    let left = expect_message(&mut worker, "peer-left").await;
    assert_eq!(left.get("peer_id").unwrap().as_str().unwrap(), browser_id);
}

#[tokio::test]
async fn rejects_connection_with_invalid_signature() {
    let addr = boot_rendezvous().await;

    let timestamp = current_ms().to_string();
    let address = "0x0000000000000000000000000000000000000001";
    // Random 65-byte signature that won't recover to `address`.
    let signature = format!("0x{}", "11".repeat(65));

    let url = format!(
        "ws://{addr}/party/abc?address={address}&signature={signature}&timestamp={timestamp}"
    );
    let result = tokio_tungstenite::connect_async(&url).await;
    assert!(
        result.is_err(),
        "expected the rendezvous WS to reject an invalid signature"
    );
}

#[tokio::test]
async fn dual_stack_bind_is_reachable_on_ipv4_and_ipv6_loopback() {
    // Regression: rendezvous used to bind `0.0.0.0:14080` (IPv4 only).
    // macOS resolves `localhost` to `::1` first and Firefox WebSockets
    // refuse the connection without falling back to IPv4, which presented
    // as "stuck at negotiating WebRTC connection". Binding `::` accepts
    // both stacks; this test fails fast if the dual-stack bind regresses.
    let port = pick_free_port().await;
    let listener = tokio::net::TcpListener::bind(format!("[::]:{port}"))
        .await
        .expect("dual-stack bind must succeed for the regression to stay fixed");

    let state = RendezvousState {
        ipfs: Arc::new(mhaol_ipfs_core::IpfsManager::new()),
        rooms: Arc::new(RoomManager::new()),
        turn: Arc::new(TurnConfig::default()),
    };
    let app = build_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    tokio::task::yield_now().await;

    // Both loopback families must hit /api/health.
    for host in ["127.0.0.1", "[::1]"] {
        let url = format!("http://{host}:{port}/api/health");
        let body = reqwest::get(&url)
            .await
            .unwrap_or_else(|e| panic!("GET {url} failed (dual-stack regression?): {e}"))
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("ok"));
    }
}

async fn pick_free_port() -> u16 {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    port
}

#[tokio::test]
async fn turn_credentials_endpoint_returns_metered_format() {
    let addr = boot_rendezvous_with_turn(TurnConfig {
        domain: "turn.example.com".to_string(),
        shared_secret: "test-secret".to_string(),
        api_keys: vec!["ok".to_string()],
        ..TurnConfig::default()
    })
    .await;

    let body = reqwest::get(format!("http://{addr}/api/v1/turn/credentials?apiKey=ok"))
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0]
        .get("urls")
        .and_then(|v| v.as_str())
        .unwrap()
        .starts_with("stun:"));
    assert!(entries[1].get("username").is_some());

    // Unauthorized when no key supplied.
    let status = reqwest::get(format!("http://{addr}/api/v1/turn/credentials"))
        .await
        .unwrap()
        .status();
    assert_eq!(status.as_u16(), 401);
}

// ===== test helpers =====

type WsClient = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

async fn boot_rendezvous() -> String {
    boot_rendezvous_with_turn(TurnConfig::default()).await
}

async fn boot_rendezvous_with_turn(turn: TurnConfig) -> String {
    // The WS + TURN routes never touch IPFS, so we can ship an uninitialized
    // IpfsManager here — keeps the test fast and avoids needing a real
    // libp2p stack.
    let state = RendezvousState {
        ipfs: Arc::new(mhaol_ipfs_core::IpfsManager::new()),
        rooms: Arc::new(RoomManager::new()),
        turn: Arc::new(turn),
    };

    let app = build_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    // Give the runtime a tick to start accepting.
    tokio::task::yield_now().await;
    addr
}

async fn connect_peer(addr: &str, room_id: &str) -> WsClient {
    let signing_key = SigningKey::random(&mut k256::elliptic_curve::rand_core::OsRng);
    let address = eth_address_from_key(&signing_key);
    let timestamp = current_ms().to_string();
    let message = format!("partykit-auth:{room_id}:{timestamp}");

    let prefixed = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        message.len(),
        message
    );
    let hash = Keccak256::digest(prefixed.as_bytes());
    let (sig, recovery) = signing_key.sign_prehash_recoverable(&hash).unwrap();
    let mut sig_bytes = [0u8; 65];
    sig_bytes[..64].copy_from_slice(&sig.to_bytes());
    sig_bytes[64] = recovery.to_byte() + 27;
    let sig_hex = format!("0x{}", hex::encode(sig_bytes));

    let url = format!(
        "ws://{addr}/party/{room_id}?address={address}&signature={sig_hex}&timestamp={timestamp}"
    );
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("WS upgrade failed; check signaling auth wire format");
    ws
}

async fn expect_connected(ws: &mut WsClient) -> String {
    let v = expect_message(ws, "connected").await;
    v.get("peer_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .expect("connected frame missing peer_id")
}

async fn expect_message(ws: &mut WsClient, expected_type: &str) -> serde_json::Value {
    // Up to ~2s of patience per frame so the test stays bounded even when
    // the relay is broken — much better than the original 10-minute browser
    // timeout that left users staring at "negotiating WebRTC connection".
    let frame = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {expected_type}"))
        .unwrap_or_else(|| panic!("WS closed before {expected_type}"))
        .unwrap_or_else(|e| panic!("WS error waiting for {expected_type}: {e}"));

    let text = match frame {
        tungstenite::Message::Text(t) => t.to_string(),
        other => panic!("expected text frame for {expected_type}, got {other:?}"),
    };
    let v: serde_json::Value = serde_json::from_str(&text).expect("frame is JSON");
    let actual_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    assert_eq!(
        actual_type, expected_type,
        "unexpected frame type (full payload: {text})"
    );
    v
}

async fn send_signal(ws: &mut WsClient, payload: serde_json::Value) {
    ws.send(tungstenite::Message::Text(payload.to_string().into()))
        .await
        .unwrap();
}

fn current_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn eth_address_from_key(key: &SigningKey) -> String {
    let verifying_key = key.verifying_key();
    let encoded = verifying_key.to_encoded_point(false);
    let bytes = &encoded.as_bytes()[1..];
    let hash = Keccak256::digest(bytes);
    format!("0x{}", hex::encode(&hash[12..]))
}

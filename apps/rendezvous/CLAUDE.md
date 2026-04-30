# Rendezvous

**Location:** `apps/rendezvous/`
**Framework:** Rust — Axum 0.8 + Tokio + `mhaol-ipfs` + `mhaol-identity`
**Crate:** `mhaol-rendezvous`
**Binary:** `mhaol-rendezvous` (default HTTP port 14080, libp2p TCP port 14001)

The rendezvous app is the **only** signaling/bootstrap server in the monorepo. It absorbs everything the previous `apps/signaling` (Rust) and `packages/signaling` (PartyKit) packages did and adds the private-swarm IPFS bootstrap node on top:

1. Long-lived **IPFS bootstrap node** for the private Mhaol swarm (Kademlia DHT in Server mode, listens on TCP 14001).
2. **WebSocket WebRTC signaling** at `/party/{room_id}` — wire-compatible with the legacy `mhaol-signaling` and PartyKit servers. EIP-191 auth, passport verification, peer relay, ICE servers in the `connected` message.
3. **DHT-backed signaling primitives** at `/signal/{room_id}/{offer,answer,candidates}` — peers exchange SDPs as Kademlia records on the same private swarm. Useful when there is no long-lived WebSocket but two peers want to discover each other through the DHT.
4. **TURN credential server** at `/api/v1/turn/credentials?apiKey=...` — Metered-compatible HMAC-SHA1 credentials.
5. **CLI deployment wizard** for Linux servers (coturn + Let's Encrypt + systemd).

## Source Structure

```
src/
├── server.rs         # Binary entry; clap CLI (serve / setup / status); builds the router
├── config.rs         # RendezvousConfig — env vars + optional TOML file
├── state.rs          # RendezvousState { ipfs, rooms, turn }
├── status.rs         # GET /api/status, GET /api/health
├── ws.rs             # WebSocket signaling: /party/{room_id} (EIP-191 auth + passport + relay)
├── rooms.rs          # In-memory RoomManager (peers, eviction, broadcast, relay)
├── turn.rs           # TurnConfig + HMAC-SHA1 credential generation + REST endpoint
├── signaling.rs      # /signal/{room_id}/{offer,answer,candidates} — DHT put/get
├── health_check.rs   # CLI helper for `mhaol-rendezvous status --url ...`
└── setup/
    ├── mod.rs            # Interactive setup wizard
    ├── detect.rs         # OS / package-manager detection
    ├── coturn.rs         # coturn install + /etc/turnserver.conf
    ├── tls.rs            # certbot / Let's Encrypt
    └── systemd.rs        # mhaol-rendezvous.service unit
```

## CLI

```bash
mhaol-rendezvous                       # Implicit `serve` (no subcommand)
mhaol-rendezvous serve [--config FILE] # Run the server, optionally with a TOML file
mhaol-rendezvous setup                 # Interactive Linux deployment wizard
mhaol-rendezvous status [--url URL]    # Health-check a running rendezvous
```

## Private swarm enforcement

The node refuses to start without a swarm key. Concretely:

- It reads (or generates) a swarm key at `IPFS_SWARM_KEY_FILE` (default `<DATA_DIR>/swarm.key`, otherwise `<home>/mhaol/swarm.key`).
- The key is passed to `mhaol-ipfs::IpfsManager::initialize`, which installs the libp2p **`pnet`** transport (TCP + pnet + noise + yamux). The `pnet` handshake fails for any peer that does not present the same pre-shared key, so non-private peers are rejected at the transport layer before anything reaches Kademlia or the application.
- The public IPFS bootstrap list is **never** dialed; rendezvous is itself the only entry point into the swarm.
- mDNS is disabled — local peers without the PSK would still be filtered, but there is nothing to gain by advertising.

The cloud app reads the same `swarm.key` file by default, so the two converge on the same private network without explicit configuration.

## WebSocket signaling protocol

`GET /party/{room_id}` upgrades to a WebSocket. Required query params:

- `address` — peer Ethereum address (lowercased on the wire).
- `signature` — EIP-191 signature over `partykit-auth:{room_id}:{timestamp}`. The string literal `partykit-auth:` is preserved for wire compatibility with existing clients (`packages/ui-lib`, `packages/p2p-stream/src/worker/signaling_client.rs`); the previous PartyKit deployment is gone, but the auth message format is unchanged.
- `timestamp` — Unix milliseconds; the request is rejected if it differs from server-now by more than 30s.
- `passport_raw` (optional) — JSON `{ "name": "...", "instanceType": "..." }`.
- `passport_signature` (optional) — EIP-191 signature over `passport_raw`. Both are required to publish a name/instance type.

After the upgrade the server emits these JSON frames:
- `{ type: "connected", peer_id, name, instance_type, ice_servers }` — `ice_servers` comes from `turn::generate_credentials`.
- `{ type: "room-peers", room_id, peers: [...] }` — existing peers in the room.
- `{ type: "peer-joined", room_id, peer_id, name, instance_type }` — broadcast to existing peers when a new one joins.
- `{ type: "peer-left", room_id, peer_id }` — broadcast on disconnect.

Clients send `{ type: "offer"|"answer"|"ice-candidate", target_peer_id, sdp|candidate, ... }`; the server relays to the target peer with `from_peer_id` set.

Duplicate connections from the same `peer_id` are evicted with close code `4002` ("Replaced by new connection").

`GET /party/{room_id}/status` returns the current peer list as JSON.

## DHT-backed signaling primitives

`/signal/{room_id}/{offer,answer,candidates}` translates each request into a `dht_put` / `dht_get` against the embedded IPFS node. Records are namespaced under `/mhaol-sig/<room>/<slot>` so they don't collide with anything else on the swarm. This is the lower-level surface for peers that don't want to maintain an open WebSocket.

## TURN credential REST endpoint

`GET /api/v1/turn/credentials?apiKey=...` — Metered-compatible JSON array of `{urls, username?, credential?}`. HMAC-SHA1 over `<expiry>:mhaol` with `TURN_SHARED_SECRET`. Returns `503` when TURN is not configured (no domain or shared secret).

If `TURN_API_KEY` (or the TOML `turn.api_keys` list) is non-empty, the endpoint requires the matching `apiKey` query param.

## Status / health

- `GET /api/status` — `{ role: "rendezvous", ipfs: IpfsStats, bootstrapMultiaddrs: [...], turnConfigured: bool }`.
- `GET /api/health` — `{ status: "ok", service: "mhaol-rendezvous" }`.

## Bootstrap discovery

On startup the node writes its full bootstrap multiaddrs (`/ip4/.../tcp/14001/p2p/<peerId>`) to `<DATA_DIR>/rendezvous/bootstrap.multiaddr` (override with `RENDEZVOUS_BOOTSTRAP_FILE`). The cloud reads that file by default if `RENDEZVOUS_BOOTSTRAP` is unset, so a single-machine setup is zero-config.

## Configuration

Two layers, env vars override the TOML file:

```toml
# /etc/mhaol-rendezvous/config.toml (written by `setup`)
[server]
host = "0.0.0.0"
http_port = 14080
ipfs_listen_port = 14001
tls_cert = "/etc/letsencrypt/live/example.com/fullchain.pem"
tls_key  = "/etc/letsencrypt/live/example.com/privkey.pem"

[turn]
domain = "turn.example.com"
shared_secret = "..."
credential_ttl_secs = 86400
stun_port = 3478
turn_port = 3478
turns_port = 5349
api_keys = ["..."]
```

| Env var | Description |
|---------|-------------|
| `RENDEZVOUS_HOST` | HTTP bind host (default: `0.0.0.0`) |
| `RENDEZVOUS_HTTP_PORT` | HTTP signaling port (default: `14080`) |
| `RENDEZVOUS_LISTEN_PORT` | libp2p TCP listen port (default: `14001`) |
| `RENDEZVOUS_REPO_DIR` | IPFS repo directory (default: `<DATA_DIR>/rendezvous/ipfs`) |
| `RENDEZVOUS_BOOTSTRAP_FILE` | Where to write advertised bootstrap multiaddrs |
| `IPFS_SWARM_KEY_FILE` | Shared private-swarm PSK path (default: `<DATA_DIR>/swarm.key`) |
| `TLS_CERT` / `TLS_KEY` | Optional TLS (rustls). When set, server speaks HTTPS/WSS. |
| `TURN_DOMAIN` / `TURN_SHARED_SECRET` / `TURN_API_KEY` | TURN configuration. Empty domain or secret disables TURN. |
| `DATA_DIR` | Top-level data directory; everything above derives from it. |

## Running

```bash
pnpm app:rendezvous        # Dev — runs `cargo run -p mhaol-rendezvous`
pnpm app:rendezvous:setup  # Linux deployment wizard
pnpm build:rendezvous      # Release build
```

Once it is running, start the cloud (`pnpm app:cloud`) — the cloud auto-discovers rendezvous via the bootstrap file and joins the same private swarm, and resolves `SIGNALING_URL` to `http://localhost:14080` by default.

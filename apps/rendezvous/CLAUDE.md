# Rendezvous

**Location:** `apps/rendezvous/`
**Framework:** Rust — Axum 0.8 + Tokio + `mhaol-ipfs`
**Crate:** `mhaol-rendezvous`
**Binary:** `mhaol-rendezvous` (default HTTP port 14080, libp2p TCP port 14001)

The rendezvous app is a long-lived IPFS bootstrap node for the private Mhaol swarm. It runs in DHT **Server** mode so other peers (the cloud, future clients) can both reach it as a bootstrap address and publish/look up records against it. It also exposes a small HTTP signaling API that stores WebRTC SDP offers, answers, and ICE candidates as Kademlia DHT records on the same private swarm — there is no centralized signaling server, only the DHT.

## Source Structure

```
src/
├── server.rs        # Binary entry point; loads/generates swarm key, starts the IPFS node, builds the HTTP router, writes the bootstrap multiaddr file
├── config.rs        # RendezvousConfig::from_env (host/ports/paths)
├── state.rs         # RendezvousState { ipfs: Arc<IpfsManager> }
├── status.rs        # GET /api/status
└── signaling.rs     # /signal/:room/{offer,answer,candidates} — DHT-backed put/get
```

## Private swarm enforcement

The node refuses to start without a swarm key. Concretely:

- It reads (or generates) a swarm key at `IPFS_SWARM_KEY_FILE` (default `<DATA_DIR>/swarm.key`, otherwise `<home>/mhaol/swarm.key`).
- The key is passed to `mhaol-ipfs::IpfsManager::initialize`, which installs the libp2p **`pnet`** transport (TCP + pnet + noise + yamux). The `pnet` handshake fails for any peer that does not present the same pre-shared key, so non-private peers are rejected at the transport layer before anything reaches Kademlia or the application.
- The public IPFS bootstrap list is **never** dialed; rendezvous is itself the only entry point into the swarm.
- mDNS is disabled — local peers without the PSK would still be filtered, but there is nothing to gain by advertising.

The cloud app reads the same `swarm.key` file by default, so the two converge on the same private network without explicit configuration.

## DHT-backed WebRTC signaling

The HTTP API speaks JSON and translates each request into a `dht_put` / `dht_get` against the running IPFS node. The records are namespaced under `/mhaol-sig/<room>/<slot>` so they don't collide with anything else on the swarm.

- `POST /signal/:room/offer`       — body `{ "sdp": "..." }`, stores the offer in the DHT (`204`).
- `GET  /signal/:room/offer`       — returns `{ "sdp": "..." }`, or `404` if absent.
- `POST /signal/:room/answer`      — body `{ "sdp": "..." }`.
- `GET  /signal/:room/answer`
- `POST /signal/:room/candidates`  — body `{ "candidates": ["..."] }`. The full list replaces whatever was there.
- `GET  /signal/:room/candidates`

Records propagate through Kademlia replication, so any other private-swarm peer that knows the room id can fetch them — the rendezvous HTTP server is just a convenience wrapper, the signaling state lives on the DHT.

## Status endpoint

- `GET /api/status` — `{ role: "rendezvous", ipfs: IpfsStats, bootstrapMultiaddrs: [...] }`. The `bootstrapMultiaddrs` list is the listen-addr set with `/p2p/<peerId>` appended; copy any of them into another node's `RENDEZVOUS_BOOTSTRAP` env var to join the swarm.

## Bootstrap discovery

On startup the node writes its full bootstrap multiaddrs (`/ip4/.../tcp/14001/p2p/<peerId>`) to `<DATA_DIR>/rendezvous/bootstrap.multiaddr` (override with `RENDEZVOUS_BOOTSTRAP_FILE`). The cloud reads that file by default if `RENDEZVOUS_BOOTSTRAP` is unset, so a single-machine setup is zero-config.

## Environment variables

- `RENDEZVOUS_HOST` — Bind address for the HTTP server (default: `0.0.0.0`).
- `RENDEZVOUS_HTTP_PORT` — HTTP signaling port (default: `14080`).
- `RENDEZVOUS_LISTEN_PORT` — libp2p TCP listen port (default: `14001`).
- `RENDEZVOUS_REPO_DIR` — IPFS repo directory (default: `<DATA_DIR>/rendezvous/ipfs`).
- `RENDEZVOUS_BOOTSTRAP_FILE` — Where to write the advertised bootstrap multiaddrs (default: `<DATA_DIR>/rendezvous/bootstrap.multiaddr`).
- `IPFS_SWARM_KEY_FILE` — Shared swarm key path (default: `<DATA_DIR>/swarm.key`).
- `DATA_DIR` — Top-level data directory; everything above derives from it.

## Running

```bash
# Dev
pnpm app:rendezvous

# Release build
pnpm build:rendezvous
```

Once it is running, start the cloud (`pnpm app:cloud`) — the cloud auto-discovers rendezvous via the bootstrap file and joins the same private swarm.

# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded SurrealKV)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 9898)

The cloud server runs an embedded SurrealDB store, an identity manager, the shared `mhaol-queue` task manager (on a separate `cloud-queue.db` SQLite file), and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ed2k`, and `mhaol-ipfs`. It loads `mhaol-p2p-stream` for the GStreamer worker, and serves the Svelte WebUI from the nested `web/` directory. It is **independent** from `mhaol-node` — node still uses its own SQLite layer, cloud has its own state.

## Source Structure

```
src/
├── server.rs            # Binary entry point — opens SurrealDB, builds router
├── db.rs                # SurrealDB connection helper (SurrealKv engine)
├── state.rs             # CloudState: { db, identity_manager, queue, ytdl_manager, torrent_manager, ed2k_manager, ipfs_manager }
├── cloud_status.rs      # GET /api/cloud/status
├── libraries.rs         # /api/libraries CRUD — SurrealDB-backed library records identified by their on-disk dir
├── documents.rs         # /api/documents CRUD — SurrealDB-backed document records (name, author, description)
├── fs_browse.rs         # /api/fs/browse — list subdirectories under a path (defaults to home), used by the WebUI directory picker
└── frontend.rs          # rust-embed wrapper that serves web/dist-static/

web/                     # SvelteKit static SPA (pnpm package `cloud`); builds to web/dist-static/
├── src/                 # routes, components, services, css
├── scripts/             # nav generator + Vite plugin
├── svelte.config.js
├── vite.config.ts
└── package.json
```

## Database

- Engine: **SurrealDB 2.x** with the embedded **SurrealKV** kv backend (pure Rust, no external server).
- Default location: `apps/cloud/cloud-surrealkv/` (a directory managed by SurrealKV).
- Namespace: `mhaol`, database: `cloud`.
- Override path via `DB_PATH` env var, or set `DATA_DIR` to put it under `<DATA_DIR>/cloud-surrealkv/`.
- The store is created fresh on first boot. There are no schemas or repos defined yet — add tables/queries as features land.

## Packages loaded by cloud

The cloud crate directly depends on these mhaol packages and reports their health on `/api/cloud/status`:

- `mhaol-queue` — task queue, backed by a dedicated `cloud-queue.db` SQLite file (sibling of the SurrealDB store).
- `mhaol-p2p-stream` — GStreamer worker subprocess + WebRTC streaming (cfg(not(target_os = "android"))).
- `mhaol-yt-dlp` — YouTube download manager (cfg(not(target_os = "android"))).
- `mhaol-torrent` — `librqbit`-backed torrent session, initialized in the background on startup so the server can bind quickly (cfg(not(target_os = "android"))).
- `mhaol-ed2k` — eDonkey/ed2k client (cfg(not(target_os = "android"))).
- `mhaol-ipfs` — embedded `rust-ipfs` node (libp2p, Bitswap, Kademlia DHT, optional mDNS), initialized in the background on startup. The blockstore lives at `<DATA_DIR>/downloads/ipfs/` (cfg(not(target_os = "android"))). The node always runs on a **private swarm**: cloud reads (or auto-generates on first boot) a swarm key at `<DATA_DIR>/downloads/ipfs/swarm.key` (override with `IPFS_SWARM_KEY_FILE`). Only nodes carrying that exact key can connect; the public bootstrap list is skipped, and the transport stack is constrained to TCP+pnet+noise+yamux. Copy the file to every other node that should join the network.

Default download paths land under `<DATA_DIR>/downloads/{torrents,ed2k,ipfs}` (or `<crate>/downloads/...` if `DATA_DIR` is unset). yt-dlp honors `YTDL_OUTPUT_DIR`/`YTDL_PO_TOKEN`/`YTDL_VISITOR_DATA`/`YTDL_COOKIES` via `YtDownloadConfig::from_env()`.

## What is NOT here (yet)

The cloud binary used to depend on `mhaol-node` and spawn its recommendation workers, peer service, library scanner, hub, etc. Those are still SQLite-backed and stay in node. When equivalent features are needed in cloud, they get re-implemented on top of SurrealDB.

## WebUI

The Svelte app lives at `apps/cloud/web/` (pnpm package name `cloud`). The user-facing port is always **9898** in both modes:

- **Dev** — Vite binds `0.0.0.0:9898` and serves the live Svelte app with hot reload. The Rust server binds `127.0.0.1:9899` (loopback only, invisible to the network). Vite proxies `/api/*` to `127.0.0.1:9899`, so the browser only ever talks to 9898.
- **Production (release builds)** — the Rust server binds `0.0.0.0:9898` and embeds `apps/cloud/web/dist-static/` via `rust-embed`, serving it directly as the fallback for any non-API path. Build it with `pnpm --filter cloud build` (or `pnpm build:cloud` to build the WebUI and the release binary together).

## Running

```bash
# Dev — starts cloud (Rust loopback :9899 + Vite WebUI :9898) + player (:9595)
pnpm dev

# Dev — Rust server only on 127.0.0.1:9899 (no UI; for API-only work)
pnpm dev:cloud

# Dev — WebUI hot-reload on 9898 (proxies /api to 127.0.0.1:9899)
pnpm dev:cloud:web

# Production build (embeds the WebUI)
pnpm build:cloud
```

## Environment Variables

- `PORT` — Server port (default: 9898; `pnpm dev:cloud` and `pnpm dev` set it to 9899)
- `HOST` — Bind address (default: 0.0.0.0; `pnpm dev:cloud` and `pnpm dev` set it to 127.0.0.1)
- `DB_PATH` — SurrealDB store path (default: `apps/cloud/cloud-surrealkv/`)
- `DATA_DIR` — If set and `DB_PATH` is unset, the store goes to `<DATA_DIR>/cloud-surrealkv/`
- `SIGNALING_URL` — PartyKit signaling URL (default: hosted instance)
- `IPFS_SWARM_KEY_FILE` — Path to the IPFS pre-shared swarm key. Default: `<DATA_DIR>/downloads/ipfs/swarm.key` (auto-generated on first boot when missing). All nodes on the same private swarm must share this file byte-for-byte.

## Worker subcommand

The binary still supports `mhaol-cloud worker`, which runs `mhaol_p2p_stream::worker::run()` for the GStreamer worker process. This subcommand does not touch the database or the identity manager.

## Public WebUI endpoints

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, signaling/client wallet addresses, db engine/namespace/version, and a `packages` block reporting health for `p2pStream`, `queue`, `ytDlp`, `torrent`, `ed2k`, and `ipfs`. No auth required (used by the embedded WebUI).
- `GET /api/libraries` — list libraries persisted in SurrealDB (`library` table). Libraries have no name; each is identified by its directory path.
- `POST /api/libraries` — create a library `{ path }`. The directory is created on disk if it does not exist; duplicate paths are rejected with `409`.
- `GET /api/libraries/:id` — fetch one library.
- `PUT /api/libraries/:id` — update the path. The new path is created on disk if missing; duplicates are rejected with `409`.
- `DELETE /api/libraries/:id` — remove the library record (the on-disk directory is left untouched).
- `GET /api/documents` — list documents persisted in SurrealDB (`document` table).
- `POST /api/documents` — create a document `{ name, author, description? }`. `name` and `author` are required.
- `GET /api/documents/:id` — fetch one document.
- `PUT /api/documents/:id` — update `name`, `author`, or `description` (any subset).
- `DELETE /api/documents/:id` — remove the document record.
- `GET /api/fs/browse?path=<optional>` — list subdirectories under `path` (defaults to the system home directory). Returns `{ path, parent, home, separator, roots, entries }` where `entries` only contains directories (hidden dot-folders are skipped). On Windows, `roots` lists available drive letters.

# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded SurrealKV)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 1540)

The cloud server runs an embedded SurrealDB store, an identity manager, the shared `mhaol-queue` task manager (on a separate `cloud-queue.db` SQLite file), and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, and `mhaol-ed2k`. It loads `mhaol-p2p-stream` for the GStreamer worker, and serves the Svelte WebUI from `apps/cloud-web/`. It is **independent** from `mhaol-node` — node still uses its own SQLite layer, cloud has its own state.

## Source Structure

```
src/
├── server.rs            # Binary entry point — opens SurrealDB, builds router
├── db.rs                # SurrealDB connection helper (SurrealKv engine)
├── state.rs             # CloudState: { db, identity_manager, queue, ytdl_manager, torrent_manager, ed2k_manager }
├── cloud_status.rs      # GET /api/cloud/status
└── frontend.rs          # rust-embed wrapper that serves apps/cloud-web/dist-static/
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

Default download paths land under `<DATA_DIR>/downloads/{torrents,ed2k}` (or `<crate>/downloads/...` if `DATA_DIR` is unset). yt-dlp honors `YTDL_OUTPUT_DIR`/`YTDL_PO_TOKEN`/`YTDL_VISITOR_DATA`/`YTDL_COOKIES` via `YtDownloadConfig::from_env()`.

## What is NOT here (yet)

The cloud binary used to depend on `mhaol-node` and spawn its recommendation workers, peer service, library scanner, hub, etc. Those are still SQLite-backed and stay in node. When equivalent features are needed in cloud, they get re-implemented on top of SurrealDB.

## WebUI

The Svelte app lives at `apps/cloud-web/` and builds to `apps/cloud-web/dist-static/`. The cloud crate embeds that directory via `rust-embed` and serves it as a fallback for any non-API path. Build it with `pnpm --filter cloud-web build` (or `pnpm build:cloud` to build the WebUI and the release binary together).

## Running

```bash
# Dev — Rust server on 1540
pnpm dev:cloud

# Dev — WebUI hot-reload on 9596 (proxies /api to :1540)
pnpm dev:cloud-web

# Production build (embeds the WebUI)
pnpm build:cloud
```

## Environment Variables

- `PORT` — Server port (default: 1540)
- `HOST` — Bind address (default: 0.0.0.0)
- `DB_PATH` — SurrealDB store path (default: `apps/cloud/cloud-surrealkv/`)
- `DATA_DIR` — If set and `DB_PATH` is unset, the store goes to `<DATA_DIR>/cloud-surrealkv/`
- `SIGNALING_URL` — PartyKit signaling URL (default: hosted instance)

## Worker subcommand

The binary still supports `mhaol-cloud worker`, which runs `mhaol_p2p_stream::worker::run()` for the GStreamer worker process. This subcommand does not touch the database or the identity manager.

## Public WebUI endpoint

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, signaling/client wallet addresses, db engine/namespace/version, and a `packages` block reporting health for `p2pStream`, `queue`, `ytDlp`, `torrent`, and `ed2k`. No auth required (used by the embedded WebUI).

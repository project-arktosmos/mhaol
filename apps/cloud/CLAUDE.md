# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded RocksDB)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 9898)

The cloud server runs an embedded SurrealDB store, an identity manager, the shared `mhaol-queue` task manager (on a separate `cloud-queue.db` SQLite file), and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ed2k`, and `mhaol-ipfs`. It loads `mhaol-p2p-stream` for the GStreamer worker, and serves the Svelte WebUI from the nested `web/` directory. It is **independent** from `mhaol-node` — node still uses its own SQLite layer, cloud has its own state.

The cloud also ships a desktop Tauri shell at `apps/cloud/src-tauri/` (crate `mhaol-cloud-shell`, productName "Mhaol Cloud"). The shell loads the shared health UI under `apps/tauri/web/` — it does **not** wrap the cloud WebUI itself; the WebUI stays browser-accessible at `http://localhost:9898`. The shell also registers a system tray icon (id `mhaol-cloud-tray`) on macOS/Windows/Linux with a single **Open** menu item that opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`.

## Source Structure

```
src/
├── server.rs            # Binary entry point — opens SurrealDB, builds router
├── db.rs                # SurrealDB connection helper (RocksDB engine)
├── state.rs             # CloudState: { db, identity_manager, queue, ytdl_manager, torrent_manager, ed2k_manager, ipfs_manager }
├── cloud_status.rs      # GET /api/cloud/status
├── libraries.rs         # /api/libraries CRUD — SurrealDB-backed library records identified by their on-disk dir
├── documents.rs         # /api/documents CRUD — SurrealDB-backed document records (name, author, description)
├── database.rs          # /api/database/tables{,/:table} — read-only SurrealDB explorer (lists tables, paginates records)
├── ipfs_pins.rs         # /api/ipfs/pins — lists pins recorded when libraries are scanned; exposes record_pin() used by the scan handler
├── fs_browse.rs         # /api/fs/browse — list subdirectories under a path (defaults to home), used by the WebUI directory picker
├── catalog.rs           # /api/catalog/* — proxies popular items + genres for tmdb / musicbrainz / openlibrary / retroachievements
├── search.rs            # /api/search/* — TMDB + ThePirateBay + LRCLIB lyrics + Wyzie subtitle proxy (drives the right-side `SubsLyricsFinder` panel)
├── player.rs            # /api/player/{stream-status,playable} — stubs so `playerService.initialize()` settles cleanly in the WebUI
├── ytdl.rs              # /api/ytdl/* — mounts `mhaol_yt_dlp::build_router(state.ytdl_manager)` so the WebUI talks to the cloud's yt-dlp manager directly (cfg(not(target_os = "android")))
└── frontend.rs          # rust-embed wrapper that serves web/dist-static/

web/                     # SvelteKit static SPA (pnpm package `cloud`); builds to web/dist-static/
├── src/                 # routes, components, services, css
├── scripts/             # nav generator + Vite plugin
├── svelte.config.js
├── vite.config.ts
└── package.json
```

## Database

- Engine: **SurrealDB 2.x** with the embedded **RocksDB** kv backend. SurrealKV was tried first but hit [surrealdb/surrealdb#5064](https://github.com/surrealdb/surrealdb/issues/5064) — concurrent writes from background scan / pin / request handlers corrupted the store and reads panicked with `Invalid revision N for type Value`. RocksDB does not have this problem.
- Default location: `<home>/mhaol/cloud-rocksdb/` — resolved via `dirs::home_dir()`, so it's OS-aware (`~/mhaol/...` on macOS/Linux, `%USERPROFILE%\mhaol\...` on Windows). The directory is managed by RocksDB.
- Namespace: `mhaol`, database: `cloud`.
- Override path via `DB_PATH` env var, or set `DATA_DIR` to put it under `<DATA_DIR>/cloud-rocksdb/`.
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

### Right-side aside

`apps/cloud/web/src/routes/+layout.svelte` mounts a fixed-width right-side aside that mirrors the player app's: `DocumentFilesPanel` (rendered when `documentPlaybackService` has a document selected), `PlayerVideo` (the playback surface — drives both yt-dlp direct streams and IPFS-pinned WebRTC sessions), and `SubsLyricsFinder` (talks to `/api/search/subs-lyrics`). The layout calls `playerService.initialize()` on mount so the aside's stores wake up; the `/api/player/stream-status` and `/api/player/playable` stubs let initialize settle without errors.

### `/youtube` route

`apps/cloud/web/src/routes/youtube/+page.svelte` is a self-contained yt-dlp UI ported from the player app. It talks **directly** to `/api/ytdl/*` via plain `fetch()` (no transport layer) — search, paste-URL info, queue audio/video/both, live progress via SSE on `/api/ytdl/downloads/events`, and "Stream" buttons that call `playerService.playUrl()` so the result plays in the right-side `PlayerVideo`.

## Running

```bash
# Dev — full desktop stack (cloud + Tauri shell + player Vite)
pnpm dev

# Dev — cloud independently with its own Tauri wrapper (Rust loopback :9899 + Vite WebUI :9898 + Tauri shell)
pnpm dev:cloud

# Dev — WebUI hot-reload on 9898 only (proxies /api to 127.0.0.1:9899; assumes the Rust server is already running, no Tauri)
pnpm dev:cloud:web

# Dev — Rust loopback server only on 127.0.0.1:9899 (no UI; for API-only work)
pnpm app:cloud

# Production build (embeds the WebUI)
pnpm build:cloud
```

## Environment Variables

- `PORT` — Server port (default: 9898; `pnpm app:cloud` / `pnpm dev:cloud` / `pnpm dev` set it to 9899 so Vite can own 9898)
- `HOST` — Bind address (default: 0.0.0.0; `pnpm app:cloud` / `pnpm dev:cloud` / `pnpm dev` set it to 127.0.0.1)
- `DB_PATH` — SurrealDB store path (default: `<home>/mhaol/cloud-rocksdb/`, resolved per-OS via `dirs::home_dir()`)
- `DATA_DIR` — If set and `DB_PATH` is unset, the store goes to `<DATA_DIR>/cloud-rocksdb/`
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
- `DELETE /api/libraries/:id` — remove the library record. Every `ipfs_pin` whose `path` lies under the library directory is unpinned from the embedded IPFS node and deleted from SurrealDB; the on-disk files and directory are left untouched.
- `GET /api/libraries/:id/scan` — recursively walk the library directory and return `{ root, total_files, total_size, entries }` where each entry is `{ path, relative_path, size, mime }`. MIME types are resolved by extension via `mime_guess`; the scan response itself is not persisted, but every entry whose mime starts with `audio/`, `video/`, or `image/` is asynchronously added to the embedded IPFS node (recursive pin) and recorded in the `ipfs_pin` table. The pin task waits for the IPFS node to reach `Running` state (up to ~60s) before it starts so the very first scan after server boot doesn't race the IPFS init. The library's `last_scanned_at` is updated on the record once the walk completes.
- `GET /api/libraries/:id/pins` — list pins from `ipfs_pin` whose `path` lies under this library's directory. Same shape as `GET /api/ipfs/pins`.
- `GET /api/ipfs/pins` — list every pin recorded by the cloud (`ipfs_pin` table). Each row is `{ id, cid, path, mime, size, created_at }`. Records are deduplicated by `(cid, path)` so re-scans don't create duplicates.
- `GET /api/documents` — list documents persisted in SurrealDB (`document` table).
- `POST /api/documents` — create a document `{ name, author, description? }`. `name` and `author` are required.
- `GET /api/documents/:id` — fetch one document.
- `PUT /api/documents/:id` — update `name`, `author`, or `description` (any subset).
- `DELETE /api/documents/:id` — remove the document record.
- `GET /api/database/tables` — list every table in the cloud SurrealDB database with its row count. Returns `{ namespace, database, tables: [{ name, record_count }] }`. Used by the embedded `/database` explorer.
- `GET /api/database/tables/:table?limit=<n>&offset=<n>` — paginate records in a single table. Table names are validated as `[A-Za-z0-9_]{1,64}`. `limit` defaults to 100 (max 1000); `offset` defaults to 0. Returns `{ table, limit, offset, total, records }` where each record is JSON with the SurrealDB `id` flattened to a `<table>:<id>` string.
- `GET /api/fs/browse?path=<optional>` — list subdirectories under `path` (defaults to the system home directory). Returns `{ path, parent, home, separator, roots, entries }` where `entries` only contains directories (hidden dot-folders are skipped). On Windows, `roots` lists available drive letters.
- `GET /api/catalog/sources` — list addons supported by the catalog browser. Each entry is `{ id, label, types: [{ id, label }], filterLabel, hasFilter }`. Currently `tmdb` (movies + tv shows, genre filter), `musicbrainz` (albums, genre filter), `openlibrary` (books, subject filter), and `retroachievements` (games, console filter).
- `GET /api/catalog/:addon/popular?type=<>&filter=<>&page=<>` — returns `{ items: [{ id, title, year, description, posterUrl, backdropUrl }], page, totalPages }` for the given addon. `type` is required for tmdb (`movie` or `tv`) and ignored for others. `filter` is the genre/subject/console id from `/genres`. TMDB and RetroAchievements need `TMDB_API_KEY` and `RA_USERNAME` + `RA_API_KEY` respectively; missing keys return `503`.
- `GET /api/catalog/:addon/genres?type=<>` — returns `[{ id, name }]` for the addon's filter dimension. TMDB requires `type=movie|tv` (queries `/genre/{type}/list` upstream); MusicBrainz/OpenLibrary/RetroAchievements return a static curated list (genres / subjects / console ids).
- `GET /api/torrent/list` — returns the cloud `TorrentManager`'s current torrents as `TorrentInfo[]` (`{ id, name, infoHash, size, progress, downloadSpeed, uploadSpeed, peers, seeds, state, addedAt, eta, outputPath }`). Returns `[]` while the session is still warming up. Used by the shared `DocumentCard` to render real-time progress.
- `POST /api/torrent/add` — adds a magnet to the cloud torrent client. Body: `{ magnet }`. Returns the initial `TorrentInfo`. `400` if the URI is not a magnet, `503` until the session has finished initializing.
- `POST /api/p2p-stream/sessions` — start a WebRTC streaming session for a previously pinned IPFS file. Body: `{ cid }`. Looks up the on-disk path in the `ipfs_pin` table, asks the `WorkerBridge` (a `mhaol-cloud worker` subprocess running `mhaol_p2p_stream::worker::run()`) to publish the file as a video stream into a fresh PartyKit room, and returns `{ sessionId, roomId, signalingUrl }`. The player connects to the same room and consumes the WebRTC stream via the existing `playerService.playRemote()`. `404` if the CID isn't pinned locally or the file is gone, `503` while the worker is still warming up.
- `POST /api/search/subs-lyrics` — body `{ type, query, externalIds?, languages? }`. For `type=track|album` queries LRCLIB by free-text query; for `type=movie|tv show|tv season|tv episode` queries Wyzie keyed by TMDB id (one entry per `externalIds[]`). Returns a flat `SubsLyrics[]`. Mirrors the node `/api/search/subs-lyrics` endpoint and powers the `SubsLyricsFinder` panel in the right-side aside.
- `GET /api/player/stream-status` — returns `{ available: false }`. The cloud has no local stream server; this stub keeps `playerService.initialize()` from rendering an error toast.
- `GET /api/player/playable` — returns `[]`. Cloud doesn't enumerate playable files like node does.
- `/api/ytdl/*` — full surface from `mhaol_yt_dlp::build_router(state.ytdl_manager)` mounted directly under the cloud router via `nest_service`. Includes `GET /search`, `GET /info/video`, `GET /info/stream-urls{,-browser}`, `GET /info/playlist`, `GET /downloads`, `POST /downloads`, `POST /downloads/playlist`, `GET /downloads/events` (SSE), `DELETE /downloads/{id}`, `DELETE /downloads/completed`, `DELETE /downloads/queue`, `GET|PUT /config`, `GET /status`, `GET /ytdlp/status`. The WebUI's `/youtube` page talks directly to this surface via plain `fetch('/api/ytdl/...')` (no transport layer). cfg(not(target_os = "android")).

## Document versioning

Documents are content-addressed: the SurrealDB record `id` is the CIDv1-raw of the document body (title, description, artists, images, files, year, type, source, version, version_hashes). Subs/lyrics are not stored on documents; the player has a sidebar finder that hits `/api/search/subs-lyrics` on its connected node. Two fields participate in this hash:

- `version: u32` — rolling-forward nonce, starts at `0`. Records persisted before this field existed deserialize as `0`.
- `version_hashes: Vec<String>` — CIDs of every prior version, oldest first. Chain integrity invariant: `version_hashes.len() == version`.

Whenever the document is updated programmatically (currently only the torrent-completion flow), the prior CID is pushed onto `version_hashes`, `version` is incremented, the new CID is computed over the full new body, the old record is deleted, and a new record is created at the new CID. Verifiers walk `version_hashes` backwards to rebuild the chain.

## Torrent → document auto-update

`apps/cloud/src/torrent_completion.rs` runs a background task that polls `TorrentManager::list()` every 5 seconds. When a torrent reaches `Seeding` (or `progress >= 1.0`):

1. Find the document whose `files` includes a `torrent magnet` whose value contains `btih:<info_hash>` (case-insensitive).
2. Walk the torrent's `output_path` recursively; skip files already represented as `ipfs` entries (matched by `title == relative_path`) so re-runs are idempotent.
3. For each remaining file: pin to the embedded IPFS node via `IpfsManager::add` and record the pin in `ipfs_pin`.
4. Append `{ type: "ipfs", value: <cid>, title: <relative_path> }` entries to `document.files`.
5. Roll the version forward (push old CID onto `version_hashes`, bump `version`), recompute the CID, delete the old record, create the new one at the new CID. `created_at` is preserved; `updated_at` is set to now.

Failures are logged and retried on the next tick; successes (including "no matching document") are remembered in-memory for the lifetime of the process so the same torrent isn't reprocessed.

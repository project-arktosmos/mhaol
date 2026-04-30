# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded RocksDB)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 9898)

The cloud server runs an embedded SurrealDB store, an identity manager, and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ed2k`, and `mhaol-ipfs`. It loads `mhaol-p2p-stream` for the GStreamer worker, and serves the Svelte WebUI from the nested `web/` directory.

The cloud also ships a desktop Tauri shell at `apps/cloud/src-tauri/` (crate `mhaol-cloud-shell`, productName "Mhaol Cloud"). The shell is **tray-only** — it never opens a window. `tauri.conf.json` has `app.windows: []`, on macOS the activation policy is set to `Accessory` (no dock icon), and `RunEvent::ExitRequested` is intercepted via `prevent_exit()` so the app stays alive without any windows. It registers a system tray icon (id `mhaol-cloud-tray`) on macOS/Windows/Linux with two menu items: **Open** opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`, **Quit** calls `app.exit(0)`. The cloud WebUI itself remains browser-accessible at `http://localhost:9898`.

## Source Structure

```
src/
├── server.rs            # Binary entry point — opens SurrealDB, builds router
├── paths.rs             # Single source of truth for on-disk paths under <data_root>
├── db.rs                # SurrealDB connection helper (RocksDB engine)
├── state.rs             # CloudState: { db, identity_manager, ytdl_manager, torrent_manager, ed2k_manager, ipfs_manager }
├── cloud_status.rs      # GET /api/cloud/status
├── users.rs             # /api/users — secp256k1 user registry (id = lowercased EVM address); register/login require an EIP-191 signature over a fresh `Mhaol Cloud auth at <RFC3339>` message
├── libraries.rs         # /api/libraries CRUD — SurrealDB-backed library records identified by their on-disk dir; carries a list of catalog `kinds` (movie / tv / album / book / game)
├── library_scan.rs      # Scan-time media detection + firkin persistence (cfg(not(target_os = "android")))
├── firkins.rs         # /api/firkins CRUD — SurrealDB-backed firkin records (id is a CIDv1-raw of the body); create also pins the body JSON to the embedded IPFS node and records an `ipfs_pin` row keyed `firkin://<id>`
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

## On-disk layout

Everything the cloud writes lives under a single root:

- Default: `<home>/mhaol-cloud/` — resolved via `dirs::home_dir()`, OS-aware (`~/mhaol-cloud/` on macOS/Linux, `%USERPROFILE%\mhaol-cloud\` on Windows).
- Override the root: set `DATA_DIR` to any path you like; everything below moves with it.

```
<data_root>/
├── db/                          # SurrealDB (RocksDB) store
├── identities/                  # Ethereum keystore (mhaol-identity)
├── swarm.key                    # IPFS PSK
├── rendezvous/
│   └── bootstrap.multiaddr      # written by the rendezvous app, read on startup
└── downloads/
    ├── torrents/                # mhaol-torrent — long-lived downloads
    ├── torrent-streams/         # ephemeral payloads for /api/torrent/stream sessions; wiped on every fresh stream
    ├── ed2k/                    # mhaol-ed2k
    ├── ipfs/                    # embedded IPFS repo (blockstore + datastore)
    ├── ipfs-stream/             # HLS segments produced by mhaol-ipfs-stream
    └── youtube/                 # yt-dlp output
```

Per-path env overrides still apply on top of `DATA_DIR`:

- `DB_PATH` — full path to the SurrealDB store (skips `<data_root>/db`).
- `IPFS_SWARM_KEY_FILE` — full path to the swarm key.
- `RENDEZVOUS_BOOTSTRAP_FILE` — full path to the bootstrap multiaddr file.
- `YTDL_OUTPUT_DIR` — full path to the yt-dlp output dir.

`apps/cloud/src/paths.rs` is the single source of truth for these defaults.

## Database

- Engine: **SurrealDB 2.x** with the embedded **RocksDB** kv backend. SurrealKV was tried first but hit [surrealdb/surrealdb#5064](https://github.com/surrealdb/surrealdb/issues/5064) — concurrent writes from background scan / pin / request handlers corrupted the store and reads panicked with `Invalid revision N for type Value`. RocksDB does not have this problem.
- Location: `<data_root>/db/` (see "On-disk layout" above).
- Namespace: `mhaol`, database: `cloud`.
- The store is created fresh on first boot. There are no schemas or repos defined yet — add tables/queries as features land.

## Packages loaded by cloud

The cloud crate directly depends on these mhaol packages and reports their health on `/api/cloud/status`:

- `mhaol-p2p-stream` — GStreamer worker subprocess + WebRTC streaming (cfg(not(target_os = "android"))).
- `mhaol-yt-dlp` — YouTube download manager (cfg(not(target_os = "android"))).
- `mhaol-torrent` — `librqbit`-backed torrent session, initialized in the background on startup so the server can bind quickly (cfg(not(target_os = "android"))).
- `mhaol-ed2k` — eDonkey/ed2k client (cfg(not(target_os = "android"))).
- `mhaol-ipfs` — embedded `rust-ipfs` node (libp2p, Bitswap, Kademlia DHT), initialized in the background on startup. The blockstore lives at `<data_root>/downloads/ipfs/` (cfg(not(target_os = "android"))). The node **always** runs on a **private swarm**: cloud reads (or auto-generates on first boot) a swarm key at `<data_root>/swarm.key` (override with `IPFS_SWARM_KEY_FILE`). Only nodes carrying that exact key can connect; the public bootstrap list is skipped, mDNS is off, and the transport stack is constrained to TCP+pnet+noise+yamux. Non-PSK peers fail at the libp2p `pnet` handshake before anything reaches Kademlia or the application — that is the only enforcement layer needed. If the swarm key cannot be loaded or generated the IPFS subsystem refuses to start (no fallback to the public swarm). The cloud bootstraps against the rendezvous node: precedence is `RENDEZVOUS_BOOTSTRAP` env var (newline- or comma-separated multiaddrs), then `<data_root>/rendezvous/bootstrap.multiaddr` (override with `RENDEZVOUS_BOOTSTRAP_FILE`), then a localhost default of `/ip4/127.0.0.1/tcp/14001`.

All download paths land under `<data_root>/downloads/{torrents,torrent-streams,ed2k,ipfs,ipfs-stream,youtube}`. The `torrents/` dir holds long-lived torrents (firkin auto-update flow); `torrent-streams/` is reserved for `/api/torrent/stream` payloads — those are deleted (torrent + on-disk files) on every new stream request. yt-dlp uses `<data_root>/downloads/youtube` by default and still honors `YTDL_OUTPUT_DIR`/`YTDL_PO_TOKEN`/`YTDL_VISITOR_DATA`/`YTDL_COOKIES`.

## WebUI

The Svelte app lives at `apps/cloud/web/` (pnpm package name `cloud`). The user-facing port is always **9898** in both modes:

- **Dev** — Vite binds `0.0.0.0:9898` and serves the live Svelte app with hot reload. The Rust server binds `127.0.0.1:9899` (loopback only, invisible to the network). Vite proxies `/api/*` to `127.0.0.1:9899`, so the browser only ever talks to 9898.
- **Production (release builds)** — the Rust server binds `0.0.0.0:9898` and embeds `apps/cloud/web/dist-static/` via `rust-embed`, serving it directly as the fallback for any non-API path. Build it with `pnpm --filter cloud build` (or `pnpm build:cloud` to build the WebUI and the release binary together).

### Right-side aside

`apps/cloud/web/src/routes/+layout.svelte` mounts a fixed-width right-side aside that mirrors the player app's: `FirkinFilesPanel` (rendered when `firkinPlaybackService` has a firkin selected), `PlayerVideo` (the playback surface — drives both yt-dlp direct streams and IPFS-pinned WebRTC sessions), and `SubsLyricsFinder` (talks to `/api/search/subs-lyrics`). The layout calls `playerService.initialize()` on mount so the aside's stores wake up; the `/api/player/stream-status` and `/api/player/playable` stubs let initialize settle without errors.

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
- `DATA_DIR` — Root directory for all cloud-managed state. Default: `<home>/mhaol-cloud/`. The DB, identities, swarm key, rendezvous bootstrap and downloads all sit under this root.
- `DB_PATH` — Override the SurrealDB store path specifically (default: `<data_root>/db/`).
- `SIGNALING_URL` — Base URL of the rendezvous WebSocket signaling server (default: `http://localhost:14080`). The cloud bakes this into passport metadata and propagates it to the GStreamer worker so `/api/p2p-stream/sessions` can return it to the player.
- `IPFS_SWARM_KEY_FILE` — Override the IPFS pre-shared swarm key path (default: `<data_root>/swarm.key`, auto-generated on first boot when missing). Note: the rendezvous app defaults to its own swarm key location; if you run both on the same machine, point one of them at the other's key (or symlink) so they share the same PSK.
- `RENDEZVOUS_BOOTSTRAP` — Newline- or comma-separated rendezvous multiaddrs to dial on startup (e.g. `/ip4/192.168.1.10/tcp/14001/p2p/12D3...`). Takes precedence over the bootstrap file.
- `RENDEZVOUS_BOOTSTRAP_FILE` — Override the rendezvous-written bootstrap multiaddr file path (default: `<data_root>/rendezvous/bootstrap.multiaddr`).
- `YTDL_OUTPUT_DIR` — Override the yt-dlp output directory (default: `<data_root>/downloads/youtube`).

## Worker subcommand

The binary still supports `mhaol-cloud worker`, which runs `mhaol_p2p_stream::worker::run()` for the GStreamer worker process. This subcommand does not touch the database or the identity manager.

## Public WebUI endpoints

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, signaling/client wallet addresses, db engine/namespace/version, and a `packages` block reporting health for `p2pStream`, `ytDlp`, `torrent`, `ed2k`, and `ipfs`. No auth required (used by the embedded WebUI).
- `GET /api/users` — list registered users (`{ address, username, created_at, updated_at, last_login_at }`).
- `GET /api/users/:address` — fetch one user by lowercased EVM address.
- `POST /api/users/register` — body `{ address, username, message, signature }`. Username is `[A-Za-z0-9-]{1,32}` (case-insensitively unique). The signature must be EIP-191 over the literal message `Mhaol Cloud auth at <RFC3339 timestamp>` (timestamp must be within ±5 minutes of the server's clock); the recovered address must equal `address`. Conflicts on duplicate address or username return `409`. The WebUI auto-registers a fresh keypair on first visit when `localStorage["mhaol-cloud-identity"]` is missing.
- `POST /api/users/login` — same auth shape as register; updates `last_login_at`. Returns `404` if the user has not registered yet.
- `PUT /api/users/:address` — body `{ username, message, signature }` rotates the username; the signature must come from the user's own private key.
- `GET /api/libraries` — list libraries persisted in SurrealDB (`library` table). Libraries have no name; each is identified by its directory path. Each record carries an `addons: string[]` field listing which `local-*` addons it serves (any subset of `local-movie`, `local-tv`, `local-album`, `local-book`, `local-game`). Records persisted under the prior schema (`kinds: ["movie", ...]`) are migrated automatically on read via a serde alias, but the values themselves don't change — set the addons explicitly via `PUT` to migrate to the new ids.
- `POST /api/libraries` — create a library `{ path, addons? }`. `addons` is an optional list of `local-*` addon ids; unknown ids are rejected with `400`. The directory is created on disk if it does not exist; duplicate paths are rejected with `409`.
- `GET /api/libraries/:id` — fetch one library.
- `PUT /api/libraries/:id` — update `path` (required) and optionally `addons`. The new path is created on disk if missing; duplicates are rejected with `409`. Omitting `addons` leaves the existing list untouched.
- `DELETE /api/libraries/:id` — remove the library record. Every `ipfs_pin` whose `path` lies under the library directory is unpinned from the embedded IPFS node and deleted from SurrealDB; the on-disk files and directory are left untouched.
- `GET /api/libraries/:id/scan` — recursively walk the library directory and return `{ root, total_files, total_size, entries }` where each entry is `{ path, relative_path, size, mime }`. MIME types are resolved by extension via `mime_guess`. The scan response itself is not persisted; the library's `last_scanned_at` is updated once the walk completes. After the walk, the scan handler hands off to `library_scan::schedule_pins_and_firkins` (see "Library scan → firkins" below). The pin task waits for the IPFS node to reach `Running` state (up to ~60s) before it starts so the very first scan after server boot doesn't race the IPFS init.
- `GET /api/libraries/:id/pins` — list pins from `ipfs_pin` whose `path` lies under this library's directory. Same shape as `GET /api/ipfs/pins`.
- `GET /api/ipfs/pins` — list every pin recorded by the cloud (`ipfs_pin` table). Each row is `{ id, cid, path, mime, size, created_at }`. Records are deduplicated by `(cid, path)` so re-scans don't create duplicates.
- `GET /api/firkins` — list firkins persisted in SurrealDB (`firkin` table). Superseded versions (any id appearing in another row's `version_hashes`) are filtered out so callers only see the head of each chain.
- `POST /api/firkins` — create a firkin `{ title, addon, description?, artists?, images?, files?, year? }`. `title` and `addon` are required (`addon` must be a known addon id; see `/api/catalog/sources` for browsable ones, plus the `local-*` family and the non-browsable `wyzie-subs-*`/`lrclib` ids). The id is the CIDv1-raw sha256 of the canonical pretty-printed JSON body, computed by `compute_firkin_cid`. Returns `200` with the existing record if a firkin with that id already exists, otherwise `201`. **Bookmark semantics**: in addition to the SurrealDB write, the handler pins the firkin's body JSON to the embedded IPFS node via `IpfsManager::add_bytes` (named `firkin-<id>.json`) and inserts an `ipfs_pin` row `{ cid: <unixfs cid>, path: "firkin://<id>", mime: "application/json", size }`. The IPFS pin is best-effort — failures are logged via `tracing::warn!` but do not fail the create, so the WebUI's Bookmark / torrent-pick flows still succeed while the IPFS node is warming up. Note: the `ipfs_pin.cid` is the UnixFS CID (which wraps the JSON in a UnixFS protobuf) and is not equal to the firkin's id (which is a raw-codec sha256 of the same bytes).
- `GET /api/firkins/:id` — fetch one firkin.
- `PUT /api/firkins/:id` — update `title`, `addon`, `description`, `artists`, `images`, `files`, `year` (any subset). Currently mutates in place at the existing id; the firkin id is **not** recomputed and the IPFS pin from the original create is **not** refreshed. Programmatic version-rollforward is the path that produces a new id (see "Firkin versioning"); this endpoint is for direct edits where preserving the id is intentional.
- `DELETE /api/firkins/:id` — remove the firkin record from SurrealDB. The IPFS pin row left by `POST /api/firkins` is currently not garbage-collected.
- `GET /api/database/tables` — list every table in the cloud SurrealDB database with its row count. Returns `{ namespace, database, tables: [{ name, record_count }] }`. Used by the embedded `/database` explorer.
- `GET /api/database/tables/:table?limit=<n>&offset=<n>` — paginate records in a single table. Table names are validated as `[A-Za-z0-9_]{1,64}`. `limit` defaults to 100 (max 1000); `offset` defaults to 0. Returns `{ table, limit, offset, total, records }` where each record is JSON with the SurrealDB `id` flattened to a `<table>:<id>` string.
- `GET /api/fs/browse?path=<optional>` — list subdirectories under `path` (defaults to the system home directory). Returns `{ path, parent, home, separator, roots, entries }` where `entries` only contains directories (hidden dot-folders are skipped). On Windows, `roots` lists available drive letters.
- `GET /api/catalog/sources` — list addons supported by the catalog browser. Each addon owns a single content kind (no nested `types`); each entry is `{ id, label, kind, filterLabel, hasFilter }`. Browsable addons are `tmdb-movie`, `tmdb-tv`, `musicbrainz`, `retroachievements`, `youtube-video`, `youtube-channel`, `openlibrary`, `iptv`, `radio`. Non-browsable addons (`wyzie-subs-movie`, `wyzie-subs-tv`, `lrclib`, and the `local-*` family used by libraries) are valid firkin `addon` values but don't appear in this list.
- `GET /api/catalog/:addon/popular?filter=<>&page=<>` — returns `{ items: [{ id, title, year, description, posterUrl, backdropUrl }], page, totalPages }` for the given addon. `filter` is the genre/subject/console/region/category/tag id from `/genres`. TMDB and RetroAchievements need `TMDB_API_KEY` and `RA_USERNAME` + `RA_API_KEY` respectively; missing keys return `503`.
- `GET /api/catalog/:addon/genres` — returns `[{ id, name }]` for the addon's filter dimension. `tmdb-movie`/`tmdb-tv` query `/genre/movie/list` and `/genre/tv/list` upstream; everyone else returns a static curated list (genres / subjects / console ids / regions / categories / tags).
- `GET /api/torrent/list` — returns the cloud `TorrentManager`'s current torrents as `TorrentInfo[]` (`{ id, name, infoHash, size, progress, downloadSpeed, uploadSpeed, peers, seeds, state, addedAt, eta, outputPath }`). Returns `[]` while the session is still warming up. Used by the shared `FirkinCard` to render real-time progress.
- `POST /api/torrent/add` — adds a magnet to the cloud torrent client. Body: `{ magnet }`. Returns the initial `TorrentInfo`. `400` if the URI is not a magnet, `503` until the session has finished initializing.
- `POST /api/torrent/evaluate` — body `{ magnet }`. Probe-only: resolves the magnet metadata via librqbit's `list_only` flag (DHT + tracker peer discovery + BEP 9/10 metadata exchange — no piece downloads, no on-disk side-effects, no session entries), checks for a streamable video file. Always returns 200; the JSON `streamable` field is the discriminator: `{ streamable: true, infoHash, name, fileIndex, fileName, fileSize, mimeType }` on success, `{ streamable: false, reason }` otherwise. The WebUI fires this on every catalog-detail mount that has a magnet attached, and only enables the "Torrent Stream" button when it returns `streamable: true`.
- `POST /api/torrent/stream` — body `{ magnet }`. Wipes any previously-started stream torrents (deletes them from the session and removes their on-disk files in `<data_root>/downloads/torrent-streams/`), resolves the magnet metadata via librqbit's `list_only` flag (same path as `/evaluate`), picks the largest streamable video file (mp4/mkv/webm/mov/avi/m4v/ogv/ts), then starts a `only_files=[idx]` download into `torrent-streams/`. Returns `{ infoHash, name, fileIndex, fileName, fileSize, mimeType, streamUrl }`.
- `GET /api/torrent/stream/:info_hash/:file_index` — serves the chosen file with HTTP byte-range support (`Accept-Ranges: bytes`, `206 Partial Content`, suffix-range parsing). Backed by librqbit's `FileStream` which lazy-fetches pieces and registers wakers for not-yet-available bytes, so the `<video>` element drives piece priority via Range requests.
- `POST /api/p2p-stream/sessions` — start a WebRTC streaming session for a previously pinned IPFS file. Body: `{ cid }`. Looks up the on-disk path in the `ipfs_pin` table, asks the `WorkerBridge` (a `mhaol-cloud worker` subprocess running `mhaol_p2p_stream::worker::run()`) to publish the file as a video stream into a fresh PartyKit room, and returns `{ sessionId, roomId, signalingUrl }`. The player connects to the same room and consumes the WebRTC stream via the existing `playerService.playRemote()`. `404` if the CID isn't pinned locally or the file is gone, `503` while the worker is still warming up.
- `POST /api/search/subs-lyrics` — body `{ addon, query, externalIds?, languages? }`. The addon implies the source: `lrclib` / `musicbrainz` / `local-album` route through LRCLIB by free-text query; `tmdb-movie` / `tmdb-tv` / `wyzie-subs-movie` / `wyzie-subs-tv` / `local-movie` / `local-tv` route through Wyzie keyed by TMDB id (one entry per `externalIds[]`). Returns a flat `SubsLyrics[]`. Mirrors the node `/api/search/subs-lyrics` endpoint and powers the `SubsLyricsFinder` panel in the right-side aside.
- `GET /api/player/stream-status` — returns `{ available: false }`. The cloud has no local stream server; this stub keeps `playerService.initialize()` from rendering an error toast.
- `GET /api/player/playable` — returns `[]`. Cloud doesn't enumerate playable files like node does.
- `/api/ytdl/*` — full surface from `mhaol_yt_dlp::build_router(state.ytdl_manager)` mounted directly under the cloud router via `nest_service`. Includes `GET /search`, `GET /info/video`, `GET /info/stream-urls{,-browser}`, `GET /info/playlist`, `GET /downloads`, `POST /downloads`, `POST /downloads/playlist`, `GET /downloads/events` (SSE), `DELETE /downloads/{id}`, `DELETE /downloads/completed`, `DELETE /downloads/queue`, `GET|PUT /config`, `GET /status`, `GET /ytdlp/status`. The WebUI's `/youtube` page talks directly to this surface via plain `fetch('/api/ytdl/...')` (no transport layer). cfg(not(target_os = "android")).

## Library scan → firkins

`apps/cloud/src/library_scan.rs` runs after every `/api/libraries/:id/scan` and turns the walked entries into `firkin` records. Behavior depends on the library's `addons`:

- Empty `addons`: legacy behavior. Every entry whose mime starts with `audio/`, `video/`, or `image/` is pinned to IPFS and recorded in `ipfs_pin`. No firkins are created.
- Non-empty `addons`: the entries are classified per addon and grouped into media items. Each group's files are pinned to IPFS, recorded in `ipfs_pin`, and persisted as a `firkin` whose `files` are the `ipfs` entries (`{ type: "ipfs", value: <cid>, title: <relative_path-or-display-title> }`). Files that are pinnable but didn't fall into any group are still pinned (kept reachable for `/api/libraries/:id/pins`).

Detection rules (one-doc-per-group; the `firkin.addon` records the local addon directly):

- `local-movie`: one firkin per video file. Title is taken from the parent directory name (or the filename if the file sits at the library root). A trailing `(YYYY)` tag is parsed into `year`. Video files that the TV detector consumed are skipped to avoid double-counting.
- `local-tv`: one firkin per show. Detection looks for either a parent directory matching `Season N` / `S01` (the show name is the directory above it) or a `S<season>E<episode>` / `<season>x<episode>` token in the filename (the show name is the top-level directory under the library, or the filename if it sits at the root). All matched episodes are appended as `ipfs` file entries with titles formatted `S01E02 - <filename>`. Re-scans append new episodes via the firkin version-roll (see "Firkin versioning") so existing CIDs are preserved as `version_hashes`.
- `local-album`: one firkin per directory containing audio files. Album title is the directory name; loose audio at the library root is grouped under `Singles`. Tracks are sorted by leading number prefix (`01 - …`) when available.
- `local-book`: one firkin per file matching a book extension (epub, pdf, mobi, azw3, cbz, cbr, djvu, fb2). Title from the filename, with `(YYYY)` parsed out.
- `local-game`: one firkin per file matching a game/ROM extension (iso, rom, smc, sfc, gba, nes, gb, gbc, n64, z64, v64, md, sms, gg, nds, 3ds, wad, cue, chd, gcm).

Re-running a scan is idempotent: existing firkins with the same `(title, addon)` are matched and version-rolled forward with any new file entries; files already present (matched by their `title`) are skipped.

## Firkin versioning

Firkins are content-addressed: the SurrealDB record `id` is the CIDv1-raw of the firkin body (title, description, artists, images, files, year, addon, version, version_hashes). The `addon` field replaces the prior split between `type` and `source` — each addon owns a single content kind, so the kind is implicit in the addon id. Subs/lyrics are not stored on firkins; the player has a sidebar finder that hits `/api/search/subs-lyrics` on its connected node. Two fields participate in this hash:

- `version: u32` — rolling-forward nonce, starts at `0`. Records persisted before this field existed deserialize as `0`.
- `version_hashes: Vec<String>` — CIDs of every prior version, oldest first. Chain integrity invariant: `version_hashes.len() == version`.

Whenever the firkin is updated programmatically (currently only the torrent-completion flow), the prior CID is pushed onto `version_hashes`, `version` is incremented, the new CID is computed over the full new body, the old record is deleted, and a new record is created at the new CID. Verifiers walk `version_hashes` backwards to rebuild the chain.

The body is also pinned to IPFS twice: once at create time via `POST /api/firkins` (the "bookmark" pin), and again at every version-rollforward (the new body is added via `IpfsManager::add_bytes` and the new `firkin://<id>` row is recorded). Each version's body is therefore independently retrievable from the swarm by its UnixFS CID; the firkin's own `id` (a raw-codec sha256 of the same JSON) remains the canonical SurrealDB key and is what `version_hashes` references.

## Torrent → firkin auto-update

`apps/cloud/src/torrent_completion.rs` runs a background task that polls `TorrentManager::list()` every 5 seconds. When a torrent reaches `Seeding` (or `progress >= 1.0`):

1. Find the firkin whose `files` includes a `torrent magnet` whose value contains `btih:<info_hash>` (case-insensitive).
2. Walk the torrent's `output_path` recursively; skip files already represented as `ipfs` entries (matched by `title == relative_path`) so re-runs are idempotent.
3. For each remaining file: pin to the embedded IPFS node via `IpfsManager::add` and record the pin in `ipfs_pin`.
4. Append `{ type: "ipfs", value: <cid>, title: <relative_path> }` entries to `firkin.files`.
5. Roll the version forward (push old CID onto `version_hashes`, bump `version`), recompute the CID, delete the old record, create the new one at the new CID. `created_at` is preserved; `updated_at` is set to now.

Failures are logged and retried on the next tick; successes (including "no matching firkin") are remembered in-memory for the lifetime of the process so the same torrent isn't reprocessed.

## Logs

Dev runs tee full stdout+stderr to `<repo-root>/logs/`:

- `pnpm dev` cloud strand → `logs/cloud.log`
- `pnpm dev` web (Vite) strand → `logs/web.log`
- `pnpm dev` tauri strand → `logs/tauri.log`

When debugging the cloud, read these files directly — don't ask the user to paste output. Each file is overwritten on the next run.

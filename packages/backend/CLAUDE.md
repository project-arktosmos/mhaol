# Backend

**Location:** `packages/backend/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded RocksDB)
**Crate:** `mhaol-backend` (library)
**Binary:** `mhaol-cloud` (default port 9898)

The backend crate runs an embedded SurrealDB store, an identity manager, and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ipfs-core`, and `mhaol-ipfs-stream`. It serves `/api/*` plus the embedded frontend SPA from `packages/frontend/dist-static/` (statically embedded at compile time via `rust-embed`).

The library exposes a single async entry: `pub async fn mhaol_backend::run()` (defined in `src/lib.rs`). The standalone bin at `src/bin/mhaol-cloud.rs` is a `#[tokio::main]` shim that calls `run().await`. The Tauri shell at `apps/cloud/` runs the standalone bin alongside the tray.

## Source Structure

```
src/
├── lib.rs               # Library entry point — exposes `pub async fn run()` (boots SurrealDB, builds the Axum router)
├── bin/mhaol-cloud.rs   # Standalone binary — #[tokio::main] async fn main() { mhaol_backend::run().await }
├── paths.rs             # Single source of truth for on-disk paths under <data_root>
├── db.rs                # SurrealDB connection helper (RocksDB engine)
├── state.rs             # CloudState: { db, identity_manager, ytdl_manager, torrent_manager, ipfs_manager, ipfs_stream_manager, track_progress, ytdl_channel_cache }
├── cloud_status.rs      # GET /api/cloud/status
├── users.rs             # /api/users — secp256k1 user registry (id = lowercased EVM address); register/login require an EIP-191 signature over a fresh `Mhaol Cloud auth at <RFC3339>` message
├── libraries.rs         # /api/libraries CRUD — SurrealDB-backed library records identified by their on-disk dir; carries an `addons` list of `local-*` ids
├── library_scan.rs      # Library scan side-effects: hash every matched file via `IpfsManager::compute_file_cid` (fast path: drives the filestore index — leaf blocks become `(path, offset, length)` references, link blocks land in the inner blockstore — so a 200 GB TV library produces megabytes of inner-tree blocks instead of 200 GB of duplicated leaf bytes). Records a lite `ipfs_pin` row per file. No firkin creation. cfg(not(target_os = "android"))
├── filestore_index.rs   # SurrealDB-backed `FilestoreIndex` impl (`ipfs_filestore_entry` table, in-process `HashMap<Cid, FilestoreEntry>` cache loaded at boot). Wired into `IpfsConfig::filestore_index` so the embedded IPFS node initialises with a `FilestoreBlockStore` decorator: bitswap reads leaf blocks by reading the original file at the recorded offset and re-running `FileAdder` to reconstruct the same UnixFS-wrapped block. cfg(not(target_os = "android"))
├── tv_build.rs          # `POST /api/libraries/:id/tv-build` background TV-show firkin builder. Spawns a `tokio::spawn` task that searches TMDB, fetches every season + episode + show metadata, then mints one `tmdb-tv` firkin via `firkins::create_firkin_record` with every episode as a `url` placeholder so the WebUI can navigate to it before any pin lands. As each local file's IPFS pin settles, the matching `(S, E)` `url` entry is rewritten to an `ipfs` CID and the firkin is rolled forward via `firkins::rollforward_firkin` (new CID, prior CID pushed onto `version_hashes`, body re-pinned). Survives page reloads; live state on `state.tv_build_progress`. cfg(not(target_os = "android"))
├── tv_build_progress.rs # In-memory `TvBuildProgressMap` keyed by `<library_id>::<show>::<year?>`; the libraries page polls `GET /api/libraries/:id/tv-builds` to render phase + current/total counters and re-hydrate after a refresh. cfg(not(target_os = "android"))
├── firkins.rs           # /api/firkins CRUD — SurrealDB-backed firkin records (stable UUID id, content-addressed `cid` field); create also pins the body JSON to the embedded IPFS node and records an `ipfs_pin` row keyed `firkin://<id>`. Firkins reference artists by CID (see `artists.rs`); incoming creates speak in inline artist objects which the server upserts before computing the firkin CID. Carries a `bookmarked` flag (default `true`, **not** part of the CID body) — the `/catalog/visit` resolver creates non-bookmarked browse-cache rows; the listing endpoint defaults to `bookmarked === true` only.
├── artists.rs           # /api/artists CRUD — SurrealDB-backed artist records (`artist` table). Each artist body is `{ name, roles: string[], imageUrl? }`; the SurrealDB id is `CIDv1-raw(sha256(normalised_name))` — *only* the name participates in the content-address (lowercased + whitespace-collapsed). Upserts merge the inbound single `role` into the existing record's `roles` array (deduped) and back-fill `imageUrl` when missing, so the same person across many firkins collapses into one record. Each merge re-pins the full body to IPFS (`artist://<id>` row).
├── database.rs          # /api/database/tables{,/:table} — read-only SurrealDB explorer (lists tables, paginates records)
├── disk.rs              # /api/disk — host disk inventory (mount, fs, total/available/used) plus per-subdir size breakdown of `<data_root>` (db, identities, swarm.key, downloads/{torrents,torrent-streams,ipfs,ipfs-stream,youtube} + any extras)
├── ipfs_pins.rs         # /api/ipfs/pins — lists pins recorded when libraries are scanned; `/api/ipfs/pins/:cid/file` streams the on-disk file for a pinned object (used by the WASM emulator modal); `/api/ipfs/pins/:cid/materialise` does the lazy `put_block` step for a lite-pinned CID; exposes `record_pin()` (materialised) + `record_lite_pin()` (compute-only) used by the scan / firkin / artist / rom-extract paths
├── media_trackers.rs    # /api/media-trackers — per-(firkin, track?, user) playback time totals. `POST /heartbeat` upserts a `media_tracker` row keyed `sha256(firkin_id:address[:track_id])` and adds the supplied `deltaSeconds`
├── recommendations.rs   # /api/recommendations — per-(user, recommended firkin) counts. `POST /ingest` dedupes per (user, source firkin) via the `recommendation_source` marker table. `POST /action` upserts a `recommendation_action` row (one per user × recommendation) recording `like` / `discard` / `bookmark`; `GET /?excludeActioned=true` filters those rows out and is what the `/feed` page consumes. `POST /rating` writes a 1–100 `userRating` directly onto the matching `recommendation` row; the listing's sort uses it as the secondary key (between `count` and the upstream review-rating tiebreaker), treating missing values as 0.
├── fs_browse.rs         # /api/fs/browse — list subdirectories under a path (defaults to home), used by the frontend directory picker
├── catalog.rs           # /api/catalog/* — proxies popular items + genres for tmdb / musicbrainz / youtube
├── search.rs            # /api/search/* — TMDB + ThePirateBay + LRCLIB lyrics + Wyzie subtitle proxy
├── tmdb_match.rs        # Per-file TMDB match used at scan time for `local-movie` libraries. cfg(not(target_os = "android"))
├── player.rs            # /api/player/{stream-status,playable} — stubs so `playerService.initialize()` settles cleanly in the frontend
├── ytdl.rs              # /api/ytdl/* — mounts `mhaol_yt_dlp::build_router(state.ytdl_manager)`. cfg(not(target_os = "android"))
├── ytdl_channel_cache.rs# In-memory cache for `/api/ytdl/channel/*` (video id → channel id, long TTL; channel id → parsed Atom feed, short TTL). cfg(not(target_os = "android"))
└── frontend.rs          # rust-embed wrapper that serves ../frontend/dist-static/
```

The bin is wired up via Cargo:

```toml
[lib]
name = "mhaol_backend"
path = "src/lib.rs"

[[bin]]
name = "mhaol-cloud"
path = "src/bin/mhaol-cloud.rs"
```

Path-deps reference sibling packages directly: `mhaol-identity = { path = "../identity" }`, etc.

## On-disk layout

Everything the backend writes lives under a single root:

- Default: `<home>/mhaol-cloud/` — resolved via `dirs::home_dir()`, OS-aware (`~/mhaol-cloud/` on macOS/Linux, `%USERPROFILE%\mhaol-cloud\` on Windows).
- Override the root: set `DATA_DIR` to any path you like; everything below moves with it.

```
<data_root>/
├── db/                          # SurrealDB (RocksDB) store
├── identities/                  # Ethereum keystore (mhaol-identity)
├── swarm.key                    # IPFS PSK
└── downloads/
    ├── torrents/                # mhaol-torrent — long-lived downloads
    ├── torrent-streams/         # ephemeral payloads for /api/torrent/stream sessions; wiped on every fresh stream
    ├── ipfs/                    # embedded IPFS repo (blockstore + datastore)
    ├── ipfs-stream/             # HLS segments produced by mhaol-ipfs-stream
    └── youtube/                 # yt-dlp output
```

Per-path env overrides still apply on top of `DATA_DIR`:

- `DB_PATH` — full path to the SurrealDB store (skips `<data_root>/db`).
- `IPFS_SWARM_KEY_FILE` — full path to the swarm key.
- `YTDL_OUTPUT_DIR` — full path to the yt-dlp output dir.

`packages/backend/src/paths.rs` is the single source of truth for these defaults.

## Database

- Engine: **SurrealDB 2.x** with the embedded **RocksDB** kv backend. SurrealKV was tried first but hit [surrealdb/surrealdb#5064](https://github.com/surrealdb/surrealdb/issues/5064) — concurrent writes from background scan / pin / request handlers corrupted the store and reads panicked with `Invalid revision N for type Value`. RocksDB does not have this problem.
- Location: `<data_root>/db/` (see "On-disk layout" above).
- Namespace: `mhaol`, database: `cloud`.
- The store is created fresh on first boot. There are no schemas or repos defined yet — add tables/queries as features land.

## Packages loaded by the backend

The crate directly depends on these mhaol packages and reports their health on `/api/cloud/status`:

- `mhaol-yt-dlp` — YouTube download manager (cfg(not(target_os = "android"))).
- `mhaol-torrent` — `librqbit`-backed torrent session, initialized in the background on startup so the server can bind quickly (cfg(not(target_os = "android"))).
- `mhaol-ipfs-core` — embedded `rust-ipfs` node (libp2p, Bitswap, Kademlia DHT), initialized in the background on startup. The blockstore lives at `<data_root>/downloads/ipfs/` (cfg(not(target_os = "android"))). The node **always** runs on a **private swarm**: the backend reads (or auto-generates on first boot) a swarm key at `<data_root>/swarm.key` (override with `IPFS_SWARM_KEY_FILE`). Only nodes carrying that exact key can connect; the public bootstrap list is skipped and the transport stack is constrained to TCP+WS+pnet+noise+yamux. Non-PSK peers fail at the libp2p `pnet` handshake before anything reaches Kademlia or the application. If the swarm key cannot be loaded or generated the IPFS subsystem refuses to start (no fallback to the public swarm). Discovery on the LAN is **mDNS-based** (no standalone bootstrap node required); two backend instances on the same network find each other automatically. **Listen ports** are fixed: TCP `9900` (`MHAOL_IPFS_TCP_PORT`) and WebSocket `9901` (`MHAOL_IPFS_WS_PORT`); the WebSocket listener exists so any future browser-resident peer can dial the swarm directly. The backend surfaces its own peer id, swarm key, and dialable multiaddrs via `GET /api/p2p/bootstrap` for that purpose.
- `mhaol-ipfs-stream` — HLS-over-IPFS streaming via GStreamer hlssink2.

All download paths land under `<data_root>/downloads/{torrents,torrent-streams,ipfs,ipfs-stream,youtube}`. The `torrents/` dir holds long-lived torrents (firkin auto-update flow); `torrent-streams/` is reserved for `/api/torrent/stream` payloads — those are deleted (torrent + on-disk files) on every new stream request. yt-dlp uses `<data_root>/downloads/youtube` by default and still honors `YTDL_OUTPUT_DIR`/`YTDL_PO_TOKEN`/`YTDL_VISITOR_DATA`/`YTDL_COOKIES`.

## Running

```bash
# Dev — backend bin only on 127.0.0.1:9899 (no UI; for API-only work)
pnpm app:cloud

# Dev — full desktop stack (backend + Vite frontend + Tauri tray shell)
pnpm dev

# Production build — embeds the SPA into the release bin
pnpm build:cloud
```

In dev, the bin binds `127.0.0.1:9899` and Vite owns `0.0.0.0:9898` (proxying `/api/*` → `127.0.0.1:9899`). In production, the release bin binds `0.0.0.0:9898` and serves the embedded `packages/frontend/dist-static/` directly as the fallback for non-API paths.

## Environment Variables

- `PORT` — Server port (default: 9898; `pnpm app:cloud` / `pnpm dev` set it to 9899 so Vite can own 9898)
- `HOST` — Bind address (default: 0.0.0.0; `pnpm app:cloud` / `pnpm dev` set it to 127.0.0.1)
- `DATA_DIR` — Root directory for all backend-managed state. Default: `<home>/mhaol-cloud/`. The DB, identities, swarm key, and downloads all sit under this root.
- `DB_PATH` — Override the SurrealDB store path specifically (default: `<data_root>/db/`).
- `IPFS_SWARM_KEY_FILE` — Override the IPFS pre-shared swarm key path (default: `<data_root>/swarm.key`, auto-generated on first boot when missing).
- `MHAOL_IPFS_TCP_PORT` — Override the embedded IPFS node's libp2p TCP listen port (default: `9900`). Useful for running multiple backend instances on one machine.
- `MHAOL_IPFS_WS_PORT` — Override the embedded IPFS node's libp2p WebSocket listen port (default: `9901`). Any future browser-resident peer can dial this address via `/api/p2p/bootstrap`.
- `YTDL_OUTPUT_DIR` — Override the yt-dlp output directory (default: `<data_root>/downloads/youtube`).

## Public API endpoints

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, the client wallet address, db engine/namespace/version, and a `packages` block reporting health for `ytDlp`, `torrent`, and `ipfs`. No auth required.
- `GET /api/p2p/bootstrap` — JSON `{ peerId, swarmKey, multiaddrs }` so any future browser-resident peer can dial the backend's libp2p node and join the same private swarm. `multiaddrs` is filtered to browser-dialable transports (`/ws`, `/wss`, `/webtransport`) and `0.0.0.0` is rewritten to loopback + the backend's primary LAN IP. Returns `503` with `Retry-After: 1` while the IPFS subsystem is still starting; callers should poll every second until ready. Trust boundary: anyone who can reach the backend's HTTP server is presumed LAN-trusted, so the swarm key is served as plain JSON over plain HTTP.
- `GET /api/users` — list registered users (`{ address, username, created_at, updated_at, last_login_at }`).
- `GET /api/users/:address` — fetch one user by lowercased EVM address.
- `POST /api/users/register` — body `{ address, username, message, signature }`. Username is `[A-Za-z0-9-]{1,32}` (case-insensitively unique). The signature must be EIP-191 over the literal message `Mhaol Cloud auth at <RFC3339 timestamp>` (timestamp must be within ±5 minutes of the server's clock); the recovered address must equal `address`. Conflicts on duplicate address or username return `409`. The frontend auto-registers a fresh keypair on first visit when `localStorage["mhaol-cloud-identity"]` is missing.
- `POST /api/users/login` — same auth shape as register; updates `last_login_at`. Returns `404` if the user has not registered yet.
- `PUT /api/users/:address` — body `{ username, message, signature }` rotates the username; the signature must come from the user's own private key.
- `GET /api/libraries` — list libraries persisted in SurrealDB (`library` table). Libraries have no name; each is identified by its directory path. Each record carries an `addons: string[]` field listing which `local-*` addons it serves (any subset of `local-movie`, `local-tv`, `local-album`, `local-book`, `local-game`).
- `POST /api/libraries` — create a library `{ path, addons? }`. `addons` is an optional list of `local-*` addon ids; unknown ids are rejected with `400`. The directory is created on disk if it does not exist; duplicate paths are rejected with `409`.
- `GET /api/libraries/:id` — fetch one library.
- `PUT /api/libraries/:id` — update `path` (required) and optionally `addons`. The new path is created on disk if missing; duplicates are rejected with `409`. Omitting `addons` leaves the existing list untouched.
- `DELETE /api/libraries/:id` — remove the library record. Every `ipfs_pin` whose `path` lies under the library directory is unpinned from the embedded IPFS node and deleted from SurrealDB; the on-disk files and directory are left untouched.
- `GET /api/libraries/:id/scan` — recursively walk the library directory and return `{ root, total_files, total_size, entries }` where each entry is `{ path, relative_path, size, mime, tmdbMatch? }`. MIME types are resolved by extension via `mime_guess`. The library's `last_scanned_at` is updated once the walk completes, and the scan response itself is persisted to the `library_scan_result` table. When the library's `addons` includes `local-movie`, every video entry runs through `tmdb_match::match_movies_parallel` (bounded concurrency 5). After the walk, the scan handler hands off to `library_scan::schedule_pins` (background task that pins every matched file to IPFS). **No firkin records are created from a library scan.**
- `GET /api/libraries/:id/scan-result` — returns the most recent persisted scan response from the `library_scan_result` table.
- `GET /api/libraries/:id/pins` — list pins from `ipfs_pin` whose `path` lies under this library's directory.
- `POST /api/libraries/:id/tv-build` — body `{ show, year?, files: [{ path, season, episode }] }`. Spawns a background task that matches the show to TMDB, fetches every season + episode + show metadata, then immediately mints one `tmdb-tv` firkin via `firkins::create_firkin_record`. The initial firkin body carries the canonical `https://www.themoviedb.org/tv/<id>` URL plus one `url` placeholder per TMDB episode pointing at `https://www.themoviedb.org/tv/<id>/season/<s>/episode/<e>` (and stamps `completed_firkin_id` on the progress entry as soon as the create returns, before any pin has landed). The task then awaits each local file's IPFS pin sequentially; on each pin landing the matching `(S, E)` placeholder is rewritten to an `ipfs` CID and the firkin is rolled forward via `firkins::rollforward_firkin` (new CID, prior CID appended to `version_hashes`, body re-pinned). Files whose pins never land within 120s keep their `url` placeholder. Returns 202 with the initial `TvBuildProgress`. Concurrent kicks of the same `<library_id>::<show>::<year>` job key surface the in-flight progress as 200 instead of starting a duplicate.
- `GET /api/libraries/:id/tv-builds` — list every `TvBuildProgress` row in the in-memory map for this library (active + recently-completed). The libraries page polls this every 2s while any non-terminal job is in flight and re-hydrates the in-progress badge on every page mount, so leaving the page or reloading does not interrupt the build.
- `DELETE /api/libraries/:id/tv-builds` — clear all terminal (`completed` / `error`) entries for this library. Active jobs are left alone.
- `GET /api/ipfs/pins` — list every pin recorded by the backend (`ipfs_pin` table). Each row is `{ id, cid, path, mime, size, materialised, created_at }`. Records are deduplicated by `(cid, path)`. `materialised: true` means the file's UnixFS blocks live in the blockstore and the CID is reachable via bitswap; `false` means only the CID was computed (library-scan lite mode) — the file is still streamable via `/file` since we hold the on-disk path, but other peers can't bitswap-fetch it until materialisation.
- `GET /api/ipfs/pins/:cid/file` — stream the on-disk bytes for a pinned object. Looks up the pin by CID, rejects metadata pins (`firkin://…`, `artist://…`), and serves the file with `Content-Type` from the pin row. Works for both materialised and lite pins (reads the file directly from disk regardless).
- `POST /api/ipfs/pins/:cid/materialise` — lazy `put_block` for a lite-pinned CID. Re-runs `IpfsManager::add` against the on-disk path, verifies the recomputed CID matches what the pin row claims (409 if the file changed), pins the result, and flips `materialised` to `true`. Idempotent on already-materialised pins. Returns the updated `IpfsPinDto`.
- `GET /api/firkins` — list firkins persisted in SurrealDB (`firkin` table). Defaults to `bookmarked === true` only — the catalog `/catalog/visit` resolver creates non-bookmarked browse-cache firkins on every catalog grid click, and they would otherwise clutter the catalog "Library" section, the `/firkins` page, and the recommendations table. Pass `?include=all` to get every row. Results are passed through `collapse_to_chain_heads`: superseded versions are dropped, parallel chains with the same `(addon, title, year)` are collapsed to the head with the higher `version`. Each row carries `artistIds` (CIDs of the referenced `artist` records, drives the firkin's own CID) and the resolved `artists` (server-side join).
- `POST /api/firkins` — create a firkin `{ title, addon, description?, artists?, images?, files?, year?, creator?, bookmarked? }`. The firkin record id is a stable UUID; the body is content-addressed via the `cid` field (CIDv1-raw sha256 of the canonical pretty-printed body). The `bookmarked` flag defaults to `true` (preserves the legacy bookmark-on-create contract); the catalog `/catalog/visit` resolver flow sends `false` to create a browse-cache firkin. Dedup by content-address: if a firkin with the same body already exists the existing row is returned (`200`), otherwise a new row is created (`201`). When the request bookmarks an already-existing browse-cache firkin (incoming `bookmarked: true`, existing `bookmarked: false`) the flag is upgraded in place via a no-CID-roll update. **Bookmark semantics**: in addition to the SurrealDB write, the handler pins the firkin's body JSON to the embedded IPFS node via `IpfsManager::add_bytes` and inserts an `ipfs_pin` row `{ cid: <unixfs cid>, path: "firkin://<id>", mime: "application/json", size }`. For fresh **bookmarked** `musicbrainz` albums (and on the false→true bookmark flip via `PUT`), the handler also spawns `spawn_resolve_album_tracks(state, id)` as a fire-and-forget `tokio::spawn` task — the browser never participates, so closing the tab never interrupts it. Browse-cache musicbrainz firkins skip the resolver until the user actually bookmarks.
- `GET /api/firkins/:id` — fetch one firkin.
- `PUT /api/firkins/:id` — update any subset of `title`, `addon`, `description`, `artists`, `images`, `files`, `year`, `trailers`, `reviews`, `bookmarked`. Applies the mutation through `rollforward_firkin`: when the new body produces a different CID, the prior CID is pushed onto `version_hashes`, `version` is incremented, the record is updated in place at its stable UUID id, and the new body JSON is re-pinned to IPFS. The `bookmarked` field is **not** part of `serialize_firkin_payload` / `compute_firkin_cid`, so flipping it in isolation persists without rolling the version. A false→true bookmark flip on a `musicbrainz` firkin spawns `spawn_resolve_album_tracks` as a background task.
- `DELETE /api/firkins/:id` — remove the firkin record from SurrealDB. The IPFS pin row left by `POST /api/firkins` is currently not garbage-collected.
- `POST /api/firkins/:id/resolve-tracks` — explicit / manual variant of the server-side per-track resolver for `musicbrainz` firkins. Loads the firkin, extracts the MusicBrainz release-group id from its `files`, fetches the album's tracklist, then for each track that's still missing a YouTube URL or lyrics entry runs both lookups in parallel via `mhaol_yt_dlp::search::search_query` and `crate::search::lrclib_search_raw`. Each resolved track adds a `url`-typed FileEntry (YouTube watch URL) and a `lyrics`-typed FileEntry whose `value` is the JSON `{ source: "lrclib", externalId, syncedLyrics, plainLyrics, instrumental }`. After all tracks are walked, the firkin is rolled forward via `rollforward_firkin`. Non-musicbrainz firkins return `400`.
- `GET /api/firkins/:id/resolution-progress` — live per-track progress for an in-flight album resolution. Backed by `state.track_progress` (in-memory `Arc<RwLock<HashMap<firkin_id, AlbumProgress>>>` defined in `src/track_progress.rs`).
- `POST /api/firkins/:id/enrich` — apply catalog-derived metadata to a firkin and roll its version forward. Body: `{ title?, year?, description?, posterUrl?, backdropUrl? }`.
- **Firkin `reviews`**: firkin records carry an optional `reviews: [{ label, score, maxScore, voteCount? }]` array — TMDB contributes `{ label: "TMDB", score: vote_average, maxScore: 10, voteCount: vote_count }` and MusicBrainz contributes `{ label: "MusicBrainz", score: rating.value, maxScore: 5, voteCount: rating.votes-count }`. Both are extracted server-side by `GET /api/catalog/:addon/:id/metadata`. Reviews with zero votes are filtered out. The field participates in `compute_firkin_cid`.
- **Firkin `trailers`**: firkin records carry an optional `trailers: [{ youtubeUrl, label?, language? }]` array — movies hold one entry; TV shows hold one show-level entry plus one per season (with `label` set to the season name). `language` is the ISO 639-1 code (e.g. `"en"`) when known. The primary source is TMDB; the frontend runs a YouTube fuzzy search as a fallback for items / seasons TMDB has no English trailer for.
- `GET /api/media-trackers?firkinId=<>&trackId=<>&address=<>` — list rows from the `media_tracker` table.
- `POST /api/media-trackers/heartbeat` — body `{ firkinId, trackId?, trackTitle?, address, deltaSeconds }`. Upserts the tracker row keyed by `sha256(firkinId:address[:trackId])`, adds `deltaSeconds` to `totalSeconds`, and stamps `last_played_at`.
- `GET /api/recommendations?address=<>&excludeActioned=<bool>` — list rows from the `recommendation` table for the given user. Sort: `count DESC, userRating DESC (None → 0), mean(score / maxScore across reviews) DESC, updated_at DESC`. `excludeActioned=true` drops any row the user has acted on (`recommendation_action` table — see `POST /action`) **and** any row whose `userRating` is set to anything (0–100). The "any rating drops it" rule applies only to the *next* load — the feed page intentionally leaves a freshly-rated card visible so the user can still bookmark it. Each row carries an optional `userRating: u8` (0–100) set via `POST /rating` (preserved across re-ingests of the same item).
- `POST /api/recommendations/ingest` — body `{ address, sourceFirkinId, items: [...] }`. The `(address, sourceFirkinId)` pair is deduped via the `recommendation_source` marker table. Only invoked from the frontend's `/catalog/[id]` after `loadRelated` resolves and **only** when the source firkin is bookmarked — non-bookmarked browse-cache firkins skip the ingest, matching the legacy virtual-page behaviour.
- `POST /api/recommendations/action` — body `{ address, firkinId, action }`. Upserts one row per `(address, firkinId)` in the `recommendation_action` table (id is `sha256("recommendation_action:{address}:{firkin_id}")`, so re-acting overwrites the previous row instead of stacking). `action` must be `"like"`, `"discard"`, or `"bookmark"`. The bookmark action is recorded separately from the actual `POST /api/firkins` call — the feed page does both, so the same recommendation card never reappears even though the recommendation row itself is left in place.
- `POST /api/recommendations/rating` — body `{ address, firkinId, rating }` where `rating` is `0..=100`. Updates the matching `recommendation` row's `userRating` field in place (404 if no row exists for the pair). The feed page's 5-star widget posts `stars * 20`; the Discard button posts `0` to hide the item under the `excludeActioned=true` filter. The value persists across re-ingests of the same item and feeds into the listing's secondary sort.
- `GET /api/artists` / `POST` / `GET /:id` / `PUT /:id` / `DELETE /:id` — artist record CRUD. POST upserts (deduping by content-addressed id `CIDv1-raw(sha256(normalised_name))`), PUT replaces in place at the existing id.
- `GET /api/database/tables` — list every table in the SurrealDB database with its row count.
- `GET /api/database/tables/:table?limit=<n>&offset=<n>` — paginate records in a single table. `limit` defaults to 100 (max 1000); `offset` defaults to 0.
- `GET /api/disk` — host disk inventory + data-root breakdown. Returns `{ dataRoot, dataRootTotalBytes, dataRootMountPoint, disks: [{ name, mountPoint, fileSystem, kind, isRemovable, totalBytes, availableBytes, usedBytes, isDataRootDisk }], subdirs: [{ name, path, kind: "Dir"|"File", exists, sizeBytes }] }`. `subdirs` covers every known top-level entry under `<data_root>` (`db`, `identities`, `swarm.key`, and the `downloads/*` subdirs from `paths.rs`) plus any extras the user has placed there; directory sizes are recursive sums via `walkdir`.
- `GET /api/fs/browse?path=<optional>` — list subdirectories under `path` (defaults to the system home directory).
- `GET /api/catalog/sources` — list addons supported by the catalog browser.
- `GET /api/catalog/:addon/popular?filter=<>&page=<>` — returns paginated popular items for the given addon. TMDB needs `TMDB_API_KEY`.
- `GET /api/catalog/:addon/genres` — returns `[{ id, name }]` for the addon's filter dimension.
- `GET /api/catalog/:addon/:id/metadata` — returns `{ artists, trailers, reviews }` for a single upstream catalog item. For TMDB collapses to one upstream HTTP call via `append_to_response=credits,videos`.
- `GET /api/catalog/tmdb-tv/:id/seasons` — returns season list for a TMDB TV show.
- `GET /api/catalog/:addon/:id/related` — returns related items for the upstream catalog item (TMDB `/recommendations`, MusicBrainz other release-groups by the same primary artist).
- `GET /api/torrent/list`, `POST /api/torrent/add`, `POST /api/torrent/evaluate`, `POST /api/torrent/stream`, `GET /api/torrent/stream/:info_hash/:file_index` — torrent client surface backed by `librqbit`.
- `POST /api/search/subs-lyrics` — LRCLIB / Wyzie subtitle proxy.
- `GET /api/player/stream-status` / `GET /api/player/playable` — stubs so `playerService.initialize()` settles cleanly.
- `/api/ytdl/*` — full surface from `mhaol_yt_dlp::build_router(state.ytdl_manager)` mounted under the cloud router. The frontend's `/youtube` page talks to this directly. cfg(not(target_os = "android")).
- `GET /api/ytdl/channel/:channel_id/rss` — returns the parsed Atom feed (`https://www.youtube.com/feeds/videos.xml?channel_id=…`) for the channel. Cached in-process for 15 minutes via `state.ytdl_channel_cache` so the public feed endpoint isn't hammered. Body: `{ channelId, channelTitle, items: [{ videoId, title, link, thumbnailUrl, publishedAt, description }] }`.
- `GET /api/ytdl/channel/by-video?url=<watch URL>` — convenience endpoint used by the catalog detail pages: resolves the video URL → video id → channel id (cached for 24h) and returns the same channel-feed JSON in one round-trip. Falls back to `fetch_video_info` only when the video → channel mapping is cold.
- `GET /api/ytdl/related?url=<watch URL>` (alt: `?videoId=<id>`) — returns the InnerTube `/next` "watch next" sidebar for a given video, parsed down to a flat list of related videos. Body: `{ videoId, items: [{ videoId, title, url, thumbnail, duration, durationText, views, viewsText, uploadedDate, uploaderName, uploaderUrl, uploaderVerified }] }`. The current YouTube web client returns the sidebar as `lockupViewModel` view-model entries (the parser keeps a `compactVideoRenderer` fallback for older clients); `reelShelfRenderer` (Shorts), `compactRadioRenderer` (mixes), playlists, and continuation slots are all filtered out — only `LOCKUP_CONTENT_TYPE_VIDEO` lockups (or legacy compact video renderers) are surfaced. Used by the right-aside "Related videos" rail on `youtube-video` catalog pages.

## Library scan → IPFS pins (lite mode)

`packages/backend/src/library_scan.rs` runs after every `/api/libraries/:id/scan`. Its side-effect is computing a UnixFS CID per matched file via `IpfsManager::compute_file_cid` and recording a **lite** `ipfs_pin` row (`materialised: false`). The blockstore is **not** written at this stage — that's the whole point of lite mode: a 200 GB TV library no longer duplicates every byte into `<data_root>/downloads/ipfs/` on scan.

- Empty `addons`: the directory walk still runs (the frontend's scan-results table populates), but no files are hashed.
- Non-empty `addons`: only entries whose type is relevant to one of the declared addons get hashed. Relevance is `local-movie`/`local-tv` → video, `local-album` → audio, `local-book` → epub/pdf/mobi/azw3/cbz/cbr/djvu/fb2, `local-game` → iso/rom/smc/sfc/gba/nes/gb/gbc/n64/z64/v64/md/sms/gg/nds/3ds/wad/cue/chd/gcm.
- Re-scans are short-circuited per path: if any `ipfs_pin` row already exists with the same `path`, `compute_file_cid` is skipped.

Each lite-pinned file lands as one row in the `ipfs_pin` table (deduped by `(cid, path)`). The libraries page treats lite pins the same as materialised pins for display purposes — the CID column populates as soon as the hash completes — and `serve_pin_file` streams the file directly from its on-disk path regardless of materialisation status, so local playback and the WASM emulator modal both work without any extra step. **No firkin records are created automatically from a library scan** — the firkin store is only written to by explicit user actions.

**Filestore decorator** (`packages/ipfs-core/src/filestore.rs` + `packages/backend/src/filestore_index.rs`): when the backend boots, it loads `ipfs_filestore_entry` rows into an in-memory `HashMap` and hands a `FilestoreIndex` arc to `IpfsConfig::filestore_index`. The embedded IPFS node then initialises with a `FilestoreBlockStore` decorator wrapping `FsBlockStore`. Bitswap reads consult the index first; on hit, the decorator reads `length` bytes from `path` at `offset` and runs them through `FileAdder` to reconstruct the leaf block — same chunker + same protobuf encoding produce a byte-identical block whose CID matches the index entry. On mismatch (the source file was edited since the scan), the read fails gracefully and bitswap moves on. With the filestore decorator wired in, **bitswap can serve every leaf without any materialisation step** — the lite pin and the live filestore are the same thing.

**Manual materialisation** is preserved at `POST /api/ipfs/pins/:cid/materialise` for deployments that don't have the decorator (or for callers that want explicit `put_block`-backed copies in the blockstore).

Other call sites still mint **materialised** pins because their use case demands swarm reachability immediately:
- `firkins::pin_firkin_body` — firkin metadata bodies (a few KB each) — peers need to be able to fetch the JSON to discover the firkin.
- `torrent_completion` — files that arrived via torrent — already on the wire, no reason to skip materialisation.
- `rom_extract` — extracted ROMs — small files served directly to the WASM emulator.
- `artists::record_pin` — artist record bodies — same as firkin metadata.

## Firkin versioning

Firkins are content-addressed: the SurrealDB record `id` is the CIDv1-raw of the firkin body (title, description, artists, images, files, year, addon, version, version_hashes). Two fields participate in this hash:

- `version: u32` — rolling-forward nonce, starts at `0`.
- `version_hashes: Vec<String>` — CIDs of every prior version, oldest first. Chain integrity invariant: `version_hashes.len() == version`.

Whenever the firkin is updated programmatically, the prior CID is pushed onto `version_hashes`, `version` is incremented, the new CID is computed, the old record is deleted, and a new record is created at the new CID. The body is pinned to IPFS at every version-rollforward.

## Torrent → firkin auto-update

`packages/backend/src/torrent_completion.rs` runs a background task that polls `TorrentManager::list()` every 5 seconds. When a torrent reaches `Seeding` (or `progress >= 1.0`):

1. Find the firkin whose `files` includes a `torrent magnet` whose value contains `btih:<info_hash>` (case-insensitive).
2. Walk the torrent's `output_path` recursively; skip files already represented as `ipfs` entries.
3. For each remaining file: pin to the embedded IPFS node and record the pin in `ipfs_pin`.
4. Append `{ type: "ipfs", value: <cid>, title: <relative_path> }` entries to `firkin.files`.
5. Roll the version forward, recompute the CID, delete the old record, create the new one at the new CID.

Failures are logged and retried on the next tick.

## Logs

Dev runs tee full stdout+stderr to `<repo-root>/logs/`:

- `pnpm dev` cloud strand → `logs/cloud.log`
- `pnpm dev` web (Vite) strand → `logs/web.log`
- `pnpm dev` tauri strand → `logs/tauri.log`

When debugging the backend, read these files directly — don't ask the user to paste output. Each file is overwritten on the next run.

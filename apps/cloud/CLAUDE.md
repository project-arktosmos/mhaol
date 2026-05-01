# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio + SurrealDB (embedded RocksDB)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 9898)

The cloud server runs an embedded SurrealDB store, an identity manager, and the desktop-only managers from `mhaol-yt-dlp`, `mhaol-torrent`, and `mhaol-ipfs-core`. It serves the Svelte WebUI from the nested `web/` directory.

The cloud also ships a desktop Tauri shell at `apps/cloud/src-tauri/` (crate `mhaol-cloud-shell`, productName "Mhaol Cloud"). The shell is **tray-only** — it never opens a window. `tauri.conf.json` has `app.windows: []`, on macOS the activation policy is set to `Accessory` (no dock icon), and `RunEvent::ExitRequested` is intercepted via `prevent_exit()` so the app stays alive without any windows. It registers a system tray icon (id `mhaol-cloud-tray`) on macOS/Windows/Linux with two menu items: **Open** opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`, **Quit** calls `app.exit(0)`. The cloud WebUI itself remains browser-accessible at `http://localhost:9898`.

## Source Structure

```
src/
├── server.rs            # Binary entry point — opens SurrealDB, builds router
├── paths.rs             # Single source of truth for on-disk paths under <data_root>
├── db.rs                # SurrealDB connection helper (RocksDB engine)
├── state.rs             # CloudState: { db, identity_manager, ytdl_manager, torrent_manager, ipfs_manager }
├── cloud_status.rs      # GET /api/cloud/status
├── users.rs             # /api/users — secp256k1 user registry (id = lowercased EVM address); register/login require an EIP-191 signature over a fresh `Mhaol Cloud auth at <RFC3339>` message
├── libraries.rs         # /api/libraries CRUD — SurrealDB-backed library records identified by their on-disk dir; carries a list of catalog `kinds` (movie / tv / album / book / game)
├── library_scan.rs      # Scan-time media detection + firkin persistence (cfg(not(target_os = "android")))
├── firkins.rs         # /api/firkins CRUD — SurrealDB-backed firkin records (id is a CIDv1-raw of the body); create also pins the body JSON to the embedded IPFS node and records an `ipfs_pin` row keyed `firkin://<id>`. Firkins reference artists by CID (see `artists.rs`); incoming creates speak in inline artist objects which the server upserts before computing the firkin CID.
├── artists.rs         # /api/artists CRUD — SurrealDB-backed artist records (`artist` table). Each artist body is `{ name, roles: string[], imageUrl? }`; the SurrealDB id is `CIDv1-raw(sha256(normalised_name))` — *only* the name participates in the content-address (lowercased + whitespace-collapsed). Upserts merge the inbound single `role` into the existing record's `roles` array (deduped) and back-fill `imageUrl` when missing, so the same person across many firkins collapses into one record. Each merge re-pins the full body to IPFS (`artist://<id>` row).
├── database.rs          # /api/database/tables{,/:table} — read-only SurrealDB explorer (lists tables, paginates records)
├── ipfs_pins.rs         # /api/ipfs/pins — lists pins recorded when libraries are scanned; `/api/ipfs/pins/:cid/file` streams the on-disk file for a pinned object (used by the WASM emulator modal); exposes record_pin() used by the scan handler
├── media_trackers.rs    # /api/media-trackers — per-(firkin, user) playback time totals. `POST /heartbeat` upserts a `media_tracker` row keyed `sha256(firkin_id:address)` and adds the supplied `deltaSeconds`; the right-side player calls it once at play-start (delta 0) and every 10 seconds while streaming.
├── recommendations.rs   # /api/recommendations — per-(user, recommended firkin) counts. `POST /ingest` dedupes per (user, source firkin) via the `recommendation_source` marker table so revisits don't re-count; only the `/catalog/[ipfsHash]` detail page (never `/catalog/virtual`) calls ingest. `compute_recommendation_cid` produces a stable CID for each related item using the same canonical body as a virtual firkin with empty artists/trailers. Each row also persists `upstream_id` (TMDB / MusicBrainz id) so the WebUI's `/recommendations` page can mint a real firkin from a row without re-querying the catalog API. Recommendation rows are **not** deleted when the user bookmarks the same item — counts keep accumulating even for items already in the user's collection.
├── fs_browse.rs         # /api/fs/browse — list subdirectories under a path (defaults to home), used by the WebUI directory picker
├── catalog.rs           # /api/catalog/* — proxies popular items + genres for tmdb / musicbrainz / youtube
├── search.rs            # /api/search/* — TMDB + ThePirateBay + LRCLIB lyrics + Wyzie subtitle proxy (drives the right-side `SubsLyricsFinder` panel)
├── player.rs            # /api/player/{stream-status,playable} — stubs so `playerService.initialize()` settles cleanly in the WebUI
├── ytdl.rs              # /api/ytdl/* — mounts `mhaol_yt_dlp::build_router(state.ytdl_manager)` so the WebUI talks to the cloud's yt-dlp manager directly (cfg(not(target_os = "android")))
└── frontend.rs          # rust-embed wrapper that serves web/dist-static/

web/                     # SvelteKit static SPA (pnpm package `cloud`); builds to web/dist-static/
├── src/
│   ├── routes/          # SvelteKit routes
│   ├── components/      # Svelte components, organised by feature
│   ├── services/        # Frontend services + runes-driven service classes (`*.svelte.ts`)
│   ├── adapters/        # Adapter classes wrapping external APIs / signaling
│   ├── transport/       # fetch / SSE / WebRTC RPC helpers (see "WebUI: transport layer" below)
│   ├── types/           # Shared TS types
│   ├── utils/           # Pure helpers
│   ├── data/            # Static data (media-registry, …)
│   ├── lib/             # SvelteKit `$lib` files (per-page services + helpers)
│   ├── app-shims/       # Svelte/Tauri environment shims
│   └── css/             # Tailwind/DaisyUI entry + theme tokens
├── scripts/             # nav generator + Vite plugin
├── svelte.config.js     # path aliases ($components, $services, $types, $adapters, $utils, $data, $transport)
├── vite.config.ts
└── package.json
```

The cloud is the only frontend-facing app in this monorepo, so the WebUI owns its full stack — there is no separate shared UI package. Aliases are defined in `svelte.config.js` so cross-module imports stay short:

```javascript
alias: {
  $components: 'src/components',
  $services: 'src/services',
  $types: 'src/types',
  $adapters: 'src/adapters',
  $utils: 'src/utils',
  $data: 'src/data',
  $transport: 'src/transport',
  'app-shims': 'src/app-shims'
}
```

Plus the SvelteKit-reserved `$lib` (→ `src/lib/`) and `$app/*` (SvelteKit modules).

## WebUI: catalog detail routes

`/catalog/virtual` and `/catalog/[ipfsHash]` share the same presentation through `$components/catalog/` and the same behaviour through resolver service classes in `$services/catalog/`. Each route only owns its route-specific wiring:

| Concern | `/catalog/virtual` | `/catalog/[ipfsHash]` |
|---|---|---|
| Source of firkin data | URL query params (synthesised `CloudFirkin`) | `+page.ts` loader → real persisted firkin |
| Header actions | `Bookmark` | `Play` / `IPFS Play` / `Torrent Stream` / `Find metadata` / `Delete firkin` |
| Identity / version history / files table | omitted | rendered |
| Resolver `persist` callbacks | none — discarded on navigate | `PUT /api/firkins/:id` (rolls the CID forward) |
| Torrent search eval column | off | on (`/api/torrent/evaluate` per row) |
| Torrent search collapsible | always open | collapsed by default |

**Shared components** (`apps/cloud/web/src/components/catalog/`):
- `CatalogPageHeader.svelte` — back link, title, addon/kind/year badges, optional `extraBadge`, action snippet slot
- `CatalogDescriptionCard.svelte` — description card
- `CatalogImagesCard.svelte` — images grid with metadata
- `CatalogTrailersCard.svelte` — trailers list driven by a `TrailerResolver`
- `CatalogTracksCard.svelte` — MusicBrainz tracks list driven by a `TrackResolver`
- `CatalogTorrentSearchCard.svelte` — torrent search results, optional collapsible + per-row streamability eval
- `CatalogIdentityCard.svelte` — CID / created / updated / version (detail only)
- `CatalogVersionHistoryCard.svelte` — `version_hashes` chain (detail only)
- `CatalogFilesTable.svelte` — firkin `files` table (detail only)

**Shared resolver services** (`apps/cloud/web/src/services/catalog/`, all `.svelte.ts` so `$state` runes work):
- `trailer-resolver.svelte.ts` — `TrailerResolver` class. `resolveMovie(...)` / `resolveTv(...)` accept TMDB-sourced trailers via `stored`, prefer them when present, and only fall back to the YouTube fuzzy search when TMDB has nothing English. Optional `persist` callback writes back via `PUT /api/firkins/:id`.
- `track-resolver.svelte.ts` — `TrackResolver` class. `loadByReleaseGroup(...)` fetches the MB tracklist and resolves YouTube URLs in series. Optional `persistTrackUrls` batches resolved URLs into one `PUT` so each resolution doesn't mint N intermediate firkin CIDs.
- `torrent-search.svelte.ts` — `TorrentSearch` class. Optional `evaluate: true` runs `/api/torrent/evaluate` per result with a sliding-window concurrency cap (default 4). Also exports `startTorrentDownload(magnet)`.

**Pattern.** When two routes need the same UI in the cloud SPA: put the markup in `$components/<feature>/`, put the behaviour in a runes-driven service class at `$services/<feature>/<thing>.svelte.ts`, and let each route compose them with route-specific inputs and (optional) persistence callbacks. The presentational components stay free of business logic; the service classes own the state machines and side-effects.

## WebUI: transport layer

All frontend-to-backend communication flows through `apps/cloud/web/src/transport/`:
- `transport.type.ts` — `Transport` interface (fetch, subscribe, resolveUrl)
- `ws-transport.ts` — WebSocket RPC implementation (sends requests over a peer connection)
- `fetch-helpers.ts` — `fetchJson()`, `fetchRaw()`, `subscribeSSE()` used by all services
- `transport-context.ts` — Module-level singleton (`setTransport` / `getTransport`); the default fallback talks plain HTTP via `globalThis.fetch`
- `rpc.type.ts` — RPC message protocol types

Services should never call `fetch` directly when they need transport-aware behaviour — go through `fetchJson` / `fetchRaw` / `subscribeSSE` so the same code paths work over HTTP and WebSocket.

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

- `mhaol-yt-dlp` — YouTube download manager (cfg(not(target_os = "android"))).
- `mhaol-torrent` — `librqbit`-backed torrent session, initialized in the background on startup so the server can bind quickly (cfg(not(target_os = "android"))).
- `mhaol-ipfs-core` — embedded `rust-ipfs` node (libp2p, Bitswap, Kademlia DHT), initialized in the background on startup. The blockstore lives at `<data_root>/downloads/ipfs/` (cfg(not(target_os = "android"))). The node **always** runs on a **private swarm**: cloud reads (or auto-generates on first boot) a swarm key at `<data_root>/swarm.key` (override with `IPFS_SWARM_KEY_FILE`). Only nodes carrying that exact key can connect; the public bootstrap list is skipped, mDNS is off, and the transport stack is constrained to TCP+pnet+noise+yamux. Non-PSK peers fail at the libp2p `pnet` handshake before anything reaches Kademlia or the application — that is the only enforcement layer needed. If the swarm key cannot be loaded or generated the IPFS subsystem refuses to start (no fallback to the public swarm). The cloud bootstraps against the rendezvous node: precedence is `RENDEZVOUS_BOOTSTRAP` env var (newline- or comma-separated multiaddrs), then `<data_root>/rendezvous/bootstrap.multiaddr` (override with `RENDEZVOUS_BOOTSTRAP_FILE`), then a localhost default of `/ip4/127.0.0.1/tcp/14001`.

All download paths land under `<data_root>/downloads/{torrents,torrent-streams,ipfs,ipfs-stream,youtube}`. The `torrents/` dir holds long-lived torrents (firkin auto-update flow); `torrent-streams/` is reserved for `/api/torrent/stream` payloads — those are deleted (torrent + on-disk files) on every new stream request. yt-dlp uses `<data_root>/downloads/youtube` by default and still honors `YTDL_OUTPUT_DIR`/`YTDL_PO_TOKEN`/`YTDL_VISITOR_DATA`/`YTDL_COOKIES`.

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
- `SIGNALING_URL` — Base URL of the rendezvous WebSocket signaling server (default: `http://localhost:14080`). The cloud bakes this into the identity manager's passport metadata so peers can discover the right rendezvous endpoint.
- `IPFS_SWARM_KEY_FILE` — Override the IPFS pre-shared swarm key path (default: `<data_root>/swarm.key`, auto-generated on first boot when missing). Note: the rendezvous app defaults to its own swarm key location; if you run both on the same machine, point one of them at the other's key (or symlink) so they share the same PSK.
- `RENDEZVOUS_BOOTSTRAP` — Newline- or comma-separated rendezvous multiaddrs to dial on startup (e.g. `/ip4/192.168.1.10/tcp/14001/p2p/12D3...`). Takes precedence over the bootstrap file.
- `RENDEZVOUS_BOOTSTRAP_FILE` — Override the rendezvous-written bootstrap multiaddr file path (default: `<data_root>/rendezvous/bootstrap.multiaddr`).
- `YTDL_OUTPUT_DIR` — Override the yt-dlp output directory (default: `<data_root>/downloads/youtube`).

## Public WebUI endpoints

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, signaling/client wallet addresses, db engine/namespace/version, and a `packages` block reporting health for `ytDlp`, `torrent`, and `ipfs`. No auth required (used by the embedded WebUI).
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
- `GET /api/ipfs/pins/:cid/file` — stream the on-disk bytes for a pinned object. Looks up the pin by CID, rejects metadata pins (`firkin://…`, `artist://…`), and serves the file with `Content-Type` from the pin row (or `application/octet-stream` when missing). Used by the WASM emulator modal so the browser can fetch a ROM directly after `extract_roms_for_firkin` has unpacked any archive.
- `GET /api/firkins` — list firkins persisted in SurrealDB (`firkin` table). Superseded versions (any id appearing in another row's `version_hashes`) are filtered out so callers only see the head of each chain. Each row carries `artistIds` (CIDs of the referenced `artist` records, drives the firkin's own CID) and the resolved `artists` (server-side join — `[{ id, name, roles: string[], imageUrl?, … }]`, where `roles` is the deduped multi-role array on the canonical artist record).
- `POST /api/firkins` — create a firkin `{ title, addon, description?, artists?, images?, files?, year?, creator? }`. `title` and `addon` are required (`addon` must be a known addon id; see `/api/catalog/sources` for browsable ones, plus the `local-*` family and the non-browsable `wyzie-subs-*`/`lrclib` ids). `artists` is an array of inline `{ name, role?, imageUrl? }` objects — the server upserts each one into the `artist` table (deduped by content-address; see `/api/artists`) and stores the resulting CIDs on the firkin body, so the firkin's own CID stays stable across presentation-only edits to an artist. The firkin id is the CIDv1-raw sha256 of the canonical pretty-printed JSON body (which references artists by CID, not by their mutable body), computed by `compute_firkin_cid`. Returns `200` with the existing record if a firkin with that id already exists, otherwise `201`. **Bookmark semantics**: in addition to the SurrealDB write, the handler pins the firkin's body JSON to the embedded IPFS node via `IpfsManager::add_bytes` (named `firkin-<id>.json`) and inserts an `ipfs_pin` row `{ cid: <unixfs cid>, path: "firkin://<id>", mime: "application/json", size }`. Each artist upsert performs the analogous pin keyed `artist://<id>`. The IPFS pins are best-effort — failures are logged via `tracing::warn!` but do not fail the create, so the WebUI's Bookmark / torrent-pick flows still succeed while the IPFS node is warming up.
- `GET /api/firkins/:id` — fetch one firkin (with the same `artistIds` + resolved `artists` shape as the list endpoint).
- `PUT /api/firkins/:id` — update `title`, `addon`, `description`, `artists`, `images`, `files`, `year`, `trailers` (any subset). `artists` is the same inline object array as on POST and is materialised through the same upsert path. Applies the mutation through the shared `rollforward_firkin` helper: when the new body produces a different `compute_firkin_cid`, the old id is pushed onto `version_hashes`, `version` is incremented, the old record is deleted, a new one is created at the new CID, and the new body JSON is pinned to IPFS via `pin_firkin_body` (so the response carries the **new** id and clients must navigate to it). When the mutation produces the same CID (only non-CID fields like `updated_at` changed), the record is updated in place at the existing id without bumping `version`. Drives the detail page's track-URL / trailer / artist back-fill flows so the IPFS pin and CID stay truthful as the WebUI hydrates resolved YouTube URLs onto the firkin.
- `DELETE /api/firkins/:id` — remove the firkin record from SurrealDB. The IPFS pin row left by `POST /api/firkins` is currently not garbage-collected. Referenced artist records are **not** deleted — they're shared across firkins.
- `POST /api/firkins/:id/enrich` — apply catalog-derived metadata to a firkin and roll its version forward. Body: `{ title?, year?, description?, posterUrl?, backdropUrl? }` (any subset). Replaces the firkin's `images` array with the supplied `posterUrl`/`backdropUrl` (in that order), updates the other listed fields, then runs the same rollforward path as `torrent_completion::rollforward` — pushes the old id onto `version_hashes`, increments `version`, recomputes the CID via `compute_firkin_cid`, deletes the old record, and creates a new one at the new id with the new body pinned to IPFS. Returns the new `FirkinDto` (under the new id). Idempotent when the supplied metadata happens to produce the same CID (returns the existing record without rolling). Drives the WebUI's "Find metadata" affordance on `/catalog` library cards and the firkin detail page when `description` or `images` are empty.
- **Firkin `trailers`**: firkin records carry an optional `trailers: [{ youtubeUrl, label?, language? }]` array — movies hold one entry; TV shows hold one show-level entry plus one per season (with `label` set to the season name, e.g. `"Season 1"`). `language` is the ISO 639-1 code (e.g. `"en"`) when known: TMDB-sourced trailers carry it; YouTube-fallback trailers usually leave it unset. The primary source is TMDB itself: `GET /api/catalog/:addon/:id/metadata` returns `trailers` extracted from the same `append_to_response=credits,videos` request that fetches credits, filtered to YouTube `Trailer` entries with `iso_639_1 == "en"` (official ones first). For TV per-season trailers (which TMDB's show-level `videos` block doesn't differentiate) and as a fallback when TMDB has no English trailer for an item, the WebUI runs a YouTube fuzzy search via `apps/cloud/web/src/lib/youtube-match.service.ts` (same double-dip pattern as music tracks: a hard ≥50% title-token gate, then a multi-dimensional score over title/year/season/trailer keyword with negative penalties for `reaction`/`review`/`recap`). `POST /api/firkins` accepts the array up-front (the virtual catalog page resolves before bookmarking); `PUT /api/firkins/:id` accepts it for in-place updates (the detail page resolves and persists when missing). The field participates in `compute_firkin_cid` (so a new trailer set means a new CID at create / enrich / rollforward time) but each subfield is `skip_serializing_if = "Option::is_none" / "is_empty"` in the canonical body view, so existing firkin CIDs (with no `language` and/or no `trailers`) stay stable across deserialise → re-serialise.
- `GET /api/media-trackers?firkinId=<>&address=<>` — list rows from the `media_tracker` table (one row per (firkin, user) pair). Each row is `{ id, firkinId, address, totalSeconds, last_played_at, created_at, updated_at }`, sorted by `last_played_at` descending. Both query params are optional; `address` is normalised to lowercase 0x-prefixed hex before the match.
- `GET /api/recommendations?address=<>` — list rows from the `recommendation` table for the given user (one row per (user, recommended firkin) pair). Each row is `{ id, address, firkinId, addon, upstreamId, title, year?, description?, posterUrl?, backdropUrl?, count, created_at, updated_at }`, sorted by `count` descending then `updated_at` descending. `address` is required and normalised to lowercase 0x-prefixed hex before the match. Rows are not deleted when the user bookmarks the matching firkin — already-collected items continue to accumulate counts when re-recommended.
- `POST /api/recommendations/ingest` — body `{ address, sourceFirkinId, items: [{ addon, id, title, year?, description?, posterUrl?, backdropUrl? }] }`. The `(address, sourceFirkinId)` pair is deduped via the `recommendation_source` marker table — if a marker row already exists for the pair, the request short-circuits with `{ processed: false, ingested: 0 }` and no counts are touched. Otherwise, for each item the server computes its virtual firkin CID via `compute_recommendation_cid` (same canonical body as `compute_firkin_cid` with no artists / trailers, version=0, creator=""), upserts the `recommendation` row keyed `sha256("recommendation":address:firkin_id)` (incrementing `count` by 1, back-filling missing presentation fields including `upstream_id`), then writes the marker row keyed `sha256("recommendation_source":address:source_firkin_id)`. Items with unknown addons or empty title/id are silently skipped. Only invoked from `/catalog/[ipfsHash]` after `loadRelated` resolves (never from `/catalog/virtual`).
- `POST /api/media-trackers/heartbeat` — body `{ firkinId, address, deltaSeconds }`. Upserts the tracker row keyed by `sha256(firkinId:address)`, adds `deltaSeconds` to `totalSeconds`, and stamps `last_played_at`. `deltaSeconds` must be a finite non-negative number (`0` is allowed and is what the right-side player sends as the play-start signal). The cloud WebUI calls this from `media-tracker.service.ts` once when a firkin file starts playing in the right-side `PlayerVideo` and again every 10 seconds while the player is `connectionState === 'streaming'` and not paused. Returns the persisted row. `404` if the firkin doesn't exist; `400` for an invalid address or a negative/NaN delta.
- `GET /api/artists` — list every `artist` record (`{ id, name, role?, imageUrl?, created_at, updated_at }`), sorted by name.
- `POST /api/artists` — upsert an artist `{ name, role?, imageUrl? }` (single-occurrence inbound shape). The id is `CIDv1-raw(sha256(normalised_name))` — same name always collapses to the same record, regardless of role/image. If the record already exists, the inbound `role` (if any) is merged into the canonical `roles` array (deduped) and `imageUrl` is back-filled when missing; if it didn't exist, a new record is seeded with `roles: [role]` (or `[]`). Returns `200` either way. Same IPFS-pin best-effort pattern as firkins (`artist://<id>` row), re-pinned on every merge.
- `GET /api/artists/:id` — fetch one artist by CID.
- `PUT /api/artists/:id` — replace `{ name, roles: string[], imageUrl? }` in place at the existing id (id is **not** recomputed; use POST with a different name to get a different content-addressed record). The full `roles` array is replaced, not merged — this is the editor path on `/artist/[ipfs]`.
- `DELETE /api/artists/:id` — remove the artist record. The `ipfs_pin` row is not garbage-collected. Referencing firkins are not updated — their `artistIds` stay intact (the resolved `artists` array on subsequent reads will simply be missing that entry).
- `GET /api/database/tables` — list every table in the cloud SurrealDB database with its row count. Returns `{ namespace, database, tables: [{ name, record_count }] }`. Used by the embedded `/database` explorer.
- `GET /api/database/tables/:table?limit=<n>&offset=<n>` — paginate records in a single table. Table names are validated as `[A-Za-z0-9_]{1,64}`. `limit` defaults to 100 (max 1000); `offset` defaults to 0. Returns `{ table, limit, offset, total, records }` where each record is JSON with the SurrealDB `id` flattened to a `<table>:<id>` string.
- `GET /api/fs/browse?path=<optional>` — list subdirectories under `path` (defaults to the system home directory). Returns `{ path, parent, home, separator, roots, entries }` where `entries` only contains directories (hidden dot-folders are skipped). On Windows, `roots` lists available drive letters.
- `GET /api/catalog/sources` — list addons supported by the catalog browser. Each addon owns a single content kind (no nested `types`); each entry is `{ id, label, kind, filterLabel, hasFilter }`. Browsable addons are `tmdb-movie`, `tmdb-tv`, `musicbrainz`, `youtube-video`, `youtube-channel`. Non-browsable addons (`wyzie-subs-movie`, `wyzie-subs-tv`, `lrclib`, and the `local-*` family used by libraries) are valid firkin `addon` values but don't appear in this list.
- `GET /api/catalog/:addon/popular?filter=<>&page=<>` — returns `{ items: [{ id, title, year, description, posterUrl, backdropUrl }], page, totalPages }` for the given addon. `filter` is the genre/region id from `/genres`. TMDB needs `TMDB_API_KEY`; missing keys return `503`.
- `GET /api/catalog/:addon/genres` — returns `[{ id, name }]` for the addon's filter dimension. `tmdb-movie`/`tmdb-tv` query `/genre/movie/list` and `/genre/tv/list` upstream; everyone else returns a static curated list (genres / subjects / console ids / regions / categories / tags).
- `GET /api/catalog/:addon/:id/metadata` — returns `{ artists: [{ name, role?, imageUrl? }], trailers: [{ youtubeUrl, label?, language? }] }` for a single upstream catalog item. For TMDB (`tmdb-movie` / `tmdb-tv`) this collapses to one upstream HTTP call via `append_to_response=credits,videos` — the `credits` block becomes `artists` (top cast + selected crew jobs), and the `videos` block is filtered to YouTube `Trailer` entries whose `iso_639_1 == "en"` (English-only; non-English trailers are dropped here so the WebUI surfaces only English trailers — when none survive, the frontend falls back to the YouTube fuzzy search). Official trailers sort first; `language` carries TMDB's `iso_639_1` (lower-cased) so persisted firkins remember the language of each trailer. For `musicbrainz` / `youtube-video` / `youtube-channel`, `artists` is populated from the upstream provider and `trailers` is `[]`. Used by the `/catalog/virtual` page on bookmark and the `/catalog/[ipfsHash]` detail page to backfill missing `artists` and to seed `trailers` (so the YouTube fuzzy-search fallback only runs when TMDB has no English trailer for the item / season).
- `GET /api/catalog/tmdb-tv/:id/seasons` — returns `[{ seasonNumber, name, airYear?, episodeCount? }]` for a TMDB TV show by upstream id. Used by the WebUI to enumerate seasons for trailer resolution: each season is searched against YouTube as `"{showTitle} season {n} trailer"` and the best match is persisted on the firkin's `trailers` array. TMDB's virtual season 0 (specials) is filtered out.
- `GET /api/catalog/:addon/:id/related` — returns `[{ id, title, year, description, posterUrl, backdropUrl }]` listing items related to the upstream catalog item. Same shape as `/popular`. For `tmdb-movie` / `tmdb-tv` this proxies TMDB's `/recommendations` endpoint (one HTTP call, no per-item credits fan-out — the related grid only renders title / year / poster). For `musicbrainz` it browses other release-groups by the same primary artist (current release-group filtered out). Unknown / unsupported addons return an empty list. Used by `/catalog/virtual` and `/catalog/[ipfsHash]` to surface related items as virtual catalog links — the response is **not persisted** to SurrealDB and **not pinned** to IPFS; clicking a related item navigates to `/catalog/virtual?…` and only persists if the user explicitly bookmarks or picks a torrent there.
- `GET /api/torrent/list` — returns the cloud `TorrentManager`'s current torrents as `TorrentInfo[]` (`{ id, name, infoHash, size, progress, downloadSpeed, uploadSpeed, peers, seeds, state, addedAt, eta, outputPath }`). Returns `[]` while the session is still warming up. Used by the shared `FirkinCard` to render real-time progress.
- `POST /api/torrent/add` — adds a magnet to the cloud torrent client. Body: `{ magnet }`. Returns the initial `TorrentInfo`. `400` if the URI is not a magnet, `503` until the session has finished initializing.
- `POST /api/torrent/evaluate` — body `{ magnet }`. Probe-only: resolves the magnet metadata via librqbit's `list_only` flag (DHT + tracker peer discovery + BEP 9/10 metadata exchange — no piece downloads, no on-disk side-effects, no session entries), checks for a streamable video file. Always returns 200; the JSON `streamable` field is the discriminator: `{ streamable: true, infoHash, name, fileIndex, fileName, fileSize, mimeType }` on success, `{ streamable: false, reason }` otherwise. The WebUI fires this on every catalog-detail mount that has a magnet attached, and only enables the "Torrent Stream" button when it returns `streamable: true`.
- `POST /api/torrent/stream` — body `{ magnet }`. Wipes any previously-started stream torrents (deletes them from the session and removes their on-disk files in `<data_root>/downloads/torrent-streams/`), resolves the magnet metadata via librqbit's `list_only` flag (same path as `/evaluate`), picks the largest streamable video file (mp4/mkv/webm/mov/avi/m4v/ogv/ts), then starts a `only_files=[idx]` download into `torrent-streams/`. Returns `{ infoHash, name, fileIndex, fileName, fileSize, mimeType, streamUrl }`.
- `GET /api/torrent/stream/:info_hash/:file_index` — serves the chosen file with HTTP byte-range support (`Accept-Ranges: bytes`, `206 Partial Content`, suffix-range parsing). Backed by librqbit's `FileStream` which lazy-fetches pieces and registers wakers for not-yet-available bytes, so the `<video>` element drives piece priority via Range requests.
- `POST /api/search/subs-lyrics` — body `{ addon, query, externalIds?, languages? }`. The addon implies the source: `lrclib` / `musicbrainz` / `local-album` route through LRCLIB by free-text query; `tmdb-movie` / `tmdb-tv` / `wyzie-subs-movie` / `wyzie-subs-tv` / `local-movie` / `local-tv` route through Wyzie keyed by TMDB id (one entry per `externalIds[]`). Returns a flat `SubsLyrics[]`. Mirrors the node `/api/search/subs-lyrics` endpoint and powers the `SubsLyricsFinder` panel in the right-side aside.
- `GET /api/player/stream-status` — returns `{ available: false }`. The cloud has no local stream server; this stub keeps `playerService.initialize()` from rendering an error toast.
- `GET /api/player/playable` — returns `[]`. Cloud doesn't enumerate playable files like node does.
- `/api/ytdl/*` — full surface from `mhaol_yt_dlp::build_router(state.ytdl_manager)` mounted directly under the cloud router via `nest_service`. Includes `GET /search`, `GET /info/video`, `GET /info/stream-urls{,-browser}`, `GET /info/playlist`, `GET /downloads`, `POST /downloads`, `POST /downloads/playlist`, `GET /downloads/events` (SSE), `DELETE /downloads/{id}`, `DELETE /downloads/completed`, `DELETE /downloads/queue`, `GET|PUT /config`, `GET /status`, `GET /ytdlp/status`. The WebUI's `/youtube` page talks directly to this surface via plain `fetch('/api/ytdl/...')` (no transport layer). cfg(not(target_os = "android")).

## Library scan → firkins

`apps/cloud/src/library_scan.rs` runs after every `/api/libraries/:id/scan` and turns the walked entries into `firkin` records. Behavior depends on the library's `addons`:

- Empty `addons`: the directory walk still runs (the WebUI's scan-results table populates), but no pins or firkins are produced — without a declared addon there is no "library type" to filter by, so nothing is considered relevant.
- Non-empty `addons`: only entries whose type is relevant to one of the declared addons participate. Relevance is `local-movie`/`local-tv` → video (mime `video/*` or known video extension), `local-album` → audio, `local-book` → epub/pdf/mobi/azw3/cbz/cbr/djvu/fb2, `local-game` → iso/rom/smc/sfc/gba/nes/gb/gbc/n64/z64/v64/md/sms/gg/nds/3ds/wad/cue/chd/gcm. Relevant entries are classified per addon and grouped into media items; each group's files are pinned to IPFS, recorded in `ipfs_pin`, and persisted as a `firkin` whose `files` are the `ipfs` entries (`{ type: "ipfs", value: <cid>, title: <relative_path-or-display-title> }`). Relevant files that didn't fall into any group are pinned as stragglers (kept reachable for `/api/libraries/:id/pins`); irrelevant files (e.g. an image or video sitting in a `local-album` directory) are ignored.

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

# Arktosmos Mhaol - Development Guidelines

This document guides Claude (and developers) on implementing features in this monorepo. Follow these conventions strictly to maintain consistency across all packages.

For package-specific conventions, see the `CLAUDE.md` in each package directory:
- `apps/cloud/CLAUDE.md` — Tray-only desktop Tauri shell (`mhaol-cloud-shell`, "Mhaol Cloud"). Thin wrapper that presents the system to users; actual logic lives in `packages/backend` and `packages/frontend`.
- `apps/headless/CLAUDE.md` — Terminal-only counterpart of `apps/cloud` (`mhaol-headless`). Same backend + embedded SPA, no Tauri shell, no tray; designed for servers and CI hosts.
- `packages/backend/CLAUDE.md` — Rust Axum server crate (`mhaol-backend`, binary `mhaol-cloud`). API routes, SurrealDB store, IPFS / torrent / yt-dlp managers, on-disk layout.
- `packages/frontend/CLAUDE.md` — Svelte SPA (`frontend`). Components, services, adapters, types, utils, CSS/themes, transport layer.
- `packages/cloud-ui/` — Shared Svelte 5 display components + firkin types used by the frontend SPA.
---

## Monorepo Overview

```
mhaol.git/
├── apps/
│   ├── cloud/                        # Tray-only Tauri shell (mhaol-cloud-shell, "Mhaol Cloud"). Embeds packages/frontend, runs alongside the mhaol-cloud server bin.
│   └── headless/                     # Thin Rust crate (mhaol-headless bin) that runs mhaol_backend::run() with no Tauri/tray. For servers and terminal-only hosts.
├── packages/
│   ├── backend/                      # Rust Axum server crate (mhaol-backend lib + mhaol-cloud bin) — port 9898, libp2p TCP 9900, libp2p /ws 9901, embedded SurrealDB.
│   ├── frontend/                     # Svelte SPA (pnpm package "frontend"). Builds to packages/frontend/dist-static which mhaol-backend embeds.
│   ├── addons/                       # Addon modules (TMDB, MusicBrainz, YouTube, LRCLIB, Wyzie subtitles, torrent search)
│   ├── cloud-ui/                     # Shared Svelte 5 display components + firkin types + game-icons.net <Icon /> set
│   ├── identity/                     # Rust Ethereum identity management (secp256k1, EIP-191)
│   ├── ipfs-stream/                  # Rust HLS-over-IPFS streaming (GStreamer hlssink2)
│   ├── torrent/                      # Rust torrent implementation
│   └── ipfs-core/                    # Rust IPFS node (libp2p + Bitswap + Kademlia DHT, embedded; TCP+WS+pnet+noise+yamux)
├── pnpm-workspace.yaml
└── package.json                      # Root workspace scripts
```

**Runtime requirements:** Node >= 18, pnpm >= 9, Rust (cargo)

---

## App Architecture

The system splits into a backend, a frontend, and one or more thin app shells that present the backend on a host:

- **`packages/backend/`** — Rust Axum server crate. Library `mhaol-backend` exposes `pub async fn run()` which boots SurrealDB, the identity manager, and the desktop-only managers (`mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ipfs-core`, `mhaol-ipfs-stream`), then serves `/api/*` plus the embedded frontend as a fallback. Bin `mhaol-cloud` is a thin `#[tokio::main]` wrapper over `mhaol_backend::run()`. Default port 9898 (in dev, the bin binds 127.0.0.1:9899 and Vite takes 9898 as the public port).
- **`packages/frontend/`** — SvelteKit static SPA (pnpm package `frontend`). Builds to `packages/frontend/dist-static/`, which the backend crate embeds at compile time via `rust-embed`. Owns its full stack: components, services, adapters, types, utils, CSS / themes, transport layer. See `packages/frontend/CLAUDE.md` for layout, aliases, and the catalog-detail / transport conventions.
- **`apps/cloud/`** — Tauri shell (`mhaol-cloud-shell`, productName "Mhaol Cloud"). Tray-only wrapper that **presents** the system to users — it does not host the SPA itself; the bin serves the SPA on 9898 and the tray's "Open" item launches that URL in the system browser. `tauri.conf.json`'s `frontendDist: ../../packages/frontend/dist-static` keeps Tauri's build tooling happy.
- **`apps/headless/`** — Terminal-only equivalent of `apps/cloud`. Crate `mhaol-headless` (binary `mhaol-headless`) is a thin `#[tokio::main]` wrapper over `mhaol_backend::run()` — no Tauri, no tray, no window. Same SPA embedded via `rust-embed`, same `/api/*` surface, same env vars. Use it on servers, CI hosts, and any machine where opening a window is impossible or unwanted.

### Backend (`packages/backend/`)

- `packages/backend/Cargo.toml` — Crate manifest. Library target `mhaol_backend`, bin target `mhaol-cloud`.
- `packages/backend/src/lib.rs` — Library entry point; declares all server modules and exposes `pub async fn run()` (opens SurrealDB, spawns workers, builds the Axum router, serves `/api/*` + the embedded frontend as a fallback). Configures the embedded IPFS node with `enable_mdns: true` for LAN cloud-to-cloud discovery, fixed TCP listen `9900` (`MHAOL_IPFS_TCP_PORT`) and WebSocket listen `9901` (`MHAOL_IPFS_WS_PORT`) so browsers can dial the swarm directly.
- `packages/backend/src/bin/mhaol-cloud.rs` — Standalone binary entry; `#[tokio::main] async fn main() { mhaol_backend::run().await }`.
- `packages/backend/src/cloud_status.rs` — Public `/api/cloud/status` route used by the frontend for health polling.
- `packages/backend/src/libraries.rs` — `/api/libraries` CRUD; library records are stored in SurrealDB and identified by their on-disk directory path.
- `packages/backend/src/p2p.rs` — `GET /api/p2p/bootstrap` returns `{ peerId, swarmKey, multiaddrs }` so any future browser-resident peer can join the same private swarm at runtime. Filters listen addrs to browser-dialable transports (`/ws`, `/wss`, `/webtransport`), substitutes `0.0.0.0` with loopback + LAN IP, and 503s with `Retry-After: 1` while the IPFS node is still starting.
- `packages/backend/src/frontend.rs` — Embeds `../frontend/dist-static/` via `rust-embed` and serves it as the fallback handler.

### Frontend (`packages/frontend/`)

The Svelte SPA at `packages/frontend/` (pnpm package `frontend`) builds to `packages/frontend/dist-static/`, which the backend crate embeds at compile time:

- `src/routes/` — SvelteKit route files (+page.svelte, +layout.svelte)
- `src/components/` — Svelte components, organised by feature (`catalog/`, `firkins/`, `core/`, `player/`, `libraries/`, …)
- `src/services/` — frontend services (catalog resolvers, firkin playback, player, theme, …)
- `src/adapters/` — adapter classes that wrap external APIs / signaling
- `src/transport/` — fetch/SSE/WebRTC RPC helpers (see "Transport Layer" below)
- `src/types/` — shared TypeScript types
- `src/utils/` — small pure helpers (string, smart-search, localStorageWritableStore)
- `src/data/` — static data (`media-registry.ts`, …)
- `src/lib/` — SvelteKit `$lib` files (per-page services + helpers like `image-cache`, `firkins.service.ts`, `youtube-match.service.ts`)
- `src/app-shims/` — Svelte/Tauri environment shims
- `src/css/app.css`, `src/css/themes.css` — CSS entry points (Tailwind + DaisyUI + theme tokens)
- `src/app.html`, `src/app.d.ts` — SvelteKit boilerplate
- Config files (`svelte.config.js`, `vite.config.ts`, `package.json`, `tsconfig.json`)

Cross-module imports use the path aliases configured in `svelte.config.js` (see "Alias configuration" below): `$components`, `$services`, `$types`, `$adapters`, `$utils`, `$data`, `$transport`, plus the SvelteKit-reserved `$lib` and `$app/*`.

### Tauri shell (`apps/cloud/`)

`apps/cloud/` — crate `mhaol-cloud-shell`, binary `mhaol-cloud-shell`. `productName: "Mhaol Cloud"`, identifier `com.arktosmos.mhaol.cloud`. **Tray-only**: `app.windows: []`, no window is ever created. macOS sets `ActivationPolicy::Accessory` (no dock icon), `RunEvent::ExitRequested` calls `prevent_exit()` so the process stays alive without windows. The system tray icon (id `mhaol-cloud-tray`, tooltip "Mhaol Cloud") has two items: **Open** opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`, **Quit** calls `app.exit(0)`. `tauri.conf.json` keeps `frontendDist: ../../packages/frontend/dist-static` / `devUrl: http://localhost:9898` so build/dev tooling resolves cleanly; nothing actually renders the assets at runtime.

The frontend stays browser-accessible at `http://localhost:9898`.

Layout:
- `apps/cloud/Cargo.toml` — crate manifest
- `apps/cloud/src/{lib.rs,main.rs}` — Tauri entry point
- `apps/cloud/tauri.conf.json` — desktop config
- `apps/cloud/capabilities/default.json`, `icons/`, `build.rs`

The cloud frontend has these screens:
- **Health** (`/`) — polls `/api/cloud/status` every 5 seconds and renders status, latency, uptime, bind, package health, and identities.
- **Profile** (`/profile`) — manages the browser-resident user identity. The layout calls `userIdentityService.initialize()` on mount: it loads `localStorage["mhaol-cloud-identity"]` (`{ address, privateKey, username }`) or generates a fresh secp256k1 keypair via viem, signs an EIP-191 `Mhaol Cloud auth at <RFC3339>` message, and either logs in or auto-registers against `/api/users`. The page exposes the address + username, a username editor, and JSON export/import (clipboard, file download, paste-or-upload) plus a regenerate button. Linked from the navbar's right end via the current username (filtered out of the central menu).
- **Libraries** (`/libraries`) — lists, creates, and removes library records via `/api/libraries`. The form lets you pick an existing directory, or browse to a parent and create a new subfolder; each library is identified by its directory path and carries an `addons` list of `local-*` addon ids (`local-movie`, `local-tv`, `local-album`, `local-book`, `local-game`). Each row has a `Scan` button that walks the directory recursively, reports file size + MIME, asynchronously pins media to IPFS, and (when `addons` is non-empty) groups files into `firkin` records per detected media item — TV shows aware of nested season directories, albums grouped by directory, books/games per file. The row also shows the IPFS pins (CID, path, MIME, size) recorded for that library, and on page load any library whose `last_scanned_at` is missing or older than 1 hour is rescanned automatically.
- **IPFS** (`/ipfs`) — reads `/api/ipfs/pins` and lists every pin recorded by the cloud (library scans plus firkin-body pins from `POST /api/firkins`).
- **Recommendations** (`/recommendations`) — per-user table of items the catalog API has recommended (via `/api/catalog/:addon/:id/related`), indexed by their *virtual* firkin CID. Counts only update when the user visits a real `/catalog/[ipfsHash]` detail page — never on `/catalog/virtual`. Each (user, source firkin) pair contributes at most once thanks to the `recommendation_source` marker table. Each row has a **Bookmark** button that fetches the upstream metadata (artists, trailers), creates a real firkin via `POST /api/firkins`, and navigates to `/catalog/[ipfsHash]` — landing on the new detail page automatically pulls that firkin's own related items into the recommendations list. Rows are **not** deleted when the user bookmarks the matching firkin: the same item being re-recommended (e.g., from another firkin's detail page) still increments the count.
- **Catalog** (`/catalog`) — pick an addon (each addon owns a single content kind: e.g. `tmdb-movie`, `tmdb-tv`, `musicbrainz`, `youtube-video`), optionally narrow by genre, and browse popular items. The Rust server proxies upstream calls via `/api/catalog/*` so addon API keys (`TMDB_API_KEY`) stay server-side. Grid items are **virtual** — nothing is written to SurrealDB and nothing is pinned to IPFS while browsing. Clicking "View details →" navigates to `/catalog/virtual?...` (a virtual detail synthesised from URL query params) which immediately runs a torrent search. The virtual page has two persistence triggers: (1) a **Bookmark** button in the header that turns the virtual item into a firkin without attaching any files (DB store + IPFS pin of the firkin metadata), or (2) picking a torrent — same DB store + IPFS pin of metadata, plus a `torrent magnet` file entry; the torrent-completion background task takes over from there to pin the resulting files to IPFS and roll the firkin version forward. Either path redirects to `/catalog/[ipfsHash]` (the real, content-addressed detail page).

**Bookmarking semantics.** Every `POST /api/firkins` (used by both the Bookmark button and the torrent-pick flow) does two things atomically from the WebUI's perspective: it writes the firkin record to SurrealDB under its content-addressed id, and it pins the firkin's serialized JSON body to the embedded IPFS node so the metadata is discoverable across the private swarm. The IPFS pin is recorded in the `ipfs_pin` table with a synthetic path `firkin://<id>` and mime `application/json`, alongside the file pins produced by library scans. The IPFS pin is best-effort — failures are logged and do not fail the request, so creates still succeed if the IPFS node is still warming up.

**YouTube extraction (music + trailers).** The `/catalog/virtual` and `/catalog/[ipfsHash]` pages share one YouTube-match stack at [apps/cloud/web/src/lib/youtube-match.service.ts](apps/cloud/web/src/lib/youtube-match.service.ts): a free-text query goes to `/api/ytdl/search`, then a "double-dip" picker filters down to the best match. **Music**: `pickBestYouTubeMatch` requires ≥50% of the track title's tokens to appear in the result, then scores by track-title overlap, artist hits in title+uploader, album hits in title, and duration delta — used to back-fill `url`-typed `files` entries on MusicBrainz firkins. **Trailers** (movies and TV-per-season): `pickBestTrailerMatch` reuses the same shape — ≥50% of the item's title tokens are required, the result must contain `"trailer"`, and (for TV) the season tag (`s01`, `season 1`, `s1`) is required; scoring rewards title overlap, the trailer keyword, year hits, and season-tag hits, while `reaction`/`review`/`recap`/`breakdown`/`fanmade`/`behind the scenes` etc. impose a negative penalty so commentary clips lose to the actual trailer. Resolved trailers are persisted on `firkin.trailers` (`{ youtubeUrl, label? }`): one entry for movies (no label), one per season for TV shows (label = `"Season N"`). `tmdb-tv` firkins also persist their upstream id as a `url` file (`https://www.themoviedb.org/tv/<id>`) so the detail page can re-fetch the season list from `/api/catalog/tmdb-tv/:id/seasons` if the stored array is empty.

### Transport Layer

All frontend-to-backend communication goes through `packages/frontend/src/transport/`:
- `transport.type.ts` — `Transport` interface (fetch, subscribe, resolveUrl)
- `fetch-helpers.ts` — `fetchJson()`, `fetchRaw()`, `subscribeSSE()` used by all services
- `transport-context.ts` — Module-level singleton (`setTransport`/`getTransport`) that defaults to plain HTTP via `globalThis.fetch`. Kept indirect so tests can swap in a mocked transport.

### How the frontend SPA wires up

`packages/frontend/src/routes/+layout.svelte` assembles the shared components, all imported through the local aliases:

```svelte
<script>
  import Navbar from '$components/core/Navbar.svelte';
  import ModalOutlet from '$components/core/ModalOutlet.svelte';
  import TorrentModalContent from '$components/torrent/TorrentModalContent.svelte';
  import { modalRouterService } from '$services/modal-router.service';
  // ...
</script>

<Navbar brand={{ label: 'Mhaol' }} items={navItems} />
<main>{@render children()}</main>
<ModalOutlet {modals} />
```

### Alias configuration

The frontend SPA's `svelte.config.js` points aliases at its own `src/`:

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

(SvelteKit reserves `$lib` for `src/lib/` and `$app/*` for its own modules; both work as expected.)

`src/css/app.css` scans the SPA's own `src/` for Tailwind classes:

```css
@import 'tailwindcss';
@plugin 'daisyui';
@source '../';
@import './themes.css';
```

### Catalog detail routes (`/catalog/virtual` and `/catalog/[ipfsHash]`)

The two catalog detail pages share their full presentation through components in `$components/catalog/` and behaviour through resolver services in `$services/catalog/`. The pages themselves only own route-specific wiring (URL params vs. loader-fed firkin, "Bookmark" vs. "Play / IPFS Play / Find metadata / Delete", and whether resolved data is persisted back to the firkin via `PUT /api/firkins/:id`).

**Shared components** (`packages/frontend/src/components/catalog/`):
- `CatalogPageHeader.svelte` — back link, title, addon/kind/year badges, optional `extraBadge`, action snippet slot
- `CatalogDescriptionPanel.svelte` — tabbed panel showing the description (default tab), identity (CID / created / updated / version, detail only), and version history (`version_hashes` chain, detail only). Tabs are only rendered when the corresponding props are supplied — virtual pages get a description-only single-tab layout with no tab strip
- `CatalogImagesCard.svelte` — images grid with metadata
- `CatalogTrailersCard.svelte` — trailers list driven by a `TrailerResolver`
- `CatalogTracksCard.svelte` — MusicBrainz tracks list driven by a `TrackResolver`
- `CatalogTorrentSearchCard.svelte` — torrent search results, optional collapsible + per-row streamability eval
- `CatalogSubsLyricsCard.svelte` — subs/lyrics search results driven by a `SubsLyricsResolver` (auto-fired on detail mount: lyrics for MusicBrainz albums, subtitles for TMDB movies/TV). Read-only — clicking a row previews lyrics inline or opens the subtitle URL
- `CatalogFilesTable.svelte` — firkin `files` table (detail only)

**Shared resolver services** (`packages/frontend/src/services/catalog/`):
- `trailer-resolver.svelte.ts` — `TrailerResolver` class. Holds `$state` for `trailers`, `status`, `playingKey`, `playError`. `resolveMovie(...)` / `resolveTv(...)` accept TMDB-sourced trailers via `stored`, prefer them when present, and only fall back to the YouTube fuzzy search when TMDB has nothing English. Optional `persist` callback (used by the detail page) lets each resolution write back to the firkin via `PUT /api/firkins/:id`.
- `track-resolver.svelte.ts` — `TrackResolver` class. Holds `$state` for `tracks`, `status`, `playingIndex`, `playError`. Pure projection: `loadFromFirkin({ releaseGroupId, files })` fetches the MusicBrainz tracklist and pairs each track with its YouTube URL + lyrics from the firkin's persisted `files`. *No in-browser searches.* All YT + LRCLIB resolution happens server-side, auto-spawned as a `tokio::spawn` background task by `POST /api/firkins` for fresh musicbrainz albums (so closing the tab never interrupts it), and rolled forward on the server. The detail page polls the firkin while any track is missing data and navigates to the rolled-forward CID when the background task completes.
- `torrent-search.svelte.ts` — `TorrentSearch` class. Holds `$state` for `matches`, `status`, `rowEvals`. Optional `evaluate: true` runs `/api/torrent/evaluate` per result with a sliding-window concurrency cap so the eval column on the detail page shows streamability without saturating the torrent client. Also exports `startTorrentDownload(magnet)`.

**Page-specific logic**:
- `/catalog/virtual` synthesises a `CloudFirkin` from URL params, instantiates the three resolvers without a `persist` callback, and only writes anything via the **Bookmark** button (which calls `firkinsService.create(...)` with `resolvedTrackFiles()` and `resolvedTrailers()` from the resolvers, then redirects to the new content-addressed detail URL).
- `/catalog/[ipfsHash]` loads the firkin via `+page.ts` and instantiates the same resolvers **with** `persist` callbacks pointing at a single `persistFirkinPatch(patch)` helper that calls `PUT /api/firkins/:id` and follows the response to its (potentially new) CID. The detail page also adds the playback / IPFS-play / torrent-stream actions, the `CatalogIdentityCard` / `CatalogVersionHistoryCard` / `CatalogFilesTable` extras, the artists backfill effect, the magnet auto-start effect, and the in-place `firkinsService.enrich(...)` flow used by the **Find metadata** modal.

This is the canonical pattern for cross-route reuse in the frontend SPA: shared presentation in `$components/<feature>/`, shared behaviour in `$services/<feature>/<thing>.svelte.ts` (the `.svelte.ts` extension lets `$state` runes work in service classes), and per-route wiring stays in the route's `+page.svelte`.

### Media Route Architecture

Media routes use slug-based routing with a data-driven registry:

```
(app)/media/
├── +layout.svelte              # Media bar (title, controls, tabs, filters)
├── [slug]/                     # movies, tv
│   ├── +page.ts               # Validates slug against MEDIA_REGISTRY
│   ├── +page.svelte           # CatalogBrowsePage + per-type extras
│   └── [id]/+page.svelte      # CatalogDetailPage + per-type meta
├── music/                      # Music hub + nested sub-slugs
│   ├── +page.svelte           # Hub (pinned, favorites, popular preview)
│   ├── [subslug]/             # album, artist
│   │   ├── +page.ts           # Validates subslug against MUSIC_REGISTRY
│   │   ├── +page.svelte       # CatalogBrowsePage with strategy
│   │   └── [id]/+page.svelte  # CatalogDetailPage + meta
├── youtube/                    # Explicit (custom UI: channels, RSS, downloads)
└── photos/                     # Explicit (custom UI: gallery, tagging)
```

**Key files** (all paths relative to `packages/frontend/`):
- `src/data/media-registry.ts` — `MEDIA_REGISTRY` and `MUSIC_REGISTRY` mapping slugs to config (kind, label, services, features)
- `src/components/catalog/CatalogBrowsePage.svelte` — Unified browse with search, tabs, filters, pinned/favorites, grid
- `src/components/catalog/filters/CatalogFilterBar.svelte` — Switch component rendering the right filter UI per kind
- `src/services/catalog.service.ts` — Strategy-pattern service (`CatalogKindStrategy` interface)
- `src/services/catalog-strategies/` — Per-kind strategies (movie, tv, album, artist, game)

**Adding a new media type:** Add an entry to `MEDIA_REGISTRY` (or `MUSIC_REGISTRY`), create a catalog strategy, a detail meta component, and add filter handling if needed. The slug routes handle everything else.

---

## Workspace Scripts

Run these from the **repo root**:

```bash
# Development
pnpm dev              # Cloud + tray-only Tauri shell ("Mhaol Cloud"): builds the mhaol-cloud binary, then runs Rust loopback :9899 + Vite WebUI :9898 + libp2p TCP :9900 + libp2p /ws :9901 + tray icon (no window).
pnpm dev:headless     # Same backend stack as `pnpm dev` but skips the Tauri tray — builds mhaol-headless, then runs Rust loopback :9899 + Vite WebUI :9898 (no tray, no window).
pnpm dev:cloud:web    # Vite dev server for the cloud WebUI only (port 9898, proxies /api → 127.0.0.1:9899)

# Building
pnpm build            # Alias for build:cloud (the only release artifact in this monorepo).
pnpm build:cloud:web  # Build cloud WebUI static assets only
pnpm build:cloud      # Builds the cloud WebUI, then the mhaol-cloud release binary which embeds it.
pnpm build:headless   # Builds the cloud WebUI, then the mhaol-headless release binary which embeds it.

# Quality
pnpm lint             # Lint all packages
pnpm check            # svelte-check + cargo check
pnpm test             # vitest
pnpm format           # Prettier write

# Tauri shell
pnpm app:tauri:cloud         # Mhaol Cloud desktop shell (apps/cloud)
pnpm app:tauri:cloud:build   # Mhaol Cloud release build

# Headless (no Tauri, no Vite — bin only with the embedded SPA)
pnpm app:headless            # Run via cargo (rebuilds on source change)
pnpm app:headless:bin        # Run the precompiled debug bin from ./target/debug/mhaol-headless

# Cleanup
pnpm clean            # Clean build artifacts, cargo clean, remove SQLite DBs
```

Never cd into a package directory to run scripts — use the root workspace scripts above.

---

## Logs

The dev scripts tee full stdout+stderr (cargo build noise, panics, `tracing` events, Vite output — everything) into `./logs/` at the repo root. **When debugging the cloud, check these files first instead of asking the user to paste output.**

| Script | Log file |
|---|---|
| `pnpm dev` (cloud strand) | `logs/cloud.log` |
| `pnpm dev` (web strand) | `logs/web.log` |
| `pnpm dev` (tauri strand) | `logs/tauri.log` |
| `pnpm dev:headless` (headless strand) | `logs/headless.log` |
| `pnpm dev:headless` (web strand) | `logs/web.log` |
| `pnpm app:headless` / `pnpm app:headless:bin` | `logs/headless.log` |

Each file is overwritten on the next run, so it always reflects the latest run. The `logs/` directory is gitignored.

---

## Git Workflow

After every change, immediately commit the affected files:

- **Who**: use the git account configured for this repo — do not override it. Never use `Co-Authored-By` or any other attribution to Claude/AI in commits.
- **What**: stage only the files actually modified in that change
- **Message**: a single short phrase in plain English, no emoji, no period, no conventional-commit prefixes
- **When**: one commit per logical change — never batch unrelated edits
- **Before committing**: run `pnpm lint`, `pnpm check`, and `pnpm test` and fix any errors. `pnpm check` runs `svelte-check` + `cargo check` which is enough to confirm everything compiles. **Do NOT run release builds** (`pnpm build`, `pnpm build:cloud`, `pnpm build:rendezvous`, `cargo build --release`, Tauri release bundles) during active development unless the user explicitly asks — they are slow, heavy, and not part of the normal verification loop.

```bash
# Verify checks pass (no release builds)
pnpm lint && pnpm check && pnpm test

# Then commit
git add packages/frontend/src/components/media/MediaCard.svelte
git commit -m "add thumbnail fallback to MediaCard"
```

---

## Feature Implementation Checklist

When adding a new feature that spans the full stack:

**Backend (`packages/backend`)**
- [ ] Create API module in `src/{feature}.rs` exposing a `pub fn router() -> Router<CloudState>`
- [ ] Add `mod {feature};` to `src/lib.rs`
- [ ] Register route in `lib.rs::run`: `.nest("/api/{feature}", {feature}::router())`
- [ ] Add any new managers/repos to `CloudState`

**Frontend (`packages/frontend`)**
- [ ] Define types in `src/types/{feature}.type.ts`
- [ ] Create adapter in `src/adapters/classes/{feature}.adapter.ts` (when wrapping an external API or signaling channel)
- [ ] Create/extend service in `src/services/{feature}.service.ts` (or `src/services/{feature}/{thing}.svelte.ts` for runes-driven service classes)
- [ ] Create component(s) in `src/components/{feature}/` using the `$components`, `$services`, `$types`, `$adapters`, `$utils`, `$transport`, `$lib` aliases
- [ ] Use `classnames` for all conditional styling — never `<style>` tags or inline styles
- [ ] Use `<Icon name="<author>/<icon>" />` from `cloud-ui` for any UI glyph — never emoji (see "Icons" below)
- [ ] Components stay presentational: callback props in, no business logic; resolvers/adapters/services own the state machines and side-effects
- [ ] When two routes need the same UI, extract the markup into `$components/<feature>/` and the behaviour into `$services/<feature>/<thing>.svelte.ts` — see "Catalog detail routes" above for the canonical pattern
- [ ] Write tests in `test/`

**Always**
- [ ] Commit each logical change immediately after completing it
- [ ] Update the relevant `CLAUDE.md` (`packages/backend`, `packages/frontend`, `apps/cloud`) if adding new modules, components, services, or adapters

---

## Icons

UI glyphs come from the game-icons.net set bundled in `packages/cloud-ui/src/icons/assets/<author>/<name>.svg` (4180 SVGs across 36 contributors, all rewritten to `fill="currentColor"`). **Never use emoji in UI**; never inline a custom SVG when one of these will do.

```svelte
<script>
  import { Icon } from 'cloud-ui';
</script>

<button class="text-primary hover:text-secondary">
  <Icon name="delapouite/save" size={18} title="Save" />
  Save
</button>
```

The icon inherits the surrounding text colour via `currentColor`, so colour it with the usual Tailwind/DaisyUI text utilities (`text-primary`, `text-error`, `text-base-content/60`, etc.). `size` accepts a number (px) or any CSS length and defaults to `1em`. Pass `title` only when the icon stands alone semantically — when it sits next to a text label, leave it off so screen readers don't double-read.

**Before writing `<Icon name="…" />`, verify the file exists on disk** — pick a name from `packages/cloud-ui/src/icons/assets/<author>/<name>.svg` (or grep `packages/cloud-ui/src/icons/icon-names.ts`). The component silently renders nothing when the name doesn't match a real file, so a typo produces an invisible icon, not a build error. If no existing icon fits, search the broader set first; only add a new SVG (under the same `<author>/<name>.svg` convention, with `fill="currentColor"`) when nothing in the set works, and re-run `node packages/cloud-ui/scripts/generate-icon-names.mjs` after adding one so the `IconName` union includes it.

`Icon`, `ICON_NAMES`, and the `IconName` type are exported from the `cloud-ui` package root.

---

## Keeping CLAUDE.md Updated

When making significant structural changes (new packages, new component directories, new services, renaming files, changing the app architecture), update the relevant CLAUDE.md files immediately:

- **Root CLAUDE.md** — Monorepo structure, app architecture, workspace scripts
- **apps/cloud/CLAUDE.md** — Tauri shell (`mhaol-cloud-shell`): tray menu, lifecycle, conf paths
- **apps/headless/CLAUDE.md** — Terminal-only shell (`mhaol-headless`): thin bin over `mhaol_backend::run()`
- **packages/backend/CLAUDE.md** — Server API modules + routes, SurrealDB store, paths, on-disk layout
- **packages/frontend/CLAUDE.md** — SPA: components, services, adapters, types, utils, CSS/themes, transport layer
- **apps/rendezvous/CLAUDE.md** — Rendezvous app

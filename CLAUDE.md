# Arktosmos Mhaol - Development Guidelines

This document guides Claude (and developers) on implementing features in this monorepo. Follow these conventions strictly to maintain consistency across all packages.

For package-specific conventions, see the `CLAUDE.md` in each package directory:
- `packages/ui-lib/CLAUDE.md` — UI components, services, types, adapters, utils, CSS/themes, transport layer
- `packages/webrtc/CLAUDE.md` — WebRTC contact handshake layer
- `apps/cloud/CLAUDE.md` — Cloud server + cloud desktop Tauri shell (`mhaol-cloud-shell`)
---

## Monorepo Overview

```
mhaol.git/
├── apps/
│   ├── cloud/                        # Rust Axum server + nested Svelte WebUI (port 9898) + tray-only Tauri shell at cloud/src-tauri (mhaol-cloud-shell, "Mhaol Cloud")
│   ├── shepperd/                     # Browser extension (Vite + Svelte, Manifest V3)
│   └── signaling/                    # Rust signaling server (self-hosted alternative to PartyKit)
├── packages/
│   ├── ui-lib/                       # Shared frontend: components, services, types, adapters, transport, CSS
│   ├── addons/                       # Addon modules (TMDB, MusicBrainz, RetroAchievements, YouTube, LRCLIB, OpenLibrary, Wyzie subtitles, torrent search)
│   ├── identity/                     # Rust Ethereum identity management (secp256k1, EIP-191)
│   ├── signaling/                    # PartyKit signaling service
│   ├── p2p-stream/                   # Rust P2P streaming library
│   ├── torrent/                      # Rust torrent implementation
│   ├── ed2k/                         # Rust eDonkey/ed2k network client (search + add)
│   ├── ipfs/                         # Rust IPFS node (libp2p + Bitswap + Kademlia DHT, embedded)
│   └── webrtc/                       # WebRTC contact handshake layer (TypeScript)
├── pnpm-workspace.yaml
└── package.json                      # Root workspace scripts
```

**Runtime requirements:** Node >= 18, pnpm >= 9, Rust (cargo)

---

## App Architecture

The cloud SPA at `apps/cloud/web/` is a thin wrapper that imports everything from `packages/ui-lib`. It contains **only**:

- `src/routes/` — SvelteKit route files (+page.svelte, +layout.svelte)
- `src/css/app.css` — CSS entry point (imports Tailwind, DaisyUI, scans ui-lib)
- `src/app.html`, `src/app.d.ts` — SvelteKit boilerplate
- Config files (svelte.config.js, vite.config.ts, package.json, tsconfig.json)

It **never** implements its own components, services, adapters, types, or utils. Everything lives in `packages/ui-lib`.

### Cloud

The cloud is a Rust Axum server at `apps/cloud/` that bootstraps an embedded SurrealDB store, an identity manager, and the desktop-only managers (`mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ed2k`, `mhaol-ipfs`, `mhaol-p2p-stream`). It hosts a nested Svelte WebUI that displays cloud health and library/IPFS state. Crate name `mhaol-cloud`, binary `mhaol-cloud`, default port 9898 (in dev, the binary binds 127.0.0.1:9899 and the Vite dev server takes 9898 as the public port).

- `apps/cloud/Cargo.toml` — Crate manifest
- `apps/cloud/src/server.rs` — Binary entry point; opens SurrealDB, spawns workers, serves the embedded WebUI as a fallback to `/api/*`
- `apps/cloud/src/cloud_status.rs` — Public `/api/cloud/status` route used by the WebUI for health polling
- `apps/cloud/src/libraries.rs` — `/api/libraries` CRUD; library records are stored in SurrealDB and identified by their on-disk directory path
- `apps/cloud/src/frontend.rs` — Embeds `apps/cloud/web/dist-static/` via `rust-embed` and serves it as the fallback handler
- `apps/cloud/web/` — SvelteKit static SPA (pnpm package `cloud`). Builds to `apps/cloud/web/dist-static/`, which is what the cloud crate embeds at compile time.

### Tauri shell

`apps/cloud/src-tauri/` — crate `mhaol-cloud-shell`, binary `mhaol-cloud-shell`. `productName: "Mhaol Cloud"`, identifier `com.arktosmos.mhaol.cloud`. **Tray-only**: `app.windows: []`, no window is ever created. macOS sets `ActivationPolicy::Accessory` (no dock icon), `RunEvent::ExitRequested` calls `prevent_exit()` so the process stays alive without windows. The system tray icon (id `mhaol-cloud-tray`, tooltip "Mhaol Cloud") has two items: **Open** opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`, **Quit** calls `app.exit(0)`. `tauri.conf.json` keeps `frontendDist: ../web/dist-static` / `devUrl: http://localhost:9898` so build/dev tooling resolves cleanly; nothing actually renders the assets at runtime.

The cloud WebUI stays browser-accessible at `http://localhost:9898`.

Layout:
- `apps/cloud/src-tauri/Cargo.toml` — crate manifest
- `apps/cloud/src-tauri/src/{lib.rs,main.rs}` — Tauri entry point
- `apps/cloud/src-tauri/tauri.conf.json` — desktop config
- `apps/cloud/src-tauri/capabilities/default.json`, `icons/`, `build.rs`

The cloud frontend has these screens:
- **Health** (`/`) — polls `/api/cloud/status` every 5 seconds and renders status, latency, uptime, bind, package health, and identities.
- **Libraries** (`/libraries`) — lists, creates, and removes library records via `/api/libraries`. The form lets you pick an existing directory, or browse to a parent and create a new subfolder; each library is identified by its directory path and carries a `kinds` list selecting which catalog types it contains (`movie`, `tv`, `album`, `book`, `game` — same ids as `/api/catalog/sources`). Each row has a `Scan` button that walks the directory recursively, reports file size + MIME, asynchronously pins media to IPFS, and (when `kinds` is non-empty) groups files into `document` records per detected media item — TV shows aware of nested season directories, albums grouped by directory, books/games per file. The row also shows the IPFS pins (CID, path, MIME, size) recorded for that library, and on page load any library whose `last_scanned_at` is missing or older than 1 hour is rescanned automatically.
- **IPFS** (`/ipfs`) — reads `/api/ipfs/pins` and lists every pin recorded by library scans (CID, path, MIME, size).
- **Catalog** (`/catalog`) — pick an addon (TMDB, MusicBrainz, OpenLibrary, RetroAchievements), optionally narrow by type (movie/tv) and genre/subject/console, and browse popular items. The Rust server proxies upstream calls via `/api/catalog/*` so addon API keys (`TMDB_API_KEY`, `RA_USERNAME` + `RA_API_KEY`) stay server-side. Grid items are **virtual** — nothing is written to SurrealDB and nothing is pinned to IPFS while browsing. Clicking "View details →" navigates to `/catalog/virtual?...` (a virtual detail synthesised from URL query params) which immediately runs a torrent search. Picking a torrent is the first persistence event: it `POST`s `/api/documents` with the magnet attached, then redirects to `/catalog/[ipfsHash]` (the real, content-addressed detail page). The torrent-completion background task on the server takes over from there to pin the resulting files to IPFS and roll the document version forward.

### Transport Layer

All frontend-to-backend communication goes through `packages/ui-lib/src/transport/`:
- `transport.type.ts` — `Transport` interface (fetch, subscribe, resolveUrl)
- `http-transport.ts` — HTTP implementation (wraps browser fetch)
- `webrtc-transport.ts` — WebRTC RPC implementation (sends requests over data channels)
- `fetch-helpers.ts` — `fetchJson()`, `fetchRaw()`, `subscribeSSE()` used by all services
- `transport-context.ts` — Module-level singleton (`setTransport`/`getTransport`)
- `rpc.type.ts` — RPC message protocol types

### How the cloud SPA wires up

`apps/cloud/web/src/routes/+layout.svelte` assembles the shared components:

```svelte
<script>
  import Navbar from 'ui-lib/components/core/Navbar.svelte';
  import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
  import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
  import { modalRouterService } from 'ui-lib/services/modal-router.service';
  // ...

  // Data-driven navbar: pass items array
  const navItems = [
    { id: 'torrent', label: 'Torrent', classes: 'btn-primary' },
    { id: 'downloads', label: 'Downloads', classes: 'btn-secondary' },
  ];

  // Data-driven modal outlet: map ids to components
  const modals = {
    torrent: { component: TorrentModalContent, maxWidth: 'max-w-5xl' },
    downloads: { component: DownloadsModalContent, maxWidth: 'max-w-5xl' },
  };
</script>

<Navbar brand={{ label: 'Mhaol' }} items={navItems} />
<main>{@render children()}</main>
<ModalOutlet {modals} />
```

### Alias configuration

The cloud SPA's `svelte.config.js` points aliases to `packages/ui-lib`:

```javascript
alias: {
  $components: '../../../packages/ui-lib/src/components',
  $services: '../../../packages/ui-lib/src/services',
  $types: '../../../packages/ui-lib/src/types',
  $adapters: '../../../packages/ui-lib/src/adapters',
  $utils: '../../../packages/ui-lib/src/utils',
  $data: '../../../packages/ui-lib/src/data',
  'ui-lib': '../../../packages/ui-lib/src'
}
```

Its `src/css/app.css` scans ui-lib for Tailwind classes:

```css
@import 'tailwindcss';
@plugin 'daisyui';
@source '../../../packages/ui-lib/src';
@import 'ui-lib/css/themes.css';
```

### Media Route Architecture

Media routes use slug-based routing with a data-driven registry:

```
(app)/media/
├── +layout.svelte              # Media bar (title, controls, tabs, filters)
├── [slug]/                     # movies, tv, books, videogames, iptv
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

**Key files:**
- `packages/ui-lib/src/data/media-registry.ts` — `MEDIA_REGISTRY` and `MUSIC_REGISTRY` mapping slugs to config (kind, label, services, features)
- `packages/ui-lib/src/components/catalog/CatalogBrowsePage.svelte` — Unified browse with search, tabs, filters, pinned/favorites, grid
- `packages/ui-lib/src/components/catalog/filters/CatalogFilterBar.svelte` — Switch component rendering the right filter UI per kind
- `packages/ui-lib/src/services/catalog.service.ts` — Strategy-pattern service (`CatalogKindStrategy` interface)
- `packages/ui-lib/src/services/catalog-strategies/` — Per-kind strategies (movie, tv, book, album, artist, game, iptv)

**Adding a new media type:** Add an entry to `MEDIA_REGISTRY` (or `MUSIC_REGISTRY`), create a catalog strategy, a detail meta component, and add filter handling if needed. The slug routes handle everything else.

---

## Workspace Scripts

Run these from the **repo root**:

```bash
# Development
pnpm dev              # Cloud + tray-only Tauri shell ("Mhaol Cloud"): Rust loopback :9899 + Vite WebUI :9898 + tray icon (no window)
pnpm dev:cloud:web    # Vite dev server for the cloud WebUI only (port 9898, proxies /api → 127.0.0.1:9899)

# Building
pnpm build            # Build cloud WebUI + mhaol-cloud release binary
pnpm build:cloud:web  # Build cloud WebUI static assets only
pnpm build:cloud      # Build cloud WebUI then build mhaol-cloud release binary (embeds the WebUI)

# Quality
pnpm lint             # Lint all packages
pnpm check            # svelte-check + cargo check
pnpm test             # vitest
pnpm format           # Prettier write

# Browser extension
pnpm app:shepperd         # Shepperd dev (watch mode)
pnpm app:shepperd:build   # Shepperd production build

# Tauri shell
pnpm app:tauri:cloud         # Mhaol Cloud desktop shell (apps/cloud/src-tauri)
pnpm app:tauri:cloud:build   # Mhaol Cloud release build

# Signaling
pnpm signaling:dev    # PartyKit local dev
pnpm signaling:deploy # Deploy PartyKit

# Cleanup
pnpm clean            # Clean build artifacts, cargo clean, remove SQLite DBs
```

Never cd into a package directory to run scripts — use the root workspace scripts above.

---

## Git Workflow

After every change, immediately commit the affected files:

- **Who**: use the git account configured for this repo — do not override it. Never use `Co-Authored-By` or any other attribution to Claude/AI in commits.
- **What**: stage only the files actually modified in that change
- **Message**: a single short phrase in plain English, no emoji, no period, no conventional-commit prefixes
- **When**: one commit per logical change — never batch unrelated edits
- **Before committing**: all CI checks must pass locally. Run `pnpm lint`, `pnpm check`, `pnpm build`, and `pnpm test` and fix any errors before committing.

```bash
# Verify checks pass
pnpm lint && pnpm check && pnpm build && pnpm test

# Then commit
git add packages/ui-lib/src/components/media/MediaCard.svelte
git commit -m "add thumbnail fallback to MediaCard"
```

---

## Feature Implementation Checklist

When adding a new feature that spans the full stack:

**Cloud (`apps/cloud`)**
- [ ] Create API module in `src/{feature}.rs` exposing a `pub fn router() -> Router<CloudState>`
- [ ] Add `mod {feature};` to `src/server.rs`
- [ ] Register route in `server.rs`: `.nest("/api/{feature}", {feature}::router())`
- [ ] Add any new managers/repos to `CloudState`

**Shared Frontend (`packages/ui-lib`)**
- [ ] Define types in `src/types/{feature}.type.ts`
- [ ] Create adapter in `src/adapters/classes/{feature}.adapter.ts`
- [ ] Create/extend service in `src/services/{feature}.service.ts`
- [ ] Create component(s) in `src/components/{feature}/`
- [ ] Use `ui-lib/...` import paths for all cross-module references
- [ ] Use `classnames` for all conditional styling
- [ ] No `<style>` tags or inline styles
- [ ] Components use callback props, contain no business logic
- [ ] Write tests in `test/`

**Cloud WebUI (`apps/cloud/web`, if the feature needs UI wiring)**
- [ ] Import components from `ui-lib/components/...`
- [ ] Import services/types from `ui-lib/services/...`, `ui-lib/types/...`
- [ ] Add to navbar items and/or modal outlet if needed
- [ ] The app only assembles — never implements logic

**Always**
- [ ] Commit each logical change immediately after completing it
- [ ] Update `packages/ui-lib/CLAUDE.md` if adding new component directories, services, or adapters

---

## Keeping CLAUDE.md Updated

When making significant structural changes (new packages, new component directories, new services, renaming files, changing the app architecture), update the relevant CLAUDE.md files immediately:

- **Root CLAUDE.md** — Monorepo structure, app architecture, workspace scripts
- **packages/ui-lib/CLAUDE.md** — Components, services, adapters, types, utils, CSS/themes
- **apps/cloud/CLAUDE.md** — API modules, routes

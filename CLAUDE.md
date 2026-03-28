# Arktosmos Mhaol - Development Guidelines

This document guides Claude (and developers) on implementing features in this monorepo. Follow these conventions strictly to maintain consistency across all packages.

For package-specific conventions, see the `CLAUDE.md` in each package directory:
- `packages/ui-lib/CLAUDE.md` — UI components, services, types, adapters, utils, CSS/themes, transport layer
- `packages/webrtc/CLAUDE.md` — WebRTC contact handshake layer
- `apps/node/CLAUDE.md` — Rust API modules, AppState, sub-crate dependencies
---

## Monorepo Overview

```
mhaol.git/
├── apps/
│   ├── frontend/                     # Unified SPA (landing + connect + media, port 1570)
│   ├── node/                         # Rust Axum server (standalone, port 1530)
│   └── shepperd/                     # Browser extension (Vite + Svelte, Manifest V3)
├── packages/
│   ├── ui-lib/                       # Shared frontend: components, services, types, adapters, transport, CSS
│   ├── addons/                       # Addon modules (TMDB, MusicBrainz, RetroAchievements, YouTube, LRCLIB, torrent search)
│   ├── identity/                     # Rust Ethereum identity management (secp256k1, EIP-191)
│   ├── signaling/                    # PartyKit signaling service
│   ├── queue/                        # Rust task queue (SQLite + broadcast)
│   ├── p2p-stream/                   # Rust P2P streaming library
│   ├── recommendations/              # Rust TMDB recommendations (queue worker + SQLite storage)
│   ├── torrent/                      # Rust torrent implementation
│   └── webrtc/                       # WebRTC contact handshake layer (TypeScript)
├── pnpm-workspace.yaml
└── package.json                      # Root workspace scripts
```

**Runtime requirements:** Node >= 18, pnpm >= 9, Rust (cargo)

---

## App Architecture: Import & Assemble

Apps under `apps/` are **thin wrappers** that import everything from `packages/ui-lib`, then assemble them. They contain **only**:

- `src/routes/` — SvelteKit route files (+page.svelte, +layout.svelte)
- `src/css/app.css` — CSS entry point (imports Tailwind, DaisyUI, scans ui-lib)
- `src/app.html`, `src/app.d.ts` — SvelteKit boilerplate
- Config files (svelte.config.js, vite.config.ts, package.json, tsconfig.json)

Apps **never** implement their own components, services, adapters, types, or utils. Everything lives in `packages/ui-lib`.

### Node

The node is a standalone Rust Axum server at `apps/node/`. Crate name `mhaol-node`, binary `mhaol-node`. Runs headless on port 1530 and exposes its API via HTTP and WebRTC RPC.

- `apps/node/Cargo.toml` — Crate manifest
- `apps/node/src/lib.rs` — AppState, modules, database layer
- `apps/node/src/server.rs` — Binary entry point (HTTP server)
- `apps/node/src/peer_service/rpc_handler.rs` — WebRTC RPC handler (routes data channel messages through Axum router)

### Transport Layer

All frontend-to-backend communication goes through `packages/ui-lib/src/transport/`:
- `transport.type.ts` — `Transport` interface (fetch, subscribe, resolveUrl)
- `http-transport.ts` — HTTP implementation (wraps browser fetch)
- `webrtc-transport.ts` — WebRTC RPC implementation (sends requests over data channels)
- `fetch-helpers.ts` — `fetchJson()`, `fetchRaw()`, `subscribeSSE()` used by all services
- `transport-context.ts` — Module-level singleton (`setTransport`/`getTransport`)
- `rpc.type.ts` — RPC message protocol types

### How apps wire up

Each app's `+layout.svelte` assembles the shared components:

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

### App alias configuration

Every app's `svelte.config.js` points aliases to `packages/ui-lib`:

```javascript
alias: {
  $components: '../../packages/ui-lib/src/components',
  $services: '../../packages/ui-lib/src/services',
  $types: '../../packages/ui-lib/src/types',
  $adapters: '../../packages/ui-lib/src/adapters',
  $utils: '../../packages/ui-lib/src/utils',
  $data: '../../packages/ui-lib/src/data',
  'ui-lib': '../../packages/ui-lib/src'
}
```

Every app's `src/css/app.css` scans ui-lib for Tailwind classes:

```css
@import 'tailwindcss';
@plugin 'daisyui';
@source '../../packages/ui-lib/src';
@import 'ui-lib/css/themes.css';
```

---

## Workspace Scripts

Run these from the **repo root**:

```bash
# Development
pnpm dev              # Frontend (port 1570) + Rust node (port 1530) in parallel
pnpm dev:node         # Rust node server only (PORT=1530)
pnpm dev:frontend     # Frontend dev server only (port 1570)

# Building
pnpm build            # Frontend build
pnpm build:node       # Rust node release build

# Quality
pnpm lint             # Lint all packages
pnpm check            # svelte-check + cargo check
pnpm test             # vitest + cargo test
pnpm format           # Prettier write

# Browser extension
pnpm app:shepperd         # Shepperd dev (watch mode)
pnpm app:shepperd:build   # Shepperd production build

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

**Node (`apps/node`)**
- [ ] Create API module in `src/api/{feature}.rs`
- [ ] Add `pub mod {feature};` to `src/api/mod.rs`
- [ ] Register route in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
- [ ] Add any new repos to `AppState`

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

**Apps (if the feature needs UI wiring)**
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
- **App CLAUDE.md files** — Which features the app uses, how it assembles them
- **apps/node/CLAUDE.md** — API modules, routes

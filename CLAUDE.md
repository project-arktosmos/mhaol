# Arktosmos Mhaol - Development Guidelines

This document guides Claude (and developers) on implementing features in this monorepo. Follow these conventions strictly to maintain consistency across all packages.

For package-specific conventions, see the `CLAUDE.md` in each package directory:
- `packages/ui-lib/CLAUDE.md` — UI components, CSS/themes
- `packages/frontend/CLAUDE.md` — Services, adapters, types, utils
- `packages/backend/CLAUDE.md` — Rust API modules, AppState, sub-crate dependencies
- `packages/database/CLAUDE.md` — SQLite schema, repository pattern

---

## Monorepo Overview

```
mhaol.git/
├── apps/                             # Thin SvelteKit wrappers (routes + assembly only)
│   ├── mhaol-video/                  # Main media app (Vite, port 1531)
│   ├── tube/                         # YouTube-focused app (has src-tauri/ for desktop build)
│   ├── cloud/                        # Cloud library management (has src-tauri/ for desktop build)
│   ├── video/                        # Full-featured video app
│   ├── signaling/                    # Signaling server management dashboard (has src-tauri/ for desktop build, port 1520)
│   ├── website/                      # Marketing landing page (base: /mhaoltube)
│   └── server/                       # Server orchestrator (starts frontend + backend)
├── packages/
│   ├── ui-lib/                       # UI components + CSS/themes (Svelte, Tailwind, DaisyUI)
│   ├── frontend/                     # Shared frontend logic (services, adapters, types, utils)
│   ├── backend/                      # Rust Axum server (port 1530)
│   ├── database/                     # SQLite schema & repositories (better-sqlite3)
│   ├── identity/                     # Rust Ethereum identity management (secp256k1, EIP-191)
│   ├── signaling/                    # PartyKit signaling service
│   ├── tauri/                        # Shared Tauri library + assets (mhaol-tauri-core)
│   ├── p2p-stream/                   # Rust P2P streaming library
│   └── torrent/                      # Rust torrent implementation
├── addons/
│   ├── common/                       # Shared addon config
│   ├── tmdb/                         # TMDB movie/TV metadata (SQLite cache)
│   └── torrent-search-thepiratebay/  # Torrent search via PirateBay API
├── pnpm-workspace.yaml
└── package.json                      # Root workspace scripts
```

**Runtime requirements:** Node >= 18, pnpm >= 9, Rust (cargo)

---

## App Architecture: Import & Assemble

Apps under `apps/` are **thin wrappers** that import UI from `packages/ui-lib` and logic from `packages/frontend`, then assemble them. They contain **only**:

- `src/routes/` — SvelteKit route files (+page.svelte, +layout.svelte)
- `src/css/app.css` — CSS entry point (imports Tailwind, DaisyUI, scans both ui-lib and frontend)
- `src/app.html`, `src/app.d.ts` — SvelteKit boilerplate
- Config files (svelte.config.js, vite.config.ts, package.json, tsconfig.json)

Apps **never** implement their own components, services, adapters, types, or utils. Components live in `packages/ui-lib`, everything else in `packages/frontend`.

### Tauri Desktop Builds

Apps that support desktop builds have a `src-tauri/` directory containing:
- `Cargo.toml` — Depends on `mhaol-tauri-core` (shared library in `packages/tauri/src-tauri/`)
- `tauri.conf.json` — App-specific product name, identifier, frontend build commands
- `src/lib.rs` + `src/main.rs` — Thin wrappers that call `mhaol_tauri_core::setup_backend()`
- `build.rs` — Standard `tauri_build::build()`

Shared assets (icons, capabilities, loading screen) live in `packages/tauri/assets/` and are copied into each app's `src-tauri/` via `bash packages/tauri/scripts/sync-assets.sh apps/{app}`. These copied assets are gitignored.

Currently enabled for: **tube**, **cloud**. To add Tauri support to another app, create `apps/{app}/src-tauri/` following the same pattern.

### How apps wire up

Each app's `+layout.svelte` assembles the shared components:

```svelte
<script>
  import Navbar from 'ui-lib/components/core/Navbar.svelte';
  import ModalOutlet from 'ui-lib/components/core/ModalOutlet.svelte';
  import TorrentModalContent from 'ui-lib/components/torrent/TorrentModalContent.svelte';
  import { modalRouterService } from 'frontend/services/modal-router.service';
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

Every app's `svelte.config.js` points aliases to both `packages/ui-lib` and `packages/frontend`:

```javascript
alias: {
  $components: '../../packages/ui-lib/src/components',
  $services: '../../packages/frontend/src/services',
  $types: '../../packages/frontend/src/types',
  $adapters: '../../packages/frontend/src/adapters',
  $utils: '../../packages/frontend/src/utils',
  $data: '../../packages/frontend/src/data',
  frontend: '../../packages/frontend/src',
  'ui-lib': '../../packages/ui-lib/src'
}
```

Every app's `src/css/app.css` scans both packages for Tailwind classes:

```css
@import 'tailwindcss';
@plugin 'daisyui';
@source '../../packages/frontend/src';
@source '../../packages/ui-lib/src';
@import 'ui-lib/css/themes.css';
```

---

## Workspace Scripts

Run these from the **repo root**:

```bash
# Development
pnpm dev              # Build + start Rust backend, then frontend dev server
pnpm dev:backend      # Rust backend only (PORT=1530)
pnpm dev:frontend     # SvelteKit dev server only (port 1531)

# Building
pnpm build            # Frontend build
pnpm build:backend    # Rust backend release build

# Quality
pnpm lint             # Lint all packages
pnpm check            # svelte-check + cargo check
pnpm test             # vitest + cargo test
pnpm format           # Prettier write

# Desktop (Tauri) — per-app builds
pnpm tauri:dev            # Default Tauri dev (tube)
pnpm tauri:dev:tube       # Tube desktop dev
pnpm tauri:dev:cloud      # Cloud desktop dev
pnpm tauri:build          # Default Tauri build (tube)
pnpm tauri:build:tube     # Tube desktop build
pnpm tauri:build:cloud    # Cloud desktop build

# Signaling
pnpm signaling:dev    # PartyKit local dev
pnpm signaling:deploy # Deploy PartyKit

# Cleanup
pnpm clean            # Clean build artifacts + cargo clean
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
git add packages/frontend/src/components/media/MediaCard.svelte
git commit -m "add thumbnail fallback to MediaCard"
```

---

## Feature Implementation Checklist

When adding a new feature that spans the full stack:

**Database (`packages/database`)**
- [ ] Add/update table in `src/schema.ts`
- [ ] Create repository in `src/repositories/`
- [ ] Export from `src/repositories/index.ts`

**Backend (`packages/backend`)**
- [ ] Create API module in `src/api/{feature}.rs`
- [ ] Add `pub mod {feature};` to `src/api/mod.rs`
- [ ] Register route in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
- [ ] Add any new repos to `AppState`

**Frontend (`packages/frontend`)**
- [ ] Define types in `src/types/{feature}.type.ts`
- [ ] Create adapter in `src/adapters/classes/{feature}.adapter.ts`
- [ ] Create/extend service in `src/services/{feature}.service.ts`
- [ ] Use `frontend/...` import paths for cross-module references
- [ ] Write tests in `test/`

**UI Library (`packages/ui-lib`)**
- [ ] Create component(s) in `src/components/{feature}/`
- [ ] Use `classnames` for all conditional styling
- [ ] No `<style>` tags or inline styles
- [ ] Components use callback props, contain no business logic
- [ ] Use `ui-lib/...` for cross-component imports, `frontend/...` for services/types

**Apps (if the feature needs UI wiring)**
- [ ] Import the new component(s) from `ui-lib/components/...`
- [ ] Import services/types from `frontend/...`
- [ ] Add to navbar items and/or modal outlet if needed
- [ ] The app only assembles — never implements logic

**Always**
- [ ] Commit each logical change immediately after completing it
- [ ] Update `packages/ui-lib/CLAUDE.md` if adding new component directories
- [ ] Update `packages/frontend/CLAUDE.md` if adding new services or adapters

---

## Keeping CLAUDE.md Updated

When making significant structural changes (new packages, new component directories, new services, renaming files, changing the app architecture), update the relevant CLAUDE.md files immediately:

- **Root CLAUDE.md** — Monorepo structure, app architecture, workspace scripts
- **packages/ui-lib/CLAUDE.md** — Component directories, CSS/themes
- **packages/frontend/CLAUDE.md** — Services list, adapters list, types, utils
- **App CLAUDE.md files** — Which features the app uses, how it assembles them
- **packages/backend/CLAUDE.md** — API modules, routes
- **packages/database/CLAUDE.md** — Schema, repositories

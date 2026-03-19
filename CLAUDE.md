# Arktosmos Mhaol - Development Guidelines

This document guides Claude (and developers) on implementing features in this monorepo. Follow these conventions strictly to maintain consistency across all packages.

For package-specific conventions, see the `CLAUDE.md` in each package directory:
- `packages/frontend/CLAUDE.md` — Components, services, adapters, styling
- `packages/backend/CLAUDE.md` — Rust API modules, AppState, sub-crate dependencies
- `packages/database/CLAUDE.md` — SQLite schema, repository pattern

---

## Monorepo Overview

```
mhaol.git/
├── apps/
│   ├── mhaol-video/              # SvelteKit 2 app (Vite, port 1531)
│   └── mhaol-server/             # Server orchestrator (starts frontend + backend)
├── packages/
│   ├── frontend/                 # SvelteKit 2 frontend (Vite, port 1531)
│   ├── backend/                  # Rust Axum server (port 1530)
│   ├── database/                 # SQLite schema & repositories (better-sqlite3)
│   ├── signaling/                # PartyKit signaling service
│   ├── tauri/                    # Desktop/mobile wrapper (macOS, Android)
│   ├── p2p-stream/               # Rust P2P streaming library
│   └── torrent/                  # Rust torrent implementation
├── addons/
│   ├── common/                   # Shared addon config
│   ├── tmdb/                     # TMDB movie/TV metadata (SQLite cache)
│   └── torrent-search-thepiratebay/  # Torrent search via PirateBay API
├── pnpm-workspace.yaml
└── package.json                  # Root workspace scripts
```

**Runtime requirements:** Node >= 18, pnpm >= 9, Rust (cargo)

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

# Desktop / Mobile
pnpm tauri:dev        # Tauri dev mode
pnpm tauri:build      # Tauri desktop build
pnpm dev:android      # Tauri Android dev
pnpm tauri:android:build  # Build Android APK

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
- [ ] Create component(s) in `src/components/{feature}/`
- [ ] Use `classnames` for all conditional styling
- [ ] No `<style>` tags or inline styles
- [ ] Components use callback props, contain no business logic
- [ ] Use path aliases (`$services`, `$adapters`, etc.)
- [ ] Write tests in `test/`

**Always**
- [ ] Commit each logical change immediately after completing it

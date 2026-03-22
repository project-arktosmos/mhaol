# App: server

**Location:** `apps/server/`
**Type:** Thin SvelteKit 2 wrapper — media app
**Adapter:** `@sveltejs/adapter-static` (fallback to `index.html`)
**Dev port:** 1531

## Architecture

This app is an **assembly-only wrapper** around `packages/ui-lib`. It contains no components, services, adapters, types, or utils of its own. All shared code is imported from `packages/ui-lib` via the `ui-lib` workspace dependency and path aliases.

## What lives here

```
backend/                 # Rust Axum server (crate: mhaol-server)
├── Cargo.toml
├── src/
│   ├── server.rs        # Binary entry point — starts Axum server
│   ├── lib.rs           # AppState definition, module declarations
│   ├── api/             # Route handlers (one module per feature)
│   ├── db/              # Database layer (rusqlite repos)
│   ├── modules/         # Plugin modules
│   ├── signaling_rooms.rs
│   └── worker_bridge.rs
└── tests/
src/
├── routes/              # SvelteKit pages
│   ├── +layout.svelte   # App shell: Navbar (brand + items) + IdentitySidebar + ModalOutlet
│   ├── +layout.ts       # SSR disabled
│   ├── +page.svelte     # Movies library grid with sub-tabs (Library, Search, Popular, Discover, Recommendations)
│   ├── +page.ts         # Page load (fetches /api/media)
│   ├── music/           # Music album browsing (MusicBrainz, genre tabs)
│   ├── tv/              # TV show browsing (TMDB browse)
│   ├── videogames/      # Videogame browsing (RetroAchievements, console tabs)
│   └── database/        # Database browser utility page
├── css/app.css          # Tailwind entry + @source for packages/ui-lib
├── app.html             # HTML template
└── app.d.ts             # SvelteKit type declarations (includes reference paths)
```

## Key features wired in layout

- **Navbar**: Brand "Mhaol" + `items` array with modal buttons
- **ModalOutlet**: Maps modal IDs to their content components from ui-lib
- **IdentitySidebar**: Always-visible identity sidebar
- **Services initialized**: `playerService`, `identityService`, `peerLibraryService`

## Dependencies

- `ui-lib` (workspace) — all shared frontend code (components, services, types, adapters, utils)
- `addons` (workspace) — TMDB metadata types and transforms (use `addons/tmdb/...` paths)

## Import pattern

All imports use `ui-lib/...` paths:

```typescript
import Navbar from "ui-lib/components/core/Navbar.svelte";
import { playerService } from "ui-lib/services/player.service";
import type { DisplayTMDBMovieDetails } from "addons/tmdb/types";
```

## Backend

The Rust backend is integrated directly at `backend/` as the `mhaol-server` crate. See `backend/CLAUDE.md` for backend-specific conventions.

```bash
# From repo root
cargo check -p mhaol-server       # Type check
cargo test -p mhaol-server        # Run tests
cargo build -p mhaol-server       # Debug build
cargo build --release --bin mhaol-server  # Release build
```

## Adding features

To add UI features to this app, add the component and service/type to `packages/ui-lib`, then import and wire it in this app's route files.

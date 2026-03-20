# App: flix

**Location:** `apps/flix/`
**Type:** Thin SvelteKit 2 wrapper — movies-only media app
**Adapter:** `@sveltejs/adapter-static` (fallback to `index.html`)
**Dev port:** 1531

## Architecture

This app is an **assembly-only wrapper** around `packages/frontend`. It contains no components, services, adapters, types, or utils of its own. All shared code is imported from `packages/frontend` via the `frontend` workspace dependency and path aliases.

## What lives here

```
src/
├── routes/              # SvelteKit pages
│   ├── +layout.svelte   # App shell: Navbar (brand + items) + IdentitySidebar + ModalOutlet
│   ├── +layout.ts       # SSR disabled
│   ├── +page.svelte     # Movies library grid with sub-tabs (Library, Search, Popular, Discover, Recommendations)
│   ├── +page.ts         # Page load (fetches /api/media)
│   └── database/        # Database browser utility page
├── css/app.css          # Tailwind entry + @source for packages/frontend
├── app.html             # HTML template
└── app.d.ts             # SvelteKit type declarations (includes reference paths)
```

## Key features wired in layout

- **Navbar**: Brand "Mhaol" + `items` array with 11 modal buttons (Torrent, Jackett, Downloads, Signaling, Peers, Identity, Plugins, Addons, Libraries, LLM, Settings)
- **ModalOutlet**: Maps all 11 modal IDs to their content components from packages/frontend
- **IdentitySidebar**: Always-visible identity sidebar
- **Services initialized**: `playerService`, `identityService`, `peerLibraryService`

## Dependencies

- `frontend` (workspace) — all shared UI code
- `addons` (workspace) — TMDB metadata types and transforms (use `addons/tmdb/...` paths)
- `fflate`, `html5-qrcode`, `qrcode`, `viem` — compression, QR codes, blockchain

## Import pattern

Component imports use `ui-lib/...` paths, services/types use `frontend/...`:

```typescript
import Navbar from "ui-lib/components/core/Navbar.svelte";
import { playerService } from "frontend/services/player.service";
import type { DisplayTMDBMovieDetails } from "addons/tmdb/types";
```

## Adding features

To add UI features to this app, add the component to `packages/ui-lib` and the service/type to `packages/frontend`, then import and wire it in this app's route files.

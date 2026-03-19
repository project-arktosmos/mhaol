# App: tube

**Location:** `apps/tube/`
**Type:** Thin SvelteKit 2 wrapper — YouTube-focused app
**Adapter:** `@sveltejs/adapter-static` (fallback to `index.html`)

## Architecture

This app is an **assembly-only wrapper** around `packages/frontend`. It contains no components, services, adapters, types, or utils of its own. All shared code is imported from `packages/frontend` via the `frontend` workspace dependency and path aliases.

## What lives here

```
src/
├── routes/              # SvelteKit pages
│   ├── +layout.svelte   # App shell: Navbar (children snippet) + ModalOutlet + RightPanel
│   ├── +layout.ts       # SSR disabled
│   ├── +page.svelte     # YouTube home/search page
│   └── +page.ts         # Page load
├── css/app.css          # Tailwind entry + @source for packages/frontend + themes
├── app.html             # HTML template
└── app.d.ts             # SvelteKit type declarations
```

## Key features wired in layout

- **Navbar**: Uses `children` snippet for custom icon buttons (settings, disk usage, download queue with active count badge)
- **ModalOutlet**: Wires `settings` → TubeSettingsContent, `disk` → DiskContent, `download-queue` → YouTubeDownloadQueue
- **RightPanel**: Video detail side panel from `frontend/components/youtube/RightPanel.svelte`
- **Services initialized**: `youtubeService`, `youtubeLibraryService`
- **Brand**: `{ label: 'Mhaol', highlight: 'Tube' }`

## Import pattern

Component imports use `ui-lib/...` paths, services/types use `frontend/...`:

```typescript
import Navbar from 'ui-lib/components/core/Navbar.svelte';
import { youtubeService } from 'frontend/services/youtube.service';
import type { YouTubeContent } from 'frontend/types/youtube.type';
```

## Desktop build (Tauri)

This app has `src-tauri/` for Tauri desktop builds. The Rust crate (`mhaol-tube-desktop`) depends on `mhaol-tauri-core` which embeds the backend server. Run with `pnpm tauri:dev:tube` from the repo root.

Shared assets (icons, capabilities, loading screen) are copied from `packages/tauri/assets/` via `sync-assets.sh` and are gitignored in `src-tauri/`.

## Adding features

To add UI features to this app, add the component to `packages/ui-lib` and the service/type to `packages/frontend`, then import and wire it in this app's route files.

# Player

**Location:** `apps/player/`
**pnpm package:** `player`
**Dev port:** `9595` (strictPort)

A SvelteKit static SPA that imports everything from `packages/ui-lib` (see the root `CLAUDE.md` for the import-and-assemble pattern). Builds to `apps/player/dist-static/`.

## Right-side sidebar

The layout's right-side aside (after the player) hosts, in order: `DocumentFilesPanel` (file list for the active document), `PlayerVideo` (video/audio surface), and `SubsLyricsFinder` (search subs or lyrics for the currently playing item). The finder hits `/api/search/subs-lyrics` on the connected node — it pre-fills with the current file name and a guessed type (audio → track, video → movie); for video kinds you must add a TMDB id before searching.

## Tauri shell

The player ships a desktop Tauri shell at `apps/player/src-tauri/` (crate `mhaol-player-shell`, productName "Mhaol Player", identifier `com.arktosmos.mhaol.player`). The shell loads the **shared health UI** under `apps/tauri/web/` — it does **not** wrap the player SPA on desktop; the player stays browser-accessible at `http://localhost:9595`.

`apps/player/src-tauri/` also carries `tauri.android.conf.json` and `tauri.ios.conf.json` overrides that wrap the player SPA directly on mobile (`frontendDist: ../dist-static`, `devUrl: http://localhost:9595`). `pnpm tauri:android:dev` runs the mobile player shell.

See `apps/tauri/CLAUDE.md` for how the two desktop shells share the health UI Vite server.

On non-Android targets, the shell also spawns an embedded yt-dlp HTTP server (`mhaol-yt-dlp`) on `127.0.0.1:9897`, exposing routes under `/api/ytdl/*`. The player Vite dev server proxies `/api/ytdl` → `9897`, so the `/youtube` page reaches yt-dlp through the player shell instead of the node. Override the port with `YTDL_PORT`.

## Routes

- `/` — landing
- `/clouds` — cloud connection setup
- `/documents` — document browser
- `/youtube` — Self-contained yt-dlp UI. Talks **directly** to the embedded yt-dlp server via plain `fetch('/api/ytdl/...')`; does **not** use `ui-lib`'s transport layer or `youtubeService`/`youtubeLibraryService` (those depend on node-only endpoints). Paste a URL → fetch info → queue audio/video/both. Live progress via SSE on `/api/ytdl/downloads/events`.

## Running

```bash
# Player + named "Mhaol Player" Tauri shell + shared health UI
pnpm dev:player

# Player Vite only (no Tauri shell)
pnpm app:player

# Player desktop release build
pnpm app:tauri:player:build

# Mobile (Android — wraps the player SPA directly)
pnpm tauri:android:dev
pnpm tauri:android:build
```

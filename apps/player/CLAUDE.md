# Player

**Location:** `apps/player/`
**pnpm package:** `player`
**Dev port:** `9595` (strictPort)

A SvelteKit static SPA that imports everything from `packages/ui-lib` (see the root `CLAUDE.md` for the import-and-assemble pattern). Builds to `apps/player/dist-static/`.

## Tauri shell

The player ships a desktop Tauri shell at `apps/player/src-tauri/` (crate `mhaol-player-shell`, productName "Mhaol Player", identifier `com.arktosmos.mhaol.player`). The shell loads the **shared health UI** under `apps/tauri/web/` — it does **not** wrap the player SPA on desktop; the player stays browser-accessible at `http://localhost:9595`.

`apps/player/src-tauri/` also carries `tauri.android.conf.json` and `tauri.ios.conf.json` overrides that wrap the player SPA directly on mobile (`frontendDist: ../dist-static`, `devUrl: http://localhost:9595`). `pnpm tauri:android:dev` runs the mobile player shell.

See `apps/tauri/CLAUDE.md` for how the two desktop shells share the health UI Vite server.

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

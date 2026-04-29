# Shared Tauri health UI

**Location:** `apps/tauri/web/`
**pnpm package:** `tauri-web`
**Dev port:** `1571` (strictPort)

This directory contains the **shared health UI** loaded by both per-app desktop Tauri shells (`apps/cloud/src-tauri/` and `apps/player/src-tauri/`). There is **no Tauri crate here anymore** — each app owns its own `src-tauri/` directory and crate so they have independent cargo targets and can run side by side.

## What it shows

A minimal SvelteKit static SPA that polls the local Mhaol apps and renders one health panel per app:

- **Cloud** — `GET http://localhost:9898/api/cloud/status` (parsed as JSON; uses `version`, `host`, `port`, `uptime_seconds`)
- **Player** — `GET http://localhost:9595/` with `mode: 'no-cors'` (success means the static SPA is being served)

Both probes refresh every 5 seconds. Each panel shows status, latency, app-specific metadata, and an "Open" button that opens the app in the system browser. `VITE_MHAOL_HEALTH_APPS` (`cloud`, `player`, `cloud,player`) selects which panels to render at Vite startup.

## How the per-app shells use it

Both desktop shells (`mhaol-cloud-shell`, `mhaol-player-shell`) point `frontendDist` at `../../tauri/web/dist-static` and `devUrl` at `http://localhost:1571`. Their `beforeDevCommand` is idempotent: `(lsof -i:1571 >/dev/null 2>&1) || pnpm --filter tauri-web dev` — so when both shells run together (`pnpm dev`) only one tauri-web Vite server is started; the second shell sees 1571 already up and skips. Vite's `strictPort: true` makes any actual collision fail loudly.

`pnpm dev` pre-starts tauri-web with `VITE_MHAOL_HEALTH_APPS=cloud,player`, then runs `dev:cloud` and `dev:player` concurrently — two named windows ("Mhaol Cloud", "Mhaol Player") share the one health UI process.

## Layout

```
apps/tauri/
├── CLAUDE.md
└── web/
    ├── package.json                # pnpm name: tauri-web
    ├── vite.config.ts              # port 1571, strictPort
    ├── svelte.config.js            # adapter-static, ui-lib aliases
    └── src/
        ├── routes/{+layout.svelte,+layout.ts,+page.svelte}
        ├── components/{HealthCard,AppHealthPanel}.svelte
        ├── lib/apps-health.service.ts
        ├── css/app.css
        └── app.html
```

The per-app Tauri crates are documented in `apps/cloud/CLAUDE.md` and `apps/player/CLAUDE.md` (and the root `CLAUDE.md`).

## Running

```bash
# Full desktop dev stack (cloud + player backends + both named Tauri windows)
pnpm dev

# Per-app shells (each starts tauri-web idempotently and opens its own named window)
pnpm dev:cloud
pnpm dev:player

# Backends only (no Tauri shell, browser workflow)
pnpm dev:apps

# Health UI Vite dev server only (no Tauri shell — quick UI tweaks in a browser)
pnpm app:tauri:web

# Per-app Tauri shells standalone (assumes the matching backend is already running)
pnpm app:tauri:cloud
pnpm app:tauri:player

# Per-app desktop release builds
pnpm app:tauri:cloud:build
pnpm app:tauri:player:build

# Mobile (Android — wraps the player SPA directly via apps/player/src-tauri mobile overrides)
pnpm tauri:android:dev
pnpm tauri:android:build
pnpm tauri:android:build:apk
```

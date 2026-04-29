# Tauri shell

**Location:** `apps/tauri/`
**Crate:** `mhaol-tauri`
**Binary:** `mhaol-tauri`

A Tauri 2 shell that bundles different frontends per platform.

## Desktop

Loads `apps/tauri/web/`, a minimal Svelte SPA (pnpm package `tauri-web`, dev port `1571`) that polls the local Mhaol apps and renders a health panel for each:

- **Cloud** — `GET http://localhost:9898/api/cloud/status` (parsed as JSON; uses `version`, `host`, `port`, `uptime_seconds`)
- **Player** — `GET http://localhost:9595/` with `mode: 'no-cors'` (success means the static SPA is being served)

Both probes refresh every 5 seconds. Each panel shows status, latency, app-specific metadata, and an "Open" button that opens the app in the system browser.

The desktop UI is the SPA at `apps/tauri/web/`; it builds to `apps/tauri/web/dist-static/`, which `tauri.conf.json` loads as `frontendDist`.

## Mobile (Android/iOS)

`tauri.android.conf.json` and `tauri.ios.conf.json` override the build block:

- `frontendDist`: `../../player/dist-static`
- `devUrl`: `http://localhost:9595`
- `beforeDevCommand`/`beforeBuildCommand`: build the player

So the mobile app is a wrapper around the existing **player** SPA (`apps/player/`). Run `pnpm tauri:android:dev` — it sets up `adb reverse tcp:9595 tcp:9595` so the player dev server on the host is reachable from the device, then runs `cargo tauri android dev`.

## Layout

```
apps/tauri/
├── src-tauri/
│   ├── Cargo.toml                  # mhaol-tauri crate
│   ├── build.rs
│   ├── tauri.conf.json             # base + desktop (loads ../web/dist-static)
│   ├── tauri.android.conf.json     # mobile override → player
│   ├── tauri.ios.conf.json         # mobile override → player
│   ├── capabilities/default.json
│   ├── icons/...                   # copied from apps/frontend/src-tauri/icons
│   └── src/{main.rs,lib.rs}        # standard Tauri 2 entry with mobile_entry_point
└── web/
    ├── package.json                # pnpm name: tauri-web
    ├── vite.config.ts              # port 1571
    ├── svelte.config.js            # adapter-static, ui-lib aliases
    └── src/
        ├── routes/{+layout.svelte,+layout.ts,+page.svelte}
        ├── components/{HealthCard,AppHealthPanel}.svelte
        ├── lib/apps-health.service.ts
        ├── css/app.css
        └── app.html
```

## Running

`pnpm dev` is the full dev stack: it boots cloud (Rust 9899 + Vite 9898) and player (Vite 9595) in the background, waits for both ports, then launches `pnpm app:tauri`. Tauri's `beforeDevCommand` brings up `tauri-web` (Vite 1571) and the native window with the health UI. Hot reload works for the cloud WebUI, the player, and the Tauri health UI; the cloud Rust binary still needs a manual restart.

```bash
# Full desktop dev stack (cloud + player + Tauri shell)
pnpm dev

# Cloud + player only (no Tauri shell — browser-based workflow)
pnpm dev:apps

# Tauri shell standalone (assumes cloud and player are already running)
pnpm app:tauri

# Health UI Vite dev server only (no Tauri shell — quick UI tweaks in a browser)
pnpm app:tauri:web

# Desktop release build
pnpm app:tauri:build

# Mobile (Android)
pnpm tauri:android:dev
pnpm tauri:android:build
pnpm tauri:android:build:apk
```

# Tauri shell

**Location:** `apps/tauri/`
**Crate:** `mhaol-tauri`
**Binary:** `mhaol-tauri`

A Tauri 2 shell that bundles different frontends per platform.

## Desktop

Loads `apps/tauri/web/`, a minimal Svelte SPA (pnpm package `tauri-web`, dev port `1571`) that polls the local Mhaol apps and renders a health panel for each:

- **Cloud** — `GET http://localhost:9898/api/cloud/status` (parsed as JSON; uses `version`, `host`, `port`, `uptime_seconds`)
- **Player** — `GET http://localhost:9595/` with `mode: 'no-cors'` (success means the static SPA is being served)

Both probes refresh every 5 seconds. Each panel shows status, latency, app-specific metadata, and an "Open" button that opens the app in the system browser. `VITE_MHAOL_HEALTH_APPS` (e.g. `cloud`, `player`, `cloud,player`) selects which panels to render.

The desktop UI is the SPA at `apps/tauri/web/`; it builds to `apps/tauri/web/dist-static/`, which `tauri.conf.json` loads as `frontendDist`. Launched via `pnpm app:tauri` and reused by both `pnpm dev:cloud` and `pnpm dev:player` — those scripts only differ in which services they boot alongside the shell. The Tauri webview always shows the health UI; the cloud and player apps themselves stay browser-accessible at `http://localhost:9898` and `http://localhost:9595`.

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

`pnpm dev:cloud` and `pnpm dev:player` both run the same `pnpm app:tauri` health shell. They only differ in which services they boot alongside it and which panels the health UI renders (`VITE_MHAOL_HEALTH_APPS`):

- `pnpm dev:cloud` boots the cloud Rust loopback server (9899), the cloud Vite WebUI (9898), then `pnpm app:tauri` (brings up `tauri-web` on 1571 as Tauri's `beforeDevCommand` and opens a native window with the health UI). Defaults `VITE_MHAOL_HEALTH_APPS=cloud`. Cloud stays reachable in the browser at `http://localhost:9898`.
- `pnpm dev:player` boots the player Vite server (9595), then `pnpm app:tauri`. Defaults `VITE_MHAOL_HEALTH_APPS=player`. Player stays reachable in the browser at `http://localhost:9595`. The Tauri webview shows the health UI, not the player itself.

`pnpm dev` runs both side-by-side: it spawns the player Vite server (`pnpm app:player`, no Tauri shell) in the background and runs `dev:cloud` in the foreground with `VITE_MHAOL_HEALTH_APPS=cloud,player` so the single Tauri health shell shows both panels. Running `dev:player` and `dev:cloud` as separate processes would launch two `cargo tauri dev` invocations competing on the same target dir, so the combined flow only spawns one Tauri shell. Hot reload works for the cloud WebUI, the player, and the Tauri health UI; the cloud Rust binary still needs a manual restart.

```bash
# Full desktop dev stack (cloud Rust + cloud Vite + player Vite + one Tauri health shell)
pnpm dev

# Cloud + player Vite servers only (no Tauri shell — browser-based workflow)
pnpm dev:apps

# Cloud + its Tauri health shell (Rust + Vite WebUI + Tauri shell)
pnpm dev:cloud

# Player + its Tauri health shell (Vite :9595 + Tauri shell)
pnpm dev:player

# Player Vite only (no Tauri shell)
pnpm app:player

# Tauri health shell standalone (assumes cloud and/or player are already running)
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

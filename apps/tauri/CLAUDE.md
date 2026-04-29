# Tauri shell

**Location:** `apps/tauri/`
**Crate:** `mhaol-tauri`
**Binary:** `mhaol-tauri`

A Tauri 2 shell that bundles different frontends per platform and per launch config.

## Desktop — cloud wrapper (default config)

Loads `apps/tauri/web/`, a minimal Svelte SPA (pnpm package `tauri-web`, dev port `1571`) that polls the local Mhaol apps and renders a health panel for each:

- **Cloud** — `GET http://localhost:9898/api/cloud/status` (parsed as JSON; uses `version`, `host`, `port`, `uptime_seconds`)
- **Player** — `GET http://localhost:9595/` with `mode: 'no-cors'` (success means the static SPA is being served)

Both probes refresh every 5 seconds. Each panel shows status, latency, app-specific metadata, and an "Open" button that opens the app in the system browser.

The desktop UI is the SPA at `apps/tauri/web/`; it builds to `apps/tauri/web/dist-static/`, which `tauri.conf.json` loads as `frontendDist`. Launched via `pnpm app:tauri` (used by `pnpm dev:cloud`).

## Desktop — player wrapper (`tauri.player.conf.json`)

`tauri.player.conf.json` overrides the desktop build block to wrap the player SPA directly:

- `productName`: `Mhaol Player`
- `identifier`: `com.arktosmos.mhaol.player` (distinct from the cloud wrapper so both can coexist)
- `frontendDist`: `../../player/dist-static`
- `devUrl`: `http://localhost:9595`
- `beforeDevCommand`/`beforeBuildCommand`: build the player

Launched via `pnpm app:tauri:player` (`cargo tauri dev --config tauri.player.conf.json`). Used by `pnpm dev:player` to give the player its own Tauri shell on desktop, mirroring how `dev:cloud` gives the cloud its Tauri shell.

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
│   ├── tauri.conf.json             # base + desktop (loads ../web/dist-static — cloud wrapper)
│   ├── tauri.player.conf.json      # desktop override → wraps player SPA at :9595
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

The cloud and player each get their own Tauri wrapper on desktop:

- `pnpm dev:cloud` boots the cloud Rust loopback server (9899), the cloud Vite WebUI (9898), then runs `pnpm app:tauri` (brings up `tauri-web` on 1571 as Tauri's `beforeDevCommand` and opens a native window with the health UI). Cloud stays reachable in the browser at `http://localhost:9898`.
- `pnpm dev:player` runs `pnpm app:tauri:player` (`cargo tauri dev --config tauri.player.conf.json`). The override config's `beforeDevCommand` brings up the player Vite server on 9595, and Tauri opens a native window pointing at it. Player stays reachable in the browser at `http://localhost:9595`.

`pnpm dev` runs both side-by-side: it spawns the player Vite server (`pnpm app:player`, no Tauri shell) in the background and runs `dev:cloud` in the foreground. Only one Tauri shell (cloud) launches in this combined flow — running both `dev:player` and `dev:cloud` simultaneously would have two `cargo tauri dev` processes competing on the same target dir. Hot reload works for the cloud WebUI, the player, and the Tauri health UI; the cloud Rust binary still needs a manual restart.

```bash
# Full desktop dev stack (cloud Tauri wrapper + player Vite in browser)
pnpm dev

# Cloud + player Vite servers only (no Tauri shell — browser-based workflow)
pnpm dev:apps

# Cloud + its Tauri wrapper (Rust + Vite WebUI + Tauri shell with health UI)
pnpm dev:cloud

# Player + its Tauri wrapper (Vite :9595 + native window wrapping the player)
pnpm dev:player

# Player Vite only (no Tauri shell)
pnpm app:player

# Cloud Tauri shell standalone (assumes cloud Vite and player are already running)
pnpm app:tauri

# Player Tauri shell standalone (its beforeDevCommand starts the player Vite server)
pnpm app:tauri:player

# Health UI Vite dev server only (no Tauri shell — quick UI tweaks in a browser)
pnpm app:tauri:web

# Desktop release builds
pnpm app:tauri:build         # cloud wrapper
pnpm app:tauri:player:build  # player wrapper

# Mobile (Android)
pnpm tauri:android:dev
pnpm tauri:android:build
pnpm tauri:android:build:apk
```

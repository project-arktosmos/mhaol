# Cloud (Tauri shell)

**Location:** `apps/cloud/`
**Crate:** `mhaol-cloud-shell` (binary `mhaol-cloud-shell`)
**Product:** "Mhaol Cloud" (`com.arktosmos.mhaol.cloud`)

This is the desktop wrapper that **presents** the system to users. It depends on `mhaol-backend` directly and **embeds the backend** — `mhaol_backend::run()` is spawned inside the Tauri `setup` hook so the release `.app` / `.msi` / `.deb` is fully self-contained:

- `packages/backend/` — Rust Axum server crate (`mhaol-backend` lib + `mhaol-cloud` bin). Runs the API, hosts the embedded frontend, owns SurrealDB / IPFS / torrent / yt-dlp / ipfs-stream subsystems.
- `packages/frontend/` — Svelte SPA (pnpm package `frontend`). Builds to `packages/frontend/dist-static/` and is embedded into the backend bin at compile time via `rust-embed`.

The shell is **tray-only** — `app.windows: []`, no window is ever created. macOS sets `ActivationPolicy::Accessory` (no dock icon), `RunEvent::ExitRequested` calls `prevent_exit()` so the process stays alive without windows. The system tray icon (id `mhaol-cloud-tray`, tooltip "Mhaol Cloud") has two items: **Open** opens `http://localhost:9898` in the system default browser via `tauri-plugin-opener`, **Quit** calls `app.exit(0)`. The frontend stays browser-accessible at `http://localhost:9898` because the spawned `mhaol_backend::run()` binds it (`0.0.0.0:9898` by default, or `PORT` if set).

## Layout

```
apps/cloud/
├── Cargo.toml              # mhaol-cloud-shell crate manifest
├── tauri.conf.json         # frontendDist → ../../packages/frontend/dist-static; devUrl http://localhost:9898; windows []
├── build.rs                # tauri_build::build()
├── capabilities/default.json
├── icons/
└── src/
    ├── lib.rs              # tauri::Builder, tray menu, RunEvent handling — setup hook spawns mhaol_backend::run()
    ├── main.rs             # mhaol_cloud_shell::run()
    └── image_cache.rs      # `image_cache_resolve` Tauri command — disk-cached fetches under <documents>/mhaol-cloud/image-cache
```

## Running

```bash
# Dev — full desktop stack (backend bin + Vite frontend dev + Tauri shell)
pnpm dev

# Dev — Tauri shell only (assumes the backend is already running on 9899 and the frontend on 9898)
pnpm app:tauri:cloud

# Release bundle
pnpm app:tauri:cloud:build
```

`pnpm dev` builds the `mhaol-cloud` debug bin first (so the tray's Open URL is reachable on launch), then runs three concurrent processes via `concurrently`: the backend bin on `127.0.0.1:9899`, the Vite dev server on `0.0.0.0:9898`, and the Tauri shell. The shell's own embedded `mhaol_backend::run()` is **not** spawned in dev — when `PORT=9899` (set by `pnpm dev`), the shell's spawned backend would collide with the standalone bin; in practice the dev script keeps the env unset for the shell so the embedded copy binds 9898, but the recommended dev path keeps the standalone bin authoritative on 9899 and Vite on 9898 for HMR.

In a **release** bundle (`.app` / `.msi` / `.deb` / `.AppImage`) there is no Vite and no separate `mhaol-cloud` bin — the shell's setup hook spawns `mhaol_backend::run()` inside the Tauri tokio runtime, which binds `0.0.0.0:9898` and serves the SPA from the backend's compile-time-embedded `dist-static` (via `rust-embed`). The tray's **Open** menu opens `http://localhost:9898` in the user's default browser.

## tauri.conf.json paths

- `frontendDist`: `../../packages/frontend/dist-static` — relative to `apps/cloud/`. Tauri's bundler reads this at release-build time; runtime never touches it (no windows render).
- `devUrl`: `http://localhost:9898` — points at the Vite dev server (which proxies `/api` → `127.0.0.1:9899`).
- `beforeBuildCommand`: `pnpm --filter frontend build` — runs the SPA build before `cargo tauri build` packages the bundle.

## image_cache Tauri command

`apps/cloud/src/image_cache.rs` registers `image_cache_resolve(url)` as a Tauri IPC command. The frontend's `<TauriImage>` component (when running inside the Tauri shell — though currently the shell never opens a window so this code path is dormant) can call it to fetch + cache remote images on disk under `<documents>/mhaol-cloud/image-cache/`. SHA3-256 of the URL becomes the on-disk filename, with the URL's path extension preserved when reasonable. Cache hits return the bytes directly; misses fetch via `reqwest` and write to disk before returning.

# Android Mobile (Tauri shell)

**Location:** `apps/android-mobile/`
**Crate:** `mhaol-android-mobile` (binary `mhaol-android-mobile`)
**Product:** "Mhaol Mobile" (`com.arktosmos.mhaol.androidmobile`)

This is the Android phone / tablet variant of the Mhaol shell. Like [`apps/android-tv/`](../android-tv/CLAUDE.md) it renders the SPA in its own Tauri viewport, but **unlike** the TV variant it **embeds the backend** — `mhaol_backend::run()` is spawned inside the Tauri `setup` hook so the app is fully self-contained and works without a network connection to any other Mhaol host.

## Layout

```
apps/android-mobile/
├── Cargo.toml              # depends on mhaol-backend
├── tauri.conf.json         # frontendDist → ../../packages/frontend/dist-static; one main window
├── build.rs                # tauri_build::build()
├── capabilities/default.json
├── icons/                  # copied from apps/cloud/icons
└── src/
    ├── lib.rs              # tauri::Builder — setup spawns mhaol_backend::run()
    └── main.rs             # mhaol_android_mobile::run()
```

## Embedded backend

`src/lib.rs` schedules the backend on Tauri's tokio runtime:

```rust
.setup(|_app| {
    tauri::async_runtime::spawn(async move {
        mhaol_backend::run().await;
    });
    Ok(())
})
```

`mhaol_backend::run()` binds `0.0.0.0:9898` by default; the SPA's default of `http://127.0.0.1:9898` reaches it via the device's loopback. All env vars from [packages/backend/CLAUDE.md](../../packages/backend/CLAUDE.md) apply unchanged (`PORT`, `DATA_DIR`, etc.) — for a packaged Android build, the backend's `<data_root>` will land under the app's sandboxed documents dir.

The `cfg(not(target_os = "android"))` guards in `packages/backend/Cargo.toml` keep the heavy desktop subsystems (`mhaol-yt-dlp`, `mhaol-torrent`, `mhaol-ipfs-core`, `mhaol-ipfs-stream`) out of the Android build — only the API surface, SurrealDB store, identity manager, and frontend embed compile on Android.

## Backend URL

The SPA's `src/lib/api-base.ts` defaults to `http://127.0.0.1:9898` for any Tauri shell, which exactly matches the embedded backend's bind. The user can override via the in-app **Settings** page (`mhaol-api-base` in localStorage) to point at a different cloud, or reset to default.

## Running

```bash
# Dev — boots the Medium_Phone_API_36.1 emulator, runs Vite (for the dev
# WebView refresh) and the mobile shell. The shell's embedded backend runs
# inside the Tauri process — no separate cloud bin in this strand.
pnpm dev:android:mobile

# Release bundle (APK / AAB)
pnpm app:tauri:android:mobile:build
```

`pnpm dev:android:mobile` runs `cargo tauri android init` once into `apps/android-mobile/gen/android/` (gitignored).

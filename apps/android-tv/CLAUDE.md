# Android TV (Tauri shell)

**Location:** `apps/android-tv/`
**Crate:** `mhaol-android-tv` (binary `mhaol-android-tv`)
**Product:** "Mhaol Android TV" (`com.arktosmos.mhaol.androidtv`)

This is the Android TV variant of the Mhaol shell. Unlike [`apps/cloud/`](../cloud/CLAUDE.md), it **renders the SPA in its own Tauri viewport** (full-screen WebView) rather than relying on a system browser. Unlike [`apps/android-mobile/`](../android-mobile/CLAUDE.md), it **does not embed a backend** — it is a pure viewer that talks to a remote `mhaol-cloud` / `mhaol-headless` instance.

## Layout

```
apps/android-tv/
├── Cargo.toml              # mhaol-android-tv crate manifest
├── tauri.conf.json         # frontendDist → ../../packages/frontend/dist-static; one full-screen window
├── build.rs                # tauri_build::build()
├── capabilities/default.json
├── icons/                  # copied from apps/cloud/icons
└── src/
    ├── lib.rs              # tauri::Builder — minimal setup, no backend
    └── main.rs             # mhaol_android_tv::run()
```

## Backend URL

The SPA's `src/lib/api-base.ts` defaults the backend URL to `http://127.0.0.1:9898` for any Tauri shell. On the Android TV emulator that resolves to the device's loopback (where nothing listens), so the user **must** override the URL in the in-app **Settings** page. Typical targets are the LAN IP of a `mhaol-cloud` desktop install, a `mhaol-headless` server, or `http://10.0.2.2:9898` when running against a host bound to localhost in the emulator.

The override persists in the WebView's localStorage under `mhaol-api-base`.

## Running

```bash
# Dev — boots the Google_TV_1080p_API_36 emulator, runs cloud bin + Vite
# (so the dev WebView has a live backend at 10.0.2.2:9898) and the TV shell.
pnpm dev:android:tv

# Release bundle (APK / AAB)
pnpm app:tauri:android:tv:build
```

`pnpm dev:android:tv` runs `cargo tauri android init` once into `apps/android-tv/gen/android/` (the Android Studio Gradle project — gitignored).

# Headless

**Location:** `apps/headless/`
**Crate:** `mhaol-headless` (binary `mhaol-headless`)

This is the terminal-only counterpart to [apps/cloud/](../cloud/CLAUDE.md). Same backend, same embedded SPA, no Tauri shell, no system tray, no desktop assumptions — designed for servers, headless boxes, and CI/automation hosts where opening a window is impossible or unwanted.

It is a thin Rust crate that does nothing but boot the backend:

```rust
// src/main.rs
#[tokio::main]
async fn main() {
    mhaol_backend::run().await;
}
```

The actual server (Axum router, SurrealDB, IPFS node, torrent / yt-dlp / ipfs-stream managers, embedded SPA) all live in `packages/backend/` — see [packages/backend/CLAUDE.md](../../packages/backend/CLAUDE.md). Releases of `mhaol-headless` embed `packages/frontend/dist-static/` exactly the same way `mhaol-cloud` does (via `rust-embed`), so the SPA is reachable in a browser at `http://<host>:<port>` once the bin is running.

## Layout

```
apps/headless/
├── Cargo.toml      # mhaol-headless crate manifest (depends on mhaol-backend)
└── src/main.rs     # #[tokio::main] async fn main() { mhaol_backend::run().await }
```

## Running

```bash
# Dev — backend bin on 127.0.0.1:9899 + Vite frontend on 0.0.0.0:9898 (proxies /api → 9899). No Tauri.
pnpm dev:headless

# Dev — bin only, embedded SPA served on its own port (no Vite, no hot reload)
pnpm app:headless

# Production build — embeds the SPA into the release bin
pnpm build:headless
./target/release/mhaol-headless     # binds 0.0.0.0:9898 by default

# Cross-compile a Linux x86_64 release from any host (handy when the target server
# is too small to run `cargo build`). Uses `cross` (https://github.com/cross-rs/cross)
# which runs the build inside a Docker image — see ../../Cross.toml for the pre-build
# hook that installs GStreamer + libssl headers needed by mhaol-ipfs-stream / reqwest.
#
# Prereqs (build host):
#   - Docker running
#   - `cargo install cross --git https://github.com/cross-rs/cross`
pnpm build:headless:linux
# Output: ./target/x86_64-unknown-linux-gnu/release/mhaol-headless
# Ship it: scp ./target/x86_64-unknown-linux-gnu/release/mhaol-headless server:/usr/local/bin/
```

## Environment

All env vars from [packages/backend/CLAUDE.md](../../packages/backend/CLAUDE.md) apply unchanged: `PORT`, `HOST`, `DATA_DIR`, `DB_PATH`, `IPFS_SWARM_KEY_FILE`, `MHAOL_IPFS_TCP_PORT`, `MHAOL_IPFS_WS_PORT`, `YTDL_OUTPUT_DIR`.

## Logs

`pnpm dev:headless` tees the bin's stdout+stderr into `logs/headless.log` and the Vite dev server into `logs/web.log`. Each file is overwritten on the next run.

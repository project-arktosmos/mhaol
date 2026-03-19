# Package: backend

**Location:** `packages/backend/`
**Framework:** Rust — Axum 0.8 + Tokio + rusqlite (SQLite)
**Binary:** `mhaol-server` (default port 1530)

## Source Structure

```
src/
├── server.rs             # Binary entry point — starts Axum server
├── lib.rs                # AppState definition, module declarations
├── api/                  # Route handlers (one module per feature)
│   ├── mod.rs            # build_router() — registers all routes
│   ├── addons.rs
│   ├── database.rs
│   ├── downloads.rs
│   ├── identities.rs
│   ├── images.rs
│   ├── libraries.rs
│   ├── lyrics.rs
│   ├── media.rs
│   ├── musicbrainz.rs
│   ├── p2p_stream.rs
│   ├── player.rs
│   ├── plugins.rs
│   ├── signaling.rs
│   ├── tmdb.rs
│   ├── torrent.rs        # cfg(not(target_os = "android"))
│   ├── youtube.rs
│   └── ytdl.rs           # cfg(not(target_os = "android"))
├── db/                   # Database layer (rusqlite repos)
├── identity/             # Identity/wallet management
├── modules/              # Plugin modules (image tagger, etc.)
├── signaling_dev.rs      # Local signaling dev server
└── worker_bridge.rs      # Background worker bridge
```

## AppState

All API handlers receive `AppState` which contains:
- Database repositories (settings, metadata, libraries, library_items, etc.)
- `IdentityManager` for identity/wallet operations
- `ModuleRegistry` for plugin modules
- `DownloadManager` (yt-dlp, desktop only)
- `TorrentManager` (desktop only)
- `ImageTaggerManager` (ONNX/ML, desktop only)
- `SignalingDevServer` and `WorkerBridge` (auto-started on server boot)

## Adding a New API Module

1. Create `src/api/{feature}.rs` with a `pub fn router() -> Router<AppState>` function
2. Add `pub mod {feature};` to `src/api/mod.rs`
3. Register in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
4. If new database access is needed, add a repo to `AppState`

## Sub-crate Dependencies (desktop only)

These are conditionally compiled with `#[cfg(not(target_os = "android"))]`:
- `mhaol-p2p-stream` — P2P streaming (`packages/p2p-stream/`)
- `mhaol-yt-dlp` — YouTube downloading (`packages/yt-dlp/`)
- `mhaol-torrent` — Torrent management (`packages/torrent/`)
- `ort` + `tokenizers` + `image` — ML image tagging (ONNX runtime)

## Running

```bash
# From repo root
pnpm dev:backend          # PORT=1530 cargo run -p mhaol-backend --bin mhaol-server
pnpm build:backend        # cargo build -p mhaol-backend --release

# Tests
cargo test -p mhaol-backend
cargo check -p mhaol-backend
```

## Environment Variables

- `PORT` — Server port (default: 1530)
- `HOST` — Bind address (default: 0.0.0.0)
- `DB_PATH` — SQLite database path (optional, in-memory if unset)

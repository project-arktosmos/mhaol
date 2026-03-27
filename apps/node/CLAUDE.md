# Node

**Location:** `apps/node/`
**Framework:** Rust — Axum 0.8 + Tokio + rusqlite (SQLite)
**Crate:** `mhaol-node`
**Binary:** `mhaol-node` (default port 1530)

## Source Structure

```
src/
├── server.rs             # Binary entry point — starts Axum server
├── lib.rs                # AppState definition, module declarations
├── api/                  # Route handlers (one module per feature)
│   ├── mod.rs            # build_router() — registers all routes
│   ├── addons.rs
│   ├── catalog.rs          # Unified media catalog (items CRUD, fetch cache, search)
│   ├── database.rs
│   ├── downloads.rs
│   ├── health.rs           # GET /api/health — simple status check
│   ├── hub.rs              # App management (list, health check, start/stop headless)
│   ├── identities.rs
│   ├── images.rs
│   ├── libraries.rs
│   ├── lyrics.rs
│   ├── media.rs
│   ├── musicbrainz.rs
│   ├── p2p_stream.rs
│   ├── player.rs
│   ├── plugins.rs
│   ├── retroachievements.rs  # RetroAchievements game metadata proxy
│   ├── queue.rs           # Queue task management (CRUD + SSE subscribe)
│   ├── game_recommendations.rs # Game recommendations via metadata matching (bulk enqueue, query, labels)
│   ├── music_recommendations.rs # Music recommendations via ListenBrainz (bulk enqueue, query, labels)
│   ├── recommendation_labels.rs # Per-user recommendation labels (CRUD, wallet-scoped)
│   ├── recommendations.rs # TMDB recommendations (bulk enqueue + query)
│   ├── roster.rs           # Roster contacts CRUD (GET/POST/DELETE /api/roster)
│   ├── signaling.rs
│   ├── smart_pair.rs      # Smart pairing: TMDB matching + pinned items
│   ├── tmdb.rs
│   ├── torrent.rs        # cfg(not(target_os = "android"))
│   ├── youtube.rs
│   └── ytdl.rs           # cfg(not(target_os = "android"))
├── db/                   # Database layer (rusqlite repos)
├── llm_worker.rs         # Background LLM queue worker (processes llm:* tasks)
├── game_recommendations_worker.rs  # Background game recommendations worker (metadata-based matching)
├── music_recommendations_worker.rs  # Background music recommendations worker (ListenBrainz similar artists)
├── recommendations_worker.rs  # Background recommendations queue worker (processes recommendations:* tasks)
├── modules/              # Plugin modules (image tagger, etc.)
├── signaling_rooms.rs    # WebSocket signaling room management
└── worker_bridge.rs      # Background worker bridge
```

## AppState

All API handlers receive `AppState` which contains:

- Database repositories (settings, metadata, libraries, library_items, etc.)
- `IdentityManager` for identity/wallet operations (from `mhaol-identity` crate)
- `ModuleRegistry` for plugin modules
- `DownloadManager` (yt-dlp, desktop only)
- `TorrentManager` (desktop only)
- `ImageTaggerManager` (ONNX/ML, desktop only)
- `HubManager` for app process management (start/stop headless apps)
- `QueueManager` for task queue management (from `mhaol-queue` crate)
- `RecommendationsRepo` for TMDB recommendation storage (from `mhaol-recommendations` crate)
- `MusicRecommendationsRepo` for music recommendation storage (from `mhaol-recommendations::music`)
- `GameRecommendationsRepo` for game recommendation storage (from `mhaol-recommendations::game`)
- `RecommendationLabelRepo` for per-user recommendation labels (wallet-scoped, in `db/repo/`)
- `SignalingRoomManager` and `WorkerBridge` (auto-started on server boot)

## Adding a New API Module

1. Create `src/api/{feature}.rs` with a `pub fn router() -> Router<AppState>` function
2. Add `pub mod {feature};` to `src/api/mod.rs`
3. Register in `build_router()`: `.nest("/api/{feature}", {feature}::router())`
4. If new database access is needed, add a repo to `AppState`

## Sub-crate Dependencies

Always included:

- `mhaol-identity` — Ethereum identity/wallet management (`packages/identity/`)
- `mhaol-queue` — Task queue management with SQLite + broadcast (`packages/queue/`)
- `mhaol-recommendations` — TMDB recommendations storage + queue types (`packages/recommendations/`)

Conditionally compiled with `#[cfg(not(target_os = "android"))]`:

- `mhaol-p2p-stream` — P2P streaming (`packages/p2p-stream/`)
- `mhaol-yt-dlp` — YouTube downloading (`packages/yt-dlp/`)
- `mhaol-torrent` — Torrent management (`packages/torrent/`)
- `ort` + `tokenizers` + `image` — ML image tagging (ONNX runtime)

## Running

```bash
# From repo root
pnpm dev:node             # PORT=1530 cargo run -p mhaol-node --bin mhaol-node
pnpm build:node           # cargo build --release --bin mhaol-node

# Tests
cargo test -p mhaol-node
cargo check -p mhaol-node
```

## Environment Variables

- `PORT` — Server port (default: 1530)
- `HOST` — Bind address (default: 0.0.0.0)
- `DB_PATH` — SQLite database path (optional, in-memory if unset)
- `RA_API_USER` — RetroAchievements API username
- `RA_API_KEY` — RetroAchievements API key

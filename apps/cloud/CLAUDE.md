# Cloud

**Location:** `apps/cloud/`
**Framework:** Rust — Axum 0.8 + Tokio (depends on `mhaol-node` as a library)
**Crate:** `mhaol-cloud`
**Binary:** `mhaol-cloud` (default port 1540)

The cloud server starts the **same** services as `mhaol-node` (database, identity manager, queue, recommendations workers, peer service) and additionally hosts an embedded Svelte WebUI that displays node health.

## Source Structure

```
src/
├── server.rs            # Binary entry point — bootstraps AppState and spawns all node workers
├── cloud_status.rs      # Public /api/cloud/status route used by the WebUI
└── frontend.rs          # rust-embed wrapper that serves apps/cloud-web/dist-static/
```

## How it reuses node code

`apps/cloud/Cargo.toml` lists `mhaol-node = { path = "../node" }` as a dependency. The binary calls `mhaol_node::AppState::new`, `state.seed_default_libraries`, `state.initialize_modules`, and spawns the four recommendations workers + peer service the same way `mhaol-node` does. The HTTP router is `mhaol_node::api::build_router(state)`, with `cloud_status::router()` merged in and `frontend::serve_frontend` set as the fallback.

## WebUI

The Svelte app lives at `apps/cloud-web/` and builds to `apps/cloud-web/dist-static/`. The cloud crate embeds that directory via `rust-embed` and serves it as a fallback for any non-API path. Build it with `pnpm --filter cloud-web build` (or `pnpm build:cloud` to build the WebUI and the release binary together).

## Running

```bash
# Dev — Rust server on 1540
pnpm dev:cloud

# Dev — WebUI hot-reload on 9596 (proxies /api to :1540)
pnpm dev:cloud-web

# Production build (embeds the WebUI)
pnpm build:cloud
```

## Environment Variables

- `PORT` — Server port (default: 1540)
- `HOST` — Bind address (default: 0.0.0.0)
- `DB_PATH` — SQLite database path (default: `apps/cloud/mhaol.db`)
- All the env vars `mhaol-node` honors (RA_API_USER, RA_API_KEY, SIGNALING_URL, DATA_DIR, …) apply unchanged.

## Public WebUI endpoint

- `GET /api/cloud/status` — JSON with status, version, uptime, host/port, local IP, signaling/client wallet addresses, library count, queue depth. No auth required (used by the embedded WebUI).

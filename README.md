# Mhaol

A family of self-hosted media apps built on shared core components. Each app ships as a standalone desktop client, headless server, or browser extension.

**Apps in this repo:**

- **Frontend** — SvelteKit SPA for browsing and managing your media library (movies, TV, music, games, books, YouTube)
- **Node** — Rust Axum headless server powering the backend (API, torrents, P2P streaming, identity)
- **Shepperd** — Browser extension that detects media as you browse and imports it into Mhaol
- **Signaling** — Self-hosted WebSocket signaling server for WebRTC peer connections

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | >= 18 | [nodejs.org](https://nodejs.org) or `nvm install 18` |
| pnpm | >= 9 | `corepack enable && corepack prepare pnpm@latest --activate` |
| Rust | stable | [rustup.rs](https://rustup.rs) |

### System dependencies (Linux only)

The Rust node requires GStreamer and native build tools on Linux:

```bash
pnpm install:deps
```

This runs:

```bash
sudo apt-get install -y \
  build-essential pkg-config libssl-dev libonig-dev \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly gstreamer1.0-libav
```

On **macOS**, the required native libraries (GStreamer, Metal for LLM) are handled automatically by Cargo.

---

## Initial Setup

```bash
# Clone the repo
git clone <repo-url> mhaol
cd mhaol

# Install JavaScript dependencies
pnpm install
```

---

## Environment Variables

Create a `.env` file in the repo root. The node server sources this automatically on startup.

```bash
# Required — TMDB movie/TV metadata
# Get a free key at: https://www.themoviedb.org/settings/api
TMDB_API_KEY=
TMDB_READ_ACCESS_TOKEN=

# Optional — RetroAchievements game metadata
# Get your key at: https://retroachievements.org/settings -> Keys
RA_API_USER=
RA_API_KEY=

# Optional — Identity wallet (auto-generated if not set)
# 32-byte hex private key for the signaling identity
SIGNALING_WALLET=

# Optional — TURN server for WebRTC (needed for P2P connections behind NAT)
METERED_DOMAIN=
METERED_SECRET_KEY=
```

Without any env vars, the node will still start — TMDB and RetroAchievements features just won't work, and a wallet will be auto-generated.

---

## Running the Apps

All commands run from the **repo root**. Never `cd` into a package directory.

### Full stack (frontend + node)

```bash
pnpm dev
```

This starts both in parallel:
- **Frontend** on [http://localhost:1570](http://localhost:1570)
- **Node** on [http://localhost:1530](http://localhost:1530)

The frontend dev server proxies all `/api/*` requests to the node automatically.

### Frontend only

```bash
pnpm dev:frontend
```

Starts the SvelteKit dev server on port **1570**. Requires the node to be running separately for API calls to work.

### Node only

```bash
pnpm dev:node
```

Starts the Rust Axum server on port **1530**. On first run, it:
1. Creates a SQLite database (`apps/node/mhaol.db`)
2. Seeds default libraries (Movies, TV, Music, Games, YouTube) under `~/Documents/mhaol/downloads/`
3. Initializes addon modules (TMDB, MusicBrainz, RetroAchievements, etc.)
4. Starts background workers (P2P streaming, LLM queue)
5. Connects to the signaling server for WebRTC peer discovery

**Node environment variables** (all optional, sensible defaults):

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `1530` | HTTP server port |
| `HOST` | `0.0.0.0` | Bind address |
| `DB_PATH` | `apps/node/mhaol.db` | SQLite database path |
| `DATA_DIR` | `~/Documents/mhaol` | Media storage directory |
| `SIGNALING_URL` | PartyKit hosted | Signaling server URL |

### Shepperd (browser extension)

```bash
# Development (watch mode)
pnpm app:shepperd

# Production build
pnpm app:shepperd:build
```

Load the built extension from `apps/shepperd/dist/` in your browser's extension manager (enable developer mode).

### Signaling (self-hosted)

For self-hosted signaling instead of the default PartyKit instance:

```bash
# Interactive setup wizard (creates config)
pnpm signaling:setup

# Start the signaling server
pnpm signaling:serve
```

For PartyKit cloud signaling (development):

```bash
pnpm signaling:dev      # Local dev
pnpm signaling:deploy   # Deploy to PartyKit
```

---

## Building for Production

### Frontend

```bash
pnpm build
```

Outputs a static site to `apps/frontend/dist-static/`.

### Node

```bash
pnpm build:node
```

Builds the release binary at `target/release/mhaol-node`.

To bundle the frontend into the node binary (single-binary deployment):

```bash
pnpm build
cargo build --release --bin mhaol-node --features embed-frontend
```

Then run with `CLIENT_STATIC_DIR` pointing to the frontend dist, or use the embedded frontend feature.

### Signaling

```bash
pnpm build:signaling
```

Builds to `target/release/mhaol-signaling`.

---

## Desktop & Mobile Builds (Tauri)

The frontend can be packaged as a native desktop or Android app via Tauri.

### Desktop

```bash
cd apps/frontend/src-tauri
cargo tauri dev       # Development
cargo tauri build     # Production
```

### Android

Requires Android SDK (min SDK 24) and the Tauri Android prerequisites.

```bash
pnpm android:dev          # Dev on connected device/emulator
pnpm android:build        # Release build (AAB)
pnpm android:build:apk    # Release build (APK)
```

The `android:dev` command automatically sets up `adb reverse` to forward port 1530 to the device.

---

## Quality Checks

Run all checks before committing:

```bash
pnpm lint       # Prettier + ESLint
pnpm check      # svelte-check + cargo check
pnpm build      # Verify frontend builds
pnpm test       # vitest + cargo test
```

Or all at once:

```bash
pnpm lint && pnpm check && pnpm build && pnpm test
```

### Formatting

```bash
pnpm format     # Auto-fix formatting (Prettier)
```

### Cleanup

```bash
pnpm clean      # Remove build artifacts + cargo clean
```

---

## Repo Structure

```
mhaol.git/
├── apps/
│   ├── frontend/          # SvelteKit SPA (port 1570)
│   ├── node/              # Rust Axum server (port 1530)
│   ├── shepperd/          # Browser extension (Manifest V3)
│   └── signaling/         # Rust signaling server
├── packages/
│   ├── ui-lib/            # Shared components, services, types, CSS
│   ├── addons/            # TMDB, MusicBrainz, RetroAchievements, YouTube, LRCLIB, OpenLibrary
│   ├── webrtc/            # WebRTC contact handshake layer
│   ├── signaling/         # PartyKit signaling (cloud)
│   ├── identity/          # Rust Ethereum identity (secp256k1)
│   ├── queue/             # Rust task queue (SQLite)
│   ├── torrent/           # Rust torrent client
│   ├── p2p-stream/        # Rust P2P streaming (GStreamer + WebRTC)
│   ├── yt-dlp/            # Rust yt-dlp wrapper
│   └── llm/               # Rust LLM (llama.cpp)
├── .env                   # API keys and secrets (not committed)
├── package.json           # Workspace scripts
├── Cargo.toml             # Rust workspace
└── pnpm-workspace.yaml
```

### Architecture

Apps under `apps/` are thin wrappers. All shared frontend code (components, services, types, adapters) lives in `packages/ui-lib`. Apps only contain route files and configuration — they import and assemble, never implement.

The frontend communicates with the node via a transport layer (`packages/ui-lib/src/transport/`) that abstracts over HTTP and WebRTC, making the same API calls work whether connecting locally or peer-to-peer.

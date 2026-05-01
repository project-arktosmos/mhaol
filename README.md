# Mhaol

A family of self-hosted media apps built on shared core components. Each app ships as a standalone desktop client, headless server, or browser extension.

**Apps in this repo:**

- **Cloud** — Rust Axum server (port 9898) with a nested Svelte WebUI; ships a tray-only desktop Tauri shell ("Mhaol Cloud")
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

The Rust cloud binary requires GStreamer and native build tools on Linux:

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

Create a `.env` file in the repo root. The cloud server sources this automatically on startup.

```bash
# Required — TMDB movie/TV metadata
# Get a free key at: https://www.themoviedb.org/settings/api
TMDB_API_KEY=
TMDB_READ_ACCESS_TOKEN=

# Optional — Identity wallet (auto-generated if not set)
# 32-byte hex private key for the signaling identity
SIGNALING_WALLET=

# Optional — TURN server for WebRTC (needed for P2P connections behind NAT)
METERED_DOMAIN=
METERED_SECRET_KEY=
```

Without any env vars, the cloud server will still start — TMDB features just won't work, and a wallet will be auto-generated.

---

## Running the Apps

All commands run from the **repo root**. Never `cd` into a package directory.

### Cloud

```bash
pnpm dev            # Rust loopback :9899 + Vite WebUI :9898 + tray-only Tauri shell
pnpm dev:cloud:web  # WebUI hot-reload only (assumes the Rust server is already running)
```



The cloud WebUI is browser-accessible at [http://localhost:9898](http://localhost:9898).

**Cloud environment variables** (all optional, sensible defaults):

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `9898` | HTTP server port (set to 9899 in dev so Vite owns 9898) |
| `HOST` | `0.0.0.0` | Bind address |
| `DB_PATH` | `~/mhaol/cloud-rocksdb/` | SurrealDB store path |
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

### Rendezvous (private-swarm IPFS bootstrap + WebRTC signaling + TURN)

Rendezvous bundles the private-swarm IPFS bootstrap node, the WebSocket WebRTC signaling relay, and the TURN credential server into a single binary. It replaces the previous PartyKit/`mhaol-signaling` stack.

```bash
# Run rendezvous (HTTP 14080, libp2p TCP 14001)
pnpm app:rendezvous

# Linux deployment wizard (coturn + Let's Encrypt + systemd)
pnpm app:rendezvous:setup
```

---

## Building for Production

### Cloud

```bash
pnpm build:cloud
```

Builds the cloud WebUI and the release binary at `target/release/mhaol-cloud` (the WebUI is embedded into the binary).

### Rendezvous

```bash
pnpm build:rendezvous
```

Builds to `target/release/mhaol-rendezvous`.

---

## Desktop Tauri shell (Mhaol Cloud)

```bash
pnpm app:tauri:cloud         # Development
pnpm app:tauri:cloud:build   # Production
```

The Mhaol Cloud shell is **tray-only** — it never opens a window. The WebUI stays browser-accessible at `http://localhost:9898`.

---

## Quality Checks

Run all checks before committing:

```bash
pnpm lint       # Prettier + ESLint
pnpm check      # svelte-check + cargo check
pnpm build      # Build cloud WebUI and the mhaol-cloud release binary
pnpm test       # vitest
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
│   ├── cloud/             # Rust Axum server (port 9898) + nested Svelte WebUI + tray-only Tauri shell
│   ├── shepperd/          # Browser extension (Manifest V3)
│   └── signaling/         # Rust signaling server
├── packages/
│   ├── ui-lib/            # Shared components, services, types, CSS
│   ├── addons/            # TMDB, MusicBrainz, YouTube, LRCLIB, Wyzie subs
│   ├── webrtc/            # WebRTC contact handshake layer
│   ├── signaling/         # PartyKit signaling (cloud)
│   ├── identity/          # Rust Ethereum identity (secp256k1)
│   ├── torrent/           # Rust torrent client
│   ├── p2p-stream/        # Rust P2P streaming (GStreamer + WebRTC)
│   ├── yt-dlp/            # Rust yt-dlp wrapper
│   ├── ed2k/              # Rust eDonkey/ed2k client
│   └── ipfs/              # Rust embedded IPFS node (libp2p, private swarm)
├── .env                   # API keys and secrets (not committed)
├── package.json           # Workspace scripts
├── Cargo.toml             # Rust workspace
└── pnpm-workspace.yaml
```

### Architecture

The cloud WebUI is a thin wrapper. All shared frontend code (components, services, types, adapters) lives in `packages/ui-lib`. The WebUI only contains route files and configuration — it imports and assembles, never implements.

The frontend communicates with the cloud server via a transport layer (`packages/ui-lib/src/transport/`) that abstracts over HTTP and WebRTC, making the same API calls work whether connecting locally or peer-to-peer.

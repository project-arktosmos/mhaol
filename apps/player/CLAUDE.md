# Player

**Location:** `apps/player/`
**Framework:** SvelteKit (static SPA, `@sveltejs/adapter-static`) + Tailwind 4 + DaisyUI 5 — same toolchain as the cloud WebUI.
**Package:** `player` (pnpm workspace)
**Default port:** 9797

The player is a **browser-only** static SPA that joins the same private IPFS swarm as the cloud and renders firkins fetched directly from IPFS. It **never** talks to the cloud HTTP API:

- Firkin metadata is fetched as a UnixFS file at the firkin's CID (the same body the cloud pinned via `pin_firkin_body`).
- `ipfs`-typed `files` entries are fetched the same way and piped into a `<video>` / `<audio>` element via a Blob URL.
- Connectivity is over **libp2p** (Helia) with `WebSocket` and `WebTransport` transports, dialing the rendezvous bootstrap multiaddr; the `pnet` connection protector enforces private-swarm membership using the same swarm key the cloud and rendezvous use.

The player has exactly one route: `/player`. It is intentionally **not** a full feature parity port of the cloud catalog detail page — only the read-only display surfaces work. Trailers/tracks/torrent search require cloud-side APIs and are excluded.

## Source structure

```
apps/player/
├── package.json
├── svelte.config.js          # static adapter + aliases ($components, $ipfs, $types, $utils)
├── vite.config.ts            # Helia/libp2p prebundling
├── tsconfig.json
├── eslint.config.js
├── .prettierrc / .prettierignore
├── scripts/
│   └── run-vite.mjs          # Dev/build wrapper: reads swarm.key + bootstrap.multiaddr from disk and injects them as VITE_* env vars before spawning Vite
└── src/
    ├── app.html
    ├── app.d.ts
    ├── css/
    │   └── app.css           # Imports themes from ../../cloud/web/src/css/themes.css to share daisyUI tokens
    ├── routes/
    │   ├── +layout.svelte    # Navbar + main slot
    │   ├── +layout.ts        # ssr=false, prerender=false
    │   ├── +page.ts          # Redirects "/" to "/player"
    │   └── player/
    │       ├── +page.svelte  # Auto-connects on mount → CID input → fetch firkin → render shared cloud-ui catalog cards → play media
    │       └── +page.ts      # static-only flags
    ├── components/
    │   └── FirkinIpfsPlayer.svelte   # Per-firkin file picker + IPFS-fetched <video>/<audio>
    ├── ipfs/
    │   ├── client.ts                  # createPlayerIpfsClient: Helia + libp2p (WebSockets + WebTransport + pnet + noise + yamux)
    │   ├── config.ts                  # Reads playerIpfsConfig from import.meta.env.VITE_RENDEZVOUS_BOOTSTRAP / VITE_SWARM_KEY
    │   └── stream-player.ts           # MSE-fed streaming pipeline (mp4 via mp4box, webm direct, blob fallback)
    └── types/
        └── mp4box.d.ts                # Local type surface for the untyped `mp4box` package
```

Shared UI is imported from the workspace package `cloud-ui`:

```ts
import {
    CatalogPageHeader,
    CatalogDescriptionCard,
    CatalogIdentityCard,
    CatalogVersionHistoryCard,
    CatalogFilesTable,
    CatalogImagesCard,
    CatalogTrailersDisplay,
    FirkinArtistsSection,
    Icon,
    addonKind,
    type Firkin
} from 'cloud-ui';
```

The cloud WebUI's local catalog components are now thin wrappers around these same shared components, so layout/visuals stay synchronised between the two apps.

For UI glyphs use `<Icon name="<author>/<icon>" />` from `cloud-ui` — **no emoji in the player UI**. Icons inherit `currentColor`, so colour them via the surrounding text colour. Verify the icon exists at `packages/cloud-ui/src/icons/assets/<author>/<name>.svg` before referencing it; the component renders nothing on a typo. Full rules in the root CLAUDE.md "Icons" section.

## Connectivity model

The browser cannot speak raw TCP, so the player can only dial **WebSocket** / **WebTransport** multiaddrs. The rendezvous app exposes a `/ws` listener on `RENDEZVOUS_WS_LISTEN_PORT` (default `14002`); the transport stack on top is still `pnet → noise → yamux`, so browser peers must carry the same swarm key.

There is **no manual configuration UI** in the player. `scripts/run-vite.mjs` reads two files at startup and bakes their contents into the bundle as `VITE_*` env vars:

| Source | Resolution order | What it becomes |
|---|---|---|
| Swarm key | `IPFS_SWARM_KEY_FILE` → `${DATA_DIR}/swarm.key` → `~/mhaol/swarm.key` → `~/mhaol-cloud/swarm.key` | `VITE_SWARM_KEY` |
| Bootstrap | `RENDEZVOUS_BOOTSTRAP` (env, newline/comma-separated) → `RENDEZVOUS_BOOTSTRAP_FILE` → `${DATA_DIR}/rendezvous/bootstrap.multiaddr` → `~/mhaol/rendezvous/bootstrap.multiaddr` | `VITE_RENDEZVOUS_BOOTSTRAP` (filtered to `/ws` / `/wss` / `/webtransport` only) |

`src/ipfs/config.ts` reads those at module load and exposes `playerIpfsConfig`, `playerIpfsConfigured`, and `playerIpfsDiagnostic`. The `/player` page calls `getPlayerIpfsClient(playerIpfsConfig)` on mount when configured, and otherwise renders an inline error explaining what's missing.

If you change either file (e.g. restart the rendezvous so it writes a new peer id) you have to **restart `pnpm dev:player`** — the values are baked in at Vite startup, not re-read on hot reload.

## Playback model

`apps/player/src/ipfs/stream-player.ts` drives an MSE-fed pipeline so playback starts as soon as the first segment is decodable, rather than waiting for the full file to download. Three modes, picked by file extension:

| Mode | Trigger | How it works |
|---|---|---|
| `mse-mp4` | `.mp4` / `.m4v` | Drives `mp4box.js` to re-mux the incoming UnixFS chunks into fragmented MP4 segments, which are appended to a `MediaSource` `SourceBuffer`. Required because most `.mp4` files (torrent rippers, yt-dlp, etc.) are *unfragmented*, and `appendBuffer` rejects them outright. The init segment is produced as soon as `mp4box` parses the moov box. |
| `mse-webm` | `.webm` | Direct-feed: each Helia chunk goes straight into a `video/webm; codecs="vp9,opus"` (or `vp8,vorbis` fallback) `SourceBuffer`. |
| `blob` | everything else (`.mkv`, `.mov`, `.avi`, audio, unknown) | Old buffered fallback: fetch all bytes, wrap in a `Blob`, hand a `URL.createObjectURL` to `<video>`/`<audio>`. Used for containers the browser can't play through MSE anyway. |

`startStream({ client, cid, title, onProgress, signal })` returns a `StreamPlayerHandle` with the `src` URL, the chosen `mode`, a `done` promise, and a `cancel()` that aborts the in-flight UnixFS read, tears down the `MediaSource`, and revokes the URL. `FirkinIpfsPlayer.svelte` calls this on Play and surfaces the running byte counter + the chosen mode under the controls.

Limitations to be aware of:

- **`mp4` files without `faststart`**: `mp4box` can't produce an init segment until it has parsed the `moov` box, and tools like ffmpeg place `moov` at the *end* of the file by default. Such files end up effectively buffered (mp4box won't fire `onReady` until everything has arrived). To make these stream, re-encode with `-movflags +faststart` on the source.
- **`mkv` / `avi` / `mov`**: not supported by `MediaSource` in any browser, so streaming wouldn't change the outcome — `blob` mode is what those get. Real fix is a server-side transmux (the cloud already has `mhaol-ipfs-stream` for this; the player intentionally skips it because it would require talking to the cloud).

## Running

```bash
pnpm dev:player        # Dev — port 9797, no API proxy (the player has no API to proxy to). Alias for app:player.
pnpm app:player        # Same dev server, lower-level alias used by other scripts.
pnpm build:player      # Static build → apps/player/dist-static/
```

`pnpm dev` only starts the cloud strand (cloud Rust binary + cloud WebUI + Tauri shell); it does **not** start the player. Run `pnpm dev:player` in a separate terminal when you want the player up.

The player does not need the cloud running — it only needs:

1. A reachable rendezvous (`pnpm app:rendezvous`).
2. At least one peer on the same private swarm hosting the firkin and its files (typically the cloud, via `pnpm dev` / `pnpm app:cloud`).

## Logs

`pnpm app:player` tees stdout+stderr to `<repo-root>/logs/player.log`. When debugging, read it directly — don't ask the user to paste output.

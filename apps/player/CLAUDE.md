# Player

**Location:** `apps/player/`
**Framework:** SvelteKit (static SPA, `@sveltejs/adapter-static`) + Tailwind 4 + DaisyUI 5 — same toolchain as the cloud WebUI.
**Package:** `player` (pnpm workspace)
**Standalone dev port:** 9797
**In production:** served by the cloud HTTP server at `http://<cloud-host>:9898/player/` — the cloud binary embeds `apps/player/dist-static/` via `rust-embed` and exposes it under the `/player/` prefix.

The player is a **browser-only** static SPA that joins the same private IPFS swarm as the cloud and renders firkins fetched directly from IPFS. The HTTP fetch to `/api/p2p/bootstrap` is the *only* thing it asks of the cloud's HTTP API; everything else (firkin metadata + attached file bytes) is fetched as UnixFS blocks via libp2p:

- Firkin metadata is fetched as a UnixFS file at the firkin's CID (the same body the cloud pinned via `pin_firkin_body`).
- `ipfs`-typed `files` entries are fetched the same way and piped into a `<video>` / `<audio>` element via the MSE-backed stream-player or a Blob URL fallback.
- Connectivity is over **libp2p** (Helia) with `WebSocket` and `WebTransport` transports, dialing the cloud's `/ws` listener; the `pnet` connection protector enforces private-swarm membership using the same swarm key the cloud generated at startup.

The player has exactly one route: `/player`. It is intentionally **not** a full feature parity port of the cloud catalog detail page — only the read-only display surfaces work. Trailers/tracks/torrent search require cloud-side APIs and are excluded.

## Source structure

```
apps/player/
├── package.json              # `dev`/`build`/`preview` invoke vite directly; the cloud build sets BASE_PATH=/player.
├── svelte.config.js          # static adapter + aliases ($components, $ipfs, $types, $utils). `paths.base` reads BASE_PATH.
├── vite.config.ts            # Helia/libp2p prebundling + dev `/api` proxy → http://localhost:9898 (override with PLAYER_API_TARGET).
├── tsconfig.json
├── eslint.config.js
└── src/
    ├── app.html
    ├── app.d.ts
    ├── css/
    │   └── app.css           # Imports themes from ../../cloud/web/src/css/themes.css to share daisyUI tokens
    ├── routes/
    │   ├── +layout.svelte    # Navbar + main slot
    │   ├── +layout.ts        # ssr=false, prerender=false
    │   ├── +page.ts          # Redirects "/" to `${base}/player` (so /player and /player/player both work)
    │   └── player/
    │       ├── +page.svelte  # Auto-fetches /api/p2p/bootstrap on mount → connect → CID input → fetch firkin → render shared cloud-ui catalog cards → play media
    │       └── +page.ts      # static-only flags
    ├── components/
    │   └── FirkinIpfsPlayer.svelte   # Per-firkin file picker + IPFS-fetched <video>/<audio>
    ├── ipfs/
    │   ├── client.ts                  # createPlayerIpfsClient: Helia + libp2p (WebSockets + WebTransport + pnet + noise + yamux)
    │   ├── config.ts                  # `fetchPlayerIpfsConfig()` — runtime fetch of /api/p2p/bootstrap with diagnostic + error
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

The browser cannot speak raw TCP, so the player can only dial **WebSocket** / **WebTransport** multiaddrs. The cloud's embedded `mhaol-ipfs-core` node binds a libp2p `/ws` listener at `MHAOL_IPFS_WS_PORT` (default `9901`); the transport stack on top is still `pnet → noise → yamux`, so browser peers must carry the same swarm key.

There is **no manual configuration UI** in the player. On mount, `+page.svelte` calls `fetchPlayerIpfsConfig()` from `src/ipfs/config.ts`, which `GET`s `/api/p2p/bootstrap` and pulls back:

| Field | Where it comes from on the cloud |
|---|---|
| `peerId` | The cloud IPFS node's libp2p peer id |
| `swarmKey` | Plain text contents of `<DATA_DIR>/swarm.key` (server-side read) |
| `multiaddrs` | The cloud's libp2p listen addrs filtered to `/ws` / `/wss` / `/webtransport`, with `0.0.0.0` rewritten to loopback + the cloud's primary LAN IP |

`/api/p2p/bootstrap` returns `503` with `Retry-After: 1` while the IPFS node is still warming up; the page renders a "Loading IPFS bootstrap from cloud…" spinner until either a valid config arrives or `fetchPlayerIpfsConfig()` reports the error inline. If the cloud is unreachable (e.g. running standalone player dev without the cloud up), the page surfaces the fetch error directly.

Because the bootstrap is fetched at runtime (not baked at build time), rotating the swarm key on the cloud just requires restarting the cloud — the player picks up the new values on its next page load with no rebuild.

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
- **`mkv` / `avi` / `mov`**: not supported by `MediaSource` in any browser, so streaming wouldn't change the outcome — `blob` mode is what those get. Real fix is a server-side transmux (the cloud already has `mhaol-ipfs-stream` for this; the player intentionally skips it because it would require talking to the cloud beyond the bootstrap fetch).

## Running

```bash
# Standalone dev (separate Vite on :9797) — useful for hot-reloading the player without rebuilding the cloud binary every time.
pnpm dev:player        # Vite proxies /api → http://localhost:9898 (override via PLAYER_API_TARGET)
pnpm app:player        # Same dev server, lower-level alias used by other scripts.
pnpm build:player      # Static build → apps/player/dist-static/ (no base path)

# Production-like — built into the cloud binary.
pnpm build             # Runs `BASE_PATH=/player pnpm --filter player build` then builds the cloud which embeds the result.
# Then visit http://localhost:9898/player/ once the cloud is running.
```

`pnpm dev` builds the player with `BASE_PATH=/player` once at startup so the cloud's `rust-embed` finds something at compile time, then serves it at `http://localhost:9898/player/` alongside the cloud WebUI. It does **not** keep the player Vite dev server running — for hot-reload iteration on the player itself, run `pnpm dev:player` in a separate terminal (and visit `http://localhost:9797`).

The player needs the cloud running to fetch its bootstrap multiaddrs and swarm key, and at least one peer (typically the cloud itself, in single-machine setups) hosting the firkin and its files.

## Logs

`pnpm app:player` tees stdout+stderr to `<repo-root>/logs/player.log`. When debugging, read it directly — don't ask the user to paste output.

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
    │       ├── +page.svelte  # CID input → fetch firkin → render shared cloud-ui catalog cards → play media
    │       └── +page.ts      # static-only flags
    ├── components/
    │   ├── IpfsConfigPanel.svelte    # Modal for editing bootstrap multiaddrs + swarm key
    │   └── FirkinIpfsPlayer.svelte   # Per-firkin file picker + IPFS-fetched <video>/<audio>
    └── ipfs/
        ├── client.ts                  # createPlayerIpfsClient: Helia + libp2p (WebSockets + WebTransport + pnet + noise + yamux)
        └── config.svelte.ts           # localStorage-backed runes store for bootstrap addrs + swarm key
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
    addonKind,
    type Firkin
} from 'cloud-ui';
```

The cloud WebUI's local catalog components are now thin wrappers around these same shared components, so layout/visuals stay synchronised between the two apps.

## Connectivity model

The browser cannot speak raw TCP, so the player can only dial **WebSocket** / **WebTransport** multiaddrs. The rendezvous app exposes a `/ws` listener on `RENDEZVOUS_WS_LISTEN_PORT` (default `14002`); the transport stack on top is still `pnet → noise → yamux`, so browser peers must carry the same swarm key.

- `apps/rendezvous` writes its full bootstrap multiaddrs (including the `/ws` ones) to `<DATA_DIR>/rendezvous/bootstrap.multiaddr`. Pick any line ending in `/ws/p2p/<peer_id>` for the player config.
- The swarm-key value is the literal contents of `<DATA_DIR>/swarm.key` (start with `/key/swarm/psk/1.0.0/`).

Both values are stored in `localStorage` under `mhaol-player:ipfs-config` and edited via the **IPFS settings** modal on the `/player` page.

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
pnpm app:player        # Dev — port 9797, no API proxy (the player has no API to proxy to)
pnpm build:player      # Static build → apps/player/dist-static/
```

The player does not need the cloud running — it only needs:

1. A reachable rendezvous (`pnpm app:rendezvous`).
2. At least one peer on the same private swarm hosting the firkin and its files (typically the cloud, via `pnpm dev` / `pnpm app:cloud`).

## Logs

`pnpm app:player` tees stdout+stderr to `<repo-root>/logs/player.log`. When debugging, read it directly — don't ask the user to paste output.

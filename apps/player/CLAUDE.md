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

For each `ipfs`-typed `FirkinFile`:

1. The browser libp2p / Helia stack fetches the UnixFS root via the bootstrap peer (and any other private-swarm peers it discovers via Bitswap / Kademlia).
2. Bytes are concatenated into a single `Uint8Array` and wrapped as a typed `Blob`.
3. `URL.createObjectURL(blob)` becomes the `src` of a `<video>` or `<audio>` element.

Limitations of this approach (not yet addressed):

- **No streaming MSE** — the entire file must finish downloading before playback starts. Suitable for short clips and audio, slow for long videos. Future work: feed Helia chunks into a `MediaSource` `SourceBuffer` for `mp4` / `webm`.
- **Container compatibility** — the browser only plays containers/codecs it understands natively (mp4/H.264, webm/VP8/VP9, opus, etc.). `mkv`, `avi`, `mov`-with-uncommon-codecs are out of scope; the cloud's `ipfs-stream` HLS transmux pipeline is server-side and not invoked by the player.

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

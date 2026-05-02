# Frontend

**Location:** `packages/frontend/`
**Framework:** SvelteKit (Svelte 5 runes) + Vite + Tailwind 4 + DaisyUI 5
**pnpm package:** `frontend`

The Svelte SPA that the backend embeds and serves. Builds to `packages/frontend/dist-static/` via `@sveltejs/adapter-static`; that directory is what `mhaol-backend`'s `rust-embed` wrapper at `packages/backend/src/frontend.rs` includes at compile time. The user-facing port is always **9898**:

- **Dev** — Vite binds `0.0.0.0:9898` and serves the live Svelte app with hot reload. The backend bin binds `127.0.0.1:9899`. Vite proxies `/api/*` to `127.0.0.1:9899`, so the browser only ever talks to 9898.
- **Production (release builds)** — the backend bin binds `0.0.0.0:9898` and embeds `packages/frontend/dist-static/` via `rust-embed`, serving it directly as the fallback for any non-API path. Build with `pnpm --filter frontend build` (or `pnpm build:cloud` to build the SPA and the release bin together).

## Source Structure

```
packages/frontend/
├── package.json           # pnpm package "frontend"
├── svelte.config.js       # path aliases ($components, $services, $types, $adapters, $utils, $data, $transport)
├── vite.config.ts         # port 9898, /api proxy to 127.0.0.1:9899, fs.allow ../..
├── tsconfig.json
├── eslint.config.js
├── scripts/               # nav generator + Vite plugin
└── src/
    ├── routes/            # SvelteKit routes (+page.svelte, +layout.svelte)
    ├── components/        # Svelte components, organised by feature (catalog/, firkins/, core/, player/, libraries/, …)
    ├── services/          # Frontend services + runes-driven service classes (`*.svelte.ts`)
    ├── adapters/          # Adapter classes wrapping external APIs / signaling
    ├── transport/         # fetch / SSE / WebRTC RPC helpers (see "Transport layer" below)
    ├── types/             # Shared TS types
    ├── utils/             # Pure helpers (string, smart-search, localStorageWritableStore)
    ├── data/              # Static data (media-registry, …)
    ├── lib/               # SvelteKit `$lib` files (per-page services + helpers like image-cache, firkins.service.ts, youtube-match.service.ts)
    ├── app-shims/         # Svelte/Tauri environment shims
    └── css/               # Tailwind/DaisyUI entry + theme tokens
```

The frontend owns its full stack — there is no separate shared UI package. Aliases in `svelte.config.js` keep cross-module imports short:

```javascript
alias: {
  $components: 'src/components',
  $services: 'src/services',
  $types: 'src/types',
  $adapters: 'src/adapters',
  $utils: 'src/utils',
  $data: 'src/data',
  $transport: 'src/transport',
  'app-shims': 'src/app-shims'
}
```

Plus the SvelteKit-reserved `$lib` (→ `src/lib/`) and `$app/*` (SvelteKit modules).

`src/css/app.css` scans the SPA's own `src/` for Tailwind classes:

```css
@import 'tailwindcss';
@plugin 'daisyui';
@source '../';
@import './themes.css';
```

## Catalog detail routes

There is only one catalog detail route, `/catalog/[id]`, and it presents both bookmarked and non-bookmarked firkins. Catalog grid clicks go through the `/catalog/visit` resolver route, whose `+page.ts` POSTs `/api/firkins` with `bookmarked: false` and `redirect(303, …)`s to `/catalog/<returnedId>`. The server dedups by content-address so revisits don't mint duplicate records, and the same record is later promoted in place when the user clicks **Bookmark** (a `PUT /api/firkins/:id` with `{ bookmarked: true }` — `bookmarked` is not part of the firkin body, so no CID roll).

The detail page reads `firkin.bookmarked` and switches between two presentations of the same surface:

| Concern                                  | `firkin.bookmarked === false` (browse cache)                                            | `firkin.bookmarked === true` (full library item)                            |
| ---------------------------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Source of firkin data                    | `+page.ts` loader → real persisted firkin (created by `/catalog/visit`)                 | `+page.ts` loader → real persisted firkin                                   |
| Header actions                           | `Bookmark` only (plus a `browse` warning badge in the header)                           | `Play` / `IPFS Play` / `Torrent Stream` / `Find metadata` / `Delete firkin` |
| Identity / version history / files table | omitted                                                                                 | rendered                                                                    |
| Tracks card mode                         | `preview` (no per-track YT/lyrics badges, no playback)                                  | full (server-side album resolver populates YT URLs + lyrics)                |
| Resolver `persist` callbacks             | trailers no-op (return `Promise.resolve()`); YT preferred-client persist short-circuits | `PUT /api/firkins/:id` (rolls the CID forward)                              |
| Torrent search card                      | hidden                                                                                  | shown (auto-fires when no real files yet)                                   |
| IPFS / Torrent stream tabs               | hidden                                                                                  | shown when applicable                                                       |
| Recommendation ingest on related items   | skipped                                                                                 | runs once per source firkin                                                 |
| Album resolver auto-spawn (server-side)  | skipped — see `firkins.rs::create`                                                      | spawned for fresh musicbrainz creates and on the false→true bookmark flip   |

**Shared components** (`src/components/catalog/`):

- `CatalogPageHeader.svelte` — back link, title, addon/kind/year badges, optional `extraBadge`, action snippet slot
- `CatalogDescriptionPanel.svelte` — tabbed panel showing the description (default tab), identity (CID / created / updated / version, detail only), and version history (`version_hashes` chain, detail only). Tabs are only rendered when the corresponding props are supplied — virtual pages get a description-only single-tab layout with no tab strip. When `reviews` is non-empty (TMDB / MusicBrainz user-rating snapshots), the panel also renders a row of compact `label score / maxScore · votes` badges above the tabs — visible on every tab.
- `CatalogImagesCard.svelte` — images grid with metadata
- `CatalogTrailersCard.svelte` — trailers list driven by a `TrailerResolver`
- `CatalogTracksCard.svelte` — MusicBrainz tracks list driven by a `TrackResolver`. `preview={true}` (used by the detail page when `!firkin.bookmarked`) hides per-track YouTube/lyrics status badges and disables play, since nothing has been resolved yet — bookmarking is what kicks off the server-side per-track YouTube + LRCLIB resolution. Per-row buttons split by source: **Play** (only when the track has a local `ipfs`-typed FileEntry — streams `/api/ipfs/pins/<cid>/file` via `playerService.playUrl`) and **Stream** (when only a YouTube URL is persisted — runs through `playYouTubeAudio`). When the firkin is bookmarked and at least one track still has a YouTube URL but no local file, a **Download album** button at the top kicks `POST /api/firkins/:id/download-album`; the page polls `/download-progress` to flip per-row status badges (`DL queued` → `DL N%` → `DL ✓`) until each track lands as an `ipfs` FileEntry
- `CatalogTorrentSearchCard.svelte` — torrent search results, optional collapsible + per-row streamability eval
- `CatalogSubsLyricsCard.svelte` — subs/lyrics search results driven by a `SubsLyricsResolver`. Lyrics for MusicBrainz albums auto-fire on detail mount; subtitles defer until the user actually starts a stream — for `tmdb-movie` that's the IPFS or torrent tab, for `tmdb-tv` it's clicking **Play** on a specific episode (the search re-fires per-episode when the user pivots, since OpenSubtitles' TV scope is `(imdb, season, episode)`). Read-only — clicking a row previews lyrics inline or opens the subtitle URL. The active sub picker (language `eng`/`cat`/`spa` → release filename → "Use subtitle") sits above the in-page player and is wired to `firkinsService.attachSubtitle(...)`, which posts `/api/firkins/:id/subtitle` so the SRT is fetched, converted to VTT, pinned to IPFS, and rolled onto the firkin as a `subtitle`-typed FileEntry whose `value` carries the `cid` the player streams via `/api/ipfs/pins/<cid>/file`. For TV the FileEntry also carries the `(season, episode)` it was pinned to, and the page's player-side filter restricts the visible subtitles to whatever episode is currently streaming so an S01E03 sub doesn't bleed onto S01E04
- `CatalogChannelLatestCard.svelte` — "Latest from channel" rail rendered on the left column of `youtube-video` catalog pages. Calls `GET /api/ytdl/channel/by-video?url=<watch URL>` once per page and renders the last ~8 entries from the channel's public Atom feed; the backend caches both the video → channel id resolve (24h) and the parsed feed (15min) so the public feed endpoint isn't hammered.
- `CatalogRelatedYoutubeCard.svelte` — "Related videos" rail rendered on the right column of `youtube-video` catalog pages (both `/catalog/[ipfsHash]` and `/catalog/virtual`). Calls `GET /api/ytdl/related?url=<watch URL>` once per page and renders up to ~12 entries from the InnerTube `/next` watch-next sidebar. Click-throughs open `youtube.com/watch?v=…` in a new tab — there's no in-app navigation back into the catalog yet.
- `CatalogFilesTable.svelte` — firkin `files` table (detail only; suppressed for `musicbrainz` firkins since the tracks card already surfaces every per-track entry)
- `FirkinLibraryGrid.svelte` — pure presentational grid that renders firkins by id (looked up from `firkinsService.state`). Always 7 cols. `collapsed={true}` (default) slices to `collapsedCount` (default 6) and renders the 7th cell as a "More (+N)" link to `moreHref`; `collapsed={false}` is the gallery mode (multi-row, full set). Optional `actions` snippet renders a per-firkin overlay (used on `/catalog` for the "Find metadata" button). Used by `/catalog` (collapsed) and `/catalog/gallery?addon=…` (full).

**Shared resolver services** (`src/services/catalog/`, all `.svelte.ts` so `$state` runes work):

- `trailer-resolver.svelte.ts` — `TrailerResolver` class. `resolveMovie(...)` / `resolveTv(...)` accept TMDB-sourced trailers via `stored`, prefer them when present, and only fall back to the YouTube fuzzy search when TMDB has nothing English. Optional `persist` callback writes back via `PUT /api/firkins/:id`.
- `track-resolver.svelte.ts` — `TrackResolver` class. Pure projection — _no in-browser searches anywhere_. Single entry point `loadFromFirkin({ releaseGroupId, files })` fetches the MusicBrainz tracklist and pairs each track with its persisted YouTube URL + lyrics from the firkin's `files`, returning `{ missingAny }`. The detail page uses `missingAny` to decide whether to poll for the rolled-forward firkin while the server's background album resolver runs. Lyrics persisted on the firkin live as `'lyrics'`-typed `FileEntry` rows whose `value` is the JSON `{ source, externalId, syncedLyrics, plainLyrics, instrumental }`; the resolver decodes this on read and parses the LRC text into the existing `SubsLyricsItem` shape. The resolver also maps `'ipfs'`-typed `FileEntry` rows by track title, exposing each track's `localCid` and a `playLocal(index, opts)` method that builds a `PlayableFile` and calls `playerService.playUrl` against `/api/ipfs/pins/<cid>/file` — the catalog tracks card uses this to render the per-row **Play** button alongside the YouTube **Stream** button. `applyDownloadProgress(payload)` overlays `/api/firkins/:id/download-progress` onto the projected tracks so per-row download status badges update live without a firkin refetch.
- `torrent-search.svelte.ts` — `TorrentSearch` class. Optional `evaluate: true` runs `/api/torrent/evaluate` per result with a sliding-window concurrency cap (default 4). Also exports `startTorrentDownload(magnet)`.
- `subs-lyrics-resolver.svelte.ts` — `SubsLyricsResolver` class. `search({ addon, query, externalIds? })` posts `/api/search/subs-lyrics` and exposes `results`, `status`, `error` as runes.

**Pattern.** When two routes need the same UI: put the markup in `$components/<feature>/`, put the behaviour in a runes-driven service class at `$services/<feature>/<thing>.svelte.ts`, and let each route compose them with route-specific inputs and (optional) persistence callbacks. The presentational components stay free of business logic; the service classes own the state machines and side-effects.

## Mutating `firkin.files` — always use `firkinsService.mutateFiles`

Any code that adds, replaces, or removes entries on a firkin's `files` array **must** go through `firkinsService.mutateFiles(id, { add?, removeTypes?, removeEntries? })` (`src/lib/firkins.service.ts`). Never PUT a full `files` array constructed from a client-side snapshot of `firkin.files` — that pattern is racy by construction: two concurrent flows (e.g. the user picking a torrent in `CatalogTorrentSearchCard` while the trailer resolver caches its preferred YouTube client) each take a snapshot, append their entry, and PUT — the later PUT wins, so one of the writes is silently lost. This is exactly how a freshly-attached `torrent magnet` got dropped, leaving the torrent permanently unmatched by `torrent_completion::run`.

`firkinsService.mutateFiles` posts to `POST /api/firkins/:id/files`; the server reads the current `files` under the per-firkin async lock, drops every entry whose `kind` is in `removeTypes` or whose `(kind, value)` matches `removeEntries`, then appends `add`, and rolls forward. Concurrent calls serialise so neither writer can lose the other's change. Examples:

```ts
// Append a torrent magnet (assignTorrent in /catalog/[id])
await firkinsService.mutateFiles(firkin.id, {
	add: [{ type: 'torrent magnet', value: magnet, title: torrent.title }]
});

// Replace the singular `youtube preferred client` cache entry
await firkinsService.mutateFiles(firkin.id, {
	removeTypes: ['youtube preferred client'],
	add: [{ type: 'youtube preferred client', value: clientName }]
});
```

`PUT /api/firkins/:id` is still the right call for whole-record edits where the user really does mean "replace all files" (e.g. the firkin editor on `/firkins`), and for non-`files` patches like `{ trailers }`, `{ reviews }`, `{ bookmarked }`, etc. — those are field-scoped on the server and don't have the read-modify-write hazard.

## Transport layer

All frontend-to-backend communication flows through `src/transport/`:

- `transport.type.ts` — `Transport` interface (fetch, subscribe, resolveUrl)
- `ws-transport.ts` — WebSocket RPC implementation (sends requests over a peer connection)
- `fetch-helpers.ts` — `fetchJson()`, `fetchRaw()`, `subscribeSSE()` used by all services
- `transport-context.ts` — Module-level singleton (`setTransport` / `getTransport`); the default fallback talks plain HTTP via `globalThis.fetch`
- `rpc.type.ts` — RPC message protocol types

Services should never call `fetch` directly when they need transport-aware behaviour — go through `fetchJson` / `fetchRaw` / `subscribeSSE` so the same code paths work over HTTP and WebSocket.

## Backend URL configuration

`src/lib/api-base.ts` is the single source of truth for the API base URL:

- **Browser** (cloud / headless served from the same backend): defaults to `''` (same-origin).
- **Tauri shells** (cloud tray, android-tv, android-mobile): defaults to `http://127.0.0.1:9898`. For `android-mobile` this matches the embedded backend; for `android-tv` the user must override via the in-app **Settings** page (`/settings`); for `apps/cloud` the tray-only shell never opens a window so the default is academic.
- **Override**: `localStorage["mhaol-api-base"]` wins. The `/settings` page exposes get/set/reset and a **Test connection** button that hits `/api/cloud/status`.

`src/lib/install-fetch-interceptor.ts` wraps `globalThis.fetch` so any request whose path starts with `/api/` is rewritten through `apiUrl(...)`. This catches the many service files that call `fetch('/api/…')` directly without going through the transport layer; without the interceptor, those would never reach the backend from a Tauri shell. The interceptor is installed by `src/routes/+layout.svelte` at module load.

## Bottom-right corner player

`src/routes/+layout.svelte` is navbar + main only; there is no right-side aside. The only persistent overlay is the fixed bottom-right `NavbarAudioPlayer` (with `NavbarLyricsPanel` and `NavbarPlaylistPanel`), shown when `playerService.displayMode === 'navbar'` and a file is loaded. The layout calls `playerService.initialize()` on mount so the player's stores wake up; the backend's `/api/player/stream-status` and `/api/player/playable` stubs let initialize settle without errors.

Audio playback uses the dedicated `displayMode === 'navbar'` mode. `NavbarAudioPlayer.svelte` is a compact horizontal strip (thumbnail, title, play/pause, position, seek bar, duration, stop) that owns its own hidden `<video>` element wired to `playerService.state.directStreamUrl`. Audio callers (the catalog tracks card via `playYouTubeAudio` in `src/lib/youtube-match.service.ts`, and the `/youtube` page when `mode === 'audio'`) call `playerService.playUrl(file, url, mime, 'navbar')` to surface playback here. Firkin in-page playback (`/catalog/[ipfsHash]`) uses `'inline'`.

## `/youtube` route

`src/routes/youtube/+page.svelte` is a self-contained yt-dlp UI. It talks **directly** to `/api/ytdl/*` via plain `fetch()` (no transport layer) — search, paste-URL info, queue audio/video/both, live progress via SSE on `/api/ytdl/downloads/events`, and "Stream" buttons that call `playerService.playUrl()` so the result plays in the navbar audio player.

## YouTube extraction (music + trailers)

`/catalog/virtual` and `/catalog/[ipfsHash]` share one YouTube-match stack at `src/lib/youtube-match.service.ts`: a free-text query goes to `/api/ytdl/search`, then a "double-dip" picker filters down to the best match.

- **Music**: `pickBestYouTubeMatch` requires ≥50% of the track title's tokens to appear in the result, then scores by track-title overlap, artist hits in title+uploader, album hits in title, and duration delta — used to back-fill `url`-typed `files` entries on MusicBrainz firkins.
- **Trailers** (movies and TV-per-season): `pickBestTrailerMatch` reuses the same shape — ≥50% of the item's title tokens are required, the result must contain `"trailer"`, and (for TV) the season tag (`s01`, `season 1`, `s1`) is required; scoring rewards title overlap, the trailer keyword, year hits, and season-tag hits, while `reaction`/`review`/`recap`/`breakdown`/`fanmade`/`behind the scenes` etc. impose a negative penalty so commentary clips lose to the actual trailer.

Resolved trailers are persisted on `firkin.trailers` (`{ youtubeUrl, label?, language? }`). `tmdb-tv` firkins also persist their upstream id as a `url` file (`https://www.themoviedb.org/tv/<id>`) so the detail page can re-fetch the season list from `/api/catalog/tmdb-tv/:id/seasons` if the stored array is empty.

## Media route architecture

Media routes use slug-based routing with a data-driven registry:

```
(app)/media/
├── +layout.svelte              # Media bar (title, controls, tabs, filters)
├── [slug]/                     # movies, tv
│   ├── +page.ts               # Validates slug against MEDIA_REGISTRY
│   ├── +page.svelte           # CatalogBrowsePage + per-type extras
│   └── [id]/+page.svelte      # CatalogDetailPage + per-type meta
├── music/                      # Music hub + nested sub-slugs
│   ├── +page.svelte           # Hub (pinned, favorites, popular preview)
│   └── [subslug]/             # album, artist
│       ├── +page.ts           # Validates subslug against MUSIC_REGISTRY
│       ├── +page.svelte       # CatalogBrowsePage with strategy
│       └── [id]/+page.svelte  # CatalogDetailPage + meta
├── youtube/                    # Explicit (custom UI: channels, RSS, downloads)
└── photos/                     # Explicit (custom UI: gallery, tagging)
```

**Key files**:

- `src/data/media-registry.ts` — `MEDIA_REGISTRY` and `MUSIC_REGISTRY` mapping slugs to config (kind, label, services, features)
- `src/components/catalog/CatalogBrowsePage.svelte` — Unified browse with search, tabs, filters, pinned/favorites, grid
- `src/components/catalog/filters/CatalogFilterBar.svelte` — Switch component rendering the right filter UI per kind
- `src/services/catalog.service.ts` — Strategy-pattern service (`CatalogKindStrategy` interface)
- `src/services/catalog-strategies/` — Per-kind strategies (movie, tv, album, artist, game)

**Adding a new media type:** Add an entry to `MEDIA_REGISTRY` (or `MUSIC_REGISTRY`), create a catalog strategy, a detail meta component, and add filter handling if needed. The slug routes handle everything else.

## Icons

Use `<Icon name="<author>/<icon>" />` from `cloud-ui` for every UI glyph. **No emoji in the SPA.** Icons inherit the surrounding text colour via `currentColor`, so colour them with the standard text utilities (`text-primary`, `text-error`, `text-base-content/60`, …). Before writing a name, verify the file exists at `packages/cloud-ui/src/icons/assets/<author>/<name>.svg` (or grep `packages/cloud-ui/src/icons/icon-names.ts`) — typos render an invisible icon, not a build error. See the root `CLAUDE.md` "Icons" section for the full rules.

## Running

```bash
# Dev — frontend hot-reload on 9898 only (proxies /api to 127.0.0.1:9899; assumes the backend is already running)
pnpm dev:cloud:web

# Dev — full desktop stack (backend + Vite frontend + Tauri tray shell)
pnpm dev

# Production build
pnpm build:cloud:web    # SPA only
pnpm build:cloud        # SPA + backend release bin (embeds the SPA)
```

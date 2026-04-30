# Package: ui-lib

**Location:** `packages/ui-lib/`
**Role:** All shared frontend code — UI components, services, types, adapters, utils, and CSS themes
**Framework:** Svelte 5 + TailwindCSS v4 + DaisyUI v5

This package contains everything shared across all apps: UI components, state management services, data adapters, type definitions, utilities, and styling.

## Source Structure

```
src/
├── components/               # UI components organized by feature
│   ├── addons/               # Addon management
│   ├── browse/               # Generic browse primitives (BrowseGrid, BrowseHeader, BrowseViewToggle)
│   ├── catalog/              # Unified media catalog (CatalogCard, CatalogBrowsePage, CatalogDetailPage, PinnedFavoritesSection, TmdbCatalogGrid, MovieLibrarySection, TvLibrarySection, detail/, filters/)
│   ├── core/                 # Shared reusable (Button, Modal, Navbar, ModalOutlet, ThemeToggle, ConnectionStatus, etc.)
│   ├── firkins/             # Cloud firkin cards (FirkinCard)
│   ├── downloads/            # Download management
│   ├── hub/                  # Hub dashboard (app management)
│   ├── identity/             # Identity/wallet
│   ├── images/               # Image tagging
│   ├── landing/              # Marketing/landing page (Hero, Features, Platforms, Footer, LandingNavbar)
│   ├── libraries/            # Media libraries (list, files, link modals, LibrarySelector, content grid/card)
│   ├── smart-search/         # Smart search modal, sections (movie/tv/music), results table, toast
│   ├── queue/                # Queue task monitor (real-time visualization)
│   ├── media/                # Media cards (Movie, TV, Audio, Image, YouTube, uncategorized)
│   ├── music/                # Music components (legacy, being replaced by catalog/)
│   ├── p2p-stream/           # P2P streaming
│   ├── peer-libraries/       # Peer library browsing
│   ├── player/               # Video/audio player (PlayerVideo, PlayerControls, MediaPlayer, LyricsPanel)
│   ├── plugins/              # Plugin management
│   ├── roster/               # Peer roster
│   ├── settings/             # Settings (SettingsModalContent, DiskContent, TubeSettingsContent)
│   ├── setup/                # Node setup (SetupGate, SetupModalContent — connection config + transport selection)
│   ├── shepperd/             # Shepperd import (ShepperdImportContent, SmartPairResults)
│   ├── share/                # Share modal
│   ├── signaling/            # Signaling/WebRTC
│   ├── subtitles/            # Subtitle search modal (Wyzie subs addon)
│   ├── tmdb-browse/          # TMDB pagination (TmdbPagination — used by movies/TV pages)
│   ├── torrent/              # Torrent management (TorrentProgressOverlay, TorrentSettings, etc.)
│   ├── ed2k/                 # ed2k/eDonkey client UI (Ed2kModalContent, Ed2kSearch, Ed2kFileList, Ed2kStatusBar)
│   ├── videogames/           # Videogame WASM emulator components (EmulatorModal mounts EmulatorJS from cdn.emulatorjs.org; emulator-cores.ts maps console name + ROM extension to a core id — currently `gambatte` for Game Boy / Game Boy Color)
│   ├── youtube/              # YouTube download (queue, settings, preview, RightPanel)
│   └── youtube-search/       # YouTube search (input, results, channel cards)
├── services/                 # State management + API calls (singleton services)
│   ├── classes/              # Base classes: ArrayServiceClass, ObjectServiceClass
│   ├── catalog-strategies/   # Per-kind browse strategies (movie, tv, book, album, artist, game, iptv, youtube, photo)
│   ├── i18n/                 # svelte-i18n locales (en.json, qq.json)
│   ├── catalog.service.ts    # Unified catalog browse service with strategy pattern
│   ├── fetch-cache.service.ts # Fetch cache for torrent download tracking on browse pages
│   ├── image-overrides.service.ts # TMDB image override management
│   └── *.service.ts          # Feature services
├── adapters/                 # Data transformation logic
│   └── classes/              # Adapter singletons (player, signaling, library-file, etc.)
├── types/                    # TypeScript type definitions (one file per domain)
├── utils/                    # Pure utility functions
│   ├── localStorageWritableStore.ts
│   └── string/               # capitalize, normalize
├── lib/                      # Platform detection + API base URL
│   ├── platform.ts           # isTauri, isMobile detection
│   └── api-base.ts           # apiUrl() helper with Tauri fallback
├── data/                     # Static data (releases.json, recommended-models.ts, media-registry.ts)
├── app-shims/                # SvelteKit virtual module shims ($app/environment)
└── css/                      # Styling
    ├── app.css               # Base Tailwind + DaisyUI
    └── themes.css            # Custom light/dark themes (OKLCH)
```

## Import Conventions

**All imports within ui-lib and from consuming apps** use `ui-lib/...` paths:

```typescript
import Modal from 'ui-lib/components/core/Modal.svelte';
import { modalRouterService } from 'ui-lib/services/modal-router.service';
import type { ID } from 'ui-lib/types/core.type';
import { apiUrl } from 'ui-lib/lib/api-base';
import { playerAdapter } from 'ui-lib/adapters/classes/player.adapter';
```

## Component Rules

1. No business logic in components — delegate to services/adapters
2. No `<style>` tags — use Tailwind classes only
3. No inline `style` attributes
4. Use `classnames` for all conditional class rendering
5. Type all props with inline type annotations on `$props()`
6. Use callback props for parent communication (e.g. `onClose`, `onSave`)
7. Keep components small — split when they grow
8. Use Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`)
9. Every new component should have tests in `test/`

## Service Classes

```typescript
// ArrayServiceClass<T> — for collections
export const myItemsService = new ArrayServiceClass<MyItem>('my-items', []);

// ObjectServiceClass<T> — for single objects
export const settingsService = new ObjectServiceClass<Settings>('settings', initialSettings);
```

## CSS & Styling

| Rule                      | Detail                      |
| ------------------------- | --------------------------- |
| NEVER use `<style>` tags  | Tailwind only               |
| NEVER use inline `style=` | Tailwind only               |
| ALWAYS use `classnames`   | For conditional classes     |
| Stack                     | TailwindCSS v4 + DaisyUI v5 |

## Testing

Tests live in `packages/ui-lib/test/` mirroring `src/`.

```bash
pnpm test             # vitest
pnpm test:ui          # interactive UI
pnpm test:coverage    # coverage report
```

## Dependencies

- `classnames` — conditional CSS class composition
- `svelte-i18n` — internationalization
- `viem` — Ethereum signing (signaling, player services)
- `fflate` — compression
- `html5-qrcode`, `qrcode` — QR code generation/scanning
- `addons` (workspace) — TMDB, torrent search, MusicBrainz, RetroAchievements, YouTube, LRCLIB (use `addons/{addon}/...` paths)
- `webrtc` (workspace) — WebRTC contact handshake layer

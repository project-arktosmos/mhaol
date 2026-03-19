# Package: frontend

**Location:** `packages/frontend/`
**Role:** Shared frontend logic — services, adapters, types, utils, and data for every app

This package is **not** a SvelteKit app itself. It is a library consumed by apps and by `packages/ui-lib`. UI components and CSS live in `packages/ui-lib`.

## Source Structure

```
src/
├── adapters/classes/         # Data transformation logic
├── services/                 # State management + API calls
│   ├── classes/              # ArrayServiceClass, ObjectServiceClass
│   ├── i18n/                 # svelte-i18n locales (en.json, qq.json)
│   └── *.service.ts          # Feature services (see list below)
├── types/                    # TypeScript type definitions (one file per domain)
├── utils/                    # Pure utility functions
│   ├── localStorageWritableStore.ts
│   ├── string/               # capitalize, normalize
│   ├── musicbrainz/          # MusicBrainz API client + transforms
│   ├── tmdb/                 # TMDB image URL helpers + transforms
│   ├── torrent-search/       # Torrent result formatting
│   └── youtube/              # YouTube embed helpers
├── data/                     # Static JSON data (releases.json, recommended-models.ts)
└── lib/                      # Platform detection + API base URL
    ├── platform.ts           # isTauri, isMobile detection
    └── api-base.ts           # apiUrl() helper with Tauri fallback
```

## Import Conventions

**Within packages/frontend** — use `frontend/...` paths:

```typescript
import { modalRouterService } from 'frontend/services/modal-router.service';
import type { ID } from 'frontend/types/core.type';
```

**From consuming apps** — same `frontend/...` paths work via the alias in svelte.config.js.

---

## Services

Feature services (all in `src/services/`):

| Service                             | Description                                             |
| ----------------------------------- | ------------------------------------------------------- |
| `downloads.service.ts`              | Download queue state                                    |
| `identity.service.ts`               | Wallet/identity                                         |
| `image-tagger.service.ts`           | Image auto-tagging                                      |
| `jackett-search.service.ts`         | Jackett torrent search                                  |
| `library.service.ts`                | Filesystem media libraries                              |
| `llm.service.ts`                    | LLM chat conversations                                  |
| `lyrics.service.ts`                 | Song lyrics fetching                                    |
| `media-detail.service.ts`           | Media detail panel state                                |
| `media-mode.service.ts`             | Audio/video mode toggle                                 |
| `modal-router.service.ts`           | URL-synced modal state                                  |
| `p2p-stream.service.ts`             | P2P streaming                                           |
| `peer-library.service.ts`           | Peer library discovery                                  |
| `player.service.ts`                 | Media player state                                      |
| `right-panel.service.ts`            | Right panel video selection                             |
| `sidebar.service.ts`                | Sidebar state                                           |
| `signaling-chat.service.ts`         | Signaling chat                                          |
| `theme.service.ts`                  | Light/dark theme + DOM sync                             |
| `tmdb-browse.service.ts`            | TMDB popular/discover                                   |
| `torrent-search.service.ts`         | Torrent search                                          |
| `torrent.service.ts`                | Torrent management                                      |
| `wallet.service.ts`                 | Crypto wallet                                           |
| `youtube.service.ts`                | YouTube download + streaming                            |
| `youtube-channel-search.service.ts` | YouTube channel search                                  |
| `youtube-library.service.ts`        | YouTube content library (favorites, content management) |
| `youtube-search.service.ts`         | YouTube video search                                    |

### Service classes

```typescript
// ArrayServiceClass<T> — for collections
export const myItemsService = new ArrayServiceClass<MyItem>('my-items', []);

// ObjectServiceClass<T> — for single objects
export const settingsService = new ObjectServiceClass<Settings>('settings', initialSettings);
```

---

## Adapters

All in `src/adapters/classes/`:

- `adapter.class.ts` — base class
- `identity.adapter.ts` — identity/wallet data
- `library-file.adapter.ts` — library file data
- `llm.adapter.ts` — LLM conversation data
- `peer-library.adapter.ts` — peer library data
- `player.adapter.ts` — media player data
- `signaling.adapter.ts` — signaling/P2P data
- `youtube-card.adapter.ts` — YouTube content → LibraryCardItem

---

## Types

One file per domain in `src/types/`:

`core.type.ts`, `download.type.ts`, `identity.type.ts`, `image-tagger.type.ts`, `library.type.ts`, `llm.type.ts`, `media-card.type.ts`, `media-detail.type.ts`, `media-list.type.ts`, `modal.type.ts`, `musicbrainz.type.ts`, `p2p-stream.type.ts`, `peer-library.type.ts`, `player.type.ts`, `route.type.ts`, `sidebar.type.ts`, `signaling.type.ts`, `tmdb-browse.type.ts`, `torrent.type.ts`, `youtube.type.ts`, `youtube-search.type.ts`

---

## Testing

Tests live in `packages/frontend/test/` mirroring `src/`.

```bash
pnpm test             # vitest
pnpm test:ui          # interactive UI
pnpm test:coverage    # coverage report
```

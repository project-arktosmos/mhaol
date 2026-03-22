# Package: ui-lib

**Location:** `packages/ui-lib/`
**Role:** All Svelte UI components and CSS themes
**Framework:** Svelte 5 + TailwindCSS v4 + DaisyUI v5

This package contains every UI component used across all apps. It depends on `packages/frontend` for services, types, adapters, and utils.

## Source Structure

```
src/
├── components/               # UI components organized by feature
│   ├── addons/               # Addon management
│   ├── core/                 # Shared reusable (Button, Modal, Navbar, ModalOutlet, ThemeToggle, etc.)
│   ├── downloads/            # Download management
│   ├── hub/                  # Hub dashboard (app management)
│   ├── identity/             # Identity/wallet
│   ├── images/               # Image tagging
│   ├── jackett/              # Jackett search
│   ├── landing/              # Marketing/landing page (Hero, Features, Platforms, Footer, LandingNavbar)
│   ├── libraries/            # Media libraries (list, files, link modals, content grid/card)
│   ├── llm/                  # LLM chat
│   ├── media/                # Media cards (Movie, TV, Audio, Image, YouTube, uncategorized)
│   ├── p2p-stream/           # P2P streaming
│   ├── peer-libraries/       # Peer library browsing
│   ├── player/               # Video/audio player (PlayerVideo, PlayerControls, MediaPlayer, LyricsPanel)
│   ├── plugins/              # Plugin management
│   ├── settings/             # Settings (SettingsModalContent, DiskContent, TubeSettingsContent)
│   ├── signaling/            # Signaling/WebRTC
│   ├── tmdb-browse/          # TMDB movie/TV browsing
│   ├── torrent/              # Torrent management
│   ├── videogames/           # Videogame browsing (GameCard — RetroAchievements)
│   ├── youtube/              # YouTube download (queue, settings, preview, RightPanel)
│   └── youtube-search/       # YouTube search (input, results, channel cards)
└── css/                      # Styling
    ├── app.css               # Base Tailwind + DaisyUI
    └── themes.css            # Custom light/dark themes (OKLCH)
```

## Import Conventions

**Within ui-lib** — use `ui-lib/...` for other components:

```typescript
import Modal from 'ui-lib/components/core/Modal.svelte';
import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';
```

**For services, types, adapters, utils** — use `frontend/...` paths:

```typescript
import { modalRouterService } from 'frontend/services/modal-router.service';
import type { ID } from 'frontend/types/core.type';
import { apiUrl } from 'frontend/lib/api-base';
```

**From consuming apps** — use `ui-lib/...` for components, `frontend/...` for everything else.

## Component Rules

1. No business logic — delegate to services/adapters in `packages/frontend`
2. No `<style>` tags — use Tailwind classes only
3. No inline `style` attributes
4. Use `classnames` for all conditional class rendering
5. Type all props with inline type annotations on `$props()`
6. Use callback props for parent communication (e.g. `onClose`, `onSave`)
7. Keep components small — split when they grow
8. Use Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`)
9. Every new component must have a `.stories.svelte` file in `apps/storybook/src/stories/{category}/`

## CSS & Styling

| Rule                      | Detail                      |
| ------------------------- | --------------------------- |
| NEVER use `<style>` tags  | Tailwind only               |
| NEVER use inline `style=` | Tailwind only               |
| ALWAYS use `classnames`   | For conditional classes     |
| Stack                     | TailwindCSS v4 + DaisyUI v5 |

## Dependencies

- `frontend` (workspace) — services, types, adapters, utils
- `classnames` — conditional CSS class composition
- `svelte-i18n` — internationalization in landing components
- `addons` (workspace) — TMDB and torrent search type imports (use `addons/tmdb/...` and `addons/torrent-search-thepiratebay/...` paths)

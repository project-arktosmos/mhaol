# App: website

**Location:** `apps/website/`
**Type:** Thin SvelteKit 2 wrapper — marketing landing page
**Adapter:** `@sveltejs/adapter-static` (base path: `/mhaoltube`)

## Architecture

This app is an **assembly-only wrapper** around `packages/frontend`. It contains no components, services, adapters, types, or utils of its own. All shared code is imported from `packages/frontend` via the `frontend` workspace dependency and path aliases.

## What lives here

```
src/
├── routes/              # SvelteKit pages
│   ├── +layout.svelte   # App shell: LandingNavbar + theme init
│   ├── +layout.ts       # SSR disabled, i18n wait
│   └── +page.svelte     # Landing page (Hero, Features, Platforms, Footer)
├── css/app.css          # Tailwind entry + @source for packages/frontend + themes
├── app.html             # HTML template
└── app.d.ts             # SvelteKit type declarations
```

## Key features wired in layout

- **LandingNavbar**: From `frontend/components/landing/LandingNavbar.svelte` (brand + ThemeToggle)
- **Theme**: `themeService.initialize()` on mount for DOM `data-theme` management
- **No ModalOutlet**: This app has no modal-based navigation

## Import pattern

Component imports use `ui-lib/...` paths, services use `frontend/...`:

```typescript
import LandingNavbar from 'ui-lib/components/landing/LandingNavbar.svelte';
import { themeService } from 'frontend/services/theme.service';
```

## Adding features

To add UI features to this app, add the component to `packages/ui-lib` and the service/type to `packages/frontend`, then import and wire it in this app's route files.

# Package: frontend

**Location:** `packages/frontend/`
**Framework:** SvelteKit 2 + Svelte 5 + TailwindCSS v4 + DaisyUI v5
**Adapter:** `@sveltejs/adapter-static` (fully pre-rendered, fallback to `index.html`)
**Dev port:** 1531

## Source Structure

```
src/
├── adapters/classes/     # Data transformation logic
├── components/           # UI components organized by feature
│   ├── core/            # Shared reusable components (Button, Navbar, etc.)
│   ├── libraries/
│   ├── media/
│   ├── player/
│   ├── settings/
│   └── youtube/
├── routes/               # SvelteKit pages (+page.svelte, +layout.svelte)
├── services/
│   ├── classes/         # ArrayServiceClass, ObjectServiceClass
│   └── i18n/            # svelte-i18n locales
├── types/                # TypeScript type definitions
├── utils/                # Pure utility functions
├── data/                 # Static JSON data
├── css/                  # Tailwind v4 imports
└── lib/                  # SvelteKit utilities
```

## Path Aliases

These aliases are configured in both `svelte.config.js` and `vite.config.ts` and are **only valid inside `packages/frontend/`**:

```typescript
$components  → src/components/*
$services    → src/services/*
$adapters    → src/adapters/*
$utils       → src/utils/*
$types       → src/types/*
$data        → src/data/*
```

## Architecture: Separation of Concerns

```
┌─────────────────────────────────────────────────────────────┐
│                    Svelte Components                         │
│              (UI only — no business logic)                   │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│   Services    │ │   Adapters    │ │    Utils      │
│ (State/Data)  │ │(Transformers) │ │(Pure helpers) │
└───────────────┘ └───────────────┘ └───────────────┘
```

**Components** — render UI, handle user interactions, call callback props
**Services** — manage Svelte store state, persist to localStorage, call the backend API
**Adapters** — transform data between API format and internal format
**Utils** — pure functions, no side effects

---

## Services

Services manage shared state using Svelte stores with automatic localStorage persistence.

### ArrayServiceClass\<T\>

For collections of items with unique IDs:

```typescript
// src/services/myItems.service.ts
import { ArrayServiceClass } from '$services/classes/array-service.class';

interface MyItem {
	id: string;
	name: string;
	value: number;
}

export const myItemsService = new ArrayServiceClass<MyItem>('my-items', []);
```

**Methods:** `add(item)`, `remove(item)`, `update(item)`, `exists(id)`, `all()`, `find(predicate)`, `filter(predicate)`

**Using in a component:**

```svelte
<script lang="ts">
	import { myItemsService } from '$services/myItems.service';

	let items = $derived($myItemsService.store);

	function addItem() {
		myItemsService.add({ id: crypto.randomUUID(), name: 'New', value: 0 });
	}
</script>
```

### ObjectServiceClass\<T\>

For managing a single object:

```typescript
// src/services/settings.service.ts
import { ObjectServiceClass } from '$services/classes/object-service.class';

export const settingsService = new ObjectServiceClass<Settings>('settings', {
	id: 'user-settings',
	theme: 'light',
	language: 'en'
});
```

**localStorage keys:**

- Array services: `array-service:{id}`
- Object services: `object-service:{id}`

**SSR safety:** Both classes use `localStorageWritableStore`, which falls back to a plain writable store when `browser` is false.

---

## Adapters

Adapters transform data between external formats (API responses, file imports) and internal types. **All data transformation lives in adapters — never in components or services.**

**Existing adapters** in `src/adapters/classes/`:

- `adapter.class.ts` — base class
- `library-file.adapter.ts` — library file data
- `player.adapter.ts` — media player data

```typescript
// src/adapters/classes/example.adapter.ts
import { AdapterClass } from '$adapters/classes/adapter.class';

export class ExampleAdapter extends AdapterClass {
	constructor() {
		super('example');
	}

	fromApi(raw: ApiExample): Example {
		return { id: raw.example_id, name: raw.display_name };
	}

	toApi(item: Example): Partial<ApiExample> {
		return { display_name: item.name };
	}
}

export const exampleAdapter = new ExampleAdapter();
```

**Rules:**

1. Always export a singleton instance
2. Name methods clearly: `fromApi`, `toApi`, `toDisplayFormat`, etc.
3. No side effects — transformations only
4. Type both input and output

---

## Svelte Components (Svelte 5 Runes)

Components contain **only UI logic**. All business logic belongs in services and adapters. This project uses **Svelte 5 runes** — do not use legacy Svelte 4 patterns.

**Rules:**

1. No business logic — delegate to services/adapters
2. No `<style>` tags — use Tailwind classes only
3. No inline `style` attributes
4. Use `classnames` for all conditional class rendering
5. Type all props with inline type annotations on `$props()`
6. Use callback props for parent communication (e.g. `onClose`, `onSave`)
7. Keep components small — split when they grow

**Component template:**

```svelte
<script lang="ts">
	import classNames from 'classnames';
	import { ThemeColors, ThemeSizes } from '$types/core.type';
	import type { Snippet } from 'svelte';

	let {
		label = '',
		variant = ThemeColors.Primary,
		disabled = false,
		classes = '',
		onclick,
		children
	}: {
		label?: string;
		variant?: ThemeColors;
		disabled?: boolean;
		classes?: string;
		onclick?: () => void;
		children?: Snippet;
	} = $props();

	const variantClasses: Record<ThemeColors, string> = {
		[ThemeColors.Primary]: 'bg-primary text-primary-content',
		[ThemeColors.Secondary]: 'bg-secondary text-secondary-content'
	};

	let computedClasses = $derived(
		classNames(
			'btn',
			variantClasses[variant],
			{ 'opacity-50 cursor-not-allowed': disabled },
			classes
		)
	);
</script>

<button class={computedClasses} {disabled} {onclick}>
	{#if label}{label}{:else if children}{@render children()}{/if}
</button>
```

**Svelte 5 runes cheat sheet:**

```svelte
<script lang="ts">
	// Props — use $props() with inline types
	let {
		show,
		item,
		onSave,
		onClose
	}: {
		show: boolean;
		item: Item | null;
		onSave: (data: ItemPayload) => void;
		onClose: () => void;
	} = $props();

	// Reactive state — use $state()
	let loading = $state(false);
	let error = $state('');
	let items: Item[] = $state([]);

	// Derived values — use $derived()
	let isEditing = $derived(item !== null);
	let filteredItems = $derived(items.filter((i) => i.active));

	// Side effects — use $effect()
	$effect(() => {
		if (show) {
			loading = true;
			fetchData();
		}
	});
</script>

<!-- Event handlers — use direct attributes, not on: directives -->
<button onclick={handleClick}>Click</button>
<input oninput={(e) => (query = e.currentTarget.value)} />
```

**Naming conflict: `$state` rune vs store variables:**

Never name a store variable `state` (e.g. `const state = myService.state;`) if the component also uses `$state()` rune calls. Svelte interprets `$state(...)` as a store auto-subscription call on the `state` variable, not the rune. Use a prefixed name instead:

```svelte
<!-- BAD: conflicts with $state() rune -->
const state = youtubeService.state; let loading = $state(false); // ERROR: tries to call store value

<!-- GOOD: use a unique name -->
const ytState = youtubeService.state; let loading = $state(false); // works correctly
```

**`classnames` usage:**

```typescript
import classNames from 'classnames';

// Always-on strings
classNames('btn', 'flex', 'gap-2');

// Conditional object syntax
classNames({ 'bg-primary': isPrimary, 'opacity-50': disabled });

// Mixed — null/undefined safely ignored
classNames('btn', typeClasses[type], { 'btn-outline': outline }, customClasses);
```

---

## CSS & Styling

| Rule                      | Detail                      |
| ------------------------- | --------------------------- |
| NEVER use `<style>` tags  | Tailwind only               |
| NEVER use inline `style=` | Tailwind only               |
| ALWAYS use `classnames`   | For conditional classes     |
| Stack                     | TailwindCSS v4 + DaisyUI v5 |

**Theme enums** from `$types/core.type.ts`:

```typescript
ThemeColors.Primary; // 'primary'
ThemeColors.Secondary; // 'secondary'
ThemeColors.Accent; // 'accent'
ThemeColors.Success; // 'success'
ThemeColors.Error; // 'error'
ThemeColors.Info; // 'info'
ThemeColors.Warning; // 'warning'
ThemeColors.Neutral; // 'neutral'

ThemeSizes.XSmall; // 'xs'
ThemeSizes.Small; // 'sm'
ThemeSizes.Medium; // 'md'
ThemeSizes.Large; // 'lg'
ThemeSizes.XLarge; // 'xl'
```

**DaisyUI patterns:**

```html
<button class="btn btn-sm btn-primary">Save</button>
<div class="card bg-base-100 shadow-xl"><div class="card-body">...</div></div>
<input class="input-bordered input w-full input-primary" />
<span class="badge badge-success">Active</span>
```

---

## Types

Define shared types in `packages/frontend/src/types/`. One file per domain:

```
src/types/
├── core.type.ts          # ID, ThemeColors, ThemeSizes
├── library.type.ts
├── media-card.type.ts
├── media-detail.type.ts
├── media-list.type.ts
├── modal.type.ts
├── player.type.ts
├── youtube.type.ts
└── youtube-search.type.ts
```

Always use the `ID` type from `$types/core.type` for entity identifiers:

```typescript
import type { ID } from '$types/core.type';

interface Entity {
	id: ID; // string | number
}
```

---

## Utils

Pure functions with no side effects. Organize by domain:

```
src/utils/
├── localStorageWritableStore.ts
├── string/
└── youtube.ts
```

```typescript
import { capitalize } from '$utils/string/capitalize';
import { normalize } from '$utils/string/normalize';
```

---

## i18n

Use `svelte-i18n` for translations. Translations are in `src/services/i18n/locales/`.

```svelte
<script lang="ts">
	import { _ } from 'svelte-i18n';
</script>

<h1>{$_('common.welcome')}</h1>
```

---

## Testing

Tests live in `packages/frontend/test/` mirroring `src/`:

```
test/
├── services/
├── adapters/
├── utils/
└── components/
```

```bash
pnpm test             # vitest
pnpm test:ui          # interactive UI
pnpm test:coverage    # coverage report
pnpm test:e2e         # Playwright
pnpm test:e2e:headed  # Playwright with visible browser
```

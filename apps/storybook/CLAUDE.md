# App: storybook

**Location:** `apps/storybook/`
**Type:** Storybook component gallery (@storybook/svelte-vite)
**Port:** 1550 (configured in `.env`)

## Purpose

Renders every Svelte component from `packages/ui-lib` in isolation using Storybook with CSF3 TypeScript stories. Provides controls, docs, and theme support.

## Structure

```
.storybook/
├── main.ts              # Framework config, viteFinal aliases, story globs
└── preview.ts           # Global CSS import, parameter defaults
src/
├── storybook.css        # Tailwind + DaisyUI + themes entry point
├── mocks/               # SvelteKit virtual module mocks ($app/environment, $app/paths)
└── stories/             # One .stories.ts per component, organized by category
    ├── core/
    ├── media/
    ├── torrent/
    ├── youtube/
    └── ...
```

## Adding stories for new components

When creating a new component in `packages/ui-lib/src/components/{category}/`:

1. Create `apps/storybook/src/stories/{category}/{ComponentName}.stories.ts`
2. Use CSF3 TypeScript format:

```typescript
import type { Meta, StoryObj } from '@storybook/svelte';
import MyComponent from 'ui-lib/components/{category}/MyComponent.svelte';

const meta = {
  title: '{Category}/MyComponent',
  component: MyComponent,
  tags: ['autodocs'],
  argTypes: { /* controls */ }
} satisfies Meta<typeof MyComponent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { /* props */ } };
export const Variant: Story = { args: { /* different props */ } };
```

## Running

```bash
pnpm app:storybook          # Browser dev server on port 1550
```

# Skill: New Frontend Component

Scaffold a new Svelte 5 component following project conventions.

## Instructions

1. Ask the user for:
   - Component name (e.g. "MediaCard")
   - Feature area (e.g. "media", "player", "core")
   - What props it needs

2. **Create the component** at `packages/frontend/src/components/{area}/{ComponentName}.svelte`:

```svelte
<script lang="ts">
	import classNames from 'classnames';

	let {
		myProp = '',
		classes = '',
		onclick
	}: {
		myProp?: string;
		classes?: string;
		onclick?: () => void;
	} = $props();

	let computedClasses = $derived(classNames('base-classes', classes));
</script>

<div class={computedClasses}>
	{myProp}
</div>
```

3. **Mandatory rules**:
   - No `<style>` tags — use Tailwind classes only
   - No inline `style` attributes
   - Use `classnames` for all conditional class rendering
   - Type all props with inline type annotations on `$props()`
   - Use callback props for parent communication (e.g. `onClose`, `onSave`, `onclick`)
   - Use `$state()` for local reactive state, `$derived()` for computed values, `$effect()` for side effects
   - No business logic — delegate to services/adapters
   - Use path aliases: `$components`, `$services`, `$adapters`, `$utils`, `$types`, `$data`
   - Use DaisyUI components where appropriate (`btn`, `card`, `input`, `badge`, etc.)
   - Use `ThemeColors` and `ThemeSizes` enums from `$types/core.type`

4. **Verify**: Run `pnpm lint && pnpm check && pnpm build && pnpm test`

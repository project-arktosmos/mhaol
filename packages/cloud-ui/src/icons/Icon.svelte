<script lang="ts">
	import type { IconName } from './icon-names.js';

	interface Props {
		name: IconName | string;
		size?: number | string;
		title?: string | null;
		class?: string;
	}

	let { name, size = '1em', title = null, class: className = '' }: Props = $props();

	const modules = import.meta.glob<string>('./assets/**/*.svg', {
		query: '?raw',
		import: 'default'
	});

	let svg = $state<string | null>(null);

	$effect(() => {
		const target = name;
		const path = `./assets/${target}.svg`;
		const loader = modules[path];
		if (!loader) {
			svg = null;
			return;
		}

		let cancelled = false;
		loader().then((value) => {
			if (cancelled) return;
			svg = value;
		});
		return () => {
			cancelled = true;
		};
	});

	let dimension = $derived(typeof size === 'number' ? `${size}px` : size);
</script>

<span
	class={`mhaol-icon ${className}`.trim()}
	style={`width:${dimension};height:${dimension}`}
	role={title ? 'img' : 'presentation'}
	aria-label={title ?? undefined}
	aria-hidden={title ? undefined : 'true'}
>
	{#if svg}
		{@html svg}
	{/if}
</span>

<style>
	.mhaol-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: inherit;
		line-height: 1;
		vertical-align: -0.125em;
	}

	.mhaol-icon :global(svg) {
		width: 100%;
		height: 100%;
		display: block;
		fill: currentColor;
	}
</style>

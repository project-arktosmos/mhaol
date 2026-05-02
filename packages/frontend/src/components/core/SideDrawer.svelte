<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		width = 'w-72',
		sidebar,
		children
	}: {
		open?: boolean;
		width?: string;
		sidebar: Snippet;
		children: Snippet;
	} = $props();

	const drawerId = `drawer-${Math.random().toString(36).slice(2, 10)}`;
</script>

<div class="drawer">
	<input id={drawerId} type="checkbox" class="drawer-toggle" bind:checked={open} />
	<div class="drawer-content">
		{@render children()}
	</div>
	<div class="drawer-side z-50">
		<label for={drawerId} class="drawer-overlay" aria-label="close sidebar"></label>
		<nav class={classNames('min-h-full bg-base-200 p-4', width)}>
			{@render sidebar()}
		</nav>
	</div>
</div>

<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		actions?: Snippet;
	}

	let { title, actions }: Props = $props();
</script>

<header class="flex flex-wrap items-start justify-between gap-3">
	<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{title}</h1>
	{#if actions}
		<div class="flex items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</header>

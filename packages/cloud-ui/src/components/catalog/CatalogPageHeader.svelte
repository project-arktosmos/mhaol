<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		year?: number | null;
		kindLabel?: string | null;
		actions?: Snippet;
	}

	let { title, year = null, kindLabel = null, actions }: Props = $props();

	const showYear = $derived(year !== null && year !== undefined && Number.isFinite(year));
</script>

<header class="flex flex-wrap items-start justify-between gap-3">
	<div class="flex flex-col gap-0.5">
		<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">
			{title}{#if showYear}&nbsp;<span class="opacity-80">{year}</span>{/if}
		</h1>
		{#if kindLabel}
			<p class="text-sm text-base-content/70 italic">{kindLabel}</p>
		{/if}
	</div>
	{#if actions}
		<div class="flex items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</header>

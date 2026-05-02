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
	const subline = $derived(
		[kindLabel, showYear ? String(year) : null].filter((s): s is string => Boolean(s)).join(' · ')
	);
</script>

<header class="flex flex-wrap items-start justify-between gap-3">
	<div class="flex flex-col gap-0.5">
		<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{title}</h1>
		{#if subline}
			<p class="text-sm text-base-content/70 italic">{subline}</p>
		{/if}
	</div>
	{#if actions}
		<div class="flex items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</header>

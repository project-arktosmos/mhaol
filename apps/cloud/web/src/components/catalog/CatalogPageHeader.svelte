<script lang="ts">
	import type { Snippet } from 'svelte';
	import { base } from '$app/paths';

	interface Props {
		title: string;
		addon: string;
		kindLabel?: string | null;
		year?: number | null;
		extraBadge?: { label: string; class: string } | null;
		backHref?: string;
		backLabel?: string;
		actions?: Snippet;
	}

	let {
		title,
		addon,
		kindLabel = null,
		year = null,
		extraBadge = null,
		backHref = `${base}/catalog`,
		backLabel = 'Catalog',
		actions
	}: Props = $props();
</script>

<header class="flex flex-wrap items-start justify-between gap-3">
	<div class="flex flex-col gap-1">
		<a class="text-xs text-base-content/60 hover:underline" href={backHref}>← {backLabel}</a>
		<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{title}</h1>
		<p class="text-sm text-base-content/70">
			<span class="badge badge-outline badge-sm">{addon}</span>
			{#if kindLabel}
				<span class="badge badge-outline badge-sm">{kindLabel}</span>
			{/if}
			{#if year !== null && year !== undefined && Number.isFinite(year)}
				<span class="badge badge-outline badge-sm">{year}</span>
			{/if}
			{#if extraBadge}
				<span class={`badge badge-sm ${extraBadge.class}`}>{extraBadge.label}</span>
			{/if}
		</p>
	</div>
	{#if actions}
		<div class="flex items-center gap-2">
			{@render actions()}
		</div>
	{/if}
</header>

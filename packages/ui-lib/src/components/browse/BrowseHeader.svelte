<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		count?: number | null;
		countLabel?: string;
		classes?: string;
		controls?: Snippet;
		tabs?: Snippet;
	}

	let {
		title,
		count = null,
		countLabel = 'items',
		classes = '',
		controls,
		tabs
	}: Props = $props();

	let headerClasses = $derived(
		classNames(
			'flex flex-wrap items-center gap-3 border-b border-base-300 px-4 py-3',
			classes
		)
	);
</script>

<div class={headerClasses}>
	<h2 class="text-lg font-bold">{title}</h2>
	{#if count !== null}
		<span class="badge badge-ghost">{count} {countLabel}</span>
	{/if}
	{#if controls}
		{@render controls()}
	{/if}
</div>
{#if tabs}
	<div class="flex flex-wrap gap-1.5 border-b border-base-300 px-4 py-2">
		{@render tabs()}
	</div>
{/if}

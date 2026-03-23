<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';

	interface Props {
		items: unknown[];
		loading?: boolean;
		error?: string | null;
		emptyTitle?: string;
		emptySubtitle?: string;
		gridClasses?: string;
		classes?: string;
		card: Snippet<[unknown, number]>;
		onretry?: () => void;
		page?: number;
		totalPages?: number;
		onpage?: (page: number) => void;
	}

	let {
		items,
		loading = false,
		error = null,
		emptyTitle = 'No items found',
		emptySubtitle = '',
		gridClasses = '',
		classes = '',
		card,
		onretry,
		page,
		totalPages,
		onpage
	}: Props = $props();

	let containerClasses = $derived(classNames('flex-1 overflow-y-auto p-4', classes));
	let gridCombinedClasses = $derived(
		classNames(
			'grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6',
			gridClasses
		)
	);
	let hasPagination = $derived(
		page !== undefined && totalPages !== undefined && onpage !== undefined && totalPages > 1
	);
</script>

<div class={containerClasses}>
	{#if loading}
		<div class="flex items-center justify-center py-16">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if error}
		<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
			<p class="text-lg">Failed to load</p>
			<p class="mt-1 text-sm">{error}</p>
			{#if onretry}
				<button class="btn mt-4 btn-sm btn-primary" onclick={onretry}>Retry</button>
			{/if}
		</div>
	{:else if items.length === 0}
		<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
			<p class="text-lg">{emptyTitle}</p>
			{#if emptySubtitle}
				<p class="mt-1 text-sm">{emptySubtitle}</p>
			{/if}
		</div>
	{:else}
		<div class={gridCombinedClasses}>
			{#each items as item, index (index)}
				{@render card(item, index)}
			{/each}
		</div>
		{#if hasPagination}
			<div class="mt-4 flex items-center justify-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					disabled={page === 0}
					onclick={() => onpage!(page! - 1)}
				>
					Prev
				</button>
				<span class="text-sm opacity-60">
					{page! + 1} / {totalPages}
				</span>
				<button
					class="btn btn-ghost btn-sm"
					disabled={page! >= totalPages! - 1}
					onclick={() => onpage!(page! + 1)}
				>
					Next
				</button>
			</div>
		{/if}
	{/if}
</div>

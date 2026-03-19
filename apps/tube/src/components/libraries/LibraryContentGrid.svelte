<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryCardItem } from '$types/library.type';
	import type { YouTubeDownloadProgress } from '$types/youtube.type';
	import LibraryContentCard from './LibraryContentCard.svelte';

	let {
		title,
		items,
		activeDownloadMap,
		onitemclick,
		onloadmore,
		loadingMore = false,
		classes = ''
	}: {
		title: string;
		items: LibraryCardItem[];
		activeDownloadMap: Map<string, YouTubeDownloadProgress>;
		onitemclick: (item: LibraryCardItem) => void;
		onloadmore?: () => void;
		loadingMore?: boolean;
		classes?: string;
	} = $props();

	const ROWS_INCREMENT = 3;

	let gridEl: HTMLDivElement | undefined = $state();
	let columns = $state(6);
	let visibleRows = $state(1);

	let effectiveRows = $derived(columns <= 2 ? Math.max(visibleRows, 3) : visibleRows);
	let maxVisibleSlots = $derived(columns * effectiveRows);
	let hasMore = $derived(items.length > maxVisibleSlots);
	let isExpanded = $derived(effectiveRows > (columns <= 2 ? 3 : 1));
	let visibleItems = $derived(
		hasMore ? items.slice(0, maxVisibleSlots - 1) : items.slice(0, maxVisibleSlots)
	);
	let remaining = $derived(items.length - visibleItems.length);

	let containerClasses = $derived(classNames('mb-4', classes));

	$effect(() => {
		if (!gridEl) return;

		function updateColumns() {
			if (!gridEl) return;
			const style = getComputedStyle(gridEl);
			const cols = style.getPropertyValue('grid-template-columns').split(' ').length;
			columns = cols;
		}

		const observer = new ResizeObserver(updateColumns);
		observer.observe(gridEl);
		updateColumns();

		return () => observer.disconnect();
	});

	function showMore() {
		visibleRows += ROWS_INCREMENT;
	}

	function showLess() {
		visibleRows = columns <= 2 ? 3 : 1;
	}
</script>

<div class={containerClasses}>
	<h2 class="sticky top-0 z-10 mb-4 rounded-lg bg-base-100 px-3 py-2 text-xl font-semibold">
		{title}
	</h2>
	<div
		bind:this={gridEl}
		class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
	>
		{#each visibleItems as item (item.videoId)}
			<LibraryContentCard
				{item}
				download={activeDownloadMap.get(item.videoId)}
				onclick={() => onitemclick(item)}
			/>
		{/each}
		{#if hasMore}
			<button
				class="flex min-h-24 items-center justify-center rounded-lg border border-base-300 text-sm opacity-70 transition-opacity hover:opacity-100"
				onclick={showMore}
			>
				+{remaining} more
			</button>
		{:else if onloadmore}
			<button
				class="flex min-h-24 items-center justify-center rounded-lg border border-base-300 text-sm opacity-70 transition-opacity hover:opacity-100"
				onclick={onloadmore}
				disabled={loadingMore}
			>
				{#if loadingMore}
					<span class="loading loading-sm loading-spinner"></span>
				{:else}
					Load more
				{/if}
			</button>
		{:else if isExpanded}
			<button
				class="flex min-h-24 items-center justify-center rounded-lg border border-base-300 text-sm opacity-70 transition-opacity hover:opacity-100"
				onclick={showLess}
			>
				Show less
			</button>
		{/if}
	</div>
</div>

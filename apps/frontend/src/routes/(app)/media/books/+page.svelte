<script lang="ts">
	import { onMount, getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import classNames from 'classnames';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import Portal from 'ui-lib/components/core/Portal.svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import BookRecommendationsModalContent from 'ui-lib/components/recommendations/BookRecommendationsModalContent.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import { bookStrategy } from 'ui-lib/services/catalog-strategies/book.strategy';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { isBook } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';

	const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);
	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	let recsModalOpen = $state(false);

	let pinnedBookKeys = $derived(
		$pins.items.filter((p) => p.service === 'openlibrary').map((p) => p.serviceId)
	);
	let favoritedBookKeys = $derived(
		$favs.items.filter((f) => f.service === 'openlibrary').map((f) => f.serviceId)
	);

	onMount(() => {
		catalogService.registerStrategy(bookStrategy);
		catalogService.activate('book');
	});

	function handleSelectItem(item: CatalogItem) {
		if (isBook(item)) {
			goto(`${base}/media/books/${item.metadata.openlibraryKey}`);
		}
	}

	function cardOverlays(item: CatalogItem) {
		return {
			favorited: $favs.items.some(
				(f) => f.service === 'openlibrary' && f.serviceId === item.sourceId
			),
			pinned: $pins.items.some(
				(p) => p.service === 'openlibrary' && p.serviceId === item.sourceId
			)
		};
	}
</script>

<Portal target={mediaBar.controlsTarget}>
	<button class="btn btn-ghost btn-sm" onclick={() => (recsModalOpen = true)}>Recs</button>
</Portal>

<CatalogBrowsePage
	browseState={$browseState}
	title="Books"
	strategy={bookStrategy}
	{cardOverlays}
	onsearch={(q) => catalogService.search(q)}
	ontabchange={(tab) => catalogService.loadTab(tab)}
	onpagechange={(p) => catalogService.loadPage(p)}
	onselectitem={handleSelectItem}
>
	{#snippet filterBar()}
		{#if $browseState.filterOptions.subject}
			<div class="flex flex-wrap gap-1">
				{#each $browseState.filterOptions.subject as option}
					<button
						class={classNames('btn btn-xs', {
							'btn-primary': $browseState.filters.subject === option.id,
							'btn-ghost': $browseState.filters.subject !== option.id
						})}
						onclick={() => catalogService.setFilter('subject', option.id)}
					>
						{option.label}
					</button>
				{/each}
			</div>
		{/if}
	{/snippet}
</CatalogBrowsePage>

<Modal open={recsModalOpen} maxWidth="max-w-[90vw]" onclose={() => (recsModalOpen = false)}>
	{#if recsModalOpen}
		<div class="p-4">
			<BookRecommendationsModalContent {pinnedBookKeys} {favoritedBookKeys} />
		</div>
	{/if}
</Modal>

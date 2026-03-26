<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import classNames from 'classnames';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import { bookStrategy } from 'ui-lib/services/catalog-strategies/book.strategy';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { isBook } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

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

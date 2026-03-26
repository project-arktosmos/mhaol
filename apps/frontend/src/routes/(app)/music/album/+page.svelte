<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import classNames from 'classnames';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import { albumStrategy } from 'ui-lib/services/catalog-strategies/album.strategy';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { isAlbum } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	onMount(() => {
		catalogService.registerStrategy(albumStrategy);
		catalogService.activate('album');
	});

	function handleSelectItem(item: CatalogItem) {
		if (isAlbum(item)) {
			goto(`${base}/music/album/${item.sourceId}`);
		}
	}

	function cardOverlays(item: CatalogItem) {
		return {
			favorited: $favs.items.some(
				(f) => f.service === 'musicbrainz-album' && f.serviceId === item.sourceId
			),
			pinned: $pins.items.some(
				(p) => p.service === 'musicbrainz-album' && p.serviceId === item.sourceId
			)
		};
	}
</script>

<CatalogBrowsePage
	browseState={$browseState}
	title="Albums"
	strategy={albumStrategy}
	{cardOverlays}
	onsearch={(q) => catalogService.search(q)}
	ontabchange={(tab) => catalogService.loadTab(tab)}
	onpagechange={(p) => catalogService.loadPage(p)}
	onselectitem={handleSelectItem}
>
	{#snippet filterBar()}
		{#if $browseState.filterOptions.genre}
			<div class="flex flex-wrap gap-1">
				{#each $browseState.filterOptions.genre as option}
					<button
						class={classNames('btn btn-xs', {
							'btn-primary': ($browseState.filters.genre || 'rock') === option.id,
							'btn-ghost': ($browseState.filters.genre || 'rock') !== option.id
						})}
						onclick={() => catalogService.setFilter('genre', option.id)}
					>
						{option.label}
					</button>
				{/each}
			</div>
		{/if}
	{/snippet}
</CatalogBrowsePage>

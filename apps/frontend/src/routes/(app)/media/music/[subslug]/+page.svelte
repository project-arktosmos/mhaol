<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import classNames from 'classnames';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { isAlbum, isArtist } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { MediaTypeConfig } from 'ui-lib/data/media-registry';
	import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';

	import { albumStrategy } from 'ui-lib/services/catalog-strategies/album.strategy';
	import { artistStrategy } from 'ui-lib/services/catalog-strategies/artist.strategy';

	interface Props {
		data: { config: MediaTypeConfig };
	}

	let { data }: Props = $props();
	const config = data.config;

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	const strategyMap: Record<string, CatalogKindStrategy> = {
		album: albumStrategy,
		artist: artistStrategy
	};

	function getStrategy(): CatalogKindStrategy {
		return strategyMap[config.kind] ?? albumStrategy;
	}

	onMount(() => {
		const strategy = getStrategy();
		catalogService.registerStrategy(strategy);
		catalogService.activate(config.kind);
	});

	function handleSelectItem(item: CatalogItem) {
		const id = config.selectItemId(item);
		goto(`${base}/media/music/${config.slug}/${id}`);
	}

	function cardOverlays(item: CatalogItem) {
		return {
			favorited: $favs.items.some(
				(f) => f.service === config.favService && f.serviceId === item.sourceId
			),
			pinned: $pins.items.some(
				(p) => p.service === config.pinService && p.serviceId === item.sourceId
			)
		};
	}
</script>

<CatalogBrowsePage
	browseState={$browseState}
	title={config.label}
	strategy={getStrategy()}
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

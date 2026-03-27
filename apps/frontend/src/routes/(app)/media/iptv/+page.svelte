<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import classNames from 'classnames';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import { iptvStrategy } from 'ui-lib/services/catalog-strategies/iptv.strategy';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { isIptvChannel } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	onMount(() => {
		catalogService.registerStrategy(iptvStrategy);
		catalogService.activate('iptv_channel');
	});

	function handleSelectItem(item: CatalogItem) {
		if (isIptvChannel(item)) {
			goto(`${base}/media/iptv/${encodeURIComponent(item.sourceId)}`);
		}
	}

	function cardOverlays(item: CatalogItem) {
		return {
			favorited: $favs.items.some(
				(f) => f.service === 'iptv' && f.serviceId === item.sourceId
			),
			pinned: $pins.items.some(
				(p) => p.service === 'iptv' && p.serviceId === item.sourceId
			)
		};
	}
</script>

<CatalogBrowsePage
	browseState={$browseState}
	title="IPTV"
	strategy={iptvStrategy}
	{cardOverlays}
	onsearch={(q) => catalogService.search(q)}
	ontabchange={(tab) => catalogService.loadTab(tab)}
	onpagechange={(p) => catalogService.loadPage(p)}
	onselectitem={handleSelectItem}
>
	{#snippet filterBar()}
		{#if $browseState.filterOptions.category || $browseState.filterOptions.country}
			<div class="flex flex-wrap items-center gap-2">
				{#if $browseState.filterOptions.category}
					<select
						class="select select-bordered select-xs"
						value={$browseState.filters.category ?? ''}
						onchange={(e) => catalogService.setFilter('category', (e.target as HTMLSelectElement).value)}
					>
						{#each $browseState.filterOptions.category as option}
							<option value={option.id}>{option.label}</option>
						{/each}
					</select>
				{/if}
				{#if $browseState.filterOptions.country}
					<select
						class="select select-bordered select-xs"
						value={$browseState.filters.country ?? ''}
						onchange={(e) => catalogService.setFilter('country', (e.target as HTMLSelectElement).value)}
					>
						{#each $browseState.filterOptions.country as option}
							<option value={option.id}>{option.label}</option>
						{/each}
					</select>
				{/if}
			</div>
		{/if}
	{/snippet}
</CatalogBrowsePage>

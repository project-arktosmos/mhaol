<script lang="ts">
	import { pinsService } from '$services/pins.service';
	import { favoritesService } from '$services/favorites.service';
	import { catalogItemToCardData } from '$adapters/classes/catalog-card.adapter';
	import CatalogCard from './CatalogCard.svelte';
	import type { CatalogKindStrategy } from '$services/catalog.service';
	import type { CatalogItem, CatalogCardData } from '$types/catalog.type';

	interface Props {
		strategy: CatalogKindStrategy;
		cardOverlays?: (item: CatalogItem) => Partial<CatalogCardData>;
		itemHref?: (item: CatalogItem) => string;
		onselectitem: (item: CatalogItem) => void;
	}

	let { strategy, cardOverlays, itemHref, onselectitem }: Props = $props();

	const pinState = pinsService.state;
	const favState = favoritesService.state;

	let pinnedItems = $state<CatalogItem[]>([]);
	let favoriteItems = $state<CatalogItem[]>([]);
	let pinnedLoading = $state(false);
	let favoritesLoading = $state(false);

	let pinnedIds = $derived(
		$pinState.items.filter((p) => p.service === strategy.pinService).map((p) => p.serviceId)
	);

	let favoriteIds = $derived(
		$favState.items.filter((f) => f.service === strategy.pinService).map((f) => f.serviceId)
	);

	$effect(() => {
		const ids = pinnedIds;
		if (!strategy.resolveByIds || ids.length === 0) {
			pinnedItems = [];
			return;
		}
		pinnedLoading = true;
		strategy
			.resolveByIds(ids)
			.then((items) => {
				pinnedItems = items;
				pinnedLoading = false;
			})
			.catch(() => {
				pinnedLoading = false;
			});
	});

	$effect(() => {
		const ids = favoriteIds;
		if (!strategy.resolveByIds || ids.length === 0) {
			favoriteItems = [];
			return;
		}
		favoritesLoading = true;
		strategy
			.resolveByIds(ids)
			.then((items) => {
				favoriteItems = items;
				favoritesLoading = false;
			})
			.catch(() => {
				favoritesLoading = false;
			});
	});

	function cardDataFor(item: CatalogItem): CatalogCardData {
		const base = catalogItemToCardData(item);
		if (cardOverlays) {
			return { ...base, ...cardOverlays(item) };
		}
		return base;
	}
</script>

{#if pinnedLoading || pinnedItems.length > 0}
	<section class="mb-6 px-4">
		<h2 class="mb-3 text-lg font-semibold">Pinned</h2>
		{#if pinnedLoading}
			<div class="flex justify-center py-6">
				<span class="loading loading-sm loading-spinner"></span>
			</div>
		{:else}
			<div
				class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each pinnedItems as item (item.id)}
					<CatalogCard
						card={cardDataFor(item)}
						href={itemHref?.(item)}
						onclick={() => onselectitem(item)}
					/>
				{/each}
			</div>
		{/if}
	</section>
{/if}

{#if favoritesLoading || favoriteItems.length > 0}
	<section class="mb-6 px-4">
		<h2 class="mb-3 text-lg font-semibold">Favorites</h2>
		{#if favoritesLoading}
			<div class="flex justify-center py-6">
				<span class="loading loading-sm loading-spinner"></span>
			</div>
		{:else}
			<div
				class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each favoriteItems as item (item.id)}
					<CatalogCard
						card={cardDataFor(item)}
						href={itemHref?.(item)}
						onclick={() => onselectitem(item)}
					/>
				{/each}
			</div>
		{/if}
	</section>
{/if}

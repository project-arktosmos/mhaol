<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import CatalogBrowsePage from 'ui-lib/components/catalog/CatalogBrowsePage.svelte';
	import CatalogFilterBar from 'ui-lib/components/catalog/filters/CatalogFilterBar.svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import { catalogService } from 'ui-lib/services/catalog.service';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { MediaTypeConfig } from 'ui-lib/data/media-registry';

	// Strategies — imported eagerly, registered on mount based on config
	import { movieStrategy } from 'ui-lib/services/catalog-strategies/movie.strategy';
	import { tvStrategy } from 'ui-lib/services/catalog-strategies/tv.strategy';
	import { bookStrategy } from 'ui-lib/services/catalog-strategies/book.strategy';
	import { gameStrategy } from 'ui-lib/services/catalog-strategies/game.strategy';
	import { iptvStrategy } from 'ui-lib/services/catalog-strategies/iptv.strategy';

	// Recommendations
	import RecommendationsModalContent from 'ui-lib/components/recommendations/RecommendationsModalContent.svelte';
	import BookRecommendationsModalContent from 'ui-lib/components/recommendations/BookRecommendationsModalContent.svelte';
	import GameRecommendationsModalContent from 'ui-lib/components/recommendations/GameRecommendationsModalContent.svelte';

	// Console selector data (for videogames)
	import { RA_CONSOLES, CONSOLE_WASM_STATUS } from 'addons/retroachievements/types';
	import { CONSOLE_IMAGES } from 'assets/game-consoles';

	import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';
	import type { CatalogKind } from 'ui-lib/types/catalog.type';

	interface Props {
		data: {
			config: MediaTypeConfig;
			mediaData?: Record<string, unknown>;
		};
	}

	let { data }: Props = $props();
	const config = data.config;

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	let recsModalOpen = $state(false);
	let selectedConsoleId = $state(5);

	const strategyMap: Record<string, CatalogKindStrategy> = {
		movie: movieStrategy,
		tv_show: tvStrategy,
		book: bookStrategy,
		game: gameStrategy,
		iptv_channel: iptvStrategy
	};

	function getStrategy(kind: CatalogKind): CatalogKindStrategy {
		return strategyMap[kind];
	}

	onMount(() => {
		const strategy = getStrategy(config.kind);
		if (!strategy) return;
		catalogService.registerStrategy(strategy);
		catalogService.activate(config.kind);
		if (config.kind === 'game') {
			catalogService.setFilter('console', String(selectedConsoleId));
		}
	});

	function handleSelectItem(item: CatalogItem) {
		const id = config.selectItemId(item);
		const encodedId = config.encodeId ? encodeURIComponent(id) : id;
		goto(`${base}/media/${config.slug}/${encodedId}`);
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

	function handleConsoleChange(consoleId: number) {
		selectedConsoleId = consoleId;
		catalogService.setFilter('console', String(consoleId));
	}

	let pinnedIds = $derived(
		$pins.items.filter((p) => p.service === config.pinService).map((p) => p.serviceId)
	);
	let favoritedIds = $derived(
		$favs.items.filter((f) => f.service === config.favService).map((f) => f.serviceId)
	);
</script>

<CatalogBrowsePage
	browseState={$browseState}
	title={config.label}
	strategy={getStrategy(config.kind)}
	{cardOverlays}
	onsearch={(q) => catalogService.search(q)}
	ontabchange={(tab) => catalogService.loadTab(tab)}
	onpagechange={(p) => catalogService.loadPage(p)}
	onselectitem={handleSelectItem}
>
	{#snippet extraControls()}
		{#if config.hasRecs}
			<button class="btn btn-ghost btn-sm" onclick={() => (recsModalOpen = true)}>Recs</button>
		{/if}
	{/snippet}

	{#snippet filterBar()}
		<CatalogFilterBar
			filterKind={config.filterKind}
			browseState={$browseState}
			onfilter={(id, value) => catalogService.setFilter(id, value)}
			consoles={RA_CONSOLES}
			consoleWasmStatus={CONSOLE_WASM_STATUS}
			consoleImages={CONSOLE_IMAGES}
			{selectedConsoleId}
			onconsolechange={handleConsoleChange}
		/>
	{/snippet}
</CatalogBrowsePage>

{#if config.hasRecs}
	<Modal open={recsModalOpen} maxWidth="max-w-[90vw]" onclose={() => (recsModalOpen = false)}>
		{#if recsModalOpen}
			<div class="p-4">
				{#if config.recsMediaType === 'movie' || config.recsMediaType === 'tv'}
					<RecommendationsModalContent
						mediaType={config.recsMediaType}
						pinnedIds={pinnedIds.map(Number)}
						favoritedIds={favoritedIds.map(Number)}
						libraryTmdbIds={[]}
					/>
				{:else if config.recsMediaType === 'book'}
					<BookRecommendationsModalContent
						pinnedBookKeys={pinnedIds}
						favoritedBookKeys={favoritedIds}
					/>
				{:else if config.recsMediaType === 'game'}
					<GameRecommendationsModalContent
						pinnedGameIds={pinnedIds.map(Number)}
						favoritedGameIds={favoritedIds.map(Number)}
					/>
				{/if}
			</div>
		{/if}
	</Modal>
{/if}

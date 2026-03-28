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

	// Movie/TV library sections
	import MovieLibrarySection from 'ui-lib/components/catalog/MovieLibrarySection.svelte';
	import TvLibrarySection from 'ui-lib/components/catalog/TvLibrarySection.svelte';
	import BrowseViewToggle from 'ui-lib/components/browse/BrowseViewToggle.svelte';
	import { fetchCacheService } from 'ui-lib/services/fetch-cache.service';
	import { imageOverridesService } from 'ui-lib/services/image-overrides.service';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';

	import type { CatalogKindStrategy } from 'ui-lib/services/catalog.service';
	import type { CatalogKind } from 'ui-lib/types/catalog.type';

	interface Props {
		data: {
			config: MediaTypeConfig;
			mediaData?: {
				itemsByType?: Record<string, MediaItem[]>;
				libraries?: Record<string, { name: string; type: string }>;
				lists?: MediaList[];
				error?: string;
			};
		};
	}

	let { data }: Props = $props();
	const config = data.config;

	const browseState = catalogService.state;
	const favs = favoritesService.state;
	const pins = pinsService.state;

	let recsModalOpen = $state(false);
	let selectedConsoleId = $state(5);

	// Movie/TV-specific state
	const fetchCacheState = fetchCacheService.state;
	const imageOverridesState = imageOverridesService.state;
	const torrentState = torrentService.state;
	const searchStore = smartSearchService.store;
	let smartSearchingId: number | null = $state(null);
	let batchSearching = $state(false);
	let batchProgress = $state({ current: 0, total: 0 });

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
		if (config.features.fetchCache) {
			fetchCacheService.load(config.pinService === 'tmdb' ? 'tmdb' : config.pinService);
		}
		if (config.features.imageOverrides) {
			imageOverridesService.load(config.features.imageOverrides);
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

	// Movie/TV browse overlay data
	let browseDownloadStatuses = $derived.by(() => {
		if (!config.features.fetchCache) return new Map();
		const torrents = $torrentState.allTorrents;
		const hashes = $fetchCacheState.hashes;
		if (torrents.length === 0 || hashes.size === 0) return new Map();
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const statuses = new Map<number, { state: string; progress: number }>();
		for (const [tmdbId, infoHash] of hashes) {
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) statuses.set(tmdbId, { state: torrent.state, progress: torrent.progress });
		}
		return statuses;
	});

	let favoritedTmdbIds = $derived(
		new Set(
			$favs.items
				.filter((f) => f.service === config.favService)
				.map((f) => Number(f.serviceId))
		)
	);
	let pinnedTmdbIds = $derived(
		new Set(
			$pins.items
				.filter((p) => p.service === config.pinService)
				.map((p) => Number(p.serviceId))
		)
	);

	// Movie smart search handler
	async function handleSmartSearch(movie: DisplayTMDBMovie) {
		smartSearchingId = movie.id;
		try {
			await smartSearchService.selectAndWaitForBest({
				title: movie.title,
				year: movie.releaseYear,
				type: 'movie',
				tmdbId: movie.id,
				mode: 'fetch'
			});
			await fetchCacheService.refresh();
		} finally {
			smartSearchingId = null;
		}
	}

	async function handleBatchSmartSearch() {
		// Batch search pinned movies that haven't been fetched yet
		// This reads from the PinnedFavoritesSection's resolved items
		// For now this is a no-op until we wire up resolved pinned items
	}

	// TV library data
	let tvLists = $derived((data.mediaData?.lists ?? []) as MediaList[]);
	let tvLibraries = $derived(
		(data.mediaData?.libraries ?? {}) as Record<string, { name: string; type: string }>
	);
	let favoritedTmdbTvIds = $derived(
		new Set(
			$favs.items
				.filter((f) => f.service === 'tmdb-tv')
				.map((f) => Number(f.serviceId))
		)
	);
	let pinnedTmdbTvIds = $derived(
		new Set(
			$pins.items
				.filter((p) => p.service === 'tmdb-tv')
				.map((p) => Number(p.serviceId))
		)
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
		{#if config.features.libraryItems}
			<BrowseViewToggle />
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

	{#snippet extraSections()}
		{#if config.features.libraryItems === 'movie' && data.mediaData}
			<MovieLibrarySection
				mediaData={{
					itemsByType: data.mediaData.itemsByType ?? {},
					libraries: (data.mediaData.libraries ?? {}) as Record<string, { name: string; type: string }>
				}}
				imageOverrides={$imageOverridesState}
				fetchCachedIds={$fetchCacheState.cachedIds}
				fetchCacheHashes={$fetchCacheState.hashes}
				fetchCacheSummaries={$fetchCacheState.summaries}
				{smartSearchingId}
				{favoritedTmdbIds}
				{pinnedTmdbIds}
				onnavigate={(tmdbId) => goto(`${base}/media/movies/${tmdbId}`)}
				onsmartsearch={handleSmartSearch}
			/>
		{:else if config.features.libraryItems === 'tv' && data.mediaData}
			<TvLibrarySection
				lists={tvLists}
				libraries={tvLibraries}
				{favoritedTmdbTvIds}
				{pinnedTmdbTvIds}
				onnavigate={(tmdbId) => goto(`${base}/media/tv/${tmdbId}`)}
			/>
		{/if}
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

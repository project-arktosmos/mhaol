<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { invalidateAll } from '$app/navigation';
	import { apiUrl } from 'frontend/lib/api-base';
	import { playerService } from 'frontend/services/player.service';
	import { playerAdapter } from 'frontend/adapters/classes/player.adapter';
	import { mediaDetailService } from 'frontend/services/media-detail.service';
	import { libraryService } from 'frontend/services/library.service';
	import { modalRouterService } from 'frontend/services/modal-router.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { MediaDetailCardType } from 'frontend/types/media-detail.type';
	import TmdbLinkModal from 'ui-lib/components/libraries/TmdbLinkModal.svelte';
	import SeasonEpisodeMatchModal from 'ui-lib/components/libraries/SeasonEpisodeMatchModal.svelte';
	import MediaCard from 'ui-lib/components/media/MediaCard.svelte';
	import MediaListCard from 'ui-lib/components/media/MediaListCard.svelte';
	import MediaDetail from 'ui-lib/components/media/MediaDetail.svelte';
	import type { MediaList, MediaListLink } from 'frontend/types/media-list.type';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import type { LibraryFile } from 'frontend/types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from 'frontend/types/media-card.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'addons/tmdb/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'addons/tmdb/transform';
	import { tmdbBrowseService } from 'frontend/services/tmdb-browse.service';
	import PopularTab from 'ui-lib/components/tmdb-browse/PopularTab.svelte';
	import DiscoverTab from 'ui-lib/components/tmdb-browse/DiscoverTab.svelte';
	import RecommendationsTab from 'ui-lib/components/tmdb-browse/RecommendationsTab.svelte';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			lists: MediaList[];
			libraries: Record<string, string>;
		};
	}

	const MOVIES_TAB = 'movies';
	const TV_TAB = 'tv';
	const POPULAR_TAB = 'popular';
	const DISCOVER_TAB = 'discover';
	const RECOMMENDATIONS_TAB = 'recommendations';

	type TabId = 'movies' | 'tv' | 'popular' | 'discover' | 'recommendations';

	let { data }: Props = $props();

	let activeTab = $state<TabId>(MOVIES_TAB);
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);
	let linkModalList: MediaList | null = $state(null);
	let linkModalListService: string | null = $state(null);
	let seasonMatchModal: {
		tmdbId: number;
		seasonNumber: number;
		showName: string;
		seasonName: string;
		files: LibraryFile[];
	} | null = $state(null);
	let selectedList: MediaList | null = $state(null);
	let selectedShowGroup: { tmdbId: string; lists: MediaList[] } | null = $state(null);

	type ListGridEntry =
		| { type: 'single'; list: MediaList }
		| { type: 'show-group'; tmdbId: string; lists: MediaList[] };

	let tvListGridEntries: ListGridEntry[] = $derived.by(() => {
		const tmdbGroups: Record<string, MediaList[]> = {};
		const ungrouped: MediaList[] = [];

		for (const list of data.lists.filter((l) => l.libraryType === 'tv')) {
			const links = getListLinks(list);
			const tmdbId = links.tmdb?.serviceId;
			if (tmdbId) {
				if (!tmdbGroups[tmdbId]) tmdbGroups[tmdbId] = [];
				tmdbGroups[tmdbId].push(list);
			} else {
				ungrouped.push(list);
			}
		}

		const entries: ListGridEntry[] = [];
		for (const [tmdbId, lists] of Object.entries(tmdbGroups)) {
			if (lists.length >= 2) {
				const sorted = [...lists].sort((a, b) => {
					const sa = getListLinks(a).tmdb?.seasonNumber ?? 999;
					const sb = getListLinks(b).tmdb?.seasonNumber ?? 999;
					return sa - sb;
				});
				entries.push({ type: 'show-group', tmdbId, lists: sorted });
			} else {
				entries.push({ type: 'single', list: lists[0] });
			}
		}
		for (const list of ungrouped) {
			entries.push({ type: 'single', list });
		}
		return entries;
	});

	let isMoviesTab = $derived(activeTab === MOVIES_TAB);
	let isTvTab = $derived(activeTab === TV_TAB);
	let isPopularTab = $derived(activeTab === POPULAR_TAB);
	let isDiscoverTab = $derived(activeTab === DISCOVER_TAB);
	let isRecommendationsTab = $derived(activeTab === RECOMMENDATIONS_TAB);

	// TMDB browse state
	const browseState = tmdbBrowseService.state;

	// Collect linked items for recommendations dropdown
	let linkedItems = $derived.by(() => {
		const items: Array<{ tmdbId: number; title: string; type: 'movie' | 'tv' }> = [];
		const seen = new Set<string>();
		for (const item of movieItems) {
			const tmdbLink = getItemLinks(item).tmdb;
			if (tmdbLink) {
				const key = `movie:${tmdbLink.serviceId}`;
				if (!seen.has(key)) {
					seen.add(key);
					const meta = tmdbMetadata[item.id] as DisplayTMDBMovieDetails | undefined;
					items.push({
						tmdbId: Number(tmdbLink.serviceId),
						title: meta?.title ?? item.name,
						type: 'movie'
					});
				}
			}
		}
		for (const list of data.lists.filter((l) => l.libraryType === 'tv')) {
			const links = getListLinks(list);
			if (links.tmdb) {
				const key = `tv:${links.tmdb.serviceId}`;
				if (!seen.has(key)) {
					seen.add(key);
					const meta = listTmdbMetadata[links.tmdb.serviceId];
					items.push({
						tmdbId: Number(links.tmdb.serviceId),
						title: meta?.name ?? list.title,
						type: 'tv'
					});
				}
			}
		}
		return items;
	});

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

	// Track category overrides so category changes are immediately reflected
	let categoryOverrides: Record<string, string> = $state({});

	// Track list link overrides
	let listLinkOverrides: Record<string, Record<string, MediaListLink | null>> = $state({});

	// List-level TMDB metadata state (keyed by tmdbId)
	let listTmdbMetadata: Record<string, DisplayTMDBTvShowDetails> = $state({});
	let listTmdbLoading: Set<string> = $state(new Set());

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

	// Scan all libraries state
	let scanning = $state(false);

	async function handleScanAll() {
		scanning = true;
		try {
			await libraryService.scanAllLibraries();
			await invalidateAll();
		} finally {
			scanning = false;
		}
	}

	onMount(async () => {
		libraryService.initialize();
	});

	function getItemLinks(item: MediaItem): Record<string, MediaItemLink> {
		const overrides = linkOverrides[item.id];
		if (!overrides) return item.links;
		const merged = { ...item.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) {
				delete merged[service];
			} else {
				merged[service] = link;
			}
		}
		return merged;
	}

	let movieItems = $derived(
		Object.values(data.itemsByType)
			.flat()
			.filter((i) => (data.libraries[i.libraryId] ?? 'movies') === 'movies')
	);

	// Apply link and category overrides to items for card rendering
	let itemsWithOverrides = $derived(
		movieItems.map((item) => {
			const linkOvr = linkOverrides[item.id];
			const catOvr = categoryOverrides[item.id];
			if (!linkOvr && catOvr === undefined) return item;
			const merged = { ...item.links };
			if (linkOvr) {
				for (const [service, link] of Object.entries(linkOvr)) {
					if (link === null) {
						delete merged[service];
					} else {
						merged[service] = link;
					}
				}
			}
			return { ...item, links: merged, ...(catOvr !== undefined ? { categoryId: catOvr } : {}) };
		})
	);

	// Player state
	const playerState = playerService.state;

	// Media detail selection
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived($mediaDetailStore?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		if (item.categoryId === 'tv' && item.links.tmdb) return 'tv';
		return 'video';
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item,
			cardType: resolveCardType(item),
			tmdbMetadata: tmdbMetadata[item.id] ?? null,
			youtubeMetadata: null,
			musicbrainzMetadata: null,
			imageTags: [],
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => {
				linkModalItem = i;
				linkModalService = service;
			},
			onunlink: (i, service) => handleUnlink(i, service)
		});
		modalRouterService.openMediaDetail(item.mediaTypeId, item.categoryId ?? '', item.id);
	}

	// Sync metadata updates into the active selection
	$effect(() => {
		const sel = $mediaDetailStore;
		if (!sel) return;
		const id = sel.item.id;
		const updatedItem = itemsWithOverrides.find((i) => i.id === id);
		if (!updatedItem) return;
		const newTmdb = tmdbMetadata[id] ?? null;
		if (newTmdb !== sel.tmdbMetadata || updatedItem !== sel.item) {
			mediaDetailService.select({
				...sel,
				item: updatedItem,
				cardType: resolveCardType(updatedItem),
				tmdbMetadata: newTmdb
			});
		}
	});

	function closeMediaDetail() {
		playerService.stop();
		mediaDetailService.clear();
		modalRouterService.closeMediaDetail();
	}

	onDestroy(() => {
		mediaDetailService.clear();
	});

	// Deep-link restoration: open media detail from URL params on load
	const routerStore = modalRouterService.store;
	let deepLinkRestored = $state(false);

	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail || deepLinkRestored) return;
		deepLinkRestored = true;
		const allItems = Object.values(data.itemsByType).flat();
		const item = allItems.find((i) => i.id === detail.id);
		if (item) {
			handleSelect(item);
		}
	});

	// Sync router popstate back to media detail
	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail && $mediaDetailStore) {
			mediaDetailService.clear();
		}
	});

	function selectTab(tab: TabId) {
		activeTab = tab;
		selectedList = null;
		selectedShowGroup = null;
		closeMediaDetail();

		if (tab === POPULAR_TAB) {
			const s = $browseState;
			if (s.popularMovies.length === 0) tmdbBrowseService.loadPopularMovies();
			if (s.popularTv.length === 0) tmdbBrowseService.loadPopularTv();
		} else if (tab === DISCOVER_TAB) {
			tmdbBrowseService.loadGenres();
			const s = $browseState;
			if (s.discoverMovies.length === 0) tmdbBrowseService.loadDiscoverMovies();
			if (s.discoverTv.length === 0) tmdbBrowseService.loadDiscoverTv();
		} else if (tab === RECOMMENDATIONS_TAB) {
			const s = $browseState;
			if (s.recommendations.length === 0 && linkedItems.length > 0) {
				const first = linkedItems[0];
				tmdbBrowseService.loadRecommendations(first.tmdbId, first.type);
			}
		}
	}

	function updateItemLinks(itemId: string, service: string, link: MediaItemLink | null) {
		linkOverrides = {
			...linkOverrides,
			[itemId]: {
				...linkOverrides[itemId],
				[service]: link
			}
		};
	}

	async function handleLink(
		tmdbId: number,
		seasonNumber: number | null,
		episodeNumber: number | null,
		type: 'movie' | 'tv'
	) {
		if (!linkModalItem) return;
		const item = linkModalItem;

		const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId, seasonNumber, episodeNumber })
		});

		if (res.ok) {
			const categoryId = type === 'movie' ? 'movies' : 'tv';
			const needsCategoryUpdate = item.categoryId !== categoryId;

			updateItemLinks(item.id, 'tmdb', {
				serviceId: String(tmdbId),
				seasonNumber,
				episodeNumber
			});
			categoryOverrides = { ...categoryOverrides, [item.id]: categoryId };

			if (needsCategoryUpdate) {
				fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/category`), {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ categoryId })
				});
			}
		}

		linkModalItem = null;
		linkModalService = null;
	}

	async function handleUnlink(item: MediaItem, service: string) {
		const res = await fetch(
			apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`),
			{
				method: 'DELETE'
			}
		);

		if (res.ok) {
			updateItemLinks(item.id, service, null);
			if (service === 'tmdb') {
				const { [item.id]: _, ...rest } = tmdbMetadata;
				tmdbMetadata = rest;
				const { [item.id]: __, ...restCat } = categoryOverrides;
				categoryOverrides = restCat;
			}
		}
	}

	function handlePlay(item: MediaItem) {
		const playableFile = playerAdapter.fromMediaItem(item);
		playerService.play(playableFile);
	}

	function getListLinks(list: MediaList): Record<string, MediaListLink> {
		const overrides = listLinkOverrides[list.id];
		if (!overrides) return list.links;
		const merged = { ...list.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) {
				delete merged[service];
			} else {
				merged[service] = link;
			}
		}
		return merged;
	}

	function listAsLibraryFile(list: MediaList): LibraryFile {
		return {
			id: list.id,
			name: list.title,
			path: '',
			extension: '',
			mediaType: list.mediaType as LibraryFile['mediaType'],
			categoryId: null,
			links: {}
		};
	}

	async function handleListTmdbLink(
		tmdbId: number,
		_seasonNumber: number | null,
		_episodeNumber: number | null,
		_type: 'movie' | 'tv'
	) {
		if (!linkModalList) return;
		const list = linkModalList;
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId })
		});
		if (res.ok) {
			const tmdbIdStr = String(tmdbId);
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: {
					...listLinkOverrides[list.id],
					tmdb: { serviceId: tmdbIdStr, seasonNumber: null }
				}
			};
			// Clear metadata for this tmdbId so it gets re-fetched
			const { [tmdbIdStr]: _, ...restTmdb } = listTmdbMetadata;
			listTmdbMetadata = restTmdb;
		}
		linkModalList = null;
		linkModalListService = null;
	}

	async function handleListUnlink(list: MediaList, service: string) {
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/${service}`), {
			method: 'DELETE'
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: { ...listLinkOverrides[list.id], [service]: null }
			};
		}
	}

	async function handleListSetSeason(list: MediaList, seasonNumber: number | null) {
		const links = getListLinks(list);
		const tmdbLink = links.tmdb;
		if (!tmdbLink) return;
		const res = await fetch(apiUrl(`/api/media-lists/${list.id}/tmdb`), {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId: Number(tmdbLink.serviceId), seasonNumber })
		});
		if (res.ok) {
			listLinkOverrides = {
				...listLinkOverrides,
				[list.id]: {
					...listLinkOverrides[list.id],
					tmdb: { serviceId: tmdbLink.serviceId, seasonNumber }
				}
			};
		}
	}

	async function handleSeasonMatchLinkAll(
		matches: Array<{ file: LibraryFile; seasonNumber: number; episodeNumber: number }>
	) {
		await Promise.all(
			matches.map(async (m) => {
				const item = data.lists.flatMap((l) => l.items).find((i) => i.id === m.file.id);
				if (!item) return;
				const tmdbId = seasonMatchModal ? seasonMatchModal.tmdbId : 0;
				const res = await fetch(apiUrl(`/api/libraries/${item.libraryId}/items/${item.id}/tmdb`), {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						tmdbId,
						seasonNumber: m.seasonNumber,
						episodeNumber: m.episodeNumber
					})
				});
				if (res.ok) {
					updateItemLinks(item.id, 'tmdb', {
						serviceId: String(tmdbId),
						seasonNumber: m.seasonNumber,
						episodeNumber: m.episodeNumber
					});
					categoryOverrides = { ...categoryOverrides, [item.id]: 'tv' };
				}
			})
		);
		seasonMatchModal = null;
	}

	async function fetchListTmdbMetadata(tmdbId: string) {
		if (listTmdbMetadata[tmdbId] || listTmdbLoading.has(tmdbId)) return;
		listTmdbLoading = new Set([...listTmdbLoading, tmdbId]);
		try {
			const res = await fetch(apiUrl(`/api/tmdb/tv/${tmdbId}`));
			if (res.ok) {
				const raw = await res.json();
				listTmdbMetadata = { ...listTmdbMetadata, [tmdbId]: tvShowDetailsToDisplay(raw) };
			}
		} catch (e) {
			console.error('Failed to load list TMDB metadata:', e);
		} finally {
			const next = new Set(listTmdbLoading);
			next.delete(tmdbId);
			listTmdbLoading = next;
		}
	}

	function itemAsLibraryFile(item: MediaItem): LibraryFile {
		return {
			id: item.id,
			name: item.name,
			path: item.path,
			extension: item.extension,
			mediaType: item.mediaTypeId as LibraryFile['mediaType'],
			categoryId: item.categoryId,
			links: getItemLinks(item)
		};
	}

	async function fetchTmdbMetadata(item: MediaItem) {
		const tmdbLink = item.links.tmdb;
		if (!tmdbLink || tmdbMetadata[item.id] || tmdbLoading.has(item.id)) return;

		tmdbLoading = new Set([...tmdbLoading, item.id]);

		const isTv = item.categoryId === 'tv';
		const endpoint = isTv
			? `/api/tmdb/tv/${tmdbLink.serviceId}`
			: `/api/tmdb/movies/${tmdbLink.serviceId}`;

		try {
			const res = await fetch(apiUrl(endpoint));
			if (res.ok) {
				const data = await res.json();
				tmdbMetadata[item.id] = isTv ? tvShowDetailsToDisplay(data) : movieDetailsToDisplay(data);
			}
		} catch (e) {
			console.error('Failed to load TMDB metadata:', e);
		} finally {
			const next = new Set(tmdbLoading);
			next.delete(item.id);
			tmdbLoading = next;
		}
	}

	$effect(() => {
		for (const item of itemsWithOverrides) {
			if (item.links.tmdb) {
				fetchTmdbMetadata(item);
			}
		}
	});

	// Fetch metadata for linked lists
	$effect(() => {
		for (const list of data.lists) {
			const links = getListLinks(list);
			if (links.tmdb) {
				fetchListTmdbMetadata(links.tmdb.serviceId);
			}
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Media</h1>
			<p class="text-sm opacity-70">Browse your media library</p>
		</div>
		<button class="btn btn-sm btn-accent" onclick={handleScanAll} disabled={scanning}>
			{#if scanning}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				Scan
			{/if}
		</button>
	</div>

	<!-- Tabs -->
	<div class="mb-6 flex gap-2">
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isMoviesTab,
				'btn-ghost': !isMoviesTab
			})}
			onclick={() => selectTab(MOVIES_TAB)}
		>
			Movies
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isTvTab,
				'btn-ghost': !isTvTab
			})}
			onclick={() => selectTab(TV_TAB)}
		>
			TV Shows
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isPopularTab,
				'btn-ghost': !isPopularTab
			})}
			onclick={() => selectTab(POPULAR_TAB)}
		>
			Popular
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isDiscoverTab,
				'btn-ghost': !isDiscoverTab
			})}
			onclick={() => selectTab(DISCOVER_TAB)}
		>
			Discover
		</button>
		<button
			class={classNames('btn btn-sm', {
				'btn-primary': isRecommendationsTab,
				'btn-ghost': !isRecommendationsTab
			})}
			onclick={() => selectTab(RECOMMENDATIONS_TAB)}
		>
			Recommendations
		</button>
	</div>

	{#if isPopularTab}
		<PopularTab
			movies={$browseState.popularMovies}
			tvShows={$browseState.popularTv}
			moviesPage={$browseState.popularMoviesPage}
			tvPage={$browseState.popularTvPage}
			moviesTotalPages={$browseState.popularMoviesTotalPages}
			tvTotalPages={$browseState.popularTvTotalPages}
			loadingMovies={$browseState.loading['popularMovies'] ?? false}
			loadingTv={$browseState.loading['popularTv'] ?? false}
			onloadMovies={(p) => tmdbBrowseService.loadPopularMovies(p)}
			onloadTv={(p) => tmdbBrowseService.loadPopularTv(p)}
		/>
	{:else if isDiscoverTab}
		<DiscoverTab
			movies={$browseState.discoverMovies}
			tvShows={$browseState.discoverTv}
			moviesPage={$browseState.discoverMoviesPage}
			tvPage={$browseState.discoverTvPage}
			moviesTotalPages={$browseState.discoverMoviesTotalPages}
			tvTotalPages={$browseState.discoverTvTotalPages}
			movieGenres={$browseState.movieGenres}
			tvGenres={$browseState.tvGenres}
			selectedGenreId={$browseState.selectedGenreId}
			loadingMovies={$browseState.loading['discoverMovies'] ?? false}
			loadingTv={$browseState.loading['discoverTv'] ?? false}
			ondiscoverMovies={(p, g) => tmdbBrowseService.loadDiscoverMovies(p, g)}
			ondiscoverTv={(p, g) => tmdbBrowseService.loadDiscoverTv(p, g)}
		/>
	{:else if isRecommendationsTab}
		<RecommendationsTab
			{linkedItems}
			recommendations={$browseState.recommendations}
			page={$browseState.recommendationsPage}
			totalPages={$browseState.recommendationsTotalPages}
			sourceId={$browseState.recommendationSourceId}
			sourceType={$browseState.recommendationSourceType}
			loading={$browseState.loading['recommendations'] ?? false}
			onload={(id, type, p) => tmdbBrowseService.loadRecommendations(id, type, p)}
		/>
	{:else if isTvTab}
		<!-- TV Shows view -->
		{#if selectedList}
			{@const currentListLinks = getListLinks(selectedList)}
			{@const listTmdb = currentListLinks.tmdb
				? (listTmdbMetadata[currentListLinks.tmdb.serviceId] ?? null)
				: null}
			{@const selectedSeason = currentListLinks.tmdb?.seasonNumber ?? null}
			{@const seasonMeta =
				listTmdb && selectedSeason != null
					? (listTmdb.seasons.find((s) => s.seasonNumber === selectedSeason) ?? null)
					: null}
			<div class="mb-4 flex items-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => {
						selectedList = null;
					}}
				>
					&larr; Back
				</button>
				<h2 class="text-xl font-semibold">{selectedList.title}</h2>
				<div class="ml-auto flex gap-2">
					{#if currentListLinks.tmdb}
						<button
							class="btn btn-outline btn-xs btn-error"
							onclick={() => handleListUnlink(selectedList!, 'tmdb')}
						>
							Unlink TV Show
						</button>
					{:else}
						<button
							class="btn btn-outline btn-xs"
							onclick={() => {
								linkModalList = selectedList;
								linkModalListService = 'tmdb';
							}}
						>
							Link TV Show
						</button>
					{/if}
				</div>
			</div>
			{#if listTmdb}
				<div class="mb-4 flex gap-4 rounded-lg bg-base-200 p-4">
					{#if seasonMeta?.posterUrl}
						<img
							src={seasonMeta.posterUrl}
							alt={seasonMeta.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{:else if listTmdb.posterUrl}
						<img
							src={listTmdb.posterUrl}
							alt={listTmdb.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{/if}
					<div class="flex flex-1 flex-col gap-1">
						<h3 class="text-lg font-semibold">{listTmdb.name}</h3>
						{#if seasonMeta}
							<p class="text-sm font-medium opacity-70">{seasonMeta.name}</p>
						{/if}
						<p class="text-xs opacity-50">
							{listTmdb.firstAirYear}{listTmdb.lastAirYear ? `–${listTmdb.lastAirYear}` : ''}
						</p>
						<p class="line-clamp-3 text-sm opacity-70">
							{seasonMeta?.overview || listTmdb.overview}
						</p>
						{#if listTmdb.seasons.length > 0}
							<div class="mt-auto">
								<select
									class="select-bordered select select-xs"
									value={selectedSeason ?? ''}
									onchange={(e) => {
										const val = (e.target as HTMLSelectElement).value;
										handleListSetSeason(selectedList!, val === '' ? null : Number(val));
									}}
								>
									<option value="">All Seasons</option>
									{#each listTmdb.seasons as season}
										<option value={season.seasonNumber}>{season.name}</option>
									{/each}
								</select>
							</div>
						{/if}
					</div>
				</div>
			{/if}
			{#if selectedList.items.length > 0}
				<div
					class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each selectedList.items as item (item.id)}
						<MediaCard
							{item}
							tmdbMetadata={tmdbMetadata[item.id] ?? null}
							metadataLoading={tmdbLoading.has(item.id)}
							selected={selectedItemId === item.id}
							onselect={(i) => handleSelect(i)}
						/>
					{/each}
				</div>
			{:else}
				<div class="rounded-lg bg-base-200 p-8 text-center">
					<p class="opacity-50">No items in this list.</p>
				</div>
			{/if}
		{:else if selectedShowGroup}
			{@const tmdbMeta = listTmdbMetadata[selectedShowGroup.tmdbId] ?? null}
			<div class="mb-4 flex items-center gap-2">
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => {
						selectedShowGroup = null;
					}}
				>
					&larr; Back
				</button>
				<h2 class="text-xl font-semibold">{tmdbMeta?.name ?? 'TV Show'}</h2>
			</div>
			{#if tmdbMeta}
				<div class="mb-6 flex gap-4 rounded-lg bg-base-200 p-4">
					{#if tmdbMeta.posterUrl}
						<img
							src={tmdbMeta.posterUrl}
							alt={tmdbMeta.name}
							class="h-40 w-auto rounded-lg object-cover"
						/>
					{/if}
					<div class="flex flex-1 flex-col gap-1">
						<h3 class="text-lg font-semibold">{tmdbMeta.name}</h3>
						<p class="text-xs opacity-50">
							{tmdbMeta.firstAirYear}{tmdbMeta.lastAirYear ? `–${tmdbMeta.lastAirYear}` : ''}
						</p>
						<p class="line-clamp-3 text-sm opacity-70">{tmdbMeta.overview}</p>
					</div>
				</div>
			{/if}
			{#each selectedShowGroup.lists as list (list.id)}
				{@const links = getListLinks(list)}
				{@const seasonNum = links.tmdb?.seasonNumber}
				{@const seasonMeta =
					tmdbMeta && seasonNum != null
						? (tmdbMeta.seasons.find((s) => s.seasonNumber === seasonNum) ?? null)
						: null}
				<div class="mb-6">
					<div class="mb-3 flex items-center gap-3">
						{#if seasonMeta?.posterUrl}
							<img
								src={seasonMeta.posterUrl}
								alt={seasonMeta.name}
								class="h-16 w-auto rounded object-cover"
							/>
						{/if}
						<div class="flex flex-1 items-center gap-3">
							<div>
								<h3 class="text-base font-semibold">
									{seasonMeta?.name ?? list.title}
								</h3>
								{#if seasonMeta?.episodeCount}
									<p class="text-xs opacity-50">{seasonMeta.episodeCount} episodes</p>
								{/if}
							</div>
							{#if tmdbMeta && seasonNum != null && list.items.length > 0}
								<button
									class="btn ml-auto btn-ghost btn-xs"
									onclick={() => {
										seasonMatchModal = {
											tmdbId: Number(selectedShowGroup!.tmdbId),
											seasonNumber: seasonNum!,
											showName: tmdbMeta!.name,
											seasonName: seasonMeta?.name ?? list.title,
											files: list.items.map((i) => itemAsLibraryFile(i))
										};
									}}
								>
									Match episodes
								</button>
							{/if}
						</div>
					</div>
					{#if list.items.length > 0}
						<div
							class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
						>
							{#each list.items as item (item.id)}
								<MediaCard
									{item}
									tmdbMetadata={tmdbMetadata[item.id] ?? null}
									metadataLoading={tmdbLoading.has(item.id)}
									selected={selectedItemId === item.id}
									onselect={(i) => handleSelect(i)}
								/>
							{/each}
						</div>
					{:else}
						<p class="text-sm opacity-50">No items in this season.</p>
					{/if}
				</div>
			{/each}
		{:else if tvListGridEntries.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each tvListGridEntries as entry}
					{#if entry.type === 'single'}
						{@const list = entry.list}
						{@const links = getListLinks(list)}
						<MediaListCard
							{list}
							tmdbMetadata={links.tmdb ? (listTmdbMetadata[links.tmdb.serviceId] ?? null) : null}
							onselect={(l) => {
								selectedList = l;
							}}
						/>
					{:else}
						{@const tmdbMeta = listTmdbMetadata[entry.tmdbId] ?? null}
						{@const representative = entry.lists[0]}
						<MediaListCard
							list={representative}
							tmdbMetadata={tmdbMeta}
							seasonCount={entry.lists.length}
							onselect={() => {
								selectedShowGroup = { tmdbId: entry.tmdbId, lists: entry.lists };
							}}
						/>
					{/if}
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No TV shows yet. Add a TV library and scan it.</p>
			</div>
		{/if}
	{:else}
		<!-- Movies grid -->
		{#if itemsWithOverrides.length > 0}
			<div
				class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each itemsWithOverrides as item (item.id)}
					<MediaCard
						{item}
						tmdbMetadata={tmdbMetadata[item.id] ?? null}
						metadataLoading={tmdbLoading.has(item.id)}
						selected={selectedItemId === item.id}
						onselect={(i) => handleSelect(i)}
					/>
				{/each}
			</div>
		{:else}
			<div class="rounded-lg bg-base-200 p-8 text-center">
				<p class="opacity-50">No movies yet. Add a Movies library and scan it.</p>
			</div>
		{/if}
	{/if}
</div>

<Modal open={!!$mediaDetailStore} maxWidth="max-w-lg" onclose={closeMediaDetail}>
	{#if $mediaDetailStore}
		<MediaDetail selection={$mediaDetailStore} onclose={closeMediaDetail} />
		{#if $playerState.currentFile && $playerState.currentFile.id !== $mediaDetailStore?.item.id}
			<div class="mt-4 border-t border-base-300 pt-4">
				<div class="mb-2 flex items-center justify-between">
					<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">
						Now Playing
					</h2>
					<button
						class="btn btn-square btn-ghost btn-xs"
						onclick={() => playerService.stop()}
						aria-label="Close player"
					>
						&times;
					</button>
				</div>
				<p class="mb-2 truncate text-xs opacity-60" title={$playerState.currentFile.name}>
					{$playerState.currentFile.name}
				</p>
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
				/>
			</div>
		{/if}
	{/if}
</Modal>

{#if linkModalItem && linkModalService === 'tmdb-movie'}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		type="movie"
		onlink={handleLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalItem && linkModalService === 'tmdb-tv'}
	<TmdbLinkModal
		file={itemAsLibraryFile(linkModalItem)}
		type="tv"
		onlink={handleLink}
		onclose={() => {
			linkModalItem = null;
			linkModalService = null;
		}}
	/>
{/if}

{#if linkModalList && linkModalListService === 'tmdb'}
	<TmdbLinkModal
		file={listAsLibraryFile(linkModalList)}
		type="tv"
		onlink={handleListTmdbLink}
		onclose={() => {
			linkModalList = null;
			linkModalListService = null;
		}}
	/>
{/if}

{#if seasonMatchModal}
	<SeasonEpisodeMatchModal
		tmdbId={seasonMatchModal.tmdbId}
		seasonNumber={seasonMatchModal.seasonNumber}
		showName={seasonMatchModal.showName}
		seasonName={seasonMatchModal.seasonName}
		files={seasonMatchModal.files}
		onlinkall={handleSeasonMatchLinkAll}
		onclose={() => {
			seasonMatchModal = null;
		}}
	/>
{/if}

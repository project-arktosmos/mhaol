<script lang="ts">
	import classNames from 'classnames';
	import { onMount, onDestroy } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { playerService } from 'frontend/services/player.service';
	import { playerAdapter } from 'frontend/adapters/classes/player.adapter';
	import { mediaDetailService } from 'frontend/services/media-detail.service';
	import { libraryService } from 'frontend/services/library.service';
	import { modalRouterService } from 'frontend/services/modal-router.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { MediaDetailCardType } from 'frontend/types/media-detail.type';
	import TmdbLinkModal from 'ui-lib/components/libraries/TmdbLinkModal.svelte';
	import LibraryTab from 'ui-lib/components/tmdb-browse/LibraryTab.svelte';
	import MediaDetail from 'ui-lib/components/media/MediaDetail.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import TmdbBrowseDetail from 'ui-lib/components/tmdb-browse/TmdbBrowseDetail.svelte';
	import type { LibraryFile } from 'frontend/types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from 'frontend/types/media-card.type';
	import type {
		DisplayTMDBMovie,
		DisplayTMDBTvShow,
		DisplayTMDBMovieDetails,
		DisplayTMDBTvShowDetails
	} from 'addons/tmdb/types';
	import { movieDetailsToDisplay, tvShowDetailsToDisplay } from 'addons/tmdb/transform';
	import { tmdbBrowseService } from 'frontend/services/tmdb-browse.service';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import type { TorrentInfo } from 'frontend/types/torrent.type';
	import type { SmartSearchTorrentResult } from 'frontend/types/smart-search.type';
	import type { PlayableFile } from 'frontend/types/player.type';
	import SearchTab from 'ui-lib/components/tmdb-browse/SearchTab.svelte';
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
			libraries: Record<string, string>;
		};
	}

	const LIBRARY_SUB = 'library';
	const SEARCH_SUB = 'search';
	const POPULAR_SUB = 'popular';
	const DISCOVER_SUB = 'discover';
	const RECOMMENDATIONS_SUB = 'recommendations';

	type SubTabId = 'library' | 'search' | 'popular' | 'discover' | 'recommendations';

	let { data }: Props = $props();

	let activeSubTab = $state<SubTabId>(LIBRARY_SUB);
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);

	let isLibrarySub = $derived(activeSubTab === LIBRARY_SUB);
	let isSearchSub = $derived(activeSubTab === SEARCH_SUB);
	let isPopularSub = $derived(activeSubTab === POPULAR_SUB);
	let isDiscoverSub = $derived(activeSubTab === DISCOVER_SUB);
	let isRecommendationsSub = $derived(activeSubTab === RECOMMENDATIONS_SUB);

	// TMDB browse state
	const browseState = tmdbBrowseService.state;

	// Browse detail selection state
	let selectedBrowseMovie: DisplayTMDBMovie | null = $state(null);
	let selectedBrowseTvShow: DisplayTMDBTvShow | null = $state(null);
	let browseMovieDetails: DisplayTMDBMovieDetails | null = $state(null);
	let browseTvShowDetails: DisplayTMDBTvShowDetails | null = $state(null);
	let browseDetailLoading = $state(false);

	let hasBrowseSelection = $derived(selectedBrowseMovie !== null || selectedBrowseTvShow !== null);

	function handleBrowseSelectMovie(movie: DisplayTMDBMovie) {
		selectedBrowseTvShow = null;
		browseTvShowDetails = null;
		if (selectedBrowseMovie?.id === movie.id) {
			selectedBrowseMovie = null;
			browseMovieDetails = null;
			return;
		}
		selectedBrowseMovie = movie;
		browseMovieDetails = null;
		fetchBrowseMovieDetails(movie.id);
	}

	function handleBrowseSelectTvShow(tvShow: DisplayTMDBTvShow) {
		selectedBrowseMovie = null;
		browseMovieDetails = null;
		if (selectedBrowseTvShow?.id === tvShow.id) {
			selectedBrowseTvShow = null;
			browseTvShowDetails = null;
			return;
		}
		selectedBrowseTvShow = tvShow;
		browseTvShowDetails = null;
		fetchBrowseTvShowDetails(tvShow.id);
	}

	function closeBrowseDetail() {
		selectedBrowseMovie = null;
		selectedBrowseTvShow = null;
		browseMovieDetails = null;
		browseTvShowDetails = null;
		fetchingTmdbId = null;
		smartSearchService.clear();
	}

	async function fetchBrowseMovieDetails(tmdbId: number) {
		browseDetailLoading = true;
		try {
			const res = await fetch(apiUrl(`/api/tmdb/movies/${tmdbId}`));
			if (res.ok) {
				const raw = await res.json();
				browseMovieDetails = movieDetailsToDisplay(raw);
			}
		} catch (e) {
			console.error('Failed to load browse movie details:', e);
		} finally {
			browseDetailLoading = false;
		}
	}

	async function fetchBrowseTvShowDetails(tmdbId: number) {
		browseDetailLoading = true;
		try {
			const res = await fetch(apiUrl(`/api/tmdb/tv/${tmdbId}`));
			if (res.ok) {
				const raw = await res.json();
				browseTvShowDetails = tvShowDetailsToDisplay(raw);
			}
		} catch (e) {
			console.error('Failed to load browse TV details:', e);
		} finally {
			browseDetailLoading = false;
		}
	}

	// Smart search store for tracking fetch state
	const searchStore = smartSearchService.store;

	let fetchingTmdbId = $state<number | null>(null);
	let currentDetailTmdbId = $derived(selectedBrowseMovie?.id ?? selectedBrowseTvShow?.id ?? null);
	let isFetching = $derived(
		fetchingTmdbId !== null &&
			fetchingTmdbId === currentDetailTmdbId &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingTmdbId === currentDetailTmdbId
	);

	function handleBrowseDetailFetch() {
		if (selectedBrowseMovie) {
			fetchingTmdbId = selectedBrowseMovie.id;
			smartSearchService.select({
				title: selectedBrowseMovie.title,
				year: selectedBrowseMovie.releaseYear,
				type: 'movie',
				tmdbId: selectedBrowseMovie.id,
				mode: 'fetch'
			});
		} else if (selectedBrowseTvShow) {
			fetchingTmdbId = selectedBrowseTvShow.id;
			smartSearchService.select({
				title: selectedBrowseTvShow.name,
				year: selectedBrowseTvShow.firstAirYear,
				type: 'tv',
				tmdbId: selectedBrowseTvShow.id,
				mode: 'fetch'
			});
		}
	}

	function handleBrowseDetailDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	function handleBrowseDetailStream() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		const title = selectedBrowseMovie?.title ?? selectedBrowseTvShow?.name ?? '';
		playerService.prepareStream(title);
		handleStreamCandidate(candidate);
	}

	async function handleStreamCandidate(candidate: SmartSearchTorrentResult) {
		smartSearchService.hide();
		const infoHash = await smartSearchService.startStream(candidate);
		if (!infoHash) return;

		const unsubscribe = torrentService.state.subscribe((state) => {
			const torrent = state.torrents.find((t) => t.infoHash === infoHash);
			if (!torrent) return;

			smartSearchService.updateStreamingProgress(torrent.progress);

			if (torrent.progress >= 0.02 || torrent.state === 'seeding') {
				unsubscribe();
				smartSearchService.clearStreaming();

				const file: PlayableFile = {
					id: `torrent:${infoHash}`,
					type: 'torrent',
					name: torrent.name,
					outputPath: torrent.outputPath ?? '',
					mode: 'video',
					format: null,
					videoFormat: null,
					thumbnailUrl: null,
					durationSeconds: null,
					size: torrent.size,
					completedAt: '',
					streamUrl: `/api/torrent/torrents/${infoHash}/stream`
				};
				playerService.playStream(file);
			}
		});
	}

	// Torrent state — match torrents to library items by path
	const torrentState = torrentService.state;

	function findTorrentForItem(item: MediaItem): TorrentInfo | null {
		const torrents = $torrentState.torrents;
		if (torrents.length === 0) return null;
		for (const t of torrents) {
			if (!t.outputPath) continue;
			if (item.path.startsWith(t.outputPath)) return t;
		}
		return null;
	}

	// Collect linked movie items for recommendations dropdown
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
		return items;
	});

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

	// Track category overrides so category changes are immediately reflected
	let categoryOverrides: Record<string, string> = $state({});

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

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

	// Convert library items to DisplayTMDBMovie[] for LibraryTab
	// Use a stable unique numeric id per item (hash from item.id string)
	function stableNumericId(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			hash = (hash * 31 + str.charCodeAt(i)) | 0;
		}
		return Math.abs(hash);
	}

	let libraryMovies = $derived(
		itemsWithOverrides.map((item): DisplayTMDBMovie => {
			const meta = tmdbMetadata[item.id];
			return {
				id: stableNumericId(item.id),
				title: meta?.title ?? item.name,
				originalTitle: meta?.originalTitle ?? item.name,
				overview: meta?.overview ?? '',
				posterUrl: meta?.posterUrl ?? null,
				backdropUrl: meta?.backdropUrl ?? null,
				releaseYear: meta?.releaseYear ?? '',
				voteAverage: meta?.voteAverage ?? 0,
				voteCount: meta?.voteCount ?? 0,
				genres: meta?.genres ?? []
			};
		})
	);

	// Map numeric id back to MediaItem for selection handling
	let libraryItemsByMovieId = $derived(
		new Map(itemsWithOverrides.map((item) => [stableNumericId(item.id), item]))
	);

	// Player state
	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	// Media detail selection
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived($mediaDetailStore?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
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

	function selectSubTab(sub: SubTabId) {
		activeSubTab = sub;
		closeMediaDetail();
		closeBrowseDetail();

		if (sub === POPULAR_SUB) {
			const s = $browseState;
			if (s.popularMovies.length === 0) tmdbBrowseService.loadPopularMovies();
		} else if (sub === DISCOVER_SUB) {
			tmdbBrowseService.loadGenres();
			const s = $browseState;
			if (s.discoverMovies.length === 0) tmdbBrowseService.loadDiscoverMovies();
		} else if (sub === RECOMMENDATIONS_SUB) {
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
			updateItemLinks(item.id, 'tmdb', {
				serviceId: String(tmdbId),
				seasonNumber,
				episodeNumber
			});
			categoryOverrides = { ...categoryOverrides, [item.id]: 'movies' };
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

		try {
			const res = await fetch(apiUrl(`/api/tmdb/movies/${tmdbLink.serviceId}`));
			if (res.ok) {
				const data = await res.json();
				tmdbMetadata[item.id] = movieDetailsToDisplay(data);
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
</script>

<div class="flex flex-1 overflow-hidden">
	<div class="min-w-0 flex-1 overflow-y-auto p-4">
		<div class="container mx-auto">
			<div class="mb-6">
				<h1 class="text-3xl font-bold">Movies</h1>
				<p class="text-sm opacity-70">Browse your movie library</p>
			</div>

			<!-- Sub Tabs -->
			<div class="mb-6 flex gap-1">
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': isLibrarySub,
						'btn-ghost': !isLibrarySub
					})}
					onclick={() => selectSubTab(LIBRARY_SUB)}
				>
					Library
				</button>
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': isSearchSub,
						'btn-ghost': !isSearchSub
					})}
					onclick={() => selectSubTab(SEARCH_SUB)}
				>
					Search
				</button>
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': isPopularSub,
						'btn-ghost': !isPopularSub
					})}
					onclick={() => selectSubTab(POPULAR_SUB)}
				>
					Popular
				</button>
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': isDiscoverSub,
						'btn-ghost': !isDiscoverSub
					})}
					onclick={() => selectSubTab(DISCOVER_SUB)}
				>
					Discover
				</button>
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': isRecommendationsSub,
						'btn-ghost': !isRecommendationsSub
					})}
					onclick={() => selectSubTab(RECOMMENDATIONS_SUB)}
				>
					Recommendations
				</button>
			</div>

			{#if isSearchSub}
				<SearchTab
					movies={$browseState.searchMovies}
					tvShows={$browseState.searchTv}
					moviesPage={$browseState.searchMoviesPage}
					tvPage={$browseState.searchTvPage}
					moviesTotalPages={$browseState.searchMoviesTotalPages}
					tvTotalPages={$browseState.searchTvTotalPages}
					query={$browseState.searchQuery}
					loadingMovies={$browseState.loading['searchMovies'] ?? false}
					loadingTv={$browseState.loading['searchTv'] ?? false}
					error={$browseState.error}
					mediaType="movies"
					selectedMovieId={selectedBrowseMovie?.id ?? null}
					onselectMovie={handleBrowseSelectMovie}
					onsearchMovies={(q, p) => tmdbBrowseService.searchMovies(q, p)}
					onsearchTv={(q, p) => tmdbBrowseService.searchTv(q, p)}
				/>
			{:else if isPopularSub}
				<PopularTab
					movies={$browseState.popularMovies}
					tvShows={$browseState.popularTv}
					moviesPage={$browseState.popularMoviesPage}
					tvPage={$browseState.popularTvPage}
					moviesTotalPages={$browseState.popularMoviesTotalPages}
					tvTotalPages={$browseState.popularTvTotalPages}
					loadingMovies={$browseState.loading['popularMovies'] ?? false}
					loadingTv={$browseState.loading['popularTv'] ?? false}
					error={$browseState.error}
					mediaType="movies"
					selectedMovieId={selectedBrowseMovie?.id ?? null}
					onselectMovie={handleBrowseSelectMovie}
					onloadMovies={(p) => tmdbBrowseService.loadPopularMovies(p)}
					onloadTv={(p) => tmdbBrowseService.loadPopularTv(p)}
				/>
			{:else if isDiscoverSub}
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
					error={$browseState.error}
					mediaType="movies"
					selectedMovieId={selectedBrowseMovie?.id ?? null}
					onselectMovie={handleBrowseSelectMovie}
					ondiscoverMovies={(p, g) => tmdbBrowseService.loadDiscoverMovies(p, g)}
					ondiscoverTv={(p, g) => tmdbBrowseService.loadDiscoverTv(p, g)}
				/>
			{:else if isRecommendationsSub}
				<RecommendationsTab
					{linkedItems}
					recommendations={$browseState.recommendations}
					page={$browseState.recommendationsPage}
					totalPages={$browseState.recommendationsTotalPages}
					sourceId={$browseState.recommendationSourceId}
					sourceType={$browseState.recommendationSourceType}
					loading={$browseState.loading['recommendations'] ?? false}
					error={$browseState.error}
					selectedMovieId={selectedBrowseMovie?.id ?? null}
					selectedTvShowId={selectedBrowseTvShow?.id ?? null}
					onselectMovie={handleBrowseSelectMovie}
					onselectTvShow={handleBrowseSelectTvShow}
					onload={(id, type, p) => tmdbBrowseService.loadRecommendations(id, type, p)}
				/>
			{:else if isLibrarySub}
				<LibraryTab
					movies={libraryMovies}
					selectedMovieId={selectedBrowseMovie?.id ?? null}
					onselectMovie={(movie) => {
						const item = libraryItemsByMovieId.get(movie.id);
						if (!item) return;
						selectedBrowseTvShow = null;
						browseTvShowDetails = null;
						if (selectedBrowseMovie?.id === movie.id) {
							selectedBrowseMovie = null;
							browseMovieDetails = null;
							return;
						}
						selectedBrowseMovie = movie;
						const meta = tmdbMetadata[item.id] ?? null;
						browseMovieDetails = meta;
						if (!meta && item.links.tmdb) {
							fetchBrowseMovieDetails(Number(item.links.tmdb.serviceId));
						}
					}}
				/>
			{/if}
		</div>
	</div>

	{#if hasBrowseSelection || ($playerState.currentFile && $playerDisplayMode === 'sidebar')}
		<div class="hidden w-85 shrink-0 border-l border-base-300 bg-base-200 lg:block">
			<TmdbBrowseDetail
				movie={selectedBrowseMovie}
				tvShow={selectedBrowseTvShow}
				movieDetails={browseMovieDetails}
				tvShowDetails={browseTvShowDetails}
				loading={browseDetailLoading}
				fetching={isFetching}
				fetched={isFetchedForCurrent}
				playerFile={$playerDisplayMode === 'sidebar' ? $playerState.currentFile : null}
				playerConnectionState={$playerState.connectionState}
				playerPositionSecs={$playerState.positionSecs}
				playerDurationSecs={$playerState.durationSecs}
				playerStreamUrl={$playerState.streamUrl}
				playerBuffering={$playerState.buffering}
				onfetch={handleBrowseDetailFetch}
				ondownload={handleBrowseDetailDownload}
				onstream={handleBrowseDetailStream}
				onfullscreen={() => playerService.setDisplayMode('fullscreen')}
				onstopplayer={() => playerService.stop()}
				onclose={closeBrowseDetail}
			/>
		</div>
	{/if}
</div>

<div class="lg:hidden">
<Modal open={hasBrowseSelection || !!($playerState.currentFile && $playerDisplayMode === 'sidebar')} maxWidth="max-w-lg" onclose={closeBrowseDetail}>
	<TmdbBrowseDetail
		movie={selectedBrowseMovie}
		tvShow={selectedBrowseTvShow}
		movieDetails={browseMovieDetails}
		tvShowDetails={browseTvShowDetails}
		loading={browseDetailLoading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		playerFile={$playerDisplayMode === 'sidebar' ? $playerState.currentFile : null}
		playerConnectionState={$playerState.connectionState}
		playerPositionSecs={$playerState.positionSecs}
		playerDurationSecs={$playerState.durationSecs}
		playerStreamUrl={$playerState.streamUrl}
		playerBuffering={$playerState.buffering}
		onfetch={handleBrowseDetailFetch}
		ondownload={handleBrowseDetailDownload}
		onstream={handleBrowseDetailStream}
		onfullscreen={() => playerService.setDisplayMode('fullscreen')}
		onstopplayer={() => playerService.stop()}
		onclose={closeBrowseDetail}
	/>
</Modal>
</div>

{#if $playerState.currentFile && !$mediaDetailStore && $playerDisplayMode === 'fullscreen'}
	<div class="fixed inset-0 z-40 flex flex-col bg-black">
		<div class="flex items-center justify-between p-3">
			<p class="truncate text-sm font-semibold text-white" title={$playerState.currentFile.name}>
				{$playerState.currentFile.name}
			</p>
			<div class="flex items-center gap-1">
				<button
					class="btn btn-square btn-ghost btn-sm text-white"
					onclick={() => playerService.setDisplayMode('sidebar')}
					aria-label="Move to sidebar"
					title="Move to sidebar"
				>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M18 8L14 12L18 16" />
						<rect x="3" y="3" width="18" height="18" rx="2" />
						<line x1="14" y1="3" x2="14" y2="21" />
					</svg>
				</button>
				<button
					class="btn btn-square btn-ghost btn-sm text-white"
					onclick={() => playerService.stop()}
					aria-label="Close player"
				>
					&times;
				</button>
			</div>
		</div>
		<div class="min-h-0 flex-1">
			<PlayerVideo
				file={$playerState.currentFile}
				connectionState={$playerState.connectionState}
				positionSecs={$playerState.positionSecs}
				durationSecs={$playerState.durationSecs}
				streamUrl={$playerState.streamUrl}
				buffering={$playerState.buffering}
				fullscreen
			/>
		</div>
	</div>
{/if}

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
					streamUrl={$playerState.streamUrl}
					buffering={$playerState.buffering}
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

<script lang="ts">
	import { onMount, onDestroy, getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { playerService } from 'ui-lib/services/player.service';
	import { playerAdapter } from 'ui-lib/adapters/classes/player.adapter';
	import { mediaDetailService } from 'ui-lib/services/media-detail.service';
	import { modalRouterService } from 'ui-lib/services/modal-router.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import Portal from 'ui-lib/components/core/Portal.svelte';
	import type { MediaDetailCardType } from 'ui-lib/types/media-detail.type';
	import TmdbLinkModal from 'ui-lib/components/libraries/TmdbLinkModal.svelte';
	import MediaDetail from 'ui-lib/components/media/MediaDetail.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import type { LibraryFile } from 'ui-lib/types/library.type';
	import type { MediaItem, MediaItemLink, MediaLinkSource, MediaCategory } from 'ui-lib/types/media-card.type';
	import type { DisplayTMDBMovie, DisplayTMDBMovieDetails, TMDBMovie } from 'addons/tmdb/types';
	import { movieDetailsToDisplay, moviesToDisplay, getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { libraryService } from 'ui-lib/services/library.service';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import classNames from 'classnames';
	import TmdbCatalogGrid from 'ui-lib/components/catalog/TmdbCatalogGrid.svelte';
	import BrowseViewToggle from 'ui-lib/components/browse/BrowseViewToggle.svelte';
	import RecommendationsModalContent from 'ui-lib/components/recommendations/RecommendationsModalContent.svelte';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';

	const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);
	mediaBar.configure({ title: 'Movies' });

	let recsModalOpen = $state(false);

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			libraries: Record<string, { name: string; type: string }>;
			error?: string;
		};
	}

	let { data }: Props = $props();

	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);
	let movieSearchInput = $state('');
	let pinnedMovies = $state<DisplayTMDBMovie[]>([]);
	let favoritedMovies = $state<DisplayTMDBMovie[]>([]);
	let movieImageOverrides = $state<Map<number, Record<string, string>>>(new Map());

	// === Browse state (inline, replaces tmdbBrowseService) ===
	interface TmdbPagedResponse { results: TMDBMovie[]; total_pages: number; page: number; }
	let popularMovies = $state<DisplayTMDBMovie[]>([]);
	let popularMoviesPage = $state(1);
	let popularMoviesTotalPages = $state(1);
	let discoverMovies = $state<DisplayTMDBMovie[]>([]);
	let discoverMoviesPage = $state(1);
	let discoverMoviesTotalPages = $state(1);
	let selectedGenreId = $state<number | null>(null);
	let movieGenres = $state<Array<{ id: number; name: string }>>([]);
	let searchMovies = $state<DisplayTMDBMovie[]>([]);
	let searchMoviesPage = $state(1);
	let searchMoviesTotalPages = $state(1);
	let searchQuery = $state('');
	let browseLoading = $state<Record<string, boolean>>({});
	let browseTab: 'popular' | 'discover' = $state('popular');

	async function loadPopularMovies(page = 1) {
		browseLoading = { ...browseLoading, popularMovies: true };
		try {
			const data = await fetchJson<TmdbPagedResponse>(`/api/tmdb/popular/movies?page=${page}`);
			popularMovies = moviesToDisplay(data.results);
			popularMoviesPage = data.page;
			popularMoviesTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		browseLoading = { ...browseLoading, popularMovies: false };
	}

	async function loadDiscoverMovies(page = 1, genreId: number | null = null) {
		browseLoading = { ...browseLoading, discoverMovies: true };
		selectedGenreId = genreId;
		try {
			let url = `/api/tmdb/discover/movies?page=${page}`;
			if (genreId) url += `&with_genres=${genreId}`;
			const data = await fetchJson<TmdbPagedResponse>(url);
			discoverMovies = moviesToDisplay(data.results);
			discoverMoviesPage = data.page;
			discoverMoviesTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		browseLoading = { ...browseLoading, discoverMovies: false };
	}

	async function loadMovieGenres() {
		try {
			const data = await fetchJson<{ genres: Array<{ id: number; name: string }> }>('/api/tmdb/genres/movie');
			movieGenres = data?.genres ?? [];
		} catch { /* best-effort */ }
	}

	async function doSearchMovies(query: string, page = 1) {
		if (!query.trim()) return;
		browseLoading = { ...browseLoading, searchMovies: true };
		searchQuery = query;
		try {
			const data = await fetchJson<TmdbPagedResponse>(`/api/tmdb/search/movies?q=${encodeURIComponent(query)}&page=${page}`);
			searchMovies = moviesToDisplay(data.results);
			searchMoviesPage = data.page;
			searchMoviesTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		browseLoading = { ...browseLoading, searchMovies: false };
	}

	// === Smart search ===
	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;
	let smartSearchingId: number | null = $state(null);
	let batchSearching = $state(false);
	let batchProgress = $state({ current: 0, total: 0 });

	// === Fetch cache ===
	let fetchCachedTmdbIds: Set<number> = $state(new Set());
	let fetchCacheHashes: Map<number, string> = $state(new Map());
	let fetchCacheSummaries: Map<number, string> = $state(new Map());

	// === Favorites/Pins ===
	const favState = favoritesService.state;
	const pinState = pinsService.state;
	let favoritedTmdbIds = $derived(
		new Set($favState.items.filter((f) => f.service === 'tmdb').map((f) => Number(f.serviceId)))
	);
	let pinnedTmdbIds = $derived(
		new Set($pinState.items.filter((p) => p.service === 'tmdb').map((p) => Number(p.serviceId)))
	);

	// === Resolve pinned/favorited movies ===
	async function resolveMovieIds(ids: number[]): Promise<DisplayTMDBMovie[]> {
		if (ids.length === 0) return [];
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<TMDBMovie>(`/api/tmdb/movies/${id}`))
		);
		return moviesToDisplay(
			results
				.filter((r): r is PromiseFulfilledResult<TMDBMovie> => r.status === 'fulfilled' && r.value != null)
				.map((r) => r.value)
		);
	}

	$effect(() => {
		const ids = [...pinnedTmdbIds];
		let cancelled = false;
		resolveMovieIds(ids).then((movies) => { if (!cancelled) pinnedMovies = movies; });
		return () => { cancelled = true; };
	});

	$effect(() => {
		const ids = [...favoritedTmdbIds];
		let cancelled = false;
		resolveMovieIds(ids).then((movies) => { if (!cancelled) favoritedMovies = movies; });
		return () => { cancelled = true; };
	});

	// === Image overrides ===
	async function loadMovieImageOverrides() {
		try {
			const res = await fetchRaw('/api/tmdb/image-overrides/movie');
			if (res.ok) {
				const overrides: Array<{ tmdb_id: number; role: string; file_path: string }> = await res.json();
				const map = new Map<number, Record<string, string>>();
				for (const o of overrides) {
					const existing = map.get(o.tmdb_id) ?? {};
					existing[o.role] = o.file_path;
					map.set(o.tmdb_id, existing);
				}
				movieImageOverrides = map;
			}
		} catch { /* best-effort */ }
	}

	function applyOverridesToMovies(movies: DisplayTMDBMovie[]): DisplayTMDBMovie[] {
		if (movieImageOverrides.size === 0) return movies;
		return movies.map((m) => {
			const overrides = movieImageOverrides.get(m.id);
			if (!overrides) return m;
			return {
				...m,
				posterUrl: overrides.poster ? getPosterUrl(overrides.poster) : m.posterUrl,
				backdropUrl: overrides.backdrop ? getBackdropUrl(overrides.backdrop) : m.backdropUrl
			};
		});
	}

	// === Fetch cache loading ===

	// Map catalog item IDs to TMDB IDs for fetch cache lookups
	let catalogToTmdbId: Map<string, number> = $state(new Map());

	async function loadFetchCacheIds() {
		try {
			// Load summaries to get catalog item IDs, then resolve TMDB IDs via catalog items
			const res = await fetchRaw('/api/catalog/fetch-cache/summaries');
			if (res.ok) {
				const entries: Array<{ catalogItemId: string; scope: string; name: string }> = await res.json();
				const catalogItemIds = [...new Set(entries.map((e) => e.catalogItemId))];
				// Resolve catalog item IDs to TMDB IDs
				const tmdbIds = new Set<number>();
				for (const itemId of catalogItemIds) {
					const itemRes = await fetchRaw(`/api/catalog/items/${itemId}`);
					if (itemRes.ok) {
						const item = await itemRes.json();
						if (item.source === 'tmdb' && (item.kind === 'movie' || item.kind === 'tv_show')) {
							const tmdbId = Number(item.sourceId);
							tmdbIds.add(tmdbId);
							catalogToTmdbId.set(itemId, tmdbId);
						}
					}
				}
				fetchCachedTmdbIds = tmdbIds;
			}
		} catch { /* best-effort */ }
	}

	async function loadFetchCacheSummaries() {
		try {
			const res = await fetchRaw('/api/catalog/fetch-cache/summaries');
			if (res.ok) {
				const entries: Array<{ catalogItemId: string; scope: string; name: string }> = await res.json();
				const map = new Map<number, string>();
				for (const e of entries) {
					const tmdbId = catalogToTmdbId.get(e.catalogItemId);
					if (tmdbId !== undefined) map.set(tmdbId, e.name);
				}
				fetchCacheSummaries = map;
			}
		} catch { /* best-effort */ }
	}

	async function loadFetchCacheHashes() {
		try {
			const res = await fetchRaw('/api/catalog/fetch-cache/hashes');
			if (res.ok) {
				const entries: Array<{ catalogItemId: string; infoHash: string }> = await res.json();
				const map = new Map<number, string>();
				for (const e of entries) {
					const tmdbId = catalogToTmdbId.get(e.catalogItemId);
					if (tmdbId !== undefined) map.set(tmdbId, e.infoHash);
				}
				fetchCacheHashes = map;
			}
		} catch { /* best-effort */ }
	}

	// === Download statuses from active torrents ===
	let browseDownloadStatuses = $derived.by(() => {
		const torrents = $torrentState.allTorrents;
		if (torrents.length === 0 || fetchCacheHashes.size === 0) return new Map();
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const statuses = new Map<number, { state: TorrentInfo['state']; progress: number }>();
		for (const [tmdbId, infoHash] of fetchCacheHashes) {
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) statuses.set(tmdbId, { state: torrent.state, progress: torrent.progress });
		}
		return statuses;
	});

	// === Smart search handlers ===
	function handleBrowseSelectMovie(movie: DisplayTMDBMovie) {
		goto(`${base}/media/movies/${movie.id}`);
	}

	async function handleSmartSearch(movie: DisplayTMDBMovie) {
		smartSearchingId = movie.id;
		try {
			await smartSearchService.selectAndWaitForBest({
				title: movie.title, year: movie.releaseYear, type: 'movie', tmdbId: movie.id, mode: 'fetch'
			});
			await Promise.all([loadFetchCacheIds(), loadFetchCacheHashes(), loadFetchCacheSummaries()]);
		} finally {
			smartSearchingId = null;
		}
	}

	async function handleBatchSmartSearch() {
		const unsearched = pinnedMovies.filter((m) => !fetchCachedTmdbIds.has(m.id));
		if (unsearched.length === 0) return;
		batchSearching = true;
		batchProgress = { current: 0, total: unsearched.length };
		for (const movie of unsearched) {
			batchProgress = { current: batchProgress.current + 1, total: batchProgress.total };
			smartSearchingId = movie.id;
			try {
				await smartSearchService.selectAndWaitForBest({
					title: movie.title, year: movie.releaseYear, type: 'movie', tmdbId: movie.id, mode: 'fetch'
				});
				await Promise.all([loadFetchCacheIds(), loadFetchCacheHashes(), loadFetchCacheSummaries()]);
			} catch { /* continue */ }
		}
		smartSearchingId = null;
		batchSearching = false;
	}

	let pinnedUnsearchedCount = $derived(
		pinnedMovies.filter((m) => !fetchCachedTmdbIds.has(m.id)).length
	);

	// === Library items ===
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});
	let categoryOverrides: Record<string, string> = $state({});
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());
	const tmdbFailed = new Set<string>();

	function getItemLinks(item: MediaItem): Record<string, MediaItemLink> {
		const overrides = linkOverrides[item.id];
		if (!overrides) return item.links;
		const merged = { ...item.links };
		for (const [service, link] of Object.entries(overrides)) {
			if (link === null) delete merged[service];
			else merged[service] = link;
		}
		return merged;
	}

	let movieItems = $derived(
		Object.values(data.itemsByType).flat()
			.filter((i) => (data.libraries[i.libraryId]?.type ?? 'movies') === 'movies')
	);

	let itemsWithOverrides = $derived(
		movieItems.map((item) => {
			const linkOvr = linkOverrides[item.id];
			const catOvr = categoryOverrides[item.id];
			if (!linkOvr && catOvr === undefined) return item;
			const merged = { ...item.links };
			if (linkOvr) {
				for (const [service, link] of Object.entries(linkOvr)) {
					if (link === null) delete merged[service];
					else merged[service] = link;
				}
			}
			return { ...item, links: merged, ...(catOvr !== undefined ? { categoryId: catOvr } : {}) };
		})
	);

	function stableNumericId(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) hash = (hash * 31 + str.charCodeAt(i)) | 0;
		return Math.abs(hash);
	}

	function itemToDisplayMovie(item: MediaItem): DisplayTMDBMovie {
		const meta = tmdbMetadata[item.id];
		const tmdbLink = getItemLinks(item).tmdb;
		const tmdbId = tmdbLink ? Number(tmdbLink.serviceId) : null;
		const overrides = tmdbId ? movieImageOverrides.get(tmdbId) : null;
		return {
			id: stableNumericId(item.id),
			title: meta?.title ?? item.name,
			originalTitle: meta?.originalTitle ?? item.name,
			overview: meta?.overview ?? '',
			posterUrl: overrides?.poster ? getPosterUrl(overrides.poster) : (meta?.posterUrl ?? null),
			backdropUrl: overrides?.backdrop ? getBackdropUrl(overrides.backdrop) : (meta?.backdropUrl ?? null),
			releaseYear: meta?.releaseYear ?? '',
			voteAverage: meta?.voteAverage ?? 0,
			voteCount: meta?.voteCount ?? 0,
			genres: meta?.genres ?? []
		};
	}

	let libraryGroups = $derived.by(() => {
		const grouped = new Map<string, MediaItem[]>();
		for (const item of itemsWithOverrides) {
			const list = grouped.get(item.libraryId);
			if (list) list.push(item);
			else grouped.set(item.libraryId, [item]);
		}
		return Array.from(grouped.entries())
			.map(([libraryId, items]) => ({
				libraryId,
				name: data.libraries[libraryId]?.name ?? libraryId,
				movies: items.map(itemToDisplayMovie)
			}))
			.filter((g) => g.movies.length > 0);
	});

	let libraryItemsByMovieId = $derived(
		new Map(itemsWithOverrides.map((item) => [stableNumericId(item.id), item]))
	);

	let libraryMovieTmdbIds = $derived(
		itemsWithOverrides
			.map((item) => {
				const tmdbLink = getItemLinks(item).tmdb;
				return tmdbLink ? Number(tmdbLink.serviceId) : null;
			})
			.filter((id): id is number => id !== null)
	);

	let fetchedDisplayIds = $derived.by(() => {
		const ids = new Set<number>();
		for (const item of itemsWithOverrides) {
			const tmdbLink = getItemLinks(item).tmdb;
			if (tmdbLink && fetchCachedTmdbIds.has(Number(tmdbLink.serviceId))) {
				ids.add(stableNumericId(item.id));
			}
		}
		return ids;
	});

	let downloadStatuses = $derived.by(() => {
		const torrents = $torrentState.allTorrents;
		if (torrents.length === 0 || fetchCacheHashes.size === 0) return new Map();
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const statuses = new Map<number, { state: TorrentInfo['state']; progress: number }>();
		for (const item of itemsWithOverrides) {
			const tmdbLink = getItemLinks(item).tmdb;
			if (!tmdbLink) continue;
			const infoHash = fetchCacheHashes.get(Number(tmdbLink.serviceId));
			if (!infoHash) continue;
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) statuses.set(stableNumericId(item.id), { state: torrent.state, progress: torrent.progress });
		}
		return statuses;
	});

	function handleLibrarySelectMovie(movie: DisplayTMDBMovie) {
		const item = libraryItemsByMovieId.get(movie.id);
		if (!item) return;
		const tmdbLink = getItemLinks(item).tmdb;
		if (tmdbLink) goto(`${base}/media/movies/${tmdbLink.serviceId}`);
	}

	// === Media detail modal ===
	const playerState = playerService.state;
	const mediaDetailStore = mediaDetailService.store;
	let selectedItemId = $derived($mediaDetailStore?.item.id ?? null);

	function resolveCardType(item: MediaItem): MediaDetailCardType {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		return 'video';
	}

	function handleSelect(item: MediaItem) {
		mediaDetailService.select({
			item, cardType: resolveCardType(item),
			tmdbMetadata: tmdbMetadata[item.id] ?? null,
			youtubeMetadata: null, musicbrainzMetadata: null, imageTags: [],
			onplay: (i) => handlePlay(i),
			onlink: (i, service) => { linkModalItem = i; linkModalService = service; },
			onunlink: (i, service) => handleUnlink(i, service)
		});
		modalRouterService.openMediaDetail(item.mediaTypeId, item.categoryId ?? '', item.id);
	}

	$effect(() => {
		const sel = $mediaDetailStore;
		if (!sel) return;
		const updatedItem = itemsWithOverrides.find((i) => i.id === sel.item.id);
		if (!updatedItem) return;
		const newTmdb = tmdbMetadata[updatedItem.id] ?? null;
		if (newTmdb !== sel.tmdbMetadata || updatedItem !== sel.item) {
			mediaDetailService.select({ ...sel, item: updatedItem, cardType: resolveCardType(updatedItem), tmdbMetadata: newTmdb });
		}
	});

	function closeMediaDetail() {
		playerService.stop();
		mediaDetailService.clear();
		modalRouterService.closeMediaDetail();
	}

	onDestroy(() => { mediaDetailService.clear(); });

	const routerStore = modalRouterService.store;
	let deepLinkRestored = $state(false);
	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail || deepLinkRestored) return;
		deepLinkRestored = true;
		const allItems = Object.values(data.itemsByType).flat();
		const item = allItems.find((i) => i.id === detail.id);
		if (item) handleSelect(item);
	});
	$effect(() => {
		const detail = $routerStore.mediaDetail;
		if (!detail && $mediaDetailStore) mediaDetailService.clear();
	});

	// === Library operations ===
	function updateItemLinks(itemId: string, service: string, link: MediaItemLink | null) {
		linkOverrides = { ...linkOverrides, [itemId]: { ...linkOverrides[itemId], [service]: link } };
	}

	async function handleLink(tmdbId: number, seasonNumber: number | null, episodeNumber: number | null, type: 'movie' | 'tv') {
		if (!linkModalItem) return;
		const item = linkModalItem;
		const res = await fetchRaw(`/api/libraries/${item.libraryId}/items/${item.id}/tmdb`, {
			method: 'PUT', headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId, seasonNumber, episodeNumber })
		});
		if (res.ok) {
			updateItemLinks(item.id, 'tmdb', { serviceId: String(tmdbId), seasonNumber, episodeNumber });
			categoryOverrides = { ...categoryOverrides, [item.id]: 'movies' };
		}
		linkModalItem = null;
		linkModalService = null;
	}

	async function handleUnlink(item: MediaItem, service: string) {
		const res = await fetchRaw(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`, { method: 'DELETE' });
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
		playerService.play(playerAdapter.fromMediaItem(item));
	}

	function itemAsLibraryFile(item: MediaItem): LibraryFile {
		return {
			id: item.id, name: item.name, path: item.path, extension: item.extension,
			mediaType: item.mediaTypeId as LibraryFile['mediaType'],
			categoryId: item.categoryId, links: getItemLinks(item)
		};
	}

	async function fetchTmdbMetadata(item: MediaItem) {
		const tmdbLink = item.links.tmdb;
		if (!tmdbLink || tmdbMetadata[item.id] || tmdbLoading.has(item.id) || tmdbFailed.has(item.id)) return;
		tmdbLoading = new Set([...tmdbLoading, item.id]);
		try {
			const res = await fetchRaw(`/api/tmdb/movies/${tmdbLink.serviceId}`);
			if (res.ok) tmdbMetadata[item.id] = movieDetailsToDisplay(await res.json());
			else tmdbFailed.add(item.id);
		} catch { tmdbFailed.add(item.id); } finally {
			const next = new Set(tmdbLoading);
			next.delete(item.id);
			tmdbLoading = next;
		}
	}

	$effect(() => {
		for (const item of itemsWithOverrides) {
			if (item.links.tmdb) fetchTmdbMetadata(item);
		}
	});

	// === Init ===
	onMount(async () => {
		libraryService.initialize();
		loadMovieImageOverrides();
		loadFetchCacheIds();
		loadFetchCacheHashes();
		loadFetchCacheSummaries();
		loadPopularMovies();
		loadMovieGenres();
		loadDiscoverMovies();
	});
</script>

<Portal target={mediaBar.controlsTarget}>
	<form class="join" onsubmit={(e) => { e.preventDefault(); if (movieSearchInput.trim()) doSearchMovies(movieSearchInput.trim()); }}>
		<input type="text" placeholder="Search movies..." class="input join-item input-bordered input-sm w-48" bind:value={movieSearchInput} />
		<button type="submit" class="btn join-item btn-sm btn-primary">Search</button>
	</form>
	<button class="btn btn-ghost btn-sm" onclick={() => (recsModalOpen = true)}>Recs</button>
	<BrowseViewToggle />
</Portal>

{#if data.error}
	<div class="flex min-h-[50vh] items-center justify-center p-4">
		<div class="alert alert-error max-w-xl">
			<span class="font-mono text-sm whitespace-pre-wrap">{data.error}</span>
		</div>
	</div>
{:else}
	<div class="container mx-auto p-4">
			{#if pinnedMovies.length > 0}
				<section class="mb-8">
					<div class="mb-3 flex items-center gap-2">
						<h2 class="text-lg font-semibold">Pinned</h2>
						{#if pinnedUnsearchedCount > 0}
							<button class="btn btn-outline btn-xs" onclick={handleBatchSmartSearch} disabled={batchSearching}>
								{#if batchSearching}
									<span class="loading loading-xs loading-spinner"></span>
									{batchProgress.current}/{batchProgress.total}
								{:else}
									Smart Search All ({pinnedUnsearchedCount})
								{/if}
							</button>
						{/if}
					</div>
					<TmdbCatalogGrid
						movies={pinnedMovies}
						fetchedIds={fetchCachedTmdbIds}
						favoritedIds={favoritedTmdbIds}
						pinnedIds={pinnedTmdbIds}
						downloadStatuses={browseDownloadStatuses}
						{fetchCacheSummaries}
						{smartSearchingId}
						onselectMovie={handleBrowseSelectMovie}
						onsmartSearch={handleSmartSearch}
					/>
				</section>
			{/if}

			{#if favoritedMovies.length > 0}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">Favorites</h2>
					<TmdbCatalogGrid
						movies={applyOverridesToMovies(favoritedMovies)}
						fetchedIds={fetchCachedTmdbIds}
						favoritedIds={favoritedTmdbIds}
						pinnedIds={pinnedTmdbIds}
						downloadStatuses={browseDownloadStatuses}
						{fetchCacheSummaries}
						{smartSearchingId}
						onselectMovie={handleBrowseSelectMovie}
						onsmartSearch={handleSmartSearch}
					/>
				</section>
			{/if}

			{#if browseLoading['searchMovies']}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
					<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
				</section>
			{:else if searchMovies.length > 0}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
					<TmdbCatalogGrid
						movies={applyOverridesToMovies(searchMovies)}
						fetchedIds={fetchCachedTmdbIds}
						favoritedIds={favoritedTmdbIds}
						pinnedIds={pinnedTmdbIds}
						downloadStatuses={browseDownloadStatuses}
						{fetchCacheSummaries}
						{smartSearchingId}
						onselectMovie={handleBrowseSelectMovie}
						onsmartSearch={handleSmartSearch}
					/>
					{#if searchMoviesTotalPages > 1}
						<div class="mt-4 flex items-center justify-center gap-2">
							<button class="btn btn-ghost btn-sm" disabled={searchMoviesPage <= 1} onclick={() => doSearchMovies(searchQuery, searchMoviesPage - 1)}>Prev</button>
							<span class="text-sm opacity-60">{searchMoviesPage} / {searchMoviesTotalPages}</span>
							<button class="btn btn-ghost btn-sm" disabled={searchMoviesPage >= searchMoviesTotalPages} onclick={() => doSearchMovies(searchQuery, searchMoviesPage + 1)}>Next</button>
						</div>
					{/if}
				</section>
			{/if}

			{#each libraryGroups as group (group.libraryId)}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">{group.name}</h2>
					<TmdbCatalogGrid
						movies={group.movies}
						fetchedIds={fetchedDisplayIds}
						favoritedIds={favoritedTmdbIds}
						pinnedIds={pinnedTmdbIds}
						downloadStatuses={downloadStatuses}
						{fetchCacheSummaries}
						{smartSearchingId}
						onselectMovie={handleLibrarySelectMovie}
						onsmartSearch={handleSmartSearch}
					/>
				</section>
			{/each}

			<section class="mb-8">
				<div class="tabs tabs-bordered mb-4">
					<button class={classNames('tab', { 'tab-active': browseTab === 'popular' })} onclick={() => (browseTab = 'popular')}>Popular</button>
					<button class={classNames('tab', { 'tab-active': browseTab === 'discover' })} onclick={() => (browseTab = 'discover')}>Discover</button>
				</div>

				{#if browseTab === 'popular'}
					{#if browseLoading['popularMovies']}
						<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
					{:else if popularMovies.length > 0}
						<TmdbCatalogGrid
							movies={applyOverridesToMovies(popularMovies)}
							fetchedIds={fetchCachedTmdbIds}
							favoritedIds={favoritedTmdbIds}
							pinnedIds={pinnedTmdbIds}
							downloadStatuses={browseDownloadStatuses}
							{fetchCacheSummaries}
							{smartSearchingId}
							onselectMovie={handleBrowseSelectMovie}
							onsmartSearch={handleSmartSearch}
						/>
						{#if popularMoviesTotalPages > 1}
							<div class="mt-4 flex items-center justify-center gap-2">
								<button class="btn btn-ghost btn-sm" disabled={popularMoviesPage <= 1} onclick={() => loadPopularMovies(popularMoviesPage - 1)}>Prev</button>
								<span class="text-sm opacity-60">{popularMoviesPage} / {popularMoviesTotalPages}</span>
								<button class="btn btn-ghost btn-sm" disabled={popularMoviesPage >= popularMoviesTotalPages} onclick={() => loadPopularMovies(popularMoviesPage + 1)}>Next</button>
							</div>
						{/if}
					{/if}
				{:else}
					{#if movieGenres.length > 0}
						<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
							{#each movieGenres as genre (genre.id)}
								<button
									class={classNames('btn btn-sm h-auto min-h-12 flex-col py-2', {
										'btn-primary': selectedGenreId === genre.id,
										'btn-ghost bg-base-200': selectedGenreId !== genre.id
									})}
									onclick={() => {
										const genreId = selectedGenreId === genre.id ? null : genre.id;
										loadDiscoverMovies(1, genreId);
									}}
								>
									{genre.name}
								</button>
							{/each}
						</div>
					{/if}
					{#if browseLoading['discoverMovies']}
						<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
					{:else if discoverMovies.length > 0}
						<div class="mt-4">
							<TmdbCatalogGrid
								movies={applyOverridesToMovies(discoverMovies)}
								fetchedIds={fetchCachedTmdbIds}
								favoritedIds={favoritedTmdbIds}
								pinnedIds={pinnedTmdbIds}
								downloadStatuses={browseDownloadStatuses}
								{fetchCacheSummaries}
								{smartSearchingId}
								onselectMovie={handleBrowseSelectMovie}
								onsmartSearch={handleSmartSearch}
							/>
							{#if discoverMoviesTotalPages > 1}
								<div class="mt-4 flex items-center justify-center gap-2">
									<button class="btn btn-ghost btn-sm" disabled={discoverMoviesPage <= 1} onclick={() => loadDiscoverMovies(discoverMoviesPage - 1, selectedGenreId)}>Prev</button>
									<span class="text-sm opacity-60">{discoverMoviesPage} / {discoverMoviesTotalPages}</span>
									<button class="btn btn-ghost btn-sm" disabled={discoverMoviesPage >= discoverMoviesTotalPages} onclick={() => loadDiscoverMovies(discoverMoviesPage + 1, selectedGenreId)}>Next</button>
								</div>
							{/if}
						</div>
					{/if}
				{/if}
			</section>
		</div>

	<Modal open={!!$mediaDetailStore} maxWidth="max-w-lg" onclose={closeMediaDetail}>
		{#if $mediaDetailStore}
			<MediaDetail selection={$mediaDetailStore} onclose={closeMediaDetail} />
			{#if $playerState.currentFile && $playerState.currentFile.id !== $mediaDetailStore?.item.id}
				<div class="mt-4 border-t border-base-300 pt-4">
					<div class="mb-2 flex items-center justify-between">
						<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">Now Playing</h2>
						<button class="btn btn-square btn-ghost btn-xs" onclick={() => playerService.stop()} aria-label="Close player">&times;</button>
					</div>
					<p class="mb-2 truncate text-xs opacity-60" title={$playerState.currentFile.name}>{$playerState.currentFile.name}</p>
					<PlayerVideo
						file={$playerState.currentFile}
						connectionState={$playerState.connectionState}
						positionSecs={$playerState.positionSecs}
						durationSecs={$playerState.durationSecs}
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
			onclose={() => { linkModalItem = null; linkModalService = null; }}
		/>
	{/if}
{/if}

<Modal open={recsModalOpen} maxWidth="max-w-[90vw]" onclose={() => (recsModalOpen = false)}>
	{#if recsModalOpen}
		<div class="p-4">
			<RecommendationsModalContent
				mediaType="movie"
				pinnedIds={pinnedMovies.map((m) => m.id)}
				favoritedIds={favoritedMovies.map((m) => m.id)}
				libraryTmdbIds={libraryMovieTmdbIds}
			/>
		</div>
	{/if}
</Modal>


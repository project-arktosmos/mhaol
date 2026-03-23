<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { playerService } from 'ui-lib/services/player.service';
	import { playerAdapter } from 'ui-lib/adapters/classes/player.adapter';
	import { mediaDetailService } from 'ui-lib/services/media-detail.service';
	import { libraryService } from 'ui-lib/services/library.service';
	import { modalRouterService } from 'ui-lib/services/modal-router.service';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import type { MediaDetailCardType } from 'ui-lib/types/media-detail.type';
	import TmdbLinkModal from 'ui-lib/components/libraries/TmdbLinkModal.svelte';
	import LibraryTab from 'ui-lib/components/tmdb-browse/LibraryTab.svelte';
	import MediaDetail from 'ui-lib/components/media/MediaDetail.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import type { LibraryFile } from 'ui-lib/types/library.type';
	import type {
		MediaItem,
		MediaItemLink,
		MediaLinkSource,
		MediaCategory
	} from 'ui-lib/types/media-card.type';
	import type {
		DisplayTMDBMovie,
		DisplayTMDBTvShow,
		DisplayTMDBMovieDetails,
		DisplayTMDBTvShowDetails
	} from 'addons/tmdb/types';
	import {
		movieDetailsToDisplay,
		tvShowDetailsToDisplay,
		getPosterUrl,
		getBackdropUrl
	} from 'addons/tmdb/transform';
	import { tmdbBrowseService } from 'ui-lib/services/tmdb-browse.service';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { smartPairService } from 'ui-lib/services/smart-pair.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { browseDetailService } from 'ui-lib/services/browse-detail.service';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import SearchTab from 'ui-lib/components/tmdb-browse/SearchTab.svelte';
	import PopularTab from 'ui-lib/components/tmdb-browse/PopularTab.svelte';
	import classNames from 'classnames';
	import TmdbBrowseGrid from 'ui-lib/components/tmdb-browse/TmdbBrowseGrid.svelte';
	import TmdbPagination from 'ui-lib/components/tmdb-browse/TmdbPagination.svelte';
	import BrowseViewToggle from 'ui-lib/components/browse/BrowseViewToggle.svelte';

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

	// Pinned movies from smart pairing
	let pinnedMovies = $state<DisplayTMDBMovie[]>([]);

	// Image overrides: tmdbId -> { poster?: filePath, backdrop?: filePath }
	let movieImageOverrides = $state<Map<number, Record<string, string>>>(new Map());

	async function loadMovieImageOverrides() {
		try {
			const res = await fetch(apiUrl('/api/tmdb/image-overrides/movie'));
			if (res.ok) {
				const overrides: Array<{ tmdb_id: number; role: string; file_path: string }> =
					await res.json();
				const map = new Map<number, Record<string, string>>();
				for (const o of overrides) {
					const existing = map.get(o.tmdb_id) ?? {};
					existing[o.role] = o.file_path;
					map.set(o.tmdb_id, existing);
				}
				movieImageOverrides = map;
			}
		} catch {
			// best-effort
		}
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

	// TMDB browse state
	const browseState = tmdbBrowseService.state;

	// Browse detail selection state
	let selectedBrowseMovie: DisplayTMDBMovie | null = $state(null);
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let selectedBrowseTvShow: DisplayTMDBTvShow | null = $state<DisplayTMDBTvShow | null>(null);
	let browseMovieDetails: DisplayTMDBMovieDetails | null = $state(null);
	let browseTvShowDetails: DisplayTMDBTvShowDetails | null = $state(null);
	let browseDetailLoading = $state(false);
	let selectedLibraryItem: MediaItem | null = $state(null);
	let relatedData: LibraryItemRelated | null = $state(null);

	async function fetchRelatedData(itemId: string) {
		try {
			const res = await fetch(apiUrl(`/api/media/library-items/${itemId}/related`));
			if (res.ok) {
				relatedData = await res.json();
			}
		} catch {
			// best-effort
		}
	}

	async function handleBrowseSelectMovie(movie: DisplayTMDBMovie) {
		goto(`/movies/${movie.id}`);
	}

	async function checkFetchCacheForTmdbId(tmdbId: number, displayId?: number) {
		const cached = await smartSearchService.checkFetchCache(tmdbId);
		if (cached) {
			fetchingTmdbId = displayId ?? tmdbId;
			setSelectionForCurrentDetail();
			smartSearchService.setFetchedCandidate(cached);
		}
	}

	function setSelectionForCurrentDetail() {
		if (selectedBrowseMovie) {
			const realTmdbId = getRealTmdbId() ?? selectedBrowseMovie.id;
			smartSearchService.setSelection({
				title: selectedBrowseMovie.title,
				year: selectedBrowseMovie.releaseYear,
				type: 'movie',
				tmdbId: realTmdbId,
				mode: 'fetch',
				existingItemId: selectedLibraryItem?.id,
				existingLibraryId: selectedLibraryItem?.libraryId
			});
		}
	}

	function closeBrowseDetail() {
		selectedBrowseMovie = null;
		selectedBrowseTvShow = null;
		browseMovieDetails = null;
		browseTvShowDetails = null;
		selectedLibraryItem = null;
		relatedData = null;
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
	function getDetailTmdbId(): number | null {
		return selectedBrowseMovie?.id ?? null;
	}
	let currentDetailTmdbId = $derived(getDetailTmdbId());
	let isFetching = $derived(
		fetchingTmdbId !== null &&
			fetchingTmdbId === currentDetailTmdbId &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingTmdbId === currentDetailTmdbId
	);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) {
			return { terms: true, search: true, searching: false, eval: true, done: true };
		}
		const s = $searchStore;
		const hasResults = s.searchResults.length > 0;
		const hasAnalysis = s.searchResults.some((r) => r.analysis !== null);
		return {
			terms: s.selection !== null,
			search: !s.searching && hasResults,
			searching: s.searching,
			eval: hasAnalysis,
			done: s.fetchedCandidate !== null
		};
	});

	// Persist fetched candidate to cache when a new fetch completes
	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		const tmdbId = getRealTmdbId() ?? fetchingTmdbId;
		if (candidate && tmdbId) {
			const mediaType = selectedBrowseMovie ? 'movie' : 'tv';
			smartSearchService.saveFetchCache(tmdbId, mediaType, candidate);
			loadFetchCacheIds();
			loadFetchCacheHashes();
		}
	});

	// Download status for currently selected detail item, checked from multiple sources
	let currentDownloadStatus = $derived.by((): { state: string; progress: number } | null => {
		// Source 1: path-match library item against active torrents (most direct)
		if (selectedLibraryItem) {
			const t = findTorrentForItem(selectedLibraryItem);
			if (t) return { state: t.state, progress: t.progress };
		}
		// Source 2: relatedData from API (DB lookup via fetch cache)
		if (relatedData?.torrentDownload) {
			return { state: relatedData.torrentDownload.state, progress: relatedData.torrentDownload.progress };
		}
		// Source 3: TMDB ID → fetch cache hash → live SSE torrent state
		const tmdbId = getRealTmdbId() ?? currentDetailTmdbId;
		if (tmdbId) {
			const status = browseDownloadStatuses.get(tmdbId);
			if (status) return status;
		}
		return null;
	});

	// Sync browse detail state to the layout-level service
	$effect(() => {
		browseDetailService.set({
			domain: selectedBrowseMovie ? 'movie' : selectedBrowseTvShow ? 'tv' : null,
			movie: selectedBrowseMovie,
			tvShow: selectedBrowseTvShow,
			movieDetails: browseMovieDetails,
			tvShowDetails: browseTvShowDetails,
			libraryItem: selectedLibraryItem,
			relatedData,
			loading: browseDetailLoading,
			fetching: isFetching,
			fetched: isFetchedForCurrent,
			downloadStatus: currentDownloadStatus,
			fetchSteps: currentFetchSteps
		});
	});

	$effect(() => {
		browseDetailService.registerCallbacks({
			onfetch: handleBrowseDetailFetch,
			ondownload: handleBrowseDetailDownload,
			onstream: handleBrowseDetailStream,
			onp2pstream: handleBrowseDetailP2pStream,
			onshowsearch: () => smartSearchService.show(),
			onclose: closeBrowseDetail
		});
	});

	function getRealTmdbId(): number | null {
		if (selectedLibraryItem?.links.tmdb) return Number(selectedLibraryItem.links.tmdb.serviceId);
		return null;
	}

	async function handleBrowseDetailFetch() {
		const isRefetch = isFetchedForCurrent;

		if (selectedBrowseMovie) {
			const realTmdbId = getRealTmdbId() ?? selectedBrowseMovie.id;
			fetchingTmdbId = selectedBrowseMovie.id;
			if (!isRefetch) {
				const cached = await smartSearchService.checkFetchCache(realTmdbId);
				if (cached) {
					setSelectionForCurrentDetail();
					smartSearchService.setFetchedCandidate(cached);
					return;
				}
			}
			smartSearchService.select({
				title: selectedBrowseMovie.title,
				year: selectedBrowseMovie.releaseYear,
				type: 'movie',
				tmdbId: realTmdbId,
				mode: 'fetch',
				existingItemId: selectedLibraryItem?.id,
				existingLibraryId: selectedLibraryItem?.libraryId
			});
		} else if (selectedBrowseTvShow) {
			fetchingTmdbId = selectedBrowseTvShow.id;
			if (!isRefetch) {
				const cached = await smartSearchService.checkFetchCache(selectedBrowseTvShow.id);
				if (cached) {
					setSelectionForCurrentDetail();
					smartSearchService.setFetchedCandidate(cached);
					return;
				}
			}
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
		playerService.setDisplayMode('sidebar');
		handleStreamCandidate(candidate);
	}

	async function handleStreamCandidate(candidate: SmartSearchTorrentResult) {
		smartSearchService.hide();
		const infoHash = await smartSearchService.startStream(candidate);
		if (!infoHash) return;

		let ready = false;
		const unsubscribe = torrentService.state.subscribe(() => {
			if (!ready) return;
			const torrent = torrentService.findByHash(infoHash);
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
		ready = true;
	}

	function handleBrowseDetailP2pStream() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		const title = selectedBrowseMovie?.title ?? selectedBrowseTvShow?.name ?? '';

		// Check if the torrent is already downloaded and has a file path
		const existingTorrent = candidate.infoHash
			? torrentService.findByHash(candidate.infoHash)
			: null;

		if (existingTorrent?.outputPath && (existingTorrent.state === 'seeding' || existingTorrent.progress >= 1.0)) {
			// File is already on disk — start P2P WebRTC session directly
			const file: PlayableFile = {
				id: `p2p:${existingTorrent.infoHash}`,
				type: 'torrent',
				name: existingTorrent.name,
				outputPath: existingTorrent.outputPath,
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: null,
				durationSeconds: null,
				size: existingTorrent.size,
				completedAt: ''
			};
			// play() calls stop() which resets displayMode, so set it after
			playerService.play(file).then(() => playerService.setDisplayMode('sidebar'));
			return;
		}

		// Not yet downloaded — start torrent download and wait for completion
		playerService.prepareStream(title);
		playerService.setDisplayMode('sidebar');
		handleP2pStreamCandidate(candidate);
	}

	async function handleP2pStreamCandidate(candidate: SmartSearchTorrentResult) {
		smartSearchService.hide();
		const infoHash = await smartSearchService.startStream(candidate);
		if (!infoHash) return;

		let ready = false;
		const unsubscribe = torrentService.state.subscribe(() => {
			if (!ready) return;
			const torrent = torrentService.findByHash(infoHash);
			if (!torrent) return;

			smartSearchService.updateStreamingProgress(torrent.progress);

			if (torrent.progress >= 1.0 || torrent.state === 'seeding') {
				unsubscribe();
				smartSearchService.clearStreaming();

				const file: PlayableFile = {
					id: `p2p:${infoHash}`,
					type: 'torrent',
					name: torrent.name,
					outputPath: torrent.outputPath ?? '',
					mode: 'video',
					format: null,
					videoFormat: null,
					thumbnailUrl: null,
					durationSeconds: null,
					size: torrent.size,
					completedAt: ''
				};
				playerService.play(file).then(() => playerService.setDisplayMode('sidebar'));
			}
		});
		ready = true;
	}

	// Torrent state — match torrents to library items by path
	const torrentState = torrentService.state;

	function findTorrentForItem(item: MediaItem): TorrentInfo | null {
		const torrents = $torrentState.allTorrents;
		if (torrents.length === 0) return null;
		for (const t of torrents) {
			if (!t.outputPath) continue;
			if (item.path.startsWith(t.outputPath)) return t;
		}
		return null;
	}

	function handleLibrarySelectMovie(movie: DisplayTMDBMovie) {
		const item = libraryItemsByMovieId.get(movie.id);
		if (!item) return;
		const tmdbLink = getItemLinks(item).tmdb;
		if (tmdbLink) {
			goto(`/movies/${tmdbLink.serviceId}`);
		}
	}

	// Track link overrides so we can update without full page reload
	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});

	// Track category overrides so category changes are immediately reflected
	let categoryOverrides: Record<string, string> = $state({});

	// TMDB metadata state
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());

	// Fetch cache — tracks which TMDB IDs have been fetched
	let fetchCachedTmdbIds: Set<number> = $state(new Set());
	// Maps TMDB ID → infoHash for matching active torrent downloads
	let fetchCacheHashes: Map<number, string> = $state(new Map());

	async function loadFetchCacheIds() {
		try {
			const res = await fetch(apiUrl('/api/torrent/fetch-cache'));
			if (res.ok) {
				const ids: number[] = await res.json();
				fetchCachedTmdbIds = new Set(ids);
			}
		} catch {
			// best-effort
		}
	}

	async function loadFetchCacheHashes() {
		try {
			const res = await fetch(apiUrl('/api/torrent/fetch-cache/hashes'));
			if (res.ok) {
				const entries: Array<{ tmdbId: number; infoHash: string }> = await res.json();
				fetchCacheHashes = new Map(entries.map((e) => [e.tmdbId, e.infoHash]));
			}
		} catch {
			// best-effort
		}
	}

	onMount(async () => {
		libraryService.initialize();
		loadMovieImageOverrides();
		loadFetchCacheIds();
		loadFetchCacheHashes();
		initBrowseSections();
		smartPairService.loadPinned().then((pinned) => {
			pinnedMovies = pinned.movies;
		});
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
			.filter((i) => (data.libraries[i.libraryId]?.type ?? 'movies') === 'movies')
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

	// Stable unique numeric id per item (hash from item.id string)
	function stableNumericId(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			hash = (hash * 31 + str.charCodeAt(i)) | 0;
		}
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
			posterUrl: overrides?.poster
				? getPosterUrl(overrides.poster)
				: (meta?.posterUrl ?? null),
			backdropUrl: overrides?.backdrop
				? getBackdropUrl(overrides.backdrop)
				: (meta?.backdropUrl ?? null),
			releaseYear: meta?.releaseYear ?? '',
			voteAverage: meta?.voteAverage ?? 0,
			voteCount: meta?.voteCount ?? 0,
			genres: meta?.genres ?? []
		};
	}

	// Group items by library for per-library grids
	let libraryGroups = $derived.by(() => {
		const grouped = new Map<string, MediaItem[]>();
		for (const item of itemsWithOverrides) {
			const list = grouped.get(item.libraryId);
			if (list) {
				list.push(item);
			} else {
				grouped.set(item.libraryId, [item]);
			}
		}
		return Array.from(grouped.entries())
			.map(([libraryId, items]) => ({
				libraryId,
				name: data.libraries[libraryId]?.name ?? libraryId,
				movies: items.map(itemToDisplayMovie)
			}))
			.filter((g) => g.movies.length > 0);
	});

	// Map numeric id back to MediaItem for selection handling
	let libraryItemsByMovieId = $derived(
		new Map(itemsWithOverrides.map((item) => [stableNumericId(item.id), item]))
	);

	// No overrides needed — card reads backdropUrl directly from movie/tvShow prop

	// Set of display IDs for items that have a fetch cache entry
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

	// Download statuses: map display movie ID → { state, progress } from active torrents
	// Use allTorrents to include downloads outside the app-filtered path (e.g. /movies/ vs /server/)
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
			if (!torrent) continue;
			statuses.set(stableNumericId(item.id), {
				state: torrent.state,
				progress: torrent.progress
			});
		}
		return statuses;
	});

	// Download statuses for browse tabs (keyed by TMDB ID directly)
	let browseDownloadStatuses = $derived.by(() => {
		const torrents = $torrentState.allTorrents;
		if (torrents.length === 0 || fetchCacheHashes.size === 0) return new Map();
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const statuses = new Map<number, { state: TorrentInfo['state']; progress: number }>();
		for (const [tmdbId, infoHash] of fetchCacheHashes) {
			const torrent = torrentsByHash.get(infoHash);
			if (!torrent) continue;
			statuses.set(tmdbId, { state: torrent.state, progress: torrent.progress });
		}
		return statuses;
	});

	// Player state
	const playerState = playerService.state;
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

	function initBrowseSections() {
		tmdbBrowseService.loadPopularMovies();
		tmdbBrowseService.loadGenres();
		tmdbBrowseService.loadDiscoverMovies();
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

	const tmdbFailed = new Set<string>();

	async function fetchTmdbMetadata(item: MediaItem) {
		const tmdbLink = item.links.tmdb;
		if (!tmdbLink || tmdbMetadata[item.id] || tmdbLoading.has(item.id) || tmdbFailed.has(item.id))
			return;

		tmdbLoading = new Set([...tmdbLoading, item.id]);

		try {
			const res = await fetch(apiUrl(`/api/tmdb/movies/${tmdbLink.serviceId}`));
			if (res.ok) {
				const data = await res.json();
				tmdbMetadata[item.id] = movieDetailsToDisplay(data);
			} else {
				tmdbFailed.add(item.id);
			}
		} catch (e) {
			tmdbFailed.add(item.id);
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

{#if data.error}
<div class="flex min-h-[50vh] items-center justify-center p-4">
	<div class="alert alert-error max-w-xl">
		<span class="font-mono text-sm whitespace-pre-wrap">{data.error}</span>
	</div>
</div>
{:else}
<div class="relative min-w-0 flex-1 overflow-y-auto p-4">
		<div class="absolute right-3 top-3 z-10">
			<BrowseViewToggle />
		</div>
		<div class="container mx-auto">
			{#if pinnedMovies.length > 0}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">Pinned</h2>
					<TmdbBrowseGrid
						movies={pinnedMovies}
						selectedMovieId={selectedBrowseMovie?.id ?? null}
						fetchedIds={fetchCachedTmdbIds}
						downloadStatuses={browseDownloadStatuses}
						onselectMovie={handleBrowseSelectMovie}
					/>
				</section>
			{/if}

			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Search Movies</h2>
				<SearchTab
					movies={applyOverridesToMovies($browseState.searchMovies)}
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
					fetchedIds={fetchCachedTmdbIds}
					downloadStatuses={browseDownloadStatuses}
					onselectMovie={handleBrowseSelectMovie}
					onsearchMovies={(q, p) => tmdbBrowseService.searchMovies(q, p)}
					onsearchTv={(q, p) => tmdbBrowseService.searchTv(q, p)}
				/>
			</section>

			{#each libraryGroups as group (group.libraryId)}
				<section class="mb-8">
					<h2 class="mb-3 text-lg font-semibold">{group.name}</h2>
					<LibraryTab
						movies={group.movies}
						selectedMovieId={selectedBrowseMovie?.id ?? null}
						fetchedIds={fetchedDisplayIds}
						downloadStatuses={downloadStatuses}
						onselectMovie={handleLibrarySelectMovie}
					/>
				</section>
			{/each}

			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Popular Movies</h2>
				<PopularTab
					movies={applyOverridesToMovies($browseState.popularMovies)}
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
					fetchedIds={fetchCachedTmdbIds}
					downloadStatuses={browseDownloadStatuses}
					onselectMovie={handleBrowseSelectMovie}
					onloadMovies={(p) => tmdbBrowseService.loadPopularMovies(p)}
					onloadTv={(p) => tmdbBrowseService.loadPopularTv(p)}
				/>
			</section>

			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Discover Movies</h2>
				{#if $browseState.movieGenres.length > 0}
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
						{#each $browseState.movieGenres as genre (genre.id)}
							<button
								class={classNames('btn btn-sm h-auto min-h-12 flex-col py-2', {
									'btn-primary': $browseState.selectedGenreId === genre.id,
									'btn-ghost bg-base-200': $browseState.selectedGenreId !== genre.id
								})}
								onclick={() => {
									const genreId = $browseState.selectedGenreId === genre.id ? null : genre.id;
									tmdbBrowseService.loadDiscoverMovies(1, genreId);
								}}
							>
								{genre.name}
							</button>
						{/each}
					</div>
				{/if}
				{#if $browseState.loading['discoverMovies']}
					<div class="flex justify-center p-8">
						<span class="loading loading-lg loading-spinner"></span>
					</div>
				{:else if $browseState.discoverMovies.length > 0}
					<div class="mt-4">
						<TmdbBrowseGrid
							movies={applyOverridesToMovies($browseState.discoverMovies)}
							selectedMovieId={selectedBrowseMovie?.id ?? null}
							fetchedIds={fetchCachedTmdbIds}
							downloadStatuses={browseDownloadStatuses}
							onselectMovie={handleBrowseSelectMovie}
						/>
						<TmdbPagination
							page={$browseState.discoverMoviesPage}
							totalPages={$browseState.discoverMoviesTotalPages}
							loading={$browseState.loading['discoverMovies'] ?? false}
							onpage={(p) => tmdbBrowseService.loadDiscoverMovies(p, $browseState.selectedGenreId)}
						/>
					</div>
				{/if}
			</section>

			</div>
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
{/if}

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fetchRaw } from '$transport/fetch-helpers';
	import { movieDetailsToDisplay, getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import type { DisplayTMDBMovie, DisplayTMDBMovieDetails } from 'addons/tmdb/types';
	import type { MediaItem, MediaItemLink } from '$types/media-card.type';
	import type { MediaList } from '$types/media-list.type';
	import type { TorrentInfo } from '$types/torrent.type';
	import type { LibraryFile } from '$types/library.type';
	import { smartSearchService } from '$services/smart-search.service';
	import { torrentService } from '$services/torrent.service';
	import { favoritesService } from '$services/favorites.service';
	import { pinsService } from '$services/pins.service';
	import { playerService } from '$services/player.service';
	import { mediaDetailService } from '$services/media-detail.service';
	import { modalRouterService } from '$services/modal-router.service';
	import { libraryService } from '$services/library.service';
	import TmdbCatalogGrid from './TmdbCatalogGrid.svelte';
	import Modal from '$components/core/Modal.svelte';
	import MediaDetail from '$components/media/MediaDetail.svelte';
	import PlayerVideo from '$components/player/PlayerVideo.svelte';
	import TmdbLinkModal from '$components/libraries/TmdbLinkModal.svelte';
	import type { MediaDetailCardType } from '$types/media-detail.type';

	export interface MatchAllApi {
		matchAll: () => void;
		unlinkedCount: number;
		matchAllState: { total: number; completed: number; matched: number } | null;
	}

	interface Props {
		mediaData: {
			itemsByType: Record<string, MediaItem[]>;
			libraries: Record<string, { name: string; type: string }>;
		};
		imageOverrides: Map<number, Record<string, string>>;
		fetchCachedIds: Set<number>;
		fetchCacheHashes: Map<number, string>;
		fetchCacheSummaries: Map<number, string>;
		smartSearchingId: number | null;
		favoritedTmdbIds: Set<number>;
		pinnedTmdbIds: Set<number>;
		onnavigate: (tmdbId: string) => void;
		onsmartsearch?: (movie: DisplayTMDBMovie) => void;
		matchAllApi?: MatchAllApi;
	}

	let {
		mediaData,
		imageOverrides,
		fetchCachedIds,
		fetchCacheHashes,
		fetchCacheSummaries,
		smartSearchingId,
		favoritedTmdbIds,
		pinnedTmdbIds,
		onnavigate,
		onsmartsearch,
		matchAllApi = $bindable({ matchAll: () => {}, unlinkedCount: 0, matchAllState: null })
	}: Props = $props();

	const torrentState = torrentService.state;
	const playerState = playerService.state;
	const mediaDetailStore = mediaDetailService.store;
	let movieMatchAllState: { total: number; completed: number; matched: number } | null =
		$state(null);

	let linkOverrides: Record<string, Record<string, MediaItemLink | null>> = $state({});
	let categoryOverrides: Record<string, string> = $state({});
	let tmdbMetadata: Record<string, DisplayTMDBMovieDetails> = $state({});
	let tmdbLoading: Set<string> = $state(new Set());
	const tmdbFailed = new Set<string>();
	let linkModalItem: MediaItem | null = $state(null);
	let linkModalService: string | null = $state(null);

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
		Object.values(mediaData.itemsByType)
			.flat()
			.filter((i) => (mediaData.libraries[i.libraryId]?.type ?? 'movies') === 'movies')
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
			return {
				...item,
				links: merged,
				...(catOvr !== undefined ? { categoryId: catOvr } : {})
			};
		})
	);

	let unlinkedMovieItems = $derived(itemsWithOverrides.filter((i) => !getItemLinks(i).tmdb));

	async function handleMatchAllMovies() {
		if (unlinkedMovieItems.length === 0) return;
		movieMatchAllState = { total: unlinkedMovieItems.length, completed: 0, matched: 0 };
		try {
			const res = await fetchRaw('/api/libraries/auto-match', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					items: unlinkedMovieItems.map((i) => ({
						itemId: i.id,
						libraryId: i.libraryId,
						title: i.name
					}))
				})
			});
			if (!res.ok) {
				movieMatchAllState = null;
				return;
			}
			const reader = res.body?.getReader();
			if (!reader) {
				movieMatchAllState = null;
				return;
			}
			const decoder = new TextDecoder();
			let buffer = '';
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() ?? '';
				for (const line of lines) {
					if (!line.trim()) continue;
					try {
						const result = JSON.parse(line);
						movieMatchAllState = {
							total: movieMatchAllState!.total,
							completed: movieMatchAllState!.completed + 1,
							matched: movieMatchAllState!.matched + (result.matched ? 1 : 0)
						};
						if (result.matched && result.tmdbId) {
							updateItemLinks(result.itemId, 'tmdb', {
								serviceId: String(result.tmdbId),
								seasonNumber: null,
								episodeNumber: null
							});
							categoryOverrides = {
								...categoryOverrides,
								[result.itemId]: 'movies'
							};
						}
					} catch {
						/* skip */
					}
				}
			}
		} finally {
			setTimeout(() => {
				movieMatchAllState = null;
			}, 3000);
		}
	}

	$effect(() => {
		matchAllApi = {
			matchAll: handleMatchAllMovies,
			unlinkedCount: unlinkedMovieItems.length,
			matchAllState: movieMatchAllState
		};
	});

	function stableNumericId(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) hash = (hash * 31 + str.charCodeAt(i)) | 0;
		return Math.abs(hash);
	}

	function itemToDisplayMovie(item: MediaItem): DisplayTMDBMovie {
		const meta = tmdbMetadata[item.id];
		const tmdbLink = getItemLinks(item).tmdb;
		const tmdbId = tmdbLink ? Number(tmdbLink.serviceId) : null;
		const overrides = tmdbId ? imageOverrides.get(tmdbId) : null;
		return {
			id: stableNumericId(item.id),
			title: meta?.title ?? item.name,
			originalTitle: meta?.originalTitle ?? item.name,
			overview: meta?.overview ?? '',
			posterUrl: overrides?.poster ? getPosterUrl(overrides.poster) : (meta?.posterUrl ?? null),
			backdropUrl: overrides?.backdrop
				? getBackdropUrl(overrides.backdrop)
				: (meta?.backdropUrl ?? null),
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
				name: mediaData.libraries[libraryId]?.name ?? libraryId,
				movies: items.map(itemToDisplayMovie)
			}))
			.filter((g) => g.movies.length > 0);
	});

	let libraryItemsByMovieId = $derived(
		new Map(itemsWithOverrides.map((item) => [stableNumericId(item.id), item]))
	);

	let fetchedDisplayIds = $derived.by(() => {
		const ids = new Set<number>();
		for (const item of itemsWithOverrides) {
			const tmdbLink = getItemLinks(item).tmdb;
			if (tmdbLink && fetchCachedIds.has(Number(tmdbLink.serviceId))) {
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
			if (torrent)
				statuses.set(stableNumericId(item.id), {
					state: torrent.state,
					progress: torrent.progress
				});
		}
		return statuses;
	});

	function handleLibrarySelectMovie(movie: DisplayTMDBMovie) {
		const item = libraryItemsByMovieId.get(movie.id);
		if (!item) return;
		const tmdbLink = getItemLinks(item).tmdb;
		if (tmdbLink) {
			onnavigate(tmdbLink.serviceId);
		} else {
			linkModalItem = item;
			linkModalService = 'tmdb-movie';
		}
	}

	// TMDB metadata resolution
	async function fetchTmdbMetadata(item: MediaItem) {
		const tmdbLink = item.links.tmdb;
		if (!tmdbLink || tmdbMetadata[item.id] || tmdbLoading.has(item.id) || tmdbFailed.has(item.id))
			return;
		tmdbLoading = new Set([...tmdbLoading, item.id]);
		try {
			const res = await fetchRaw(`/api/tmdb/movies/${tmdbLink.serviceId}`);
			if (res.ok) tmdbMetadata[item.id] = movieDetailsToDisplay(await res.json());
			else tmdbFailed.add(item.id);
		} catch {
			tmdbFailed.add(item.id);
		} finally {
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

	// Link/unlink
	function updateItemLinks(itemId: string, service: string, link: MediaItemLink | null) {
		linkOverrides = {
			...linkOverrides,
			[itemId]: { ...linkOverrides[itemId], [service]: link }
		};
	}

	async function handleLink(
		tmdbId: number,
		seasonNumber: number | null,
		episodeNumber: number | null,
		_type: 'movie' | 'tv'
	) {
		if (!linkModalItem) return;
		const item = linkModalItem;
		const res = await fetchRaw(`/api/libraries/${item.libraryId}/items/${item.id}/tmdb`, {
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
		const res = await fetchRaw(`/api/libraries/${item.libraryId}/items/${item.id}/${service}`, {
			method: 'DELETE'
		});
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

	// Media detail modal
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
			onplay: () => {},
			onlink: (i, service) => {
				linkModalItem = i;
				linkModalService = service;
			},
			onunlink: (i, service) => handleUnlink(i, service)
		});
		modalRouterService.openMediaDetail(item.mediaTypeId, item.categoryId ?? '', item.id);
	}

	function closeMediaDetail() {
		playerService.stop();
		mediaDetailService.clear();
		modalRouterService.closeMediaDetail();
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

	onMount(() => {
		libraryService.initialize();
	});

	onDestroy(() => {
		mediaDetailService.clear();
	});
</script>

{#each libraryGroups as group (group.libraryId)}
	<section class="mb-8 px-4">
		<h2 class="mb-3 text-lg font-semibold">{group.name}</h2>
		<TmdbCatalogGrid
			movies={group.movies}
			fetchedIds={fetchedDisplayIds}
			favoritedIds={favoritedTmdbIds}
			pinnedIds={pinnedTmdbIds}
			{downloadStatuses}
			{fetchCacheSummaries}
			{smartSearchingId}
			onselectMovie={handleLibrarySelectMovie}
			onsmartSearch={onsmartsearch}
		/>
	</section>
{/each}

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
						aria-label="Close player">&times;</button
					>
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

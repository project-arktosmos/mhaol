<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { playerService } from 'ui-lib/services/player.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { movieDetailsToDisplay, getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import type { DisplayTMDBMovieDetails } from 'addons/tmdb/types';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import type { CatalogMovie } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import MovieDetailMeta from 'ui-lib/components/catalog/detail/MovieDetailMeta.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';

	let catalogItem = $state<CatalogMovie | null>(null);
	let movieDetails = $state<DisplayTMDBMovieDetails | null>(null);
	let libraryItem = $state<MediaItem | null>(null);
	let relatedData = $state<LibraryItemRelated | null>(null);
	let loading = $state(true);
	let fetchingTmdbId = $state<number | null>(null);
	let imageOverrides = $state<Record<string, string> | null>(null);

	const favState = favoritesService.state;
	const pinState = pinsService.state;
	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;
	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	let id = $derived($page.params.id ?? '');
	let tmdbId = $derived(Number(id));

	let isFavorite = $derived(
		$favState.items.some((f) => f.service === 'tmdb' && f.serviceId === String(tmdbId))
	);
	let isPinned = $derived(
		$pinState.items.some((p) => p.service === 'tmdb' && p.serviceId === String(tmdbId))
	);

	let isFetching = $derived(
		fetchingTmdbId !== null &&
			fetchingTmdbId === tmdbId &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingTmdbId === tmdbId
	);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) return { terms: true, search: true, searching: false, eval: true, done: true };
		const s = $searchStore;
		return {
			terms: s.selection !== null,
			search: !s.searching && s.searchResults.length > 0,
			searching: s.searching,
			eval: s.searchResults.some((r) => r.analysis !== null),
			done: s.fetchedCandidate !== null
		};
	});

	let matchedTorrent = $derived.by((): TorrentInfo | null => {
		const torrents = $torrentState.allTorrents;
		if (libraryItem) {
			for (const t of torrents) {
				if (t.outputPath && libraryItem.path.startsWith(t.outputPath)) return t;
			}
		}
		if (relatedData?.torrentDownload?.infoHash) {
			const t = torrentService.findByHash(relatedData.torrentDownload.infoHash);
			if (t) return t;
		}
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingTmdbId) {
			smartSearchService.saveFetchCache(fetchingTmdbId, 'movie', candidate);
		}
	});

	function buildCatalogItem(details: DisplayTMDBMovieDetails): CatalogMovie {
		return {
			id: String(details.id),
			kind: 'movie',
			title: details.title,
			sortTitle: details.title.toLowerCase(),
			year: details.releaseYear || null,
			overview: details.overview || null,
			posterUrl: details.posterUrl,
			backdropUrl: details.backdropUrl,
			voteAverage: details.voteAverage,
			voteCount: details.voteCount,
			parentId: null,
			position: null,
			source: 'tmdb',
			sourceId: String(details.id),
			createdAt: '',
			updatedAt: '',
			metadata: {
				tmdbId: details.id,
				originalTitle: details.originalTitle,
				runtime: details.runtime,
				director: details.director,
				cast: details.cast.map((c) => ({
					id: c.id,
					name: c.name,
					character: c.character,
					profileUrl: c.profileUrl
				})),
				genres: details.genres,
				tagline: details.tagline,
				budget: details.budget,
				revenue: details.revenue,
				imdbId: details.imdbId,
				images: details.images.map((img) => ({
					thumbnailUrl: img.thumbnailUrl,
					fullUrl: img.fullUrl,
					width: img.width,
					height: img.height,
					filePath: img.filePath,
					imageType: img.imageType
				})),
				imageOverrides: imageOverrides ?? {}
			}
		};
	}

	async function fetchMovie(showId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/tmdb/movies/${showId}`);
			if (res.ok) {
				const raw = await res.json();
				movieDetails = movieDetailsToDisplay(raw);
				catalogItem = buildCatalogItem(movieDetails);
			}

			await fetchLibraryItem(showId);
			await fetchImageOverrides(showId);

			if (catalogItem && imageOverrides) {
				catalogItem = {
					...catalogItem,
					posterUrl: imageOverrides.poster ? getPosterUrl(imageOverrides.poster) : catalogItem.posterUrl,
					backdropUrl: imageOverrides.backdrop ? getBackdropUrl(imageOverrides.backdrop) : catalogItem.backdropUrl,
					metadata: { ...catalogItem.metadata, imageOverrides: imageOverrides ?? {} }
				};
			}

			const cached = await smartSearchService.checkFetchCache(showId);
			if (cached) {
				fetchingTmdbId = showId;
				smartSearchService.setSelection({
					title: catalogItem?.title ?? '',
					year: catalogItem?.year ?? '',
					type: 'movie',
					tmdbId: showId,
					mode: 'fetch',
					existingItemId: libraryItem?.id,
					existingLibraryId: libraryItem?.libraryId
				});
				smartSearchService.setFetchedCandidate(cached);
			}
		} catch (e) {
			console.error('Failed to load movie details:', e);
		}
		loading = false;
	}

	async function fetchLibraryItem(showId: number) {
		try {
			const res = await fetchRaw('/api/media');
			if (!res.ok) return;
			const data = await res.json();
			const allItems: MediaItem[] = Object.values(data.itemsByType ?? {}).flat() as MediaItem[];
			const match = allItems.find((item) => item.links?.tmdb?.serviceId === String(showId));
			if (match) {
				libraryItem = match;
				const relRes = await fetchRaw(`/api/media/library-items/${match.id}/related`);
				if (relRes.ok) relatedData = await relRes.json();
			}
		} catch { /* best-effort */ }
	}

	async function fetchImageOverrides(showId: number) {
		try {
			const res = await fetchRaw(`/api/tmdb/image-overrides/movie/${showId}`);
			if (res.ok) {
				const overrides: Array<{ role: string; file_path: string }> = await res.json();
				const map: Record<string, string> = {};
				for (const o of overrides) map[o.role] = o.file_path;
				imageOverrides = Object.keys(map).length > 0 ? map : null;
			}
		} catch { /* best-effort */ }
	}

	async function handleFetch() {
		if (!catalogItem) return;
		const tid = Number(catalogItem.sourceId);
		fetchingTmdbId = tid;
		if (!isFetchedForCurrent) {
			const cached = await smartSearchService.checkFetchCache(tid);
			if (cached) {
				smartSearchService.setSelection({
					title: catalogItem.title, year: catalogItem.year ?? '', type: 'movie',
					tmdbId: tid, mode: 'fetch',
					existingItemId: libraryItem?.id, existingLibraryId: libraryItem?.libraryId
				});
				smartSearchService.setFetchedCandidate(cached);
				return;
			}
		}
		smartSearchService.select({
			title: catalogItem.title, year: catalogItem.year ?? '', type: 'movie',
			tmdbId: tid, mode: 'fetch',
			existingItemId: libraryItem?.id, existingLibraryId: libraryItem?.libraryId
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	function handleP2pStream() {
		const torrent = matchedTorrent;
		if (torrent?.outputPath && (torrent.state === 'seeding' || torrent.progress >= 1.0)) {
			const file: PlayableFile = {
				id: `p2p:${torrent.infoHash}`, type: 'torrent', name: torrent.name,
				outputPath: torrent.outputPath, mode: 'video', format: null,
				videoFormat: null, thumbnailUrl: null, durationSeconds: null,
				size: torrent.size, completedAt: ''
			};
			playerService.play(file, 'inline');
		}
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('tmdb', catalogItem.sourceId, catalogItem.title);
	}

	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('tmdb', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => fetchMovie(tmdbId));
</script>

{#if catalogItem}
	<CatalogDetailPage
		item={catalogItem}
		{loading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		{isFavorite}
		{isPinned}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onstream={handleP2pStream}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/movies`)}
		ontogglefavorite={handleToggleFavorite}
		ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			<MovieDetailMeta item={catalogItem!} />
		{/snippet}
		{#snippet cellB()}
			{#if $playerState.currentFile && $playerDisplayMode === 'inline'}
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<h2 class="text-sm font-semibold tracking-wide text-base-content/50 uppercase">Now Playing</h2>
						<button class="btn btn-square btn-ghost btn-xs" onclick={() => playerService.stop()} aria-label="Close player">&times;</button>
					</div>
					<p class="truncate text-xs opacity-60" title={$playerState.currentFile.name}>{$playerState.currentFile.name}</p>
					<PlayerVideo
						file={$playerState.currentFile}
						connectionState={$playerState.connectionState}
						positionSecs={$playerState.positionSecs}
						durationSecs={$playerState.durationSecs}
						buffering={$playerState.buffering}
					/>
				</div>
			{/if}
		{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Movie not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/movies`)}>Back to movies</button>
	</div>
{/if}

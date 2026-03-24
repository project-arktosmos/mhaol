<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { playerService } from 'ui-lib/services/player.service';
	import { movieDetailsToDisplay, getPosterUrl, getBackdropUrl } from 'addons/tmdb/transform';
	import type { DisplayTMDBMovie, DisplayTMDBMovieDetails } from 'addons/tmdb/types';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import MovieDetailPage from 'ui-lib/components/tmdb-browse/MovieDetailPage.svelte';

	let movie = $state<DisplayTMDBMovie | null>(null);
	let movieDetails = $state<DisplayTMDBMovieDetails | null>(null);
	let libraryItem = $state<MediaItem | null>(null);
	let relatedData = $state<LibraryItemRelated | null>(null);
	let loading = $state(true);
	let fetchingTmdbId = $state<number | null>(null);
	let imagesVisible = $state(false);
	let imageOverrides = $state<Record<string, string> | null>(null);

	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;

	let id = $derived($page.params.id ?? '');
	let tmdbId = $derived(Number(id));

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

	let matchedTorrent = $derived.by((): TorrentInfo | null => {
		const torrents = $torrentState.allTorrents;
		// 1. By library item path
		if (libraryItem) {
			for (const t of torrents) {
				if (t.outputPath && libraryItem.path.startsWith(t.outputPath)) return t;
			}
		}
		// 2. By related data info hash
		if (relatedData?.torrentDownload?.infoHash) {
			const t = torrentService.findByHash(relatedData.torrentDownload.infoHash);
			if (t) return t;
		}
		// 3. By fetched candidate hash
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	let currentDownloadStatus = $derived.by((): { state: string; progress: number } | null => {
		if (matchedTorrent) return { state: matchedTorrent.state, progress: matchedTorrent.progress };
		if (relatedData?.torrentDownload) {
			return { state: relatedData.torrentDownload.state, progress: relatedData.torrentDownload.progress };
		}
		return null;
	});

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		const tid = fetchingTmdbId;
		if (candidate && tid) {
			smartSearchService.saveFetchCache(tid, 'movie', candidate);
		}
	});

	async function fetchMovie(showId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			// Fetch details
			const res = await fetch(apiUrl(`/api/tmdb/movies/${showId}`));
			if (res.ok) {
				const raw = await res.json();
				movieDetails = movieDetailsToDisplay(raw);
				movie = {
					id: showId,
					title: movieDetails?.title ?? '',
					originalTitle: movieDetails?.originalTitle ?? '',
					posterUrl: movieDetails?.posterUrl ?? null,
					backdropUrl: movieDetails?.backdropUrl ?? null,
					overview: movieDetails?.overview ?? '',
					releaseYear: movieDetails?.releaseYear ?? '',
					voteAverage: movieDetails?.voteAverage ?? 0,
					voteCount: movieDetails?.voteCount ?? 0,
					genres: movieDetails?.genres ?? []
				};
			}

			// Look up library item and image overrides
			await fetchLibraryItem(showId);
			await fetchImageOverrides(showId);

			// Apply image overrides to display URLs
			if (movie && imageOverrides) {
				if (imageOverrides.poster) {
					movie = { ...movie, posterUrl: getPosterUrl(imageOverrides.poster) };
				}
				if (imageOverrides.backdrop) {
					movie = { ...movie, backdropUrl: getBackdropUrl(imageOverrides.backdrop) };
				}
			}

			imagesVisible = false;

			// Check fetch cache
			const cached = await smartSearchService.checkFetchCache(showId);
			if (cached) {
				fetchingTmdbId = showId;
				smartSearchService.setSelection({
					title: movie?.title ?? '',
					year: movie?.releaseYear ?? '',
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
			const res = await fetch(apiUrl('/api/media'));
			if (!res.ok) return;
			const data = await res.json();
			const allItems: MediaItem[] = Object.values(data.itemsByType ?? {}).flat() as MediaItem[];
			const match = allItems.find(
				(item) => item.links?.tmdb?.serviceId === String(showId)
			);
			if (match) {
				libraryItem = match;
				// Fetch related data for library item
				const relRes = await fetch(apiUrl(`/api/media/library-items/${match.id}/related`));
				if (relRes.ok) {
					relatedData = await relRes.json();
				}
			}
		} catch {
			// best-effort
		}
	}

	async function fetchImageOverrides(showId: number) {
		try {
			const res = await fetch(apiUrl(`/api/tmdb/image-overrides/movie/${showId}`));
			if (res.ok) {
				const overrides: Array<{ role: string; file_path: string }> = await res.json();
				const map: Record<string, string> = {};
				for (const o of overrides) {
					map[o.role] = o.file_path;
				}
				imageOverrides = Object.keys(map).length > 0 ? map : null;
			}
		} catch {
			// best-effort
		}
	}

	async function handleSetImageOverride(filePath: string, role: 'poster' | 'backdrop') {
		if (!movie) return;
		try {
			const res = await fetch(apiUrl(`/api/tmdb/image-overrides/movie/${movie.id}/${role}`), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ file_path: filePath })
			});
			if (res.ok) {
				imageOverrides = { ...imageOverrides, [role]: filePath };
				if (role === 'poster') {
					movie = { ...movie, posterUrl: getPosterUrl(filePath) };
				} else {
					movie = { ...movie, backdropUrl: getBackdropUrl(filePath) };
				}
			}
		} catch {
			// best-effort
		}
	}

	async function handleFetch() {
		if (!movie) return;
		const isRefetch = isFetchedForCurrent;
		fetchingTmdbId = movie.id;
		if (!isRefetch) {
			const cached = await smartSearchService.checkFetchCache(movie.id);
			if (cached) {
				smartSearchService.setSelection({
					title: movie.title,
					year: movie.releaseYear,
					type: 'movie',
					tmdbId: movie.id,
					mode: 'fetch',
					existingItemId: libraryItem?.id,
					existingLibraryId: libraryItem?.libraryId
				});
				smartSearchService.setFetchedCandidate(cached);
				return;
			}
		}
		smartSearchService.select({
			title: movie.title,
			year: movie.releaseYear,
			type: 'movie',
			tmdbId: movie.id,
			mode: 'fetch',
			existingItemId: libraryItem?.id,
			existingLibraryId: libraryItem?.libraryId
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	function handleP2pStream() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		const torrent = matchedTorrent;
		if (torrent?.outputPath && (torrent.state === 'seeding' || torrent.progress >= 1.0)) {
			const file: PlayableFile = {
				id: `p2p:${torrent.infoHash}`,
				type: 'torrent',
				name: torrent.name,
				outputPath: torrent.outputPath,
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
	}

	onMount(() => {
		fetchMovie(tmdbId);
	});
</script>

{#if movie}
	<MovieDetailPage
		{movie}
		{movieDetails}
		{libraryItem}
		{relatedData}
		{loading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		downloadStatus={currentDownloadStatus}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		{imagesVisible}
		{imageOverrides}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onp2pstream={handleP2pStream}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto('/movies')}
		ontoggleimages={() => { imagesVisible = true; }}
		onsetimageoverride={handleSetImageOverride}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Movie not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/movies')}>Back to movies</button>
	</div>
{/if}

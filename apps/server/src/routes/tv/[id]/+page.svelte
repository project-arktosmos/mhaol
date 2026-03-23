<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { playerService } from 'ui-lib/services/player.service';
	import { tvShowDetailsToDisplay, seasonDetailsToDisplay } from 'addons/tmdb/transform';
	import type {
		DisplayTMDBTvShow,
		DisplayTMDBTvShowDetails,
		DisplayTMDBSeasonDetails
	} from 'addons/tmdb/types';
	import type { SmartSearchTorrentResult, TvSeasonMeta } from 'ui-lib/types/smart-search.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import TvDetailPage from 'ui-lib/components/tmdb-browse/TvDetailPage.svelte';

	let tvShow = $state<DisplayTMDBTvShow | null>(null);
	let tvShowDetails = $state<DisplayTMDBTvShowDetails | null>(null);
	let tvSeasonDetailsList = $state<DisplayTMDBSeasonDetails[]>([]);
	let tvSeasonsMeta = $state<TvSeasonMeta[]>([]);
	let loading = $state(true);
	let fetchingTmdbId = $state<number | null>(null);

	const searchStore = smartSearchService.store;

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

	let currentDownloadStatus = $derived.by((): { state: string; progress: number } | null => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return { state: t.state, progress: t.progress };
		}
		return null;
	});

	let tvMatchedSeasons = $derived.by(() => {
		const tvResults = $searchStore.tvResults;
		if (!tvResults) return { hasComplete: false, seasons: new Map<number, Set<number>>() };
		const hasComplete = tvResults.complete.length > 0;
		const seasons = new Map<number, Set<number>>();
		for (const [snStr, data] of Object.entries(tvResults.seasons)) {
			const sn = Number(snStr);
			const eps = new Set<number>();
			if (data.seasonPacks.length > 0) eps.add(-1);
			for (const en of Object.keys(data.episodes).map(Number)) {
				if (data.episodes[en].length > 0) eps.add(en);
			}
			if (eps.size > 0) seasons.set(sn, eps);
		}
		return { hasComplete, seasons };
	});

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		const tid = fetchingTmdbId;
		if (candidate && tid) {
			const analysis = candidate.analysis;
			let scope = 'complete';
			let seasonNumber: number | null = null;
			let episodeNumber: number | null = null;
			if (analysis) {
				if (analysis.isCompleteSeries) {
					scope = 'complete';
				} else if (analysis.seasonNumber != null && analysis.episodeNumber != null) {
					scope = 'episode';
					seasonNumber = analysis.seasonNumber;
					episodeNumber = analysis.episodeNumber;
				} else if (analysis.seasonNumber != null) {
					scope = 'season';
					seasonNumber = analysis.seasonNumber;
				}
			}
			smartSearchService.saveTvFetchCache(tid, scope, seasonNumber, episodeNumber, candidate);
		}
	});

	async function fetchTvShow(showId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			// Fetch full details
			const res = await fetch(apiUrl(`/api/tmdb/tv/${showId}`));
			if (res.ok) {
				const raw = await res.json();
				tvShowDetails = tvShowDetailsToDisplay(raw);
				tvShow = {
					id: showId,
					name: tvShowDetails?.name ?? '',
					originalName: tvShowDetails?.originalName ?? '',
					posterUrl: tvShowDetails?.posterUrl ?? null,
					backdropUrl: tvShowDetails?.backdropUrl ?? null,
					overview: tvShowDetails?.overview ?? '',
					firstAirYear: tvShowDetails?.firstAirYear ?? '',
					lastAirYear: tvShowDetails?.lastAirYear ?? null,
					voteAverage: tvShowDetails?.voteAverage ?? 0,
					voteCount: tvShowDetails?.voteCount ?? 0,
					genres: tvShowDetails?.genres ?? [],
					numberOfSeasons: tvShowDetails?.numberOfSeasons ?? null,
					numberOfEpisodes: tvShowDetails?.numberOfEpisodes ?? null
				};

				// Fetch season details
				if (tvShowDetails?.seasons) {
					const seasonPromises = tvShowDetails.seasons
						.filter((s) => s.seasonNumber > 0)
						.map(async (s) => {
							try {
								const sRes = await fetch(apiUrl(`/api/tmdb/tv/${showId}/season/${s.seasonNumber}`));
								if (sRes.ok) {
									const sRaw = await sRes.json();
									return seasonDetailsToDisplay(sRaw);
								}
							} catch {
								// best-effort
							}
							return null;
						});
					const results = await Promise.all(seasonPromises);
					const details = results.filter((r): r is DisplayTMDBSeasonDetails => r !== null);
					tvSeasonDetailsList = details;
					tvSeasonsMeta = details.map((d) => ({
						seasonNumber: d.seasonNumber,
						name: d.name,
						episodeCount: d.episodes.length,
						episodes: d.episodes.map((ep) => ({
							episodeNumber: ep.episodeNumber,
							seasonNumber: ep.seasonNumber,
							name: ep.name
						}))
					}));
				}
			}

			// Check fetch cache
			const cached = await smartSearchService.checkTvFetchCache(showId);
			if (cached && cached.length > 0) {
				fetchingTmdbId = showId;
				const completeEntry = cached.find((e) => e.scope === 'complete');
				const bestEntry = completeEntry ?? cached[0];
				smartSearchService.setSelection({
					title: tvShow?.name ?? '',
					year: tvShow?.firstAirYear ?? '',
					type: 'tv',
					tmdbId: showId,
					mode: 'fetch',
					seasons: tvSeasonsMeta
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch (e) {
			console.error('Failed to load TV show details:', e);
		}
		loading = false;
	}

	async function handleFetch() {
		if (!tvShow) return;
		const isRefetch = isFetchedForCurrent;
		fetchingTmdbId = tvShow.id;
		if (!isRefetch) {
			const cached = await smartSearchService.checkTvFetchCache(tvShow.id);
			if (cached && cached.length > 0) {
				const completeEntry = cached.find((e) => e.scope === 'complete');
				const bestEntry = completeEntry ?? cached[0];
				smartSearchService.setSelection({
					title: tvShow.name,
					year: tvShow.firstAirYear,
					type: 'tv',
					tmdbId: tvShow.id,
					mode: 'fetch',
					seasons: tvSeasonsMeta
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: tvShow.name,
			year: tvShow.firstAirYear,
			type: 'tv',
			tmdbId: tvShow.id,
			mode: 'fetch',
			seasons: tvSeasonsMeta
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	function handleStream() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		playerService.prepareStream(tvShow?.name ?? '');
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

	function handleP2pStream() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		const existingTorrent = candidate.infoHash ? torrentService.findByHash(candidate.infoHash) : null;
		if (existingTorrent?.outputPath && (existingTorrent.state === 'seeding' || existingTorrent.progress >= 1.0)) {
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
			playerService.play(file).then(() => playerService.setDisplayMode('sidebar'));
			return;
		}
		playerService.prepareStream(tvShow?.name ?? '');
		playerService.setDisplayMode('sidebar');
		handleStreamCandidate(candidate);
	}

	onMount(() => {
		fetchTvShow(tmdbId);
	});
</script>

{#if tvShow}
	<TvDetailPage
		{tvShow}
		{tvShowDetails}
		tvSeasonDetails={tvSeasonDetailsList}
		{loading}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		downloadStatus={currentDownloadStatus}
		{tvMatchedSeasons}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onstream={handleStream}
		onp2pstream={handleP2pStream}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto('/tv')}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">TV show not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/tv')}>Back to TV</button>
	</div>
{/if}

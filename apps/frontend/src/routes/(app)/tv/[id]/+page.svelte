<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
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
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import TvDetailPage from 'ui-lib/components/tmdb-browse/TvDetailPage.svelte';
	import type { LibraryEpisodeFile } from 'ui-lib/components/tmdb-browse/TvDetailPage.svelte';

	let tvShow = $state<DisplayTMDBTvShow | null>(null);
	let tvShowDetails = $state<DisplayTMDBTvShowDetails | null>(null);
	let tvSeasonDetailsList = $state<DisplayTMDBSeasonDetails[]>([]);
	let tvSeasonsMeta = $state<TvSeasonMeta[]>([]);
	let loading = $state(true);
	let fetchingTmdbId = $state<number | null>(null);
	let libraryFiles = $state<LibraryEpisodeFile[]>([]);

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

	let matchedTorrent = $derived.by(() => {
		const candidate = $searchStore.fetchedCandidate;
		const _ = $torrentState;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
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

	function parseEpisodeFromFilename(name: string): { season: number; episode: number } | null {
		const match = name.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
		if (match) {
			return { season: parseInt(match[1], 10), episode: parseInt(match[2], 10) };
		}
		return null;
	}

	async function fetchLibraryData(showId: number) {
		try {
			const res = await fetch(apiUrl('/api/media'));
			if (!res.ok) return;
			const data = await res.json();
			const lists: MediaList[] = data.lists ?? [];

			// Find ALL show-level lists linked to this TMDB ID (multiple season folders may each be a top-level list)
			const showLists = lists.filter(
				(l) => l.parentListId === null && l.links?.tmdb?.serviceId === String(showId)
			);
			if (showLists.length === 0) return;

			const files: LibraryEpisodeFile[] = [];

			for (const showList of showLists) {
				// Flat show: episodes directly on the show list
				for (const item of showList.items) {
					const parsed = parseEpisodeFromFilename(item.name);
					if (parsed) {
						files.push({
							seasonNumber: parsed.season,
							episodeNumber: parsed.episode,
							name: item.name,
							path: item.path
						});
					}
				}

				// Season children
				const seasonLists = lists.filter((l) => l.parentListId === showList.id);
				for (const seasonList of seasonLists) {
					// Try to determine season number from the list's TMDB link or folder name
					const seasonNum = seasonList.links?.tmdb?.seasonNumber
						?? parseSeasonFromTitle(seasonList.title);

					for (const item of seasonList.items) {
						const parsed = parseEpisodeFromFilename(item.name);
						if (parsed) {
							files.push({
								seasonNumber: parsed.season,
								episodeNumber: parsed.episode,
								name: item.name,
								path: item.path
							});
						} else if (seasonNum != null) {
							// Can't parse episode from filename — assign position-based episode number
							const idx = seasonList.items.indexOf(item);
							files.push({
								seasonNumber: seasonNum,
								episodeNumber: idx + 1,
								name: item.name,
								path: item.path
							});
						}
					}
				}
			}

			libraryFiles = files;
		} catch {
			// best-effort
		}
	}

	function parseSeasonFromTitle(title: string): number | null {
		const match = title.match(/[Ss]eason\s*(\d+)/i);
		if (match) return parseInt(match[1], 10);
		const numMatch = title.match(/^(\d+)$/);
		if (numMatch) return parseInt(numMatch[1], 10);
		return null;
	}

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

			// Fetch library data (show list + season children)
			await fetchLibraryData(showId);

			// Only check smart search cache if no library files
			if (libraryFiles.length === 0) {
				const cached = await smartSearchService.checkTvFetchCache(showId);
				if (cached && cached.length > 0) {
					fetchingTmdbId = showId;
					const completeEntry = cached.find((e) => e.scope === 'complete');
					const bestEntry = completeEntry ?? cached[0];
					const sel = {
						title: tvShow?.name ?? '',
						year: tvShow?.firstAirYear ?? '',
						type: 'tv' as const,
						tmdbId: showId,
						mode: 'fetch' as const,
						seasons: tvSeasonsMeta
					};
					smartSearchService.setSelection(sel);
					smartSearchService.setFetchedCandidate(bestEntry.candidate);
					smartSearchService.ensurePendingItem(sel);
				}
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
				const sel = {
					title: tvShow.name,
					year: tvShow.firstAirYear,
					type: 'tv' as const,
					tmdbId: tvShow.id,
					mode: 'fetch' as const,
					seasons: tvSeasonsMeta
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				smartSearchService.ensurePendingItem(sel);
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
			playerService.play(file, 'inline');
		}
	}

	function handlePlayFile(libFile: LibraryEpisodeFile) {
		const file: PlayableFile = {
			id: `library:${libFile.path}`,
			type: 'library',
			name: libFile.name,
			outputPath: libFile.path,
			mode: 'video',
			format: null,
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: null,
			size: 0,
			completedAt: ''
		};
		playerService.play(file, 'inline');
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
		torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		{tvMatchedSeasons}
		{libraryFiles}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onp2pstream={handleP2pStream}
		onshowsearch={() => smartSearchService.show()}
		onplayfile={handlePlayFile}
		onback={() => goto(`${base}/tv`)}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">TV show not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/tv`)}>Back to TV</button>
	</div>
{/if}

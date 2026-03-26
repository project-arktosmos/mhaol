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
	import { tvShowDetailsToDisplay, seasonDetailsToDisplay } from 'addons/tmdb/transform';
	import type { DisplayTMDBTvShowDetails, DisplayTMDBSeasonDetails } from 'addons/tmdb/types';
	import type { TvSeasonMeta } from 'ui-lib/types/smart-search.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import type { CatalogTvShow } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import TvDetailMeta from 'ui-lib/components/catalog/detail/TvDetailMeta.svelte';

	interface LibraryEpisodeFile {
		seasonNumber: number;
		episodeNumber: number;
		name: string;
		path: string;
	}

	let catalogItem = $state<CatalogTvShow | null>(null);
	let tvSeasonsMeta = $state<TvSeasonMeta[]>([]);
	let loading = $state(true);
	let fetchingTmdbId = $state<number | null>(null);
	let libraryFiles = $state<LibraryEpisodeFile[]>([]);
	let resyncing = $state(false);

	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let id = $derived($page.params.id ?? '');
	let tmdbId = $derived(Number(id));

	let isFavorite = $derived(
		$favState.items.some((f) => f.service === 'tmdb-tv' && f.serviceId === String(tmdbId))
	);
	let isPinned = $derived(
		$pinState.items.some((p) => p.service === 'tmdb-tv' && p.serviceId === String(tmdbId))
	);

	let isFetching = $derived(
		fetchingTmdbId !== null && fetchingTmdbId === tmdbId &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
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

	let matchedTorrent = $derived.by(() => {
		const candidate = $searchStore.fetchedCandidate;
		const _ = $torrentState;
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingTmdbId) {
			const analysis = candidate.analysis;
			let scope = 'complete';
			let seasonNumber: number | null = null;
			let episodeNumber: number | null = null;
			if (analysis) {
				if (analysis.isCompleteSeries) scope = 'complete';
				else if (analysis.seasonNumber != null && analysis.episodeNumber != null) {
					scope = 'episode'; seasonNumber = analysis.seasonNumber; episodeNumber = analysis.episodeNumber;
				} else if (analysis.seasonNumber != null) {
					scope = 'season'; seasonNumber = analysis.seasonNumber;
				}
			}
			smartSearchService.saveTvFetchCache(fetchingTmdbId, scope, seasonNumber, episodeNumber, candidate);
		}
	});

	function buildCatalogItem(details: DisplayTMDBTvShowDetails, seasonDetails: DisplayTMDBSeasonDetails[]): CatalogTvShow {
		return {
			id: String(details.id), kind: 'tv_show',
			title: details.name, sortTitle: details.name.toLowerCase(),
			year: details.firstAirYear || null, overview: details.overview || null,
			posterUrl: details.posterUrl, backdropUrl: details.backdropUrl,
			voteAverage: details.voteAverage, voteCount: details.voteCount,
			parentId: null, position: null,
			source: 'tmdb', sourceId: String(details.id),
			createdAt: '', updatedAt: '',
			metadata: {
				tmdbId: details.id, originalName: details.originalName,
				lastAirYear: details.lastAirYear, status: details.status,
				networks: details.networks, createdBy: details.createdBy,
				cast: details.cast.map((c) => ({ id: c.id, name: c.name, character: c.character, profileUrl: c.profileUrl })),
				genres: details.genres, tagline: details.tagline,
				numberOfSeasons: details.numberOfSeasons, numberOfEpisodes: details.numberOfEpisodes,
				seasons: (details.seasons ?? []).map((s) => ({
					id: s.id, name: s.name, overview: s.overview, airDate: s.airDate,
					episodeCount: s.episodeCount, posterUrl: s.posterUrl, seasonNumber: s.seasonNumber
				})),
				images: details.images.map((img) => ({
					thumbnailUrl: img.thumbnailUrl, fullUrl: img.fullUrl,
					width: img.width, height: img.height, filePath: img.filePath, imageType: img.imageType
				})),
				imageOverrides: {}
			}
		};
	}

	function parseEpisodeFromFilename(name: string): { season: number; episode: number } | null {
		const match = name.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
		return match ? { season: parseInt(match[1], 10), episode: parseInt(match[2], 10) } : null;
	}

	function parseSeasonFromTitle(title: string): number | null {
		const match = title.match(/[Ss]eason\s*(\d+)/i);
		if (match) return parseInt(match[1], 10);
		const numMatch = title.match(/^(\d+)$/);
		return numMatch ? parseInt(numMatch[1], 10) : null;
	}

	async function fetchLibraryData(showId: number) {
		try {
			const res = await fetchRaw('/api/media');
			if (!res.ok) return;
			const data = await res.json();
			const lists: MediaList[] = data.lists ?? [];
			const showLists = lists.filter((l) => l.parentListId === null && l.links?.tmdb?.serviceId === String(showId));
			if (showLists.length === 0) return;
			const files: LibraryEpisodeFile[] = [];
			for (const showList of showLists) {
				for (const item of showList.items) {
					const parsed = parseEpisodeFromFilename(item.name);
					if (parsed) files.push({ seasonNumber: parsed.season, episodeNumber: parsed.episode, name: item.name, path: item.path });
				}
				const seasonLists = lists.filter((l) => l.parentListId === showList.id);
				for (const seasonList of seasonLists) {
					const seasonNum = seasonList.links?.tmdb?.seasonNumber ?? parseSeasonFromTitle(seasonList.title);
					for (const item of seasonList.items) {
						const parsed = parseEpisodeFromFilename(item.name);
						if (parsed) files.push({ ...parsed, seasonNumber: parsed.season, episodeNumber: parsed.episode, name: item.name, path: item.path });
						else if (seasonNum != null) {
							const idx = seasonList.items.indexOf(item);
							files.push({ seasonNumber: seasonNum, episodeNumber: idx + 1, name: item.name, path: item.path });
						}
					}
				}
			}
			libraryFiles = files;
		} catch { /* best-effort */ }
	}

	async function fetchTvShow(showId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/tmdb/tv/${showId}`);
			if (res.ok) {
				const raw = await res.json();
				const details = tvShowDetailsToDisplay(raw);
				let seasonDetailsList: DisplayTMDBSeasonDetails[] = [];
				if (details?.seasons) {
					const results = await Promise.all(
						details.seasons.filter((s) => s.seasonNumber > 0).map(async (s) => {
							try {
								const sRes = await fetchRaw(`/api/tmdb/tv/${showId}/season/${s.seasonNumber}`);
								if (sRes.ok) return seasonDetailsToDisplay(await sRes.json());
							} catch { /* best-effort */ }
							return null;
						})
					);
					seasonDetailsList = results.filter((r): r is DisplayTMDBSeasonDetails => r !== null);
					tvSeasonsMeta = seasonDetailsList.map((d) => ({
						seasonNumber: d.seasonNumber, name: d.name, episodeCount: d.episodes.length,
						episodes: d.episodes.map((ep) => ({ episodeNumber: ep.episodeNumber, seasonNumber: ep.seasonNumber, name: ep.name }))
					}));
				}
				catalogItem = buildCatalogItem(details, seasonDetailsList);
			}
			await fetchLibraryData(showId);
			if (libraryFiles.length === 0) {
				const cached = await smartSearchService.checkTvFetchCache(showId);
				if (cached && cached.length > 0) {
					fetchingTmdbId = showId;
					const bestEntry = cached.find((e) => e.scope === 'complete') ?? cached[0];
					const sel = { title: catalogItem?.title ?? '', year: catalogItem?.year ?? '', type: 'tv' as const, tmdbId: showId, mode: 'fetch' as const, seasons: tvSeasonsMeta };
					smartSearchService.setSelection(sel);
					smartSearchService.setFetchedCandidate(bestEntry.candidate);
					smartSearchService.ensurePendingItem(sel);
				}
			}
		} catch (e) { console.error('Failed to load TV show:', e); }
		loading = false;
	}

	async function handleFetch() {
		if (!catalogItem) return;
		const tid = Number(catalogItem.sourceId);
		fetchingTmdbId = tid;
		if (!isFetchedForCurrent) {
			const cached = await smartSearchService.checkTvFetchCache(tid);
			if (cached && cached.length > 0) {
				const bestEntry = cached.find((e) => e.scope === 'complete') ?? cached[0];
				const sel = { title: catalogItem.title, year: catalogItem.year ?? '', type: 'tv' as const, tmdbId: tid, mode: 'fetch' as const, seasons: tvSeasonsMeta };
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				smartSearchService.ensurePendingItem(sel);
				return;
			}
		}
		smartSearchService.select({ title: catalogItem.title, year: catalogItem.year ?? '', type: 'tv', tmdbId: tid, mode: 'fetch', seasons: tvSeasonsMeta });
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
				size: torrent.size, completedAt: '',
				infoHash: torrent.infoHash
			};
			playerService.play(file, 'inline');
		}
	}

	function handlePlayFile(file: LibraryEpisodeFile) {
		const pf: PlayableFile = {
			id: `library:${file.path}`, type: 'library', name: file.name,
			outputPath: file.path, mode: 'video', format: null,
			videoFormat: null, thumbnailUrl: null, durationSeconds: null, size: 0, completedAt: ''
		};
		playerService.play(pf, 'inline');
	}

	async function handleResync() {
		resyncing = true;
		try {
			const res = await fetchRaw('/api/media');
			if (!res.ok) return;
			const data = await res.json();
			const lists: MediaList[] = data.lists ?? [];
			const showLists = lists.filter((l: MediaList) => l.parentListId === null && l.links?.tmdb?.serviceId === String(tmdbId));
			const libraryIds = [...new Set(showLists.map((l: MediaList) => l.libraryId))];
			await Promise.all(libraryIds.map((lid: string) => fetchRaw(`/api/libraries/${lid}/scan`, { method: 'POST' })));
			await fetchLibraryData(tmdbId);
		} finally { resyncing = false; }
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('tmdb-tv', catalogItem.sourceId, catalogItem.title);
	}
	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('tmdb-tv', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => fetchTvShow(tmdbId));
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
			? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' }
			: null}
		{isFavorite}
		{isPinned}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onstream={handleP2pStream}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/tv`)}
		ontogglefavorite={handleToggleFavorite}
		ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			<TvDetailMeta item={catalogItem!} />
			{#if libraryFiles.length > 0}
				<div>
					<div class="flex items-center justify-between">
						<h3 class="text-xs font-semibold tracking-wide uppercase opacity-50">Library Files ({libraryFiles.length})</h3>
						<button class="btn btn-ghost btn-xs" onclick={handleResync} disabled={resyncing}>
							{resyncing ? 'Syncing...' : 'Resync'}
						</button>
					</div>
					<div class="mt-1 flex flex-col gap-0.5">
						{#each libraryFiles as file}
							<button
								class="flex items-center justify-between rounded p-1.5 text-left text-sm hover:bg-base-200"
								onclick={() => handlePlayFile(file)}
							>
								<span class="truncate">{file.name}</span>
								<span class="badge badge-ghost badge-xs">S{String(file.seasonNumber).padStart(2, '0')}E{String(file.episodeNumber).padStart(2, '0')}</span>
							</button>
						{/each}
					</div>
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
		<p class="text-sm opacity-60">TV show not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/tv`)}>Back to TV</button>
	</div>
{/if}

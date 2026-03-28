<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { error } from '@sveltejs/kit';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { getMediaConfig } from 'ui-lib/data/media-registry';
	import type { MediaTypeConfig } from 'ui-lib/data/media-registry';
	import type {
		CatalogItem,
		CatalogBook,
		CatalogGame,
		CatalogMovie,
		CatalogTvShow
	} from 'ui-lib/types/catalog.type';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import type { TvSeasonMeta } from 'ui-lib/types/smart-search.type';
	import { playerService } from 'ui-lib/services/player.service';

	// Detail components
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import BookDetailMeta from 'ui-lib/components/catalog/detail/BookDetailMeta.svelte';
	import GameDetailMeta from 'ui-lib/components/catalog/detail/GameDetailMeta.svelte';
	import MovieDetailMeta from 'ui-lib/components/catalog/detail/MovieDetailMeta.svelte';
	import TvDetailMeta from 'ui-lib/components/catalog/detail/TvDetailMeta.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import IptvChannelDetail from 'ui-lib/components/iptv/IptvChannelDetail.svelte';

	// Type-specific transforms
	import { workToDisplayDetails, authorToDisplay, getCoverUrl } from 'addons/openlibrary/transform';
	import type { OpenLibraryWork, OpenLibraryAuthor } from 'addons/openlibrary/types';
	import { gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameExtended } from 'addons/retroachievements/types';
	import {
		movieDetailsToDisplay,
		tvShowDetailsToDisplay,
		seasonDetailsToDisplay,
		getPosterUrl,
		getBackdropUrl
	} from 'addons/tmdb/transform';
	import type {
		DisplayTMDBMovieDetails,
		DisplayTMDBSeasonDetails
	} from 'addons/tmdb/types';
	import { iptvService } from 'ui-lib/services/iptv.service';

	let slug = $derived($page.params.slug ?? '');
	let id = $derived($page.params.id ?? '');
	let config = $derived(getMediaConfig(slug));

	// Shared state
	let catalogItem = $state<CatalogItem | null>(null);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let isFavorite = $derived(
		config ? $favState.items.some((f) => f.service === config.favService && f.serviceId === id) : false
	);
	let isPinned = $derived(
		config ? $pinState.items.some((p) => p.service === config.pinService && p.serviceId === id) : false
	);

	// Smart search state (shared across catalog types)
	let isFetching = $derived(
		fetchingId !== null && fetchingId === id &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingId === id
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
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	// IPTV-specific state
	let iptvChannel = $state<IptvChannel | null>(null);
	let iptvStreams = $state<IptvStream[]>([]);
	let iptvStreamUrl = $state('');
	let iptvEpgPrograms = $state<IptvEpgProgram[]>([]);
	let iptvEpgAvailable = $state(false);
	let togglingFavorite = $state(false);
	let togglingPin = $state(false);

	// Movie-specific state
	let libraryItem = $state<MediaItem | null>(null);
	let relatedData = $state<LibraryItemRelated | null>(null);
	let movieImageOverrides = $state<Record<string, string> | null>(null);
	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	// TV-specific state
	let tvSeasonsMeta = $state<TvSeasonMeta[]>([]);
	let libraryFiles = $state<Array<{ seasonNumber: number; episodeNumber: number; name: string; path: string }>>([]);
	let resyncing = $state(false);

	// === Per-type fetch functions ===

	async function fetchBook(bookKey: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const workRes = await fetchRaw(`/api/openlibrary/works/${bookKey}`);
			if (!workRes.ok) throw new Error('Failed to fetch');
			const work: OpenLibraryWork = await workRes.json();
			const coverId = work.covers?.[0] ?? null;
			const authorKeys = work.authors?.map((a) => a.author.key.replace('/authors/', '')) ?? [];
			const authors = await Promise.all(
				authorKeys.slice(0, 3).map(async (key) => {
					try {
						const res = await fetchRaw(`/api/openlibrary/authors/${key}`);
						if (res.ok) return authorToDisplay(await res.json() as OpenLibraryAuthor);
					} catch { /* best-effort */ }
					return { key, name: 'Unknown', birthDate: null, deathDate: null, bio: null, photoUrl: null };
				})
			);
			const details = workToDisplayDetails(work, authors, {
				key: bookKey, title: work.title, authors: authorKeys, authorKeys,
				firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
				coverId, coverUrl: getCoverUrl(coverId, 'M'),
				subjects: (work.subjects ?? []).slice(0, 10), publishers: [],
				pageCount: null, editionCount: 0, isbn: null, ratingsAverage: null, ratingsCount: 0
			});
			catalogItem = {
				id: bookKey, kind: 'book',
				title: work.title, sortTitle: work.title.toLowerCase(),
				year: work.first_publish_date?.split('-')[0] ?? null,
				overview: details?.description ?? null,
				posterUrl: getCoverUrl(coverId, 'M'), backdropUrl: null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'openlibrary', sourceId: bookKey,
				createdAt: '', updatedAt: '',
				metadata: {
					openlibraryKey: bookKey, authors: authors.map((a) => a.name), authorKeys,
					firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
					coverId, coverUrl: getCoverUrl(coverId, 'M'),
					subjects: (work.subjects ?? []).slice(0, 10), publishers: [],
					pageCount: null, editionCount: 0, isbn: null,
					ratingsAverage: null, ratingsCount: 0,
					description: details?.description ?? null, authorDetails: authors
				}
			};
			const cached = await smartSearchService.checkBookFetchCache(bookKey);
			if (cached) {
				fetchingId = bookKey;
				const sel = {
					title: work.title, year: catalogItem.year ?? '',
					type: 'book' as const, openlibraryKey: bookKey,
					author: authors[0]?.name ?? 'Unknown', mode: 'fetch' as const
				};
				smartSearchService.setSelection(sel);
				smartSearchService.setFetchedCandidate(cached);
				smartSearchService.ensurePendingItem(sel);
			}
		} catch { catalogItem = null; }
		loading = false;
	}

	async function fetchGame(gameId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/retroachievements/games/${gameId}`);
			if (!res.ok) throw new Error('Failed to fetch');
			const detail = gameExtendedToDisplay(await res.json() as RaGameExtended);
			catalogItem = {
				id: String(detail.id), kind: 'game',
				title: detail.title, sortTitle: detail.title.toLowerCase(),
				year: detail.released ?? null, overview: null,
				posterUrl: detail.imageBoxArtUrl ?? detail.imageIconUrl, backdropUrl: detail.imageTitleUrl ?? null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'retroachievements', sourceId: String(detail.id),
				createdAt: '', updatedAt: '',
				metadata: {
					retroachievementsId: detail.id, consoleId: detail.consoleId,
					consoleName: detail.consoleName, imageIconUrl: detail.imageIconUrl,
					numAchievements: detail.numAchievements, points: detail.points,
					developer: detail.developer ?? null, publisher: detail.publisher ?? null,
					genre: detail.genre ?? null, released: detail.released ?? null,
					imageTitleUrl: detail.imageTitleUrl ?? null,
					imageIngameUrl: detail.imageIngameUrl ?? null,
					imageBoxArtUrl: detail.imageBoxArtUrl ?? null,
					achievements: (detail.achievements ?? []).map((a) => ({
						id: a.id, title: a.title, description: a.description,
						points: a.points, trueRatio: a.trueRatio, badgeUrl: a.badgeUrl,
						displayOrder: a.displayOrder, numAwarded: a.numAwarded,
						numAwardedHardcore: a.numAwardedHardcore
					}))
				}
			};
		} catch { catalogItem = null; }
		loading = false;
	}

	async function fetchIptv(channelId: string) {
		loading = true;
		const detail = await iptvService.getChannel(channelId);
		if (detail) {
			iptvChannel = detail.channel;
			iptvStreams = detail.streams;
			if (detail.streams.length > 0) {
				iptvStreamUrl = iptvService.getStreamUrl(channelId);
			}
		}
		loading = false;
		const epg = await iptvService.getEpg(channelId);
		if (epg) {
			iptvEpgAvailable = epg.available;
			iptvEpgPrograms = epg.programs;
		}
	}

	// === Movie fetch ===

	async function fetchMovie(tmdbId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/tmdb/movies/${tmdbId}`);
			if (res.ok) {
				const details = movieDetailsToDisplay(await res.json());
				// Fetch image overrides
				try {
					const ovRes = await fetchRaw(`/api/tmdb/image-overrides/movie/${tmdbId}`);
					if (ovRes.ok) {
						const entries: Array<{ role: string; file_path: string }> = await ovRes.json();
						const map: Record<string, string> = {};
						for (const o of entries) map[o.role] = o.file_path;
						movieImageOverrides = Object.keys(map).length > 0 ? map : null;
					}
				} catch { /* best-effort */ }
				// Fetch library item
				try {
					const mRes = await fetchRaw('/api/media');
					if (mRes.ok) {
						const mData = await mRes.json();
						const allItems: MediaItem[] = Object.values(mData.itemsByType ?? {}).flat() as MediaItem[];
						const match = allItems.find((item) => item.links?.tmdb?.serviceId === String(tmdbId));
						if (match) {
							libraryItem = match;
							const relRes = await fetchRaw(`/api/media/library-items/${match.id}/related`);
							if (relRes.ok) relatedData = await relRes.json();
						}
					}
				} catch { /* best-effort */ }

				catalogItem = {
					id: String(details.id), kind: 'movie',
					title: details.title, sortTitle: details.title.toLowerCase(),
					year: details.releaseYear || null, overview: details.overview || null,
					posterUrl: movieImageOverrides?.poster ? getPosterUrl(movieImageOverrides.poster) : details.posterUrl,
					backdropUrl: movieImageOverrides?.backdrop ? getBackdropUrl(movieImageOverrides.backdrop) : details.backdropUrl,
					voteAverage: details.voteAverage, voteCount: details.voteCount,
					parentId: null, position: null,
					source: 'tmdb', sourceId: String(details.id),
					createdAt: '', updatedAt: '',
					metadata: {
						tmdbId: details.id, originalTitle: details.originalTitle,
						runtime: details.runtime, director: details.director,
						cast: details.cast.map((c) => ({ id: c.id, name: c.name, character: c.character, profileUrl: c.profileUrl })),
						genres: details.genres, tagline: details.tagline,
						budget: details.budget, revenue: details.revenue,
						imdbId: details.imdbId,
						images: details.images.map((img) => ({ thumbnailUrl: img.thumbnailUrl, fullUrl: img.fullUrl, width: img.width, height: img.height, filePath: img.filePath, imageType: img.imageType })),
						imageOverrides: movieImageOverrides ?? {}
					}
				};

				const cached = await smartSearchService.checkFetchCache(tmdbId);
				if (cached) {
					fetchingId = String(tmdbId);
					smartSearchService.setSelection({
						title: details.title, year: details.releaseYear ?? '', type: 'movie',
						tmdbId, mode: 'fetch',
						existingItemId: libraryItem?.id, existingLibraryId: libraryItem?.libraryId
					});
					smartSearchService.setFetchedCandidate(cached);
				}
			}
		} catch { catalogItem = null; }
		loading = false;
	}

	// === TV fetch ===

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

	async function fetchTvLibraryData(showId: number) {
		try {
			const res = await fetchRaw('/api/media');
			if (!res.ok) return;
			const mData = await res.json();
			const lists: MediaList[] = mData.lists ?? [];
			const showLists = lists.filter((l: MediaList) => l.parentListId === null && l.links?.tmdb?.serviceId === String(showId));
			if (showLists.length === 0) return;
			const files: typeof libraryFiles = [];
			for (const showList of showLists) {
				for (const item of showList.items) {
					const parsed = parseEpisodeFromFilename(item.name);
					if (parsed) files.push({ ...parsed, seasonNumber: parsed.season, episodeNumber: parsed.episode, name: item.name, path: item.path });
				}
				const seasonLists = lists.filter((l: MediaList) => l.parentListId === showList.id);
				for (const seasonList of seasonLists) {
					const seasonNum = seasonList.links?.tmdb?.seasonNumber ?? parseSeasonFromTitle(seasonList.title);
					for (const item of seasonList.items) {
						const parsed = parseEpisodeFromFilename(item.name);
						if (parsed) files.push({ seasonNumber: parsed.season, episodeNumber: parsed.episode, name: item.name, path: item.path });
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
				const details = tvShowDetailsToDisplay(await res.json());
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
				catalogItem = {
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
						seasons: (details.seasons ?? []).map((s) => ({ id: s.id, name: s.name, overview: s.overview, airDate: s.airDate, episodeCount: s.episodeCount, posterUrl: s.posterUrl, seasonNumber: s.seasonNumber })),
						images: details.images.map((img) => ({ thumbnailUrl: img.thumbnailUrl, fullUrl: img.fullUrl, width: img.width, height: img.height, filePath: img.filePath, imageType: img.imageType })),
						imageOverrides: {}
					}
				};
				await fetchTvLibraryData(showId);
				if (libraryFiles.length === 0) {
					const cached = await smartSearchService.checkTvFetchCache(showId);
					if (cached && cached.length > 0) {
						fetchingId = String(showId);
						const bestEntry = cached.find((e) => e.scope === 'complete') ?? cached[0];
						const sel = { title: details.name, year: details.firstAirYear ?? '', type: 'tv' as const, tmdbId: showId, mode: 'fetch' as const, seasons: tvSeasonsMeta };
						smartSearchService.setSelection(sel);
						smartSearchService.setFetchedCandidate(bestEntry.candidate);
						smartSearchService.ensurePendingItem(sel);
					}
				}
			}
		} catch { catalogItem = null; }
		loading = false;
	}

	function handlePlayFile(file: { name: string; path: string }) {
		const pf: PlayableFile = {
			id: `library:${file.path}`, type: 'library', name: file.name,
			outputPath: file.path, mode: 'video', format: null,
			videoFormat: null, thumbnailUrl: null, durationSeconds: null, size: 0, completedAt: ''
		};
		playerService.play(pf, 'inline');
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

	async function handleResync() {
		if (!config || config.kind !== 'tv_show') return;
		resyncing = true;
		try {
			const res = await fetchRaw('/api/media');
			if (!res.ok) return;
			const mData = await res.json();
			const lists: MediaList[] = mData.lists ?? [];
			const showLists = lists.filter((l: MediaList) => l.parentListId === null && l.links?.tmdb?.serviceId === id);
			const libraryIds = [...new Set(showLists.map((l: MediaList) => l.libraryId))];
			await Promise.all(libraryIds.map((lid: string) => fetchRaw(`/api/libraries/${lid}/scan`, { method: 'POST' })));
			await fetchTvLibraryData(Number(id));
		} finally { resyncing = false; }
	}

	// === Shared smart search handlers ===

	async function handleFetch() {
		if (!catalogItem || !config) return;
		fetchingId = catalogItem.sourceId;
		if (catalogItem.kind === 'book') {
			const book = catalogItem as CatalogBook;
			smartSearchService.select({
				title: book.title, year: book.year ?? '', type: 'book',
				openlibraryKey: book.sourceId,
				author: book.metadata.authors[0] ?? 'Unknown', mode: 'fetch'
			});
		} else if (catalogItem.kind === 'game') {
			const game = catalogItem as CatalogGame;
			smartSearchService.select({
				title: game.title, year: '', type: 'game',
				retroachievementsId: game.metadata.retroachievementsId,
				consoleName: game.metadata.consoleName, mode: 'fetch'
			});
		} else if (catalogItem.kind === 'movie') {
			const tid = Number(catalogItem.sourceId);
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
		} else if (catalogItem.kind === 'tv_show') {
			const tid = Number(catalogItem.sourceId);
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
			smartSearchService.select({ title: catalogItem.title, year: catalogItem.year ?? '', type: 'tv', tmdbId: Number(catalogItem.sourceId), mode: 'fetch', seasons: tvSeasonsMeta });
		}
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (!config) return;
		if (config.kind === 'iptv_channel') {
			if (!iptvChannel) return;
			togglingFavorite = true;
			try { await favoritesService.toggle(config.favService, iptvChannel.id, iptvChannel.name); }
			finally { togglingFavorite = false; }
		} else if (catalogItem) {
			await favoritesService.toggle(config.favService, catalogItem.sourceId, catalogItem.title);
		}
	}

	async function handleTogglePin() {
		if (!config) return;
		if (config.kind === 'iptv_channel') {
			if (!iptvChannel) return;
			togglingPin = true;
			try { await pinsService.toggle(config.pinService, iptvChannel.id, iptvChannel.name); }
			finally { togglingPin = false; }
		} else if (catalogItem) {
			await pinsService.toggle(config.pinService, catalogItem.sourceId, catalogItem.title);
		}
	}

	// Fetch cache auto-save
	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (!candidate || !fetchingId || !config) return;
		if (config.kind === 'book') {
			smartSearchService.saveBookFetchCache(fetchingId, candidate);
		} else if (config.kind === 'movie') {
			smartSearchService.saveFetchCache(Number(fetchingId), 'movie', candidate);
		} else if (config.kind === 'tv_show') {
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
			smartSearchService.saveTvFetchCache(Number(fetchingId), scope, seasonNumber, episodeNumber, candidate);
		}
	});

	onMount(() => {
		if (!config) return;
		smartSearchService.initializeConfig();
		if (config.kind === 'book') fetchBook(id);
		else if (config.kind === 'game') fetchGame(id);
		else if (config.kind === 'iptv_channel') fetchIptv(id);
		else if (config.kind === 'movie') fetchMovie(Number(id));
		else if (config.kind === 'tv_show') fetchTvShow(Number(id));
	});
</script>

{#if !config}
	<div class="flex flex-1 items-center justify-center">
		<p class="text-sm opacity-60">Not found</p>
	</div>
{:else if config.kind === 'iptv_channel'}
	{#if iptvChannel}
		<IptvChannelDetail
			channel={iptvChannel}
			streams={iptvStreams}
			streamUrl={iptvStreamUrl}
			{loading}
			epgPrograms={iptvEpgPrograms}
			epgAvailable={iptvEpgAvailable}
			{isFavorite}
			{togglingFavorite}
			{isPinned}
			{togglingPin}
			onback={() => goto(`${base}/media/iptv`)}
			onstreamselect={() => { iptvStreamUrl = iptvService.getStreamUrl(iptvChannel?.id ?? id); }}
			ontogglefavorite={handleToggleFavorite}
			ontogglepin={handleTogglePin}
		/>
	{:else if loading}
		<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
	{:else}
		<div class="flex flex-1 flex-col items-center justify-center gap-2">
			<p class="text-sm opacity-60">Channel not found</p>
			<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/iptv`)}>Back</button>
		</div>
	{/if}
{:else if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={isFetching} fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps} torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null}
		{isFavorite} {isPinned}
		onfetch={handleFetch} ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/media/${config.slug}`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
		onstream={(config?.kind === 'movie' || config?.kind === 'tv_show') ? handleP2pStream : undefined}
	>
		{#snippet extra()}
			{#if catalogItem?.kind === 'book'}
				<BookDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'game'}
				<GameDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'movie'}
				<MovieDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'tv_show'}
				<TvDetailMeta item={catalogItem} />
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
			{/if}
		{/snippet}
		{#snippet cellB()}
			{#if (config?.kind === 'movie' || config?.kind === 'tv_show') && $playerState.currentFile && $playerDisplayMode === 'inline'}
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<h2 class="text-sm font-semibold tracking-wide uppercase text-base-content/50">Now Playing</h2>
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
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/${config.slug}`)}>Back</button>
	</div>
{/if}

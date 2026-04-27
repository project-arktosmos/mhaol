<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
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
		CatalogAuthor,
		CatalogAlbum,
		CatalogBook,
		CatalogGame,
		CatalogMovie,
		CatalogTvShow,
		AlbumRelease
	} from 'ui-lib/types/catalog.type';
	import { formatAuthors } from 'ui-lib/types/catalog.type';
	import type { IptvChannel, IptvStream, IptvEpgProgram } from 'ui-lib/types/iptv.type';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import type { TvSeasonMeta, TvFetchedCandidates, SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import { playerService } from 'ui-lib/services/player.service';

	// Detail components
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import AlbumDetailMeta from 'ui-lib/components/catalog/detail/AlbumDetailMeta.svelte';
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
		getBackdropUrl,
		getProfileUrl
	} from 'addons/tmdb/transform';
	import type {
		TMDBMovieDetails,
		TMDBTvShowDetails,
		DisplayTMDBMovieDetails,
		DisplayTMDBSeasonDetails
	} from 'addons/tmdb/types';
	import { iptvService } from 'ui-lib/services/iptv.service';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'addons/musicbrainz/transform';
	import type { MusicBrainzReleaseGroup, MusicBrainzRelease, MusicBrainzArtistCredit } from 'addons/musicbrainz/types';

	function mbCreditsToAuthors(credits: MusicBrainzArtistCredit[]): CatalogAuthor[] {
		return credits.map((c) => ({
			id: c.artist.id, name: c.name, role: 'artist' as const, source: 'musicbrainz' as const,
			imageUrl: null, joinPhrase: c.joinphrase || undefined
		}));
	}

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
	const torrentState = torrentService.state;

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
		// Check library-related torrent first (movies with existing library items)
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


	// TV-specific state
	let tvSeasonsMeta = $state<TvSeasonMeta[]>([]);
	let libraryFiles = $state<Array<{ seasonNumber: number; episodeNumber: number; name: string; path: string }>>([]);
	let resyncing = $state(false);
	const savedTvHashes = new Set<string>();

	let tvFetchedTorrents = $derived.by(() => {
		if (config?.kind !== 'tv_show') return null;
		const tvc = $searchStore.fetchedTvCandidates;
		if (!tvc) return null;
		const list: Array<{ label: string; name: string; quality: string; languages: string }> = [];
		if (tvc.complete) {
			list.push({
				label: 'Complete series',
				name: tvc.complete.name,
				quality: tvc.complete.analysis?.quality ?? '',
				languages: tvc.complete.analysis?.languages ?? ''
			});
		}
		const seasonNums = Object.keys(tvc.seasons).map(Number).sort((a, b) => a - b);
		for (const sn of seasonNums) {
			const c = tvc.seasons[sn];
			if (!c) continue;
			list.push({
				label: `Season ${sn}`,
				name: c.name,
				quality: c.analysis?.quality ?? '',
				languages: c.analysis?.languages ?? ''
			});
		}
		return list.length > 0 ? list : null;
	});

	let tvSeasonEpisodes = $derived.by(() => {
		const map: Record<number, typeof tvSeasonsMeta[number]['episodes']> = {};
		for (const s of tvSeasonsMeta) map[s.seasonNumber] = s.episodes;
		return map;
	});

	// Keyed by lowercase infoHash because the backend lowercases magnet hashes,
	// while TPB returns uppercase — strict-equality lookup would always miss.
	let torrentByHash = $derived.by(() => {
		const map: Record<string, import('ui-lib/types/torrent.type').TorrentInfo> = {};
		for (const t of $torrentState.allTorrents) map[t.infoHash.toLowerCase()] = t;
		return map;
	});

	// Auto-start downloads for each per-scope TV candidate (idempotent: backend dedupes by infoHash)
	const autoStartedHashes = new Set<string>();
	$effect(() => {
		if (config?.kind !== 'tv_show') return;
		const tvc = $searchStore.fetchedTvCandidates;
		if (!tvc) return;
		const candidates = [tvc.complete, ...Object.values(tvc.seasons)].filter(
			(c): c is NonNullable<typeof c> => c !== null
		);
		for (const c of candidates) {
			const key = c.infoHash.toLowerCase();
			if (autoStartedHashes.has(key)) continue;
			if (torrentByHash[key]) {
				autoStartedHashes.add(key);
				continue;
			}
			autoStartedHashes.add(key);
			smartSearchService.startDownload(c).catch(() => {
				autoStartedHashes.delete(key);
			});
		}
	});

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
			const authorDisplays = await Promise.all(
				authorKeys.slice(0, 3).map(async (key) => {
					try {
						const res = await fetchRaw(`/api/openlibrary/authors/${key}`);
						if (res.ok) return authorToDisplay(await res.json() as OpenLibraryAuthor);
					} catch { /* best-effort */ }
					return { key, name: 'Unknown', birthDate: null, deathDate: null, bio: null, photoUrl: null };
				})
			);
			const bookAuthors: CatalogAuthor[] = authorDisplays.map((a) => ({
				id: a.key, name: a.name, role: 'author' as const, source: 'openlibrary' as const,
				imageUrl: a.photoUrl ?? null,
				bio: a.bio ?? undefined, birthDate: a.birthDate ?? undefined, deathDate: a.deathDate ?? undefined
			}));
			const details = workToDisplayDetails(work, authorDisplays, {
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
					openlibraryKey: bookKey, authors: bookAuthors,
					firstPublishYear: work.first_publish_date?.split('-')[0] ?? '',
					coverId, coverUrl: getCoverUrl(coverId, 'M'),
					subjects: (work.subjects ?? []).slice(0, 10), publishers: [],
					pageCount: null, editionCount: 0, isbn: null,
					ratingsAverage: null, ratingsCount: 0,
					description: details?.description ?? null
				}
			};
			const cached = await smartSearchService.checkBookFetchCache(bookKey);
			if (cached) {
				fetchingId = bookKey;
				const sel = {
					title: work.title, year: catalogItem.year ?? '',
					type: 'book' as const, openlibraryKey: bookKey,
					author: bookAuthors[0]?.name ?? 'Unknown', mode: 'fetch' as const
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
					authors: [
						...(detail.developer ? [{ id: detail.developer, name: detail.developer, role: 'developer' as const, source: 'retroachievements' as const, imageUrl: null }] : []),
						...(detail.publisher ? [{ id: detail.publisher, name: detail.publisher, role: 'publisher' as const, source: 'retroachievements' as const, imageUrl: null }] : [])
					] satisfies CatalogAuthor[],
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

	async function fetchAlbum(albumId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const rgRes = await fetchRaw(`/api/musicbrainz/release-group/${albumId}`);
			if (!rgRes.ok) throw new Error('Failed to fetch');
			const rgData = await rgRes.json();
			const display = releaseGroupsToDisplay([rgData as MusicBrainzReleaseGroup]);
			const album = display[0];
			if (!album) throw new Error('No album data');

			let catalogReleases: AlbumRelease[] = [];
			const rawReleases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (rawReleases.length > 0) {
				const official = rawReleases.find((r) => r.status === 'Official') ?? rawReleases[0];
				const relRes = await fetchRaw(`/api/musicbrainz/release/${official.id}`);
				if (relRes.ok) {
					const rel = releaseToDisplay((await relRes.json()) as MusicBrainzRelease);
					if (rel) catalogReleases = [{
						id: rel.id, title: rel.title, date: rel.date, status: rel.status,
						country: rel.country, authors: mbCreditsToAuthors(rel.rawArtistCredits),
						trackCount: rel.trackCount, label: rel.label,
						tracks: rel.tracks.map((t) => ({
							id: t.id, number: t.number, title: t.title,
							duration: t.duration, durationMs: t.durationMs,
							authors: mbCreditsToAuthors(t.rawArtistCredits)
						}))
					}];
				}
			}

			const albumCredits: MusicBrainzArtistCredit[] = (rgData as MusicBrainzReleaseGroup)['artist-credit'] ?? [];
			const albumAuthors = mbCreditsToAuthors(albumCredits);

			catalogItem = {
				id: albumId, kind: 'album',
				title: album.title, sortTitle: album.title.toLowerCase(),
				year: album.firstReleaseYear || null, overview: null,
				posterUrl: album.coverArtUrl, backdropUrl: null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'musicbrainz', sourceId: albumId,
				createdAt: '', updatedAt: '',
				metadata: {
					musicbrainzId: albumId, primaryType: album.primaryType,
					secondaryTypes: album.secondaryTypes, authors: albumAuthors,
					firstReleaseYear: album.firstReleaseYear,
					coverArtUrl: album.coverArtUrl, releases: catalogReleases
				}
			};

			let cached = await smartSearchService.checkMusicFetchCache(albumId);
			// If no album-level cache, check the parent artist's fetch cache
			if (!cached || cached.length === 0) {
				const artistId = albumAuthors[0]?.id;
				if (artistId) cached = await smartSearchService.checkMusicFetchCache(artistId);
			}
			if (cached && cached.length > 0) {
				fetchingId = albumId;
				const bestEntry = cached.find((e) => e.scope === 'album') ?? cached[0];
				smartSearchService.setSelection({
					title: album.title, year: album.firstReleaseYear,
					type: 'music', musicbrainzId: albumId,
					artist: formatAuthors(albumAuthors, 'artist'),
					mode: 'fetch', musicSearchMode: 'album'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
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

	function tmdbMovieAuthors(raw: TMDBMovieDetails): CatalogAuthor[] {
		const authors: CatalogAuthor[] = [];
		const director = raw.credits?.crew.find((c) => c.job === 'Director');
		if (director) authors.push({ id: String(director.id), name: director.name, role: 'director', source: 'tmdb', imageUrl: getProfileUrl(director.profile_path) });
		for (const c of (raw.credits?.cast ?? []).slice(0, 10)) {
			authors.push({ id: String(c.id), name: c.name, role: 'actor', source: 'tmdb', imageUrl: getProfileUrl(c.profile_path), character: c.character });
		}
		return authors;
	}

	function tmdbTvAuthors(raw: TMDBTvShowDetails): CatalogAuthor[] {
		const authors: CatalogAuthor[] = [];
		for (const c of raw.created_by ?? []) {
			authors.push({ id: String(c.id), name: c.name, role: 'creator', source: 'tmdb', imageUrl: getProfileUrl(c.profile_path) });
		}
		for (const c of (raw.credits?.cast ?? []).slice(0, 10)) {
			authors.push({ id: String(c.id), name: c.name, role: 'actor', source: 'tmdb', imageUrl: getProfileUrl(c.profile_path), character: c.character });
		}
		return authors;
	}

	async function fetchMovie(tmdbId: number) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/tmdb/movies/${tmdbId}`);
			if (res.ok) {
				const rawMovie: TMDBMovieDetails = await res.json();
				const details = movieDetailsToDisplay(rawMovie);
				const movieAuthors = tmdbMovieAuthors(rawMovie);
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
						runtime: details.runtime,
						authors: movieAuthors,
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

	function buildTvCandidatesFromCache(
		rows: Array<{ scope: string; seasonNumber: number | null; candidate: SmartSearchTorrentResult }>
	): TvFetchedCandidates {
		const candidates: TvFetchedCandidates = { complete: null, seasons: {} };
		for (const row of rows) {
			if (row.scope === 'complete') {
				candidates.complete = row.candidate;
				savedTvHashes.add(`complete:${row.candidate.infoHash}`);
			} else if (row.scope === 'season' && row.seasonNumber != null) {
				candidates.seasons[row.seasonNumber] = row.candidate;
				savedTvHashes.add(`season:${row.seasonNumber}:${row.candidate.infoHash}`);
			}
		}
		return candidates;
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
				const rawTv: TMDBTvShowDetails = await res.json();
				const details = tvShowDetailsToDisplay(rawTv);
				const tvAuthors = tmdbTvAuthors(rawTv);
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
						networks: details.networks,
						authors: tvAuthors,
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
						const sel = { title: details.name, year: details.firstAirYear ?? '', type: 'tv' as const, tmdbId: showId, mode: 'fetch' as const, seasons: tvSeasonsMeta };
						smartSearchService.setSelection(sel);
						smartSearchService.setFetchedTvCandidates(buildTvCandidatesFromCache(cached));
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

	function handleStreamOrPlay() {
		if (libraryItem) {
			handlePlayFile({ name: libraryItem.name, path: libraryItem.path });
		} else {
			handleP2pStream();
		}
	}

	let hasLibraryItem = $derived(libraryItem !== null);

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
			if (!isFetchedForCurrent) {
				const cached = await smartSearchService.checkBookFetchCache(book.sourceId);
				if (cached) {
					smartSearchService.setSelection({
						title: book.title, year: book.year ?? '', type: 'book',
						openlibraryKey: book.sourceId,
						author: book.metadata.authors[0]?.name ?? 'Unknown', mode: 'fetch'
					});
					smartSearchService.setFetchedCandidate(cached);
					return;
				}
			}
			smartSearchService.select({
				title: book.title, year: book.year ?? '', type: 'book',
				openlibraryKey: book.sourceId,
				author: book.metadata.authors[0]?.name ?? 'Unknown', mode: 'fetch'
			});
		} else if (catalogItem.kind === 'game') {
			const game = catalogItem as CatalogGame;
			if (!isFetchedForCurrent) {
				const cached = await smartSearchService.checkGameFetchCache(game.metadata.retroachievementsId);
				if (cached) {
					smartSearchService.setSelection({
						title: game.title, year: '', type: 'game',
						retroachievementsId: game.metadata.retroachievementsId,
						consoleName: game.metadata.consoleName, mode: 'fetch'
					});
					smartSearchService.setFetchedCandidate(cached);
					return;
				}
			}
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
					const sel = { title: catalogItem.title, year: catalogItem.year ?? '', type: 'tv' as const, tmdbId: tid, mode: 'fetch' as const, seasons: tvSeasonsMeta };
					smartSearchService.setSelection(sel);
					smartSearchService.setFetchedTvCandidates(buildTvCandidatesFromCache(cached));
					smartSearchService.ensurePendingItem(sel);
					return;
				}
			}
			// Re-fetch path: drop stale per-scope rows so the next save can repopulate cleanly
			await smartSearchService.clearTvFetchCache(tid);
			savedTvHashes.clear();
			smartSearchService.select({ title: catalogItem.title, year: catalogItem.year ?? '', type: 'tv', tmdbId: Number(catalogItem.sourceId), mode: 'fetch', seasons: tvSeasonsMeta });
		} else if (catalogItem.kind === 'album') {
			const album = catalogItem as CatalogAlbum;
			const artistName = formatAuthors(album.metadata.authors, 'artist');
			if (!isFetchedForCurrent) {
				let cached = await smartSearchService.checkMusicFetchCache(album.sourceId);
				if (!cached || cached.length === 0) {
					const artistId = album.metadata.authors?.[0]?.id;
					if (artistId) cached = await smartSearchService.checkMusicFetchCache(artistId);
				}
				if (cached && cached.length > 0) {
					const bestEntry = cached.find((e) => e.scope === 'album') ?? cached[0];
					smartSearchService.setSelection({
						title: album.title, year: album.year ?? '', type: 'music',
						musicbrainzId: album.sourceId, artist: artistName,
						mode: 'fetch', musicSearchMode: 'album'
					});
					smartSearchService.setFetchedCandidate(bestEntry.candidate);
					return;
				}
			}
			smartSearchService.select({
				title: album.title, year: album.year ?? '', type: 'music',
				musicbrainzId: album.sourceId, artist: artistName,
				mode: 'fetch', musicSearchMode: 'album'
			});
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

	// Fetch cache auto-save (non-TV)
	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (!candidate || !fetchingId || !config) return;
		if (config.kind === 'book') {
			smartSearchService.saveBookFetchCache(fetchingId, candidate);
		} else if (config.kind === 'movie') {
			smartSearchService.saveFetchCache(Number(fetchingId), 'movie', candidate);
		} else if (config.kind === 'album') {
			const scope = candidate.analysis?.isDiscography ? 'discography' : 'album';
			smartSearchService.saveMusicFetchCache(fetchingId, scope, candidate);
		} else if (config.kind === 'game') {
			smartSearchService.saveGameFetchCache(Number(fetchingId), candidate);
		}
	});

	// TV fetch cache auto-save (per-scope)
	$effect(() => {
		const candidates = $searchStore.fetchedTvCandidates;
		if (!candidates || !fetchingId || config?.kind !== 'tv_show') return;
		const tid = Number(fetchingId);
		if (candidates.complete) {
			const key = `complete:${candidates.complete.infoHash}`;
			if (!savedTvHashes.has(key)) {
				savedTvHashes.add(key);
				smartSearchService.saveTvFetchCache(tid, 'complete', null, null, candidates.complete);
			}
		}
		for (const [snStr, c] of Object.entries(candidates.seasons)) {
			if (!c) continue;
			const sn = Number(snStr);
			const key = `season:${sn}:${c.infoHash}`;
			if (!savedTvHashes.has(key)) {
				savedTvHashes.add(key);
				smartSearchService.saveTvFetchCache(tid, 'season', sn, null, c);
			}
		}
	});

	onMount(() => {
		if (!config) return;
		smartSearchService.initializeConfig();
		if (config.kind === 'book') fetchBook(id);
		else if (config.kind === 'game') fetchGame(id);
		else if (config.kind === 'iptv_channel') fetchIptv(id);
		else if (config.kind === 'album') fetchAlbum(id);
		else if (config.kind === 'movie') fetchMovie(Number(id));
		else if (config.kind === 'tv_show') fetchTvShow(Number(id));
	});

	onDestroy(() => {
		playerService.stop();
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
			onback={() => goto(`${base}/media/${slug}`)}
			onstreamselect={() => { iptvStreamUrl = iptvService.getStreamUrl(iptvChannel?.id ?? id); }}
			ontogglefavorite={handleToggleFavorite}
			ontogglepin={handleTogglePin}
		/>
	{:else if loading}
		<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
	{:else}
		<div class="flex flex-1 flex-col items-center justify-center gap-2">
			<p class="text-sm opacity-60">Channel not found</p>
			<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/${slug}`)}>Back</button>
		</div>
	{/if}
{:else if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={hasLibraryItem ? false : isFetching}
		fetched={hasLibraryItem ? false : isFetchedForCurrent}
		fetchSteps={hasLibraryItem ? null : currentFetchSteps}
		torrentStatus={hasLibraryItem ? null : matchedTorrent}
		fetchedTorrent={hasLibraryItem || config?.kind === 'tv_show' ? null : ($searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null)}
		fetchedTorrents={hasLibraryItem ? null : tvFetchedTorrents}
		{isFavorite} {isPinned}
		onfetch={hasLibraryItem ? undefined : handleFetch}
		ondownload={hasLibraryItem || config?.kind === 'tv_show' ? undefined : handleDownload}
		onshowsearch={hasLibraryItem ? undefined : () => smartSearchService.show()}
		onback={() => goto(`${base}/media/${config.slug}`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
		onstream={(config?.kind === 'movie' || config?.kind === 'tv_show') ? handleStreamOrPlay : undefined}
		streaming={$playerState.connectionState !== 'idle'}
	>
		{#snippet extra()}
			{#if catalogItem?.kind === 'album'}
				<AlbumDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'book'}
				<BookDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'game'}
				<GameDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'movie'}
				<MovieDetailMeta item={catalogItem} />
				{#if libraryItem}
					<div>
						<h3 class="text-xs font-semibold tracking-wide uppercase opacity-50">Library File</h3>
						<button
							class="mt-1 flex w-full items-center gap-2 rounded p-1.5 text-left text-sm hover:bg-base-200"
							onclick={() => handlePlayFile({ name: libraryItem!.name, path: libraryItem!.path })}
						>
							<span class="truncate">{libraryItem.name}</span>
						</button>
					</div>
				{/if}
			{:else if catalogItem?.kind === 'tv_show'}
				<TvDetailMeta
					item={catalogItem}
					completeCandidate={$searchStore.fetchedTvCandidates?.complete ?? null}
					seasonCandidates={$searchStore.fetchedTvCandidates?.seasons ?? {}}
					seasonEpisodes={tvSeasonEpisodes}
					{torrentByHash}
				/>
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
		{#snippet rightPanel()}
			{#if config?.kind === 'movie' || config?.kind === 'tv_show'}
				<div class="flex flex-col gap-2">
					<PlayerVideo
						file={$playerState.currentFile}
						connectionState={$playerState.connectionState}
						positionSecs={$playerState.positionSecs}
						durationSecs={$playerState.durationSecs}
						buffering={$playerState.buffering}
						poster={catalogItem?.backdropUrl ?? catalogItem?.posterUrl}
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

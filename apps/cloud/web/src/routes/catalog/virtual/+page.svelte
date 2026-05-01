<script lang="ts">
	import classNames from 'classnames';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import FirkinArtistsSection from 'ui-lib/components/firkins/FirkinArtistsSection.svelte';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import { cachedImageUrl } from '$lib/image-cache';
	import {
		firkinsService,
		addonKind,
		type FirkinAddon,
		type Firkin,
		type ImageMeta,
		type Artist,
		type Trailer,
		type FileEntry
	} from '$lib/firkins.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type TorrentResultItem
	} from '$lib/search.service';
	import {
		playYouTubeAudio,
		playYouTubeVideo,
		resolveYouTubeTrailerForMovie,
		resolveYouTubeTrailerForSeason,
		resolveYouTubeUrlForTrack
	} from '$lib/youtube-match.service';
	import { userIdentityService } from '$lib/user-identity.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page as pageStore } from '$app/state';

	const params = $derived(pageStore.url.searchParams);
	const addon = $derived(params.get('addon') ?? '');
	const itemId = $derived(params.get('id') ?? '');
	const title = $derived(params.get('title') ?? '');
	const yearParam = $derived(params.get('year'));
	const year = $derived(
		yearParam !== null && yearParam !== '' ? Number.parseInt(yearParam, 10) : null
	);
	const description = $derived(params.get('description') ?? '');
	const posterUrl = $derived(params.get('posterUrl'));
	const backdropUrl = $derived(params.get('backdropUrl'));

	const kindLabel = $derived(addonKind(addon) ?? '');
	const isMusicBrainz = $derived(addon === 'musicbrainz');
	const isTmdbMovie = $derived(addon === 'tmdb-movie');
	const isTmdbTv = $derived(addon === 'tmdb-tv');

	const images = $derived<ImageMeta[]>(
		[posterUrl, backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }))
	);

	type ArtistsStatus = 'idle' | 'loading' | 'done' | 'error';
	let artists = $state<Artist[]>([]);
	let artistsStatus = $state<ArtistsStatus>('idle');
	let artistsError = $state<string | null>(null);
	let artistsRun = 0;
	let artistsInitForKey: string | null = null;
	// TMDB-sourced trailers for the current item, populated by /metadata.
	// Movies usually carry one official trailer here; TV shows carry the
	// show-level trailer(s). Per-season trailers (TV) still come from the
	// YouTube fallback below.
	let tmdbTrailers = $state<Trailer[]>([]);

	$effect(() => {
		if (!addon || !itemId) return;
		const key = `${addon}:${itemId}`;
		if (artistsInitForKey === key) return;
		artistsInitForKey = key;
		void loadMetadata(addon, itemId, ++artistsRun);
	});

	async function loadMetadata(addon: string, id: string, myRun: number) {
		artistsStatus = 'loading';
		artistsError = null;
		artists = [];
		tmdbTrailers = [];
		try {
			const res = await fetch(
				`/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(id)}/metadata`,
				{ cache: 'no-store' }
			);
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as { artists?: Artist[]; trailers?: Trailer[] };
			if (myRun !== artistsRun) return;
			artists = Array.isArray(body.artists) ? body.artists : [];
			tmdbTrailers = Array.isArray(body.trailers) ? body.trailers : [];
			artistsStatus = 'done';
		} catch (err) {
			if (myRun !== artistsRun) return;
			artistsError = err instanceof Error ? err.message : 'Unknown error';
			artistsStatus = 'error';
		}
	}

	const userIdentityState = userIdentityService.state;
	const virtualFirkin = $derived<CloudFirkin>({
		id: `virtual:${addon}:${itemId}`,
		title,
		artists,
		description,
		images,
		files: [],
		year,
		addon,
		creator: $userIdentityState.identity?.address ?? '',
		created_at: '',
		updated_at: '',
		version: 0,
		version_hashes: []
	});

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentMatches = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;

	let bookmarking = $state(false);
	let bookmarkError = $state<string | null>(null);

	async function bookmark() {
		if (bookmarking || !title) return;
		bookmarkError = null;
		bookmarking = true;
		try {
			// Persist the upstream catalog id alongside the firkin so the detail
			// page can deterministically refetch upstream data (musicbrainz
			// tracks; tmdb-tv seasons for trailer rebuild) instead of doing a
			// fuzzy search by title that may return a different upstream item.
			const sourceFiles = buildUpstreamSourceFiles();
			const created: Firkin = await firkinsService.create({
				title,
				artists,
				description,
				images,
				files: [...sourceFiles, ...resolvedTrackFiles()],
				year,
				addon: addon as FirkinAddon,
				trailers: resolvedTrailers()
			});
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			bookmarkError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			bookmarking = false;
		}
	}

	// Carry the YouTube URLs already resolved on this virtual page into the
	// firkin body so the detail page does not have to re-run yt-dlp searches
	// for tracks we just looked up. Only `found` tracks contribute — pending /
	// missing / errored tracks fall through to the detail page's resolver.
	function resolvedTrackFiles(): FileEntry[] {
		if (!isMusicBrainz) return [];
		return tracks
			.filter(
				(t): t is Track & { youtubeUrl: string } =>
					t.youtubeStatus === 'found' && Boolean(t.youtubeUrl) && Boolean(t.title.trim())
			)
			.map((t) => ({ type: 'url', value: t.youtubeUrl, title: t.title }));
	}

	function buildUpstreamSourceFiles() {
		if (!itemId) return [];
		if (isMusicBrainz) {
			return [
				{
					type: 'url' as const,
					value: `https://musicbrainz.org/release-group/${itemId}`,
					title: 'MusicBrainz Release Group'
				}
			];
		}
		if (isTmdbTv) {
			return [
				{
					type: 'url' as const,
					value: `https://www.themoviedb.org/tv/${itemId}`,
					title: 'TMDB TV Show'
				}
			];
		}
		if (isTmdbMovie) {
			return [
				{
					type: 'url' as const,
					value: `https://www.themoviedb.org/movie/${itemId}`,
					title: 'TMDB Movie'
				}
			];
		}
		return [];
	}

	type Track = {
		id: string;
		position: number;
		title: string;
		lengthMs: number | null;
		youtubeUrl: string | null;
		youtubeStatus: 'pending' | 'searching' | 'found' | 'missing' | 'error';
	};

	type TrailerEntry = {
		key: string;
		label: string | null;
		seasonNumber: number | null;
		airYear: number | null;
		youtubeUrl: string | null;
		language: string | null;
		status: 'pending' | 'searching' | 'found' | 'missing' | 'error';
	};
	let trailers = $state<TrailerEntry[]>([]);
	let trailersStatus = $state<'idle' | 'loading' | 'done' | 'error'>('idle');
	let trailersError = $state<string | null>(null);
	let trailersRun = 0;
	let trailersInitForKey: string | null = null;
	let playingTrailerKey = $state<string | null>(null);
	let trailerPlayError = $state<string | null>(null);

	$effect(() => {
		if (!title) return;
		if (!isTmdbMovie && !isTmdbTv) return;
		// Wait for /metadata to settle so resolve* can prefer TMDB-sourced
		// trailers over a YouTube fuzzy search.
		if (artistsStatus !== 'done' && artistsStatus !== 'error') return;
		const key = `${addon}:${itemId}:${title}:${year ?? ''}:${artistsStatus}`;
		if (trailersInitForKey === key) return;
		trailersInitForKey = key;
		const myRun = ++trailersRun;
		if (isTmdbMovie) {
			void resolveMovieTrailer(myRun);
		} else if (isTmdbTv) {
			void resolveTvTrailers(myRun);
		}
	});

	async function resolveMovieTrailer(myRun: number) {
		trailersStatus = 'loading';
		trailersError = null;
		// Prefer the official TMDB trailer when /metadata returned one;
		// only fall back to a fuzzy YouTube search if TMDB has nothing.
		const tmdb = tmdbTrailers[0];
		if (tmdb) {
			trailers = [
				{
					key: 'movie',
					label: tmdb.label ?? null,
					seasonNumber: null,
					airYear: year,
					youtubeUrl: tmdb.youtubeUrl,
					language: tmdb.language ?? null,
					status: 'found'
				}
			];
			trailersStatus = 'done';
			return;
		}
		trailers = [
			{
				key: 'movie',
				label: null,
				seasonNumber: null,
				airYear: year,
				youtubeUrl: null,
				language: null,
				status: 'searching'
			}
		];
		try {
			const url = await resolveYouTubeTrailerForMovie(title, year);
			if (myRun !== trailersRun) return;
			trailers = trailers.map((t) =>
				t.key === 'movie' ? { ...t, youtubeUrl: url, status: url ? 'found' : 'missing' } : t
			);
			trailersStatus = 'done';
		} catch (err) {
			if (myRun !== trailersRun) return;
			trailers = trailers.map((t) => (t.key === 'movie' ? { ...t, status: 'error' } : t));
			trailersError = err instanceof Error ? err.message : 'Unknown error';
			trailersStatus = 'error';
		}
	}

	async function resolveTvTrailers(myRun: number) {
		trailersStatus = 'loading';
		trailersError = null;
		trailers = [];
		if (!itemId) {
			trailersStatus = 'done';
			return;
		}
		let seasons: { seasonNumber: number; name: string; airYear: number | null }[] = [];
		try {
			const res = await fetch(`/api/catalog/tmdb-tv/${encodeURIComponent(itemId)}/seasons`, {
				cache: 'no-store'
			});
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as {
				seasonNumber: number;
				name: string;
				airYear?: number | null;
			}[];
			if (myRun !== trailersRun) return;
			seasons = body.map((s) => ({
				seasonNumber: s.seasonNumber,
				name: s.name,
				airYear: s.airYear ?? null
			}));
		} catch (err) {
			if (myRun !== trailersRun) return;
			trailersError = err instanceof Error ? err.message : 'Unknown error';
			trailersStatus = 'error';
			return;
		}
		// TMDB-sourced show-level trailers come from /metadata; per-season
		// trailers are still resolved by the YouTube fallback because TMDB's
		// show-level /videos block doesn't differentiate them.
		const tmdbEntries: TrailerEntry[] = tmdbTrailers.map((t, i) => ({
			key: `tmdb-${i}`,
			label: t.label ?? 'Trailer',
			seasonNumber: null,
			airYear: null,
			youtubeUrl: t.youtubeUrl,
			language: t.language ?? null,
			status: 'found'
		}));
		const seasonEntries: TrailerEntry[] = seasons.map((s) => ({
			key: `season-${s.seasonNumber}`,
			label: s.name,
			seasonNumber: s.seasonNumber,
			airYear: s.airYear,
			youtubeUrl: null,
			language: null,
			status: 'pending'
		}));
		trailers = [...tmdbEntries, ...seasonEntries];
		trailersStatus = 'done';
		// Resolve sequentially so we don't pin too many concurrent yt-dlp
		// search requests at once (mirrors the per-track flow). Skip TMDB
		// entries — they're already 'found' from the metadata response.
		for (let i = 0; i < trailers.length; i++) {
			if (myRun !== trailersRun) return;
			const entry = trailers[i];
			if (entry.status === 'found') continue;
			trailers = trailers.map((t, idx) => (idx === i ? { ...t, status: 'searching' } : t));
			try {
				const url = await resolveYouTubeTrailerForSeason(
					title,
					entry.seasonNumber ?? 0,
					entry.airYear
				);
				if (myRun !== trailersRun) return;
				trailers = trailers.map((t, idx) =>
					idx === i ? { ...t, youtubeUrl: url, status: url ? 'found' : 'missing' } : t
				);
			} catch {
				if (myRun !== trailersRun) return;
				trailers = trailers.map((t, idx) =>
					idx === i ? { ...t, youtubeUrl: null, status: 'error' } : t
				);
			}
		}
	}

	function resolvedTrailers(): Trailer[] {
		return trailers
			.filter((t): t is TrailerEntry & { youtubeUrl: string } => Boolean(t.youtubeUrl))
			.map((t) => ({
				youtubeUrl: t.youtubeUrl,
				label: t.label ?? undefined,
				language: t.language ?? undefined
			}));
	}

	async function playTrailer(entry: TrailerEntry) {
		if (!entry.youtubeUrl || playingTrailerKey !== null) return;
		playingTrailerKey = entry.key;
		trailerPlayError = null;
		try {
			const thumb = images[0]?.url ?? null;
			const playTitle = entry.label ? `${title} — ${entry.label} trailer` : `${title} trailer`;
			await playYouTubeVideo(entry.youtubeUrl, playTitle, thumb, null);
		} catch (err) {
			trailerPlayError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			playingTrailerKey = null;
		}
	}
	type TracksStatus = 'idle' | 'loading' | 'done' | 'error';
	let tracksStatus = $state<TracksStatus>('idle');
	let tracksError = $state<string | null>(null);
	let tracks = $state<Track[]>([]);
	let tracksRun = 0;

	const albumArtist = $derived(description.split(' · ')[0]?.trim() ?? '');

	let playingTrackIndex = $state<number | null>(null);
	let trackPlayError = $state<string | null>(null);

	async function playTrack(index: number) {
		const t = tracks[index];
		if (!t || !t.youtubeUrl || playingTrackIndex !== null) return;
		playingTrackIndex = index;
		trackPlayError = null;
		try {
			const durationSeconds = t.lengthMs ? Math.round(t.lengthMs / 1000) : null;
			const thumb = images[0]?.url ?? null;
			await playYouTubeAudio(t.youtubeUrl, t.title, thumb, durationSeconds);
		} catch (err) {
			trackPlayError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			playingTrackIndex = null;
		}
	}

	$effect(() => {
		if (!title) return;
		if (isMusicBrainz) {
			const id = itemId;
			void loadMusicBrainzTracks(id);
			return;
		}
		const t = title;
		const a = addon;
		const y = year;
		void runTorrentSearch(t, a, y);
	});

	async function loadMusicBrainzTracks(releaseGroupId: string) {
		const myRun = ++tracksRun;
		tracksStatus = 'loading';
		tracksError = null;
		tracks = [];
		if (!releaseGroupId) {
			tracksStatus = 'done';
			return;
		}
		try {
			const res = await fetch(
				`/api/catalog/musicbrainz/release-groups/${encodeURIComponent(releaseGroupId)}/tracks`,
				{ cache: 'no-store' }
			);
			if (!res.ok) {
				let message = `HTTP ${res.status}`;
				try {
					const body = await res.json();
					if (body && typeof body.error === 'string') message = body.error;
				} catch {
					// ignore
				}
				throw new Error(message);
			}
			const body = (await res.json()) as {
				id: string;
				position: number;
				title: string;
				lengthMs: number | null;
			}[];
			if (myRun !== tracksRun) return;
			tracks = body.map((t) => ({
				id: t.id,
				position: t.position,
				title: t.title,
				lengthMs: t.lengthMs,
				youtubeUrl: null,
				youtubeStatus: 'pending'
			}));
			tracksStatus = 'done';
			void resolveYouTubeForAllTracks(myRun);
		} catch (err) {
			if (myRun !== tracksRun) return;
			tracksError = err instanceof Error ? err.message : 'Unknown error';
			tracksStatus = 'error';
		}
	}

	async function resolveYouTubeForAllTracks(myRun: number) {
		const album = title;
		const artist = albumArtist;
		for (let i = 0; i < tracks.length; i++) {
			if (myRun !== tracksRun) return;
			const t = tracks[i];
			tracks = tracks.map((tr, idx) => (idx === i ? { ...tr, youtubeStatus: 'searching' } : tr));
			try {
				const url = await resolveYouTubeUrlForTrack(t.title, artist, album, t.lengthMs);
				if (myRun !== tracksRun) return;
				tracks = tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: url, youtubeStatus: url ? 'found' : 'missing' } : tr
				);
			} catch {
				if (myRun !== tracksRun) return;
				tracks = tracks.map((tr, idx) =>
					idx === i ? { ...tr, youtubeUrl: null, youtubeStatus: 'error' } : tr
				);
			}
		}
	}

	function formatDuration(ms: number | null): string {
		if (!ms || !Number.isFinite(ms) || ms <= 0) return '—';
		const total = Math.round(ms / 1000);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	async function runTorrentSearch(title: string, addon: string, year: number | null) {
		const myRun = ++searchRun;
		torrentStatus = 'searching';
		torrentError = null;
		torrentMatches = [];
		try {
			const torrents = await searchTorrents(addon, title);
			if (myRun !== searchRun) return;
			const matches = matchTorrentsForResult(
				{ title, description: '', artists: [], images: [], files: [], year, raw: null },
				torrents
			);
			torrentMatches = matches;
			torrentStatus = 'done';
		} catch (err) {
			if (myRun !== searchRun) return;
			torrentMatches = [];
			torrentError = err instanceof Error ? err.message : 'Unknown error';
			torrentStatus = 'error';
		}
	}

	async function startTorrentDownload(magnet: string): Promise<void> {
		const res = await fetch('/api/torrent/add', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ magnet })
		});
		if (!res.ok) {
			let message = `HTTP ${res.status}`;
			try {
				const body = await res.json();
				if (body && typeof body.error === 'string') message = body.error;
			} catch {
				// ignore
			}
			throw new Error(message);
		}
	}

	async function assignTorrent(torrent: TorrentResultItem) {
		if (!torrent.magnetLink || addingHash) return;
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const created: Firkin = await firkinsService.create({
				title,
				artists,
				description,
				images,
				files: [
					...buildUpstreamSourceFiles(),
					{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }
				],
				year,
				addon: addon as FirkinAddon,
				trailers: resolvedTrailers()
			});
			await startTorrentDownload(torrent.magnetLink);
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '—';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let value = bytes;
		let unit = 0;
		while (value >= 1024 && unit < units.length - 1) {
			value /= 1024;
			unit++;
		}
		return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {title || 'Catalog'}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{title}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-outline badge-sm">{addon}</span>
				{#if kindLabel}
					<span class="badge badge-outline badge-sm">{kindLabel}</span>
				{/if}
				{#if year !== null && year !== undefined && Number.isFinite(year)}
					<span class="badge badge-outline badge-sm">{year}</span>
				{/if}
				<span class="badge badge-sm badge-warning">virtual</span>
			</p>
		</div>
		<div class="flex items-center gap-2">
			<button
				type="button"
				class="btn gap-2 btn-sm btn-primary"
				onclick={bookmark}
				disabled={bookmarking || !title}
				aria-label="Bookmark"
				title="Persist this virtual item as a firkin in the catalog"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="currentColor"
					stroke="none"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				>
					<path d="M6 3h12a1 1 0 0 1 1 1v17l-7-4-7 4V4a1 1 0 0 1 1-1z" />
				</svg>
				<span>{bookmarking ? 'Bookmarking…' : 'Bookmark'}</span>
			</button>
		</div>
	</header>

	{#if bookmarkError}
		<div class="alert alert-error">
			<span>{bookmarkError}</span>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<FirkinCard firkin={virtualFirkin} />
		</aside>

		<section class="flex flex-col gap-6">
			{#if description}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
					<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
				</div>
			{/if}

			<FirkinArtistsSection
				{artists}
				loading={artistsStatus === 'loading'}
				error={artistsStatus === 'error' ? artistsError : null}
				emptyLabel="No people or groups attached to this item upstream."
				artistHref={(id) => `${base}/artist/${encodeURIComponent(id)}`}
			/>

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Status</h2>
				<p class="text-xs text-base-content/70">
					This item is virtual — no record exists in the database yet, and nothing is pinned to
					IPFS. Picking a torrent below will create the firkin, pin its files, and bring it into the
					catalog properly.
				</p>
			</div>

			{#if images.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Images ({images.length})
					</h2>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
						{#each images as image, i (i)}
							<figure
								class="flex flex-col gap-1 overflow-hidden rounded-box border border-base-content/10 bg-base-300"
							>
								<img
									src={cachedImageUrl(image.url)}
									alt={`Image ${i + 1}`}
									class="block h-auto w-full"
									loading="lazy"
								/>
								<figcaption class="px-2 py-1 text-[10px] text-base-content/70">
									{image.width || '?'}×{image.height || '?'}
									{#if image.fileSize}· {formatBytes(image.fileSize)}{/if}
									{#if image.mimeType}· {image.mimeType}{/if}
								</figcaption>
							</figure>
						{/each}
					</div>
				</div>
			{/if}

			{#if isTmdbMovie || isTmdbTv}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="mb-2 flex items-center justify-between gap-2">
						<h2 class="text-sm font-semibold text-base-content/70 uppercase">
							Trailers{trailers.length > 0 ? ` (${trailers.length})` : ''}
						</h2>
					</div>
					{#if trailerPlayError}
						<div class="mb-2 alert alert-error">
							<span>{trailerPlayError}</span>
						</div>
					{/if}
					{#if trailersStatus === 'loading' && trailers.length === 0}
						<p class="text-sm text-base-content/60">Loading…</p>
					{:else if trailersStatus === 'error' && trailers.length === 0}
						<p class="text-sm text-error">{trailersError ?? 'Failed'}</p>
					{:else if trailers.length === 0}
						<p class="text-sm text-base-content/60">No trailers found.</p>
					{:else}
						<ol class="flex flex-col gap-1">
							{#each trailers as trailer (trailer.key)}
								{@const playable = trailer.status === 'found' && !!trailer.youtubeUrl}
								{@const isPlaying = playingTrailerKey === trailer.key}
								<li>
									<button
										type="button"
										class={classNames(
											'flex w-full flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs',
											{
												'cursor-pointer hover:bg-base-100': playable && !isPlaying,
												'opacity-60': isPlaying,
												'cursor-default': !playable
											}
										)}
										disabled={!playable || playingTrailerKey !== null}
										onclick={() => playTrailer(trailer)}
										title={playable
											? `Play ${trailer.label ?? 'trailer'}`
											: (trailer.label ?? 'Trailer')}
									>
										<span class="flex-1 truncate">
											{trailer.label ?? 'Trailer'}
										</span>
										{#if trailer.status === 'pending'}
											<span class="badge badge-ghost badge-xs">YT queued</span>
										{:else if trailer.status === 'searching'}
											<span class="badge badge-ghost badge-xs">YT…</span>
										{:else if playable}
											{#if isPlaying}
												<span class="badge badge-xs badge-primary">starting…</span>
											{:else}
												<span class="badge badge-xs badge-primary">▶ Play</span>
											{/if}
										{:else if trailer.status === 'missing'}
											<span class="badge badge-xs badge-warning">no match</span>
										{:else if trailer.status === 'error'}
											<span class="badge badge-xs badge-error">error</span>
										{/if}
									</button>
								</li>
							{/each}
						</ol>
					{/if}
				</div>
			{/if}

			{#if isMusicBrainz}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="mb-2 flex items-center justify-between gap-2">
						<h2 class="text-sm font-semibold text-base-content/70 uppercase">
							Tracks{tracks.length > 0 ? ` (${tracks.length})` : ''}
						</h2>
						<button
							type="button"
							class="btn btn-outline btn-xs"
							onclick={() => loadMusicBrainzTracks(itemId)}
							disabled={tracksStatus === 'loading'}
						>
							{tracksStatus === 'loading' ? 'Loading…' : 'Refresh'}
						</button>
					</div>
					{#if tracksStatus === 'loading' && tracks.length === 0}
						<p class="text-sm text-base-content/60">Loading…</p>
					{:else if tracksStatus === 'error'}
						<p class="text-sm text-error">{tracksError ?? 'Failed'}</p>
					{:else if tracks.length === 0}
						<p class="text-sm text-base-content/60">No tracks found.</p>
					{:else}
						{#if trackPlayError}
							<div class="mb-2 alert alert-error">
								<span>{trackPlayError}</span>
							</div>
						{/if}
						<ol class="flex flex-col gap-1">
							{#each tracks as track, idx (track.id || `${track.position}-${track.title}`)}
								{@const playable = track.youtubeStatus === 'found' && !!track.youtubeUrl}
								{@const isPlaying = playingTrackIndex === idx}
								<li>
									<button
										type="button"
										class={classNames(
											'flex w-full flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs',
											{
												'cursor-pointer hover:bg-base-100': playable && !isPlaying,
												'opacity-60': isPlaying,
												'cursor-default': !playable
											}
										)}
										disabled={!playable || playingTrackIndex !== null}
										onclick={() => playTrack(idx)}
										title={playable ? `Play "${track.title}"` : track.title}
									>
										<span class="w-6 shrink-0 text-right font-mono text-base-content/60"
											>{track.position}</span
										>
										<span class="flex-1 truncate">{track.title}</span>
										<span class="text-base-content/60">{formatDuration(track.lengthMs)}</span>
										{#if track.youtubeStatus === 'pending'}
											<span class="badge badge-ghost badge-xs">YT queued</span>
										{:else if track.youtubeStatus === 'searching'}
											<span class="badge badge-ghost badge-xs">YT…</span>
										{:else if playable}
											{#if isPlaying}
												<span class="badge badge-xs badge-primary">starting…</span>
											{:else}
												<span class="badge badge-xs badge-primary">▶ Play</span>
											{/if}
										{:else if track.youtubeStatus === 'missing'}
											<span class="badge badge-xs badge-warning">no match</span>
										{:else if track.youtubeStatus === 'error'}
											<span class="badge badge-xs badge-error">error</span>
										{/if}
									</button>
								</li>
							{/each}
						</ol>
					{/if}
				</div>
			{:else}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<div class="mb-2 flex items-center justify-between gap-2">
						<h2 class="text-sm font-semibold text-base-content/70 uppercase">
							Torrent search{torrentMatches.length > 0 ? ` (${torrentMatches.length})` : ''}
						</h2>
						<button
							type="button"
							class="btn btn-outline btn-xs"
							onclick={() => runTorrentSearch(title, addon, year)}
							disabled={torrentStatus === 'searching'}
						>
							{torrentStatus === 'searching' ? 'Searching…' : 'Refresh'}
						</button>
					</div>
					{#if assignError}
						<div class="mb-2 alert alert-error">
							<span>{assignError}</span>
						</div>
					{/if}
					{#if torrentStatus === 'searching' && torrentMatches.length === 0}
						<p class="text-sm text-base-content/60">Searching…</p>
					{:else if torrentStatus === 'error'}
						<p class="text-sm text-error">{torrentError ?? 'Failed'}</p>
					{:else if torrentMatches.length === 0}
						<p class="text-sm text-base-content/60">No matching torrents.</p>
					{:else}
						<div class="flex flex-col gap-1">
							{#each torrentMatches as torrent (torrent.infoHash)}
								{@const adding = addingHash === torrent.magnetLink}
								<button
									type="button"
									class={classNames(
										'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-100',
										{ 'opacity-60': adding }
									)}
									onclick={() => assignTorrent(torrent)}
									disabled={addingHash !== null}
									title={torrent.title}
								>
									<span class="font-medium">{torrent.quality ?? '—'}</span>
									<span class="text-success">↑{torrent.seeders}</span>
									<span class="text-warning">↓{torrent.leechers}</span>
									<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
									<span class="truncate text-base-content/70"
										>{torrent.parsedTitle || torrent.title}</span
									>
									{#if adding}
										<span class="ml-auto">…</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</section>
	</div>
</div>

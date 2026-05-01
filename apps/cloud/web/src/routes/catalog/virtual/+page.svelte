<script lang="ts">
	import FirkinCard from '$components/firkins/FirkinCard.svelte';
	import FirkinArtistsSection from '$components/firkins/FirkinArtistsSection.svelte';
	import CatalogPageHeader from '$components/catalog/CatalogPageHeader.svelte';
	import CatalogDescriptionCard from '$components/catalog/CatalogDescriptionCard.svelte';
	import CatalogImagesCard from '$components/catalog/CatalogImagesCard.svelte';
	import CatalogTrailersCard from '$components/catalog/CatalogTrailersCard.svelte';
	import CatalogTracksCard from '$components/catalog/CatalogTracksCard.svelte';
	import CatalogTorrentSearchCard from '$components/catalog/CatalogTorrentSearchCard.svelte';
	import type { CloudFirkin } from '$types/firkin.type';
	import {
		firkinsService,
		addonKind,
		type FirkinAddon,
		type Firkin,
		type ImageMeta,
		type Artist,
		type Trailer
	} from '$lib/firkins.service';
	import type { TorrentResultItem } from '$lib/search.service';
	import { TrailerResolver } from '$services/catalog/trailer-resolver.svelte';
	import { TrackResolver } from '$services/catalog/track-resolver.svelte';
	import { TorrentSearch, startTorrentDownload } from '$services/catalog/torrent-search.svelte';
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
	const thumb = $derived(images[0]?.url ?? null);

	type ArtistsStatus = 'idle' | 'loading' | 'done' | 'error';
	let artists = $state<Artist[]>([]);
	let artistsStatus = $state<ArtistsStatus>('idle');
	let artistsError = $state<string | null>(null);
	let tmdbTrailers = $state<Trailer[]>([]);
	let metadataInitForKey: string | null = null;
	let metadataRun = 0;

	$effect(() => {
		if (!addon || !itemId) return;
		const key = `${addon}:${itemId}`;
		if (metadataInitForKey === key) return;
		metadataInitForKey = key;
		void loadMetadata(addon, itemId, ++metadataRun);
	});

	async function loadMetadata(addon: string, id: string, myRun: number) {
		artistsStatus = 'loading';
		artistsError = null;
		artists = [];
		tmdbTrailers = [];
		try {
			const res = await fetch(
				`${base}/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(id)}/metadata`,
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
			if (myRun !== metadataRun) return;
			artists = Array.isArray(body.artists) ? body.artists : [];
			tmdbTrailers = Array.isArray(body.trailers) ? body.trailers : [];
			artistsStatus = 'done';
		} catch (err) {
			if (myRun !== metadataRun) return;
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

	const trailerResolver = new TrailerResolver();
	let trailersInitForKey: string | null = null;

	$effect(() => {
		if (!title) return;
		if (!isTmdbMovie && !isTmdbTv) return;
		// Wait for /metadata so the resolver can prefer TMDB-sourced trailers
		// over a fuzzy YouTube search.
		if (artistsStatus !== 'done' && artistsStatus !== 'error') return;
		const key = `${addon}:${itemId}:${title}:${year ?? ''}:${artistsStatus}`;
		if (trailersInitForKey === key) return;
		trailersInitForKey = key;
		if (isTmdbMovie) {
			void trailerResolver.resolveMovie({
				addon,
				tmdbMovieId: itemId || null,
				title,
				year,
				stored: tmdbTrailers
			});
		} else {
			void trailerResolver.resolveTv({
				addon,
				tmdbTvId: itemId,
				title,
				stored: tmdbTrailers
			});
		}
	});

	const trackResolver = new TrackResolver();
	const albumArtist = $derived(description.split(' · ')[0]?.trim() ?? '');
	let tracksInitForKey: string | null = null;

	$effect(() => {
		if (!isMusicBrainz || !title) return;
		const key = `${addon}:${itemId}:${title}`;
		if (tracksInitForKey === key) return;
		tracksInitForKey = key;
		void trackResolver.loadByReleaseGroup(
			{ releaseGroupId: itemId },
			{ albumTitle: title, artist: albumArtist, thumb }
		);
	});

	const torrentSearch = new TorrentSearch();
	let torrentSearchInitForKey: string | null = null;

	$effect(() => {
		if (!title || isMusicBrainz) return;
		const key = `${addon}:${title}:${year ?? ''}`;
		if (torrentSearchInitForKey === key) return;
		torrentSearchInitForKey = key;
		void torrentSearch.search({ addon, title, year });
	});

	let bookmarking = $state(false);
	let bookmarkError = $state<string | null>(null);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);

	async function bookmark() {
		if (bookmarking || !title) return;
		bookmarkError = null;
		bookmarking = true;
		try {
			const created: Firkin = await firkinsService.create({
				title,
				artists,
				description,
				images,
				files: [...buildUpstreamSourceFiles(), ...trackResolver.resolvedTrackFiles()],
				year,
				addon: addon as FirkinAddon,
				trailers: trailerResolver.resolvedTrailers()
			});
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			bookmarkError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			bookmarking = false;
		}
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
				trailers: trailerResolver.resolvedTrailers()
			});
			await startTorrentDownload(torrent.magnetLink);
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {title || 'Catalog'}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<CatalogPageHeader
		{title}
		{addon}
		{kindLabel}
		{year}
		extraBadge={{ label: 'virtual', class: 'badge-warning' }}
	>
		{#snippet actions()}
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
		{/snippet}
	</CatalogPageHeader>

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
			<CatalogDescriptionCard {description} />

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

			<CatalogImagesCard {images} />

			{#if isTmdbMovie || isTmdbTv}
				<CatalogTrailersCard resolver={trailerResolver} firkinTitle={title} {thumb} />
			{/if}

			{#if isMusicBrainz}
				<CatalogTracksCard resolver={trackResolver} {thumb} />
			{:else}
				<CatalogTorrentSearchCard
					search={torrentSearch}
					onAssign={assignTorrent}
					{addingHash}
					{assignError}
					onRefresh={() => torrentSearch.search({ addon, title, year })}
				/>
			{/if}
		</section>
	</div>
</div>

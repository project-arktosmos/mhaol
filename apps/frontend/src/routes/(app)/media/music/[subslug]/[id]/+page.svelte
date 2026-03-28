<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import {
		releaseGroupsToDisplay,
		releaseToDisplay,
		artistsToDisplay
	} from 'addons/musicbrainz/transform';
	import type {
		MusicBrainzReleaseGroup,
		MusicBrainzRelease,
		MusicBrainzArtistCredit,
		MusicBrainzArtist
	} from 'addons/musicbrainz/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { getMusicConfig } from 'ui-lib/data/media-registry';
	import type { CatalogAlbum, CatalogArtist, CatalogItem } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import AlbumDetailMeta from 'ui-lib/components/catalog/detail/AlbumDetailMeta.svelte';
	import ArtistDetailMeta from 'ui-lib/components/catalog/detail/ArtistDetailMeta.svelte';
	import CatalogCard from 'ui-lib/components/catalog/CatalogCard.svelte';
	import { catalogItemToCardData } from 'ui-lib/adapters/classes/catalog-card.adapter';

	let subslug = $derived($page.params.subslug ?? '');
	let id = $derived($page.params.id ?? '');
	let config = $derived(getMusicConfig(subslug));
	let isAlbumMode = $derived(subslug === 'album');

	// Shared state
	let catalogItem = $state<CatalogAlbum | CatalogArtist | null>(null);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	// Artist-specific
	let discography = $state<CatalogItem[]>([]);
	// Album-specific
	let albumArtistCredits = $state<MusicBrainzArtistCredit[]>([]);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let isFavorite = $derived(
		config ? $favState.items.some((f) => f.service === config.favService && f.serviceId === id) : false
	);
	let isPinned = $derived(
		config ? $pinState.items.some((p) => p.service === config.pinService && p.serviceId === id) : false
	);

	let isFetching = $derived(
		fetchingId !== null &&
			fetchingId === id &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived($searchStore.fetchedCandidate !== null && fetchingId === id);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent)
			return { terms: true, search: true, searching: false, eval: true, done: true };
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

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingId) {
			const scope = isAlbumMode
				? candidate.analysis?.isDiscography
					? 'discography'
					: 'album'
				: 'discography';
			smartSearchService.saveMusicFetchCache(fetchingId, scope, candidate);
		}
	});

	async function fetchAlbum(albumId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const rgRes = await fetchRaw(`/api/musicbrainz/release-group/${albumId}`);
			if (!rgRes.ok) throw new Error('Failed to fetch');
			const rgData = await rgRes.json();
			albumArtistCredits = (rgData as MusicBrainzReleaseGroup)['artist-credit'] ?? [];
			const display = releaseGroupsToDisplay([rgData as MusicBrainzReleaseGroup]);
			const album = display[0];
			if (!album) throw new Error('No album data');

			let releases: Array<{
				id: string;
				title: string;
				date: string | null;
				status: string | null;
				country: string | null;
				artistCredits: string;
				trackCount: number;
				label: string | null;
				tracks: Array<{
					id: string;
					number: string;
					title: string;
					duration: string | null;
					durationMs: number | null;
					artistCredits: string;
				}>;
			}> = [];
			const rawReleases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (rawReleases.length > 0) {
				const official = rawReleases.find((r) => r.status === 'Official') ?? rawReleases[0];
				const relRes = await fetchRaw(`/api/musicbrainz/release/${official.id}`);
				if (relRes.ok) {
					const rel = releaseToDisplay((await relRes.json()) as MusicBrainzRelease);
					if (rel) releases = [rel];
				}
			}

			catalogItem = {
				id: albumId,
				kind: 'album',
				title: album.title,
				sortTitle: album.title.toLowerCase(),
				year: album.firstReleaseYear || null,
				overview: null,
				posterUrl: album.coverArtUrl,
				backdropUrl: null,
				voteAverage: null,
				voteCount: null,
				parentId: null,
				position: null,
				source: 'musicbrainz',
				sourceId: albumId,
				createdAt: '',
				updatedAt: '',
				metadata: {
					musicbrainzId: albumId,
					primaryType: album.primaryType,
					secondaryTypes: album.secondaryTypes,
					artistCredits: album.artistCredits,
					firstReleaseYear: album.firstReleaseYear,
					coverArtUrl: album.coverArtUrl,
					releases
				}
			};

			const cached = await smartSearchService.checkMusicFetchCache(albumId);
			if (cached && cached.length > 0) {
				fetchingId = albumId;
				const bestEntry = cached.find((e) => e.scope === 'album') ?? cached[0];
				smartSearchService.setSelection({
					title: album.title,
					year: album.firstReleaseYear,
					type: 'music',
					musicbrainzId: albumId,
					artist: album.artistCredits,
					mode: 'fetch',
					musicSearchMode: 'album'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch {
			catalogItem = null;
		}
		loading = false;
	}

	async function fetchArtist(artistId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/musicbrainz/artist/${artistId}`);
			if (!res.ok) throw new Error('Failed to fetch');
			const data: MusicBrainzArtist = await res.json();
			const display = artistsToDisplay([data]);
			const artist = display[0];
			if (!artist) throw new Error('No artist data');

			catalogItem = {
				id: artistId,
				kind: 'artist',
				title: artist.name,
				sortTitle: artist.sortName.toLowerCase(),
				year: artist.beginYear || null,
				overview: null,
				posterUrl: artist.imageUrl,
				backdropUrl: null,
				voteAverage: null,
				voteCount: null,
				parentId: null,
				position: null,
				source: 'musicbrainz',
				sourceId: artistId,
				createdAt: '',
				updatedAt: '',
				metadata: {
					musicbrainzId: artistId,
					sortName: artist.sortName,
					type: artist.type,
					country: artist.country,
					disambiguation: artist.disambiguation,
					beginYear: artist.beginYear,
					endYear: artist.endYear,
					ended: artist.ended,
					tags: artist.tags,
					imageUrl: artist.imageUrl
				}
			};

			const rgs = data['release-groups'] ?? [];
			discography = releaseGroupsToDisplay(rgs)
				.sort(
					(a, b) =>
						(parseInt(b.firstReleaseYear) || 0) - (parseInt(a.firstReleaseYear) || 0)
				)
				.map((a) => ({
					id: a.id,
					kind: 'album' as const,
					title: a.title,
					sortTitle: a.title.toLowerCase(),
					year: a.firstReleaseYear || null,
					overview: null,
					posterUrl: a.coverArtUrl,
					backdropUrl: null,
					voteAverage: null,
					voteCount: null,
					parentId: artistId,
					position: null,
					source: 'musicbrainz' as const,
					sourceId: a.id,
					createdAt: '',
					updatedAt: '',
					metadata: {
						musicbrainzId: a.id,
						primaryType: a.primaryType,
						secondaryTypes: a.secondaryTypes,
						artistCredits: a.artistCredits,
						firstReleaseYear: a.firstReleaseYear,
						coverArtUrl: a.coverArtUrl,
						releases: []
					}
				}));

			const cached = await smartSearchService.checkMusicFetchCache(artistId);
			if (cached && cached.length > 0) {
				fetchingId = artistId;
				const bestEntry = cached.find((e) => e.scope === 'discography') ?? cached[0];
				smartSearchService.setSelection({
					title: artist.name,
					year: '',
					type: 'music',
					musicbrainzId: artistId,
					artist: artist.name,
					mode: 'fetch',
					musicSearchMode: 'artist'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch {
			catalogItem = null;
		}
		loading = false;
	}

	async function handleFetch() {
		if (!catalogItem || !config) return;
		fetchingId = catalogItem.sourceId;
		const musicSearchMode = isAlbumMode ? 'album' : 'artist';
		const artistName = isAlbumMode
			? (catalogItem as CatalogAlbum).metadata.artistCredits
			: catalogItem.title;

		if (!isFetchedForCurrent) {
			const cached = await smartSearchService.checkMusicFetchCache(catalogItem.sourceId);
			if (cached && cached.length > 0) {
				const scope = isAlbumMode ? 'album' : 'discography';
				const bestEntry = cached.find((e) => e.scope === scope) ?? cached[0];
				smartSearchService.setSelection({
					title: catalogItem.title,
					year: catalogItem.year ?? '',
					type: 'music',
					musicbrainzId: catalogItem.sourceId,
					artist: artistName,
					mode: 'fetch',
					musicSearchMode
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: catalogItem.title,
			year: catalogItem.year ?? '',
			type: 'music',
			musicbrainzId: catalogItem.sourceId,
			artist: artistName,
			mode: 'fetch',
			musicSearchMode
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (!catalogItem || !config) return;
		if (isAlbumMode) {
			if (isFavorite) {
				await favoritesService.remove('musicbrainz-album', catalogItem.sourceId);
			} else {
				await favoritesService.add(
					'musicbrainz-album',
					catalogItem.sourceId,
					catalogItem.title
				);
				if (albumArtistCredits.length > 0) {
					await Promise.all(
						albumArtistCredits.map((credit) =>
							favoritesService.addSilent(
								'musicbrainz-artist',
								credit.artist.id,
								credit.artist.name
							)
						)
					);
					await favoritesService.refresh();
				}
			}
		} else {
			await favoritesService.toggle(config.favService, catalogItem.sourceId, catalogItem.title);
		}
	}

	async function handleTogglePin() {
		if (!catalogItem || !config) return;
		await pinsService.toggle(config.pinService, catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => {
		smartSearchService.initializeConfig();
		if (isAlbumMode) fetchAlbum(id);
		else fetchArtist(id);
	});
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
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/media/music/${subslug}`)}
		ontogglefavorite={handleToggleFavorite}
		ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			{#if catalogItem?.kind === 'album'}
				<AlbumDetailMeta item={catalogItem} />
			{:else if catalogItem?.kind === 'artist'}
				<ArtistDetailMeta item={catalogItem} />
				{#if discography.length > 0}
					<div>
						<h3
							class="mb-2 text-xs font-semibold tracking-wide uppercase opacity-50"
						>
							Discography ({discography.length})
						</h3>
						<div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
							{#each discography as album (album.id)}
								<CatalogCard
									card={catalogItemToCardData(album)}
									onclick={() => goto(`${base}/media/music/album/${album.sourceId}`)}
								/>
							{/each}
						</div>
					</div>
				{/if}
			{/if}
		{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/media/music/${subslug}`)}
			>Back</button
		>
	</div>
{/if}

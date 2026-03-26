<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { artistsToDisplay, releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
	import type { MusicBrainzArtist } from 'addons/musicbrainz/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { CatalogArtist } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import ArtistDetailMeta from 'ui-lib/components/catalog/detail/ArtistDetailMeta.svelte';
	import CatalogCard from 'ui-lib/components/catalog/CatalogCard.svelte';
	import { catalogItemToCardData } from 'ui-lib/adapters/classes/catalog-card.adapter';
	import type { CatalogItem } from 'ui-lib/types/catalog.type';

	let catalogItem = $state<CatalogArtist | null>(null);
	let discography = $state<CatalogItem[]>([]);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let id = $derived($page.params.id ?? '');
	let isFavorite = $derived($favState.items.some((f) => f.service === 'musicbrainz-artist' && f.serviceId === id));
	let isPinned = $derived($pinState.items.some((p) => p.service === 'musicbrainz-artist' && p.serviceId === id));

	let isFetching = $derived(
		fetchingId !== null && fetchingId === id &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived($searchStore.fetchedCandidate !== null && fetchingId === id);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) return { terms: true, search: true, searching: false, eval: true, done: true };
		const s = $searchStore;
		return {
			terms: s.selection !== null, search: !s.searching && s.searchResults.length > 0,
			searching: s.searching, eval: s.searchResults.some((r) => r.analysis !== null),
			done: s.fetchedCandidate !== null
		};
	});

	let matchedTorrent = $derived.by(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) { const t = torrentService.findByHash(candidate.infoHash); if (t) return t; }
		return null;
	});

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate && fetchingId) smartSearchService.saveMusicFetchCache(fetchingId, 'discography', candidate);
	});

	async function fetchArtist(artistId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/musicbrainz/artist/${artistId}`);
			if (!res.ok) throw new Error('Failed to fetch artist');
			const data: MusicBrainzArtist = await res.json();
			const display = artistsToDisplay([data]);
			const artist = display[0];
			if (!artist) throw new Error('No artist data');

			catalogItem = {
				id: artistId, kind: 'artist',
				title: artist.name, sortTitle: artist.sortName.toLowerCase(),
				year: artist.beginYear || null, overview: null,
				posterUrl: artist.imageUrl, backdropUrl: null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'musicbrainz', sourceId: artistId,
				createdAt: '', updatedAt: '',
				metadata: {
					musicbrainzId: artistId, sortName: artist.sortName,
					type: artist.type, country: artist.country,
					disambiguation: artist.disambiguation,
					beginYear: artist.beginYear, endYear: artist.endYear,
					ended: artist.ended, tags: artist.tags, imageUrl: artist.imageUrl
				}
			};

			const rgs = data['release-groups'] ?? [];
			discography = releaseGroupsToDisplay(rgs)
				.sort((a, b) => (parseInt(b.firstReleaseYear) || 0) - (parseInt(a.firstReleaseYear) || 0))
				.map((a) => ({
					id: a.id, kind: 'album' as const,
					title: a.title, sortTitle: a.title.toLowerCase(),
					year: a.firstReleaseYear || null, overview: null,
					posterUrl: a.coverArtUrl, backdropUrl: null,
					voteAverage: null, voteCount: null,
					parentId: artistId, position: null,
					source: 'musicbrainz' as const, sourceId: a.id,
					createdAt: '', updatedAt: '',
					metadata: {
						musicbrainzId: a.id, primaryType: a.primaryType,
						secondaryTypes: a.secondaryTypes, artistCredits: a.artistCredits,
						firstReleaseYear: a.firstReleaseYear, coverArtUrl: a.coverArtUrl, releases: []
					}
				}));

			const cached = await smartSearchService.checkMusicFetchCache(artistId);
			if (cached && cached.length > 0) {
				fetchingId = artistId;
				const bestEntry = cached.find((e) => e.scope === 'discography') ?? cached[0];
				smartSearchService.setSelection({
					title: artist.name, year: '', type: 'music',
					musicbrainzId: artistId, artist: artist.name, mode: 'fetch', musicSearchMode: 'artist'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch { catalogItem = null; }
		loading = false;
	}

	async function handleFetch() {
		if (!catalogItem) return;
		fetchingId = catalogItem.sourceId;
		if (!isFetchedForCurrent) {
			const cached = await smartSearchService.checkMusicFetchCache(catalogItem.sourceId);
			if (cached && cached.length > 0) {
				const bestEntry = cached.find((e) => e.scope === 'discography') ?? cached[0];
				smartSearchService.setSelection({
					title: catalogItem.title, year: '', type: 'music',
					musicbrainzId: catalogItem.sourceId, artist: catalogItem.title,
					mode: 'fetch', musicSearchMode: 'artist'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: catalogItem.title, year: '', type: 'music',
			musicbrainzId: catalogItem.sourceId, artist: catalogItem.title,
			mode: 'fetch', musicSearchMode: 'artist'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('musicbrainz-artist', catalogItem.sourceId, catalogItem.title);
	}
	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('musicbrainz-artist', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => { smartSearchService.initializeConfig(); fetchArtist(id); });
</script>

{#if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={isFetching} fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps} torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null}
		{isFavorite} {isPinned}
		onfetch={handleFetch} ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/music/artist`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
	>
		{#snippet extra()}
			<ArtistDetailMeta item={catalogItem!} />
			{#if discography.length > 0}
				<div>
					<h3 class="mb-2 text-xs font-semibold tracking-wide uppercase opacity-50">Discography ({discography.length})</h3>
					<div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
						{#each discography as album (album.id)}
							<CatalogCard card={catalogItemToCardData(album)} onclick={() => goto(`${base}/music/album/${album.sourceId}`)} />
						{/each}
					</div>
				</div>
			{/if}
		{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Artist not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/music/artist`)}>Back to artists</button>
	</div>
{/if}

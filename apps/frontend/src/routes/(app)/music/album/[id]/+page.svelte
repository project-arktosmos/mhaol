<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'addons/musicbrainz/transform';
	import type { MusicBrainzReleaseGroup, MusicBrainzRelease } from 'addons/musicbrainz/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import type { CatalogAlbum } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import AlbumDetailMeta from 'ui-lib/components/catalog/detail/AlbumDetailMeta.svelte';

	let catalogItem = $state<CatalogAlbum | null>(null);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let id = $derived($page.params.id ?? '');
	let isFavorite = $derived($favState.items.some((f) => f.service === 'musicbrainz-album' && f.serviceId === id));
	let isPinned = $derived($pinState.items.some((p) => p.service === 'musicbrainz-album' && p.serviceId === id));

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
		if (candidate && fetchingId) {
			const scope = candidate.analysis?.isDiscography ? 'discography' : 'album';
			smartSearchService.saveMusicFetchCache(fetchingId, scope, candidate);
		}
	});

	async function fetchAlbum(albumId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const rgRes = await fetchRaw(`/api/musicbrainz/release-group/${albumId}`);
			if (!rgRes.ok) throw new Error('Failed to fetch release group');
			const rgData = await rgRes.json();
			const display = releaseGroupsToDisplay([rgData as MusicBrainzReleaseGroup]);
			const album = display[0];
			if (!album) throw new Error('No album data');

			let releases: Array<{ id: string; title: string; date: string | null; status: string | null; country: string | null; artistCredits: string; trackCount: number; label: string | null; tracks: Array<{ id: string; number: string; title: string; duration: string | null; durationMs: number | null; artistCredits: string }> }> = [];
			const rawReleases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (rawReleases.length > 0) {
				const official = rawReleases.find((r) => r.status === 'Official') ?? rawReleases[0];
				const relRes = await fetchRaw(`/api/musicbrainz/release/${official.id}`);
				if (relRes.ok) {
					const rel = releaseToDisplay(await relRes.json() as MusicBrainzRelease);
					if (rel) releases = [rel];
				}
			}

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
					secondaryTypes: album.secondaryTypes, artistCredits: album.artistCredits,
					firstReleaseYear: album.firstReleaseYear, coverArtUrl: album.coverArtUrl,
					releases
				}
			};

			const cached = await smartSearchService.checkMusicFetchCache(albumId);
			if (cached && cached.length > 0) {
				fetchingId = albumId;
				const bestEntry = cached.find((e) => e.scope === 'album') ?? cached[0];
				smartSearchService.setSelection({
					title: album.title, year: album.firstReleaseYear, type: 'music',
					musicbrainzId: albumId, artist: album.artistCredits, mode: 'fetch', musicSearchMode: 'album'
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
				const bestEntry = cached.find((e) => e.scope === 'album') ?? cached[0];
				smartSearchService.setSelection({
					title: catalogItem.title, year: catalogItem.year ?? '', type: 'music',
					musicbrainzId: catalogItem.sourceId, artist: catalogItem.metadata.artistCredits,
					mode: 'fetch', musicSearchMode: 'album'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: catalogItem.title, year: catalogItem.year ?? '', type: 'music',
			musicbrainzId: catalogItem.sourceId, artist: catalogItem.metadata.artistCredits,
			mode: 'fetch', musicSearchMode: 'album'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('musicbrainz-album', catalogItem.sourceId, catalogItem.title);
	}
	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('musicbrainz-album', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => { smartSearchService.initializeConfig(); fetchAlbum(id); });
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
		onback={() => goto(`${base}/music/album`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
	>
		{#snippet extra()}<AlbumDetailMeta item={catalogItem!} />{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Album not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/music/album`)}>Back to albums</button>
	</div>
{/if}

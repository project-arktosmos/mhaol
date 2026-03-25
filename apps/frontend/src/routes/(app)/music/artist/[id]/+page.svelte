<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { artistsToDisplay, releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzArtist,
		DisplayMusicBrainzReleaseGroup,
		MusicBrainzArtist
	} from 'addons/musicbrainz/types';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import ArtistDetailPage from 'ui-lib/components/music/ArtistDetailPage.svelte';

	let artist = $state<DisplayMusicBrainzArtist | null>(null);
	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let loading = $state(true);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;

	let id = $derived($page.params.id ?? '');

	let isFetching = $derived(
		fetchingId !== null &&
			fetchingId === id &&
			$searchStore.fetchedCandidate === null &&
			$searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived(
		$searchStore.fetchedCandidate !== null && fetchingId === id
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
		if (candidate?.infoHash) {
			const t = torrentService.findByHash(candidate.infoHash);
			if (t) return t;
		}
		return null;
	});

	let currentTorrentStatus = $derived(matchedTorrent);

	$effect(() => {
		const candidate = $searchStore.fetchedCandidate;
		const key = fetchingId;
		if (candidate && key) {
			smartSearchService.saveMusicFetchCache(key, 'discography', candidate);
		}
	});

	async function fetchArtist(artistId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/artist/${artistId}`));
			if (!res.ok) throw new Error('Failed to fetch artist');
			const data: MusicBrainzArtist = await res.json();
			const display = artistsToDisplay([data]);
			if (display.length > 0) {
				artist = display[0];
			}

			// Extract release groups (discography)
			const rgs = data['release-groups'] ?? [];
			if (rgs.length > 0) {
				albums = releaseGroupsToDisplay(rgs).sort((a, b) => {
					const ya = parseInt(a.firstReleaseYear) || 0;
					const yb = parseInt(b.firstReleaseYear) || 0;
					return yb - ya;
				});
			}

			// Check fetch cache
			const cached = await smartSearchService.checkMusicFetchCache(artistId);
			if (cached && cached.length > 0) {
				fetchingId = artistId;
				const discoEntry = cached.find((e) => e.scope === 'discography');
				const bestEntry = discoEntry ?? cached[0];
				smartSearchService.setSelection({
					title: artist?.name ?? '',
					year: '',
					type: 'music',
					musicbrainzId: artistId,
					artist: artist?.name ?? '',
					mode: 'fetch',
					musicSearchMode: 'artist'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch {
			artist = null;
		}
		loading = false;
	}

	async function handleFetch() {
		if (!artist) return;
		const isRefetch = isFetchedForCurrent;
		fetchingId = artist.id;
		if (!isRefetch) {
			const cached = await smartSearchService.checkMusicFetchCache(artist.id);
			if (cached && cached.length > 0) {
				const discoEntry = cached.find((e) => e.scope === 'discography');
				const bestEntry = discoEntry ?? cached[0];
				smartSearchService.setSelection({
					title: artist.name,
					year: '',
					type: 'music',
					musicbrainzId: artist.id,
					artist: artist.name,
					mode: 'fetch',
					musicSearchMode: 'artist'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: artist.name,
			year: '',
			type: 'music',
			musicbrainzId: artist.id,
			artist: artist.name,
			mode: 'fetch',
			musicSearchMode: 'artist'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	onMount(() => {
		smartSearchService.initializeConfig();
		fetchArtist(id);
	});
</script>

{#if artist}
	<ArtistDetailPage
		{artist}
		{albums}
		fetching={isFetching}
		fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps}
		torrentStatus={currentTorrentStatus}
		fetchedTorrent={$searchStore.fetchedCandidate
			? {
					name: $searchStore.fetchedCandidate.name,
					quality: $searchStore.fetchedCandidate.analysis?.quality ?? '',
					languages: $searchStore.fetchedCandidate.analysis?.languages ?? ''
				}
			: null}
		onfetch={handleFetch}
		ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/music/artist`)}
		onalbumclick={(albumId) => goto(`${base}/music/album/${albumId}`)}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Artist not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/music/artist`)}>Back to artists</button>
	</div>
{/if}

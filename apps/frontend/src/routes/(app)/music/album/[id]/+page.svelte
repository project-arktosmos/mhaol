<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease,
		MusicBrainzReleaseGroup,
		MusicBrainzRelease
	} from 'addons/musicbrainz/types';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import AlbumDetailPage from 'ui-lib/components/music/AlbumDetailPage.svelte';

	let album = $state<DisplayMusicBrainzReleaseGroup | null>(null);
	let release = $state<DisplayMusicBrainzRelease | null>(null);
	let loading = $state(true);
	let tracksLoading = $state(false);
	let fetchingId = $state<string | null>(null);

	const searchStore = smartSearchService.store;
	const torrentState = torrentService.state;

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
			const scope = candidate.analysis?.isDiscography ? 'discography' : 'album';
			smartSearchService.saveMusicFetchCache(key, scope, candidate);
		}
	});

	async function fetchAlbum(albumId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const rgRes = await fetchRaw(`/api/musicbrainz/release-group/${albumId}`);
			if (!rgRes.ok) throw new Error('Failed to fetch release group');
			const rgData = await rgRes.json();

			const releaseGroups: MusicBrainzReleaseGroup[] = [rgData];
			const display = releaseGroupsToDisplay(releaseGroups);
			if (display.length > 0) {
				album = display[0];
			}

			const releases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (releases.length > 0) {
				tracksLoading = true;
				const official = releases.find((r) => r.status === 'Official') ?? releases[0];
				const relRes = await fetchRaw(`/api/musicbrainz/release/${official.id}`);
				if (relRes.ok) {
					const relData: MusicBrainzRelease = await relRes.json();
					release = releaseToDisplay(relData);
				}
				tracksLoading = false;
			}

			// Check fetch cache
			const cached = await smartSearchService.checkMusicFetchCache(albumId);
			if (cached && cached.length > 0) {
				fetchingId = albumId;
				const albumEntry = cached.find((e) => e.scope === 'album');
				const bestEntry = albumEntry ?? cached[0];
				smartSearchService.setSelection({
					title: album?.title ?? '',
					year: album?.firstReleaseYear ?? '',
					type: 'music',
					musicbrainzId: albumId,
					artist: album?.artistCredits ?? '',
					mode: 'fetch',
					musicSearchMode: 'album'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
			}
		} catch {
			album = null;
		}
		loading = false;
	}

	async function handleFetch() {
		if (!album) return;
		const isRefetch = isFetchedForCurrent;
		fetchingId = album.id;
		if (!isRefetch) {
			const cached = await smartSearchService.checkMusicFetchCache(album.id);
			if (cached && cached.length > 0) {
				const albumEntry = cached.find((e) => e.scope === 'album');
				const bestEntry = albumEntry ?? cached[0];
				smartSearchService.setSelection({
					title: album.title,
					year: album.firstReleaseYear,
					type: 'music',
					musicbrainzId: album.id,
					artist: album.artistCredits,
					mode: 'fetch',
					musicSearchMode: 'album'
				});
				smartSearchService.setFetchedCandidate(bestEntry.candidate);
				return;
			}
		}
		smartSearchService.select({
			title: album.title,
			year: album.firstReleaseYear,
			type: 'music',
			musicbrainzId: album.id,
			artist: album.artistCredits,
			mode: 'fetch',
			musicSearchMode: 'album'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (!candidate) return;
		smartSearchService.startDownload(candidate);
	}

	onMount(() => {
		smartSearchService.initializeConfig();
		fetchAlbum(id);
	});
</script>

{#if album}
	<AlbumDetailPage
		{album}
		{release}
		loading={tracksLoading}
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
		onback={() => goto(`${base}/music/album`)}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Album not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/music/album`)}>Back to albums</button>
	</div>
{/if}

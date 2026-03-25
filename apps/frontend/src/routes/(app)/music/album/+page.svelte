<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		MusicBrainzReleaseGroup
	} from 'addons/musicbrainz/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import AlbumCard from 'ui-lib/components/music/AlbumCard.svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import classNames from 'classnames';

	const GENRES = [
		'rock', 'pop', 'electronic', 'hip hop', 'jazz', 'classical', 'r&b', 'metal',
		'folk', 'soul', 'punk', 'blues', 'country', 'ambient', 'indie', 'alternative'
	];

	let selectedGenre = $state('rock');
	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let searchResults = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let searchLoading = $state(false);

	let filteredAlbums = $derived.by(() => {
		if (!searchQuery.trim()) return albums;
		const lower = searchQuery.toLowerCase();
		return albums.filter((a) => a.title.toLowerCase().includes(lower) || a.artistCredits.toLowerCase().includes(lower));
	});

	async function handleSearch() {
		const q = searchQuery.trim();
		if (!q) { searchResults = []; return; }
		searchLoading = true;
		try {
			const res = await fetchRaw(`/api/musicbrainz/search/release-groups?q=${encodeURIComponent(q)}`);
			if (!res.ok) throw new Error('Search failed');
			const data = await res.json();
			const releaseGroups: MusicBrainzReleaseGroup[] = data['release-groups'] ?? [];
			searchResults = releaseGroupsToDisplay(releaseGroups);
		} catch {
			searchResults = [];
		}
		searchLoading = false;
	}

	let albumTorrentMap = $state<Record<string, string>>({});
	let fetchCacheHashes: Map<string, string> = $state(new Map());
	const torrentState = torrentService.state;
	const searchStore = smartSearchService.store;

	async function loadMusicFetchCacheHashes() {
		try {
			const res = await fetchRaw('/api/torrent/music-fetch-cache/hashes');
			if (res.ok) {
				const entries: Array<{ musicbrainzId: string; infoHash: string }> = await res.json();
				fetchCacheHashes = new Map(entries.map((e) => [e.musicbrainzId, e.infoHash]));
			}
		} catch {
			// best-effort
		}
	}

	$effect(() => {
		const s = $searchStore;
		if (s.selection?.type === 'music' && s.downloadedHash) {
			albumTorrentMap[s.selection.musicbrainzId] = s.downloadedHash;
			loadMusicFetchCacheHashes();
		}
	});

	let albumTorrents = $derived.by(() => {
		const torrents = $torrentState.allTorrents;
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const result: Record<string, TorrentInfo> = {};
		for (const [mbId, infoHash] of fetchCacheHashes) {
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) result[mbId] = torrent;
		}
		for (const [albumId, infoHash] of Object.entries(albumTorrentMap)) {
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) result[albumId] = torrent;
		}
		return result;
	});

	let genreCache: Record<string, DisplayMusicBrainzReleaseGroup[]> = {};

	async function fetchPopularAlbums(genre: string) {
		if (genreCache[genre]) { albums = genreCache[genre]; return; }
		loading = true;
		error = null;
		try {
			const res = await fetchRaw(`/api/musicbrainz/popular?genre=${encodeURIComponent(genre)}`);
			if (!res.ok) throw new Error('Failed to fetch popular albums');
			const data = await res.json();
			const releaseGroups: MusicBrainzReleaseGroup[] = data['release-groups'] ?? [];
			const display = releaseGroupsToDisplay(releaseGroups);
			genreCache[genre] = display;
			albums = display;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			albums = [];
		}
		loading = false;
	}

	function handleGenreChange(genre: string) {
		selectedGenre = genre;
		searchResults = [];
		searchQuery = '';
		fetchPopularAlbums(genre);
	}

	function handleSelectAlbum(album: DisplayMusicBrainzReleaseGroup) {
		goto(`${base}/music/album/${album.id}`);
	}

	onMount(() => {
		fetchPopularAlbums(selectedGenre);
		loadMusicFetchCacheHashes();
	});
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-hidden">
	<BrowseHeader title="Popular Albums" count={filteredAlbums.length} countLabel="albums">
		{#snippet controls()}
			<form class="join" onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
				<input
					type="text"
					placeholder="Search albums..."
					class="input join-item input-sm input-bordered w-48"
					bind:value={searchQuery}
				/>
				<button type="submit" class="btn join-item btn-sm btn-primary" disabled={searchLoading}>
					{#if searchLoading}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						Search
					{/if}
				</button>
			</form>
		{/snippet}
		{#snippet tabs()}
			{#each GENRES as genre}
				<button
					class={classNames('btn btn-xs', {
						'btn-primary': selectedGenre === genre,
						'btn-ghost': selectedGenre !== genre
					})}
					onclick={() => handleGenreChange(genre)}
				>
					{genre}
				</button>
			{/each}
		{/snippet}
	</BrowseHeader>

	{#if searchResults.length > 0}
		<div class="border-b border-base-300 px-4 py-3">
			<h3 class="mb-2 text-sm font-semibold opacity-50">Search Results ({searchResults.length})</h3>
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
				{#each searchResults as album (album.id)}
					{@const torrent = albumTorrents[album.id]}
					<AlbumCard
						{album}
						torrentProgress={torrent?.progress ?? null}
						torrentState={torrent?.state ?? null}
						torrentSpeed={torrent?.downloadSpeed ?? null}
						torrentEta={torrent?.eta ?? null}
						onselect={handleSelectAlbum}
					/>
				{/each}
			</div>
		</div>
	{/if}

	<BrowseGrid
		items={filteredAlbums}
		{loading}
		{error}
		emptyTitle="No albums found"
		onretry={() => fetchPopularAlbums(selectedGenre)}
	>
		{#snippet card(item)}
			{@const album = item as DisplayMusicBrainzReleaseGroup}
			{@const torrent = albumTorrents[album.id]}
			<AlbumCard
				{album}
				torrentProgress={torrent?.progress ?? null}
				torrentState={torrent?.state ?? null}
				torrentSpeed={torrent?.downloadSpeed ?? null}
				torrentEta={torrent?.eta ?? null}
				onselect={handleSelectAlbum}
			/>
		{/snippet}
	</BrowseGrid>
</div>

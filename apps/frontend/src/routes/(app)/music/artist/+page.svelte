<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { artistsToDisplay } from 'addons/musicbrainz/transform';
	import type { DisplayMusicBrainzArtist, MusicBrainzArtist } from 'addons/musicbrainz/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import ArtistCard from 'ui-lib/components/music/ArtistCard.svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import classNames from 'classnames';

	const GENRES = [
		'rock', 'pop', 'electronic', 'hip hop', 'jazz', 'classical', 'r&b', 'metal',
		'folk', 'soul', 'punk', 'blues', 'country', 'ambient', 'indie', 'alternative'
	];

	let selectedGenre = $state('rock');
	let artists = $state<DisplayMusicBrainzArtist[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let searchResults = $state<DisplayMusicBrainzArtist[]>([]);
	let searchLoading = $state(false);

	let filteredArtists = $derived.by(() => {
		if (!searchQuery.trim()) return artists;
		const lower = searchQuery.toLowerCase();
		return artists.filter((a) => a.name.toLowerCase().includes(lower));
	});

	async function handleSearch() {
		const q = searchQuery.trim();
		if (!q) { searchResults = []; return; }
		searchLoading = true;
		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/search/artists?q=${encodeURIComponent(q)}`));
			if (!res.ok) throw new Error('Search failed');
			const data = await res.json();
			const rawArtists: MusicBrainzArtist[] = data.artists ?? [];
			searchResults = artistsToDisplay(rawArtists);
		} catch {
			searchResults = [];
		}
		searchLoading = false;
	}

	const torrentState = torrentService.state;
	let fetchCacheHashes: Map<string, string> = $state(new Map());

	async function loadMusicFetchCacheHashes() {
		try {
			const res = await fetch(apiUrl('/api/torrent/music-fetch-cache/hashes'));
			if (res.ok) {
				const entries: Array<{ musicbrainzId: string; infoHash: string }> = await res.json();
				fetchCacheHashes = new Map(entries.map((e) => [e.musicbrainzId, e.infoHash]));
			}
		} catch {
			// best-effort
		}
	}

	let artistTorrents = $derived.by(() => {
		const torrents = $torrentState.allTorrents;
		if (torrents.length === 0 || fetchCacheHashes.size === 0) return new Map<string, TorrentInfo>();
		const torrentsByHash = new Map(torrents.map((t) => [t.infoHash, t]));
		const result = new Map<string, TorrentInfo>();
		for (const [mbId, infoHash] of fetchCacheHashes) {
			const torrent = torrentsByHash.get(infoHash);
			if (torrent) result.set(mbId, torrent);
		}
		return result;
	});

	let genreCache: Record<string, DisplayMusicBrainzArtist[]> = {};

	async function fetchPopularArtists(genre: string) {
		if (genreCache[genre]) { artists = genreCache[genre]; return; }
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/popular-artists?genre=${encodeURIComponent(genre)}`));
			if (!res.ok) throw new Error('Failed to fetch popular artists');
			const data = await res.json();
			const rawArtists: MusicBrainzArtist[] = data.artists ?? [];
			const display = artistsToDisplay(rawArtists);
			genreCache[genre] = display;
			artists = display;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			artists = [];
		}
		loading = false;
	}

	function handleGenreChange(genre: string) {
		selectedGenre = genre;
		searchResults = [];
		searchQuery = '';
		fetchPopularArtists(genre);
	}

	function handleSelectArtist(artist: DisplayMusicBrainzArtist) {
		goto(`/music/artist/${artist.id}`);
	}

	onMount(() => {
		fetchPopularArtists(selectedGenre);
		loadMusicFetchCacheHashes();
	});
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-hidden">
	<BrowseHeader title="Popular Artists" count={filteredArtists.length} countLabel="artists">
		{#snippet controls()}
			<form class="join" onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
				<input
					type="text"
					placeholder="Search artists..."
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
				{#each searchResults as artist (artist.id)}
					{@const torrent = artistTorrents.get(artist.id)}
					<ArtistCard
						{artist}
						torrentProgress={torrent?.progress ?? null}
						torrentState={torrent?.state ?? null}
						torrentSpeed={torrent?.downloadSpeed ?? null}
						torrentEta={torrent?.eta ?? null}
						onselect={handleSelectArtist}
					/>
				{/each}
			</div>
		</div>
	{/if}

	<BrowseGrid
		items={filteredArtists}
		{loading}
		{error}
		emptyTitle="No artists found"
		onretry={() => fetchPopularArtists(selectedGenre)}
	>
		{#snippet card(item)}
			{@const artist = item as DisplayMusicBrainzArtist}
			{@const torrent = artistTorrents.get(artist.id)}
			<ArtistCard
				{artist}
				torrentProgress={torrent?.progress ?? null}
				torrentState={torrent?.state ?? null}
				torrentSpeed={torrent?.downloadSpeed ?? null}
				torrentEta={torrent?.eta ?? null}
				onselect={handleSelectArtist}
			/>
		{/snippet}
	</BrowseGrid>
</div>

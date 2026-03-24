<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
	import { artistsToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzArtist,
		MusicBrainzReleaseGroup,
		MusicBrainzArtist
	} from 'addons/musicbrainz/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import AlbumCard from 'ui-lib/components/music/AlbumCard.svelte';
	import ArtistCard from 'ui-lib/components/music/ArtistCard.svelte';
	import { torrentService } from 'ui-lib/services/torrent.service';

	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let artists = $state<DisplayMusicBrainzArtist[]>([]);
	let albumsLoading = $state(false);
	let artistsLoading = $state(false);

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

	let musicTorrents = $derived.by(() => {
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

	async function fetchAlbums() {
		albumsLoading = true;
		try {
			const res = await fetch(apiUrl('/api/musicbrainz/popular?genre=rock'));
			if (!res.ok) throw new Error('Failed to fetch albums');
			const data = await res.json();
			const releaseGroups: MusicBrainzReleaseGroup[] = data['release-groups'] ?? [];
			albums = releaseGroupsToDisplay(releaseGroups).slice(0, 6);
		} catch {
			albums = [];
		}
		albumsLoading = false;
	}

	async function fetchArtists() {
		artistsLoading = true;
		try {
			const res = await fetch(apiUrl('/api/musicbrainz/popular-artists?genre=rock'));
			if (!res.ok) throw new Error('Failed to fetch artists');
			const data = await res.json();
			const rawArtists: MusicBrainzArtist[] = data.artists ?? [];
			artists = artistsToDisplay(rawArtists).slice(0, 6);
		} catch {
			artists = [];
		}
		artistsLoading = false;
	}

	onMount(() => {
		fetchAlbums();
		fetchArtists();
		loadMusicFetchCacheHashes();
	});
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-y-auto p-4">
	<h1 class="mb-6 text-2xl font-bold">Music</h1>

	<section class="mb-8">
		<div class="mb-3 flex items-center justify-between">
			<h2 class="text-lg font-semibold">Albums</h2>
			<a href="/music/album" class="btn btn-ghost btn-sm">View all</a>
		</div>
		{#if albumsLoading}
			<div class="flex items-center justify-center py-12">
				<span class="loading loading-spinner loading-md"></span>
			</div>
		{:else if albums.length === 0}
			<p class="py-8 text-center text-sm opacity-50">No albums available</p>
		{:else}
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
				{#each albums as album (album.id)}
					{@const torrent = musicTorrents.get(album.id)}
					<AlbumCard
						{album}
						torrentProgress={torrent?.progress ?? null}
						torrentState={torrent?.state ?? null}
						torrentSpeed={torrent?.downloadSpeed ?? null}
						torrentEta={torrent?.eta ?? null}
						onselect={(a) => goto(`/music/album/${a.id}`)}
					/>
				{/each}
			</div>
		{/if}
	</section>

	<section>
		<div class="mb-3 flex items-center justify-between">
			<h2 class="text-lg font-semibold">Artists</h2>
			<a href="/music/artist" class="btn btn-ghost btn-sm">View all</a>
		</div>
		{#if artistsLoading}
			<div class="flex items-center justify-center py-12">
				<span class="loading loading-spinner loading-md"></span>
			</div>
		{:else if artists.length === 0}
			<p class="py-8 text-center text-sm opacity-50">No artists available</p>
		{:else}
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
				{#each artists as artist (artist.id)}
					{@const torrent = musicTorrents.get(artist.id)}
					<ArtistCard
						{artist}
						torrentProgress={torrent?.progress ?? null}
						torrentState={torrent?.state ?? null}
						torrentSpeed={torrent?.downloadSpeed ?? null}
						torrentEta={torrent?.eta ?? null}
						onselect={(a) => goto(`/music/artist/${a.id}`)}
					/>
				{/each}
			</div>
		{/if}
	</section>
</div>

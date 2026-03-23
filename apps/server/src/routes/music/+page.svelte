<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { releaseGroupsToDisplay } from 'addons/musicbrainz/transform';
	import { artistsToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzArtist,
		MusicBrainzReleaseGroup,
		MusicBrainzArtist
	} from 'addons/musicbrainz/types';
	import AlbumCard from 'ui-lib/components/music/AlbumCard.svelte';
	import ArtistCard from 'ui-lib/components/music/ArtistCard.svelte';

	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let artists = $state<DisplayMusicBrainzArtist[]>([]);
	let albumsLoading = $state(false);
	let artistsLoading = $state(false);

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
					<AlbumCard {album} />
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
					<ArtistCard {artist} />
				{/each}
			</div>
		{/if}
	</section>
</div>

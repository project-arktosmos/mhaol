<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { artistsToDisplay } from 'addons/musicbrainz/transform';
	import type { DisplayMusicBrainzArtist, MusicBrainzArtist } from 'addons/musicbrainz/types';
	import ArtistDetailPage from 'ui-lib/components/music/ArtistDetailPage.svelte';

	let artist = $state<DisplayMusicBrainzArtist | null>(null);
	let loading = $state(true);

	let id = $derived($page.params.id ?? '');

	async function fetchArtist(artistId: string) {
		loading = true;
		try {
			const res = await fetch(apiUrl(`/api/musicbrainz/artist/${artistId}`));
			if (!res.ok) throw new Error('Failed to fetch artist');
			const data: MusicBrainzArtist = await res.json();
			const display = artistsToDisplay([data]);
			if (display.length > 0) {
				artist = display[0];
			}
		} catch {
			artist = null;
		}
		loading = false;
	}

	onMount(() => {
		fetchArtist(id);
	});
</script>

{#if artist}
	<ArtistDetailPage {artist} onback={() => goto('/music/artist')} />
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Artist not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/music/artist')}>Back to artists</button>
	</div>
{/if}

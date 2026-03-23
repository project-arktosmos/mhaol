<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'addons/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease,
		MusicBrainzReleaseGroup,
		MusicBrainzRelease
	} from 'addons/musicbrainz/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import AlbumDetailPage from 'ui-lib/components/music/AlbumDetailPage.svelte';

	let album = $state<DisplayMusicBrainzReleaseGroup | null>(null);
	let release = $state<DisplayMusicBrainzRelease | null>(null);
	let loading = $state(true);
	let tracksLoading = $state(false);
	let torrentHash = $state<string | null>(null);

	const torrentState = torrentService.state;
	const searchStore = smartSearchService.store;

	let torrent = $derived.by(() => {
		if (!torrentHash) return null;
		return $torrentState.torrents.find((t) => t.infoHash === torrentHash) ?? null;
	});

	$effect(() => {
		const s = $searchStore;
		if (s.selection?.type === 'music' && s.downloadedHash) {
			torrentHash = s.downloadedHash;
		}
	});

	let id = $derived($page.params.id ?? '');

	async function fetchAlbum(albumId: string) {
		loading = true;
		try {
			const rgRes = await fetch(apiUrl(`/api/musicbrainz/release-group/${albumId}`));
			if (!rgRes.ok) throw new Error('Failed to fetch release group');
			const rgData = await rgRes.json();

			// Build display album from the release-group data
			const releaseGroups: MusicBrainzReleaseGroup[] = [rgData];
			const display = releaseGroupsToDisplay(releaseGroups);
			if (display.length > 0) {
				album = display[0];
			}

			// Fetch tracks from first official release
			const releases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (releases.length > 0) {
				tracksLoading = true;
				const official = releases.find((r) => r.status === 'Official') ?? releases[0];
				const relRes = await fetch(apiUrl(`/api/musicbrainz/release/${official.id}`));
				if (relRes.ok) {
					const relData: MusicBrainzRelease = await relRes.json();
					release = releaseToDisplay(relData);
				}
				tracksLoading = false;
			}
		} catch {
			album = null;
		}
		loading = false;
	}

	function handleDownload() {
		if (!album) return;
		smartSearchService.select({
			title: album.title,
			year: album.firstReleaseYear,
			type: 'music',
			musicbrainzId: album.id,
			artist: album.artistCredits,
			mode: 'download'
		});
	}

	onMount(() => {
		fetchAlbum(id);
	});
</script>

{#if album}
	<AlbumDetailPage
		{album}
		{release}
		{torrent}
		loading={tracksLoading}
		ondownloadalbum={handleDownload}
		onback={() => goto('/music/album')}
	/>
{:else if loading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Album not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/music/album')}>Back to albums</button>
	</div>
{/if}

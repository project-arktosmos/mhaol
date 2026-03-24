<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
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

	let albumTorrentMap = $state<Record<string, string>>({});
	let fetchCacheHashes: Map<string, string> = $state(new Map());
	const torrentState = torrentService.state;
	const searchStore = smartSearchService.store;

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
			const res = await fetch(apiUrl(`/api/musicbrainz/popular?genre=${encodeURIComponent(genre)}`));
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
		fetchPopularAlbums(genre);
	}

	function handleSelectAlbum(album: DisplayMusicBrainzReleaseGroup) {
		goto(`/music/album/${album.id}`);
	}

	onMount(() => {
		fetchPopularAlbums(selectedGenre);
		loadMusicFetchCacheHashes();
	});
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-hidden">
	<BrowseHeader title="Popular Albums" count={albums.length} countLabel="albums">
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

	<BrowseGrid
		items={albums}
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

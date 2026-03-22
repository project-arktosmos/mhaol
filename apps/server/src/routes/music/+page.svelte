<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'ui-lib/utils/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease,
		MusicBrainzReleaseGroup,
		MusicBrainzRelease
	} from 'ui-lib/types/musicbrainz.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';
	import AlbumCard from 'ui-lib/components/music/AlbumCard.svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { browseDetailService } from 'ui-lib/services/browse-detail.service';
	import classNames from 'classnames';

	const GENRES = [
		'rock', 'pop', 'electronic', 'hip hop', 'jazz', 'classical', 'r&b', 'metal',
		'folk', 'soul', 'punk', 'blues', 'country', 'ambient', 'indie', 'alternative'
	];

	let selectedGenre = $state('rock');
	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let selectedAlbum = $state<DisplayMusicBrainzReleaseGroup | null>(null);
	let selectedRelease = $state<DisplayMusicBrainzRelease | null>(null);
	let tracksLoading = $state(false);

	let albumTorrentMap = $state<Record<string, string>>({});
	const torrentState = torrentService.state;
	const searchStore = smartSearchService.store;

	$effect(() => {
		const s = $searchStore;
		if (s.selection?.type === 'music' && s.downloadedHash) {
			albumTorrentMap[s.selection.musicbrainzId] = s.downloadedHash;
		}
	});

	let albumTorrents = $derived.by(() => {
		const torrents = $torrentState.torrents;
		const result: Record<string, TorrentInfo> = {};
		for (const [albumId, infoHash] of Object.entries(albumTorrentMap)) {
			const torrent = torrents.find((t) => t.infoHash === infoHash);
			if (torrent) result[albumId] = torrent;
		}
		return result;
	});

	let genreCache: Record<string, DisplayMusicBrainzReleaseGroup[]> = {};
	let releaseCache: Record<string, DisplayMusicBrainzRelease> = {};

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

	async function fetchAlbumTracks(albumId: string) {
		if (releaseCache[albumId]) { selectedRelease = releaseCache[albumId]; syncToService(); return; }
		tracksLoading = true;
		selectedRelease = null;
		try {
			const rgRes = await fetch(apiUrl(`/api/musicbrainz/release-group/${albumId}`));
			if (!rgRes.ok) throw new Error('Failed to fetch release group');
			const rgData = await rgRes.json();
			const releases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (releases.length === 0) { tracksLoading = false; syncToService(); return; }
			const official = releases.find((r) => r.status === 'Official') ?? releases[0];
			const relRes = await fetch(apiUrl(`/api/musicbrainz/release/${official.id}`));
			if (!relRes.ok) throw new Error('Failed to fetch release');
			const relData: MusicBrainzRelease = await relRes.json();
			const display = releaseToDisplay(relData);
			releaseCache[albumId] = display;
			selectedRelease = display;
		} catch { selectedRelease = null; }
		tracksLoading = false;
		syncToService();
	}

	function handleGenreChange(genre: string) {
		selectedGenre = genre;
		selectedAlbum = null;
		selectedRelease = null;
		browseDetailService.close();
		fetchPopularAlbums(genre);
	}

	function handleSelectAlbum(album: DisplayMusicBrainzReleaseGroup) {
		if (selectedAlbum?.id === album.id) {
			selectedAlbum = null;
			selectedRelease = null;
			browseDetailService.close();
		} else {
			selectedAlbum = album;
			selectedRelease = null;
			syncToService();
			fetchAlbumTracks(album.id);
		}
	}

	function handleDownloadAlbum() {
		if (!selectedAlbum) return;
		smartSearchService.select({
			title: selectedAlbum.title,
			year: selectedAlbum.firstReleaseYear,
			type: 'music',
			musicbrainzId: selectedAlbum.id,
			artist: selectedAlbum.artistCredits,
			mode: 'download'
		});
	}

	function syncToService() {
		if (!selectedAlbum) return;
		const torrent = albumTorrents[selectedAlbum.id] ?? null;
		browseDetailService.set({
			domain: 'music',
			musicAlbum: selectedAlbum,
			musicRelease: selectedRelease,
			musicTorrent: torrent,
			loading: tracksLoading
		});
		browseDetailService.registerCallbacks({
			onclose: () => { selectedAlbum = null; selectedRelease = null; browseDetailService.close(); },
			ondownloadalbum: handleDownloadAlbum
		});
	}

	// Keep torrent state in sync with the service
	$effect(() => {
		if (selectedAlbum) {
			const torrent = albumTorrents[selectedAlbum.id] ?? null;
			browseDetailService.update(() => ({ musicTorrent: torrent }));
		}
	});

	onMount(() => { fetchPopularAlbums(selectedGenre); });
	onDestroy(() => { browseDetailService.close(); });
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
				selected={selectedAlbum?.id === album.id}
				torrentProgress={torrent?.progress ?? null}
				torrentState={torrent?.state ?? null}
				torrentSpeed={torrent?.downloadSpeed ?? null}
				torrentEta={torrent?.eta ?? null}
				onselect={handleSelectAlbum}
			/>
		{/snippet}
	</BrowseGrid>
</div>

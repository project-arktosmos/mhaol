<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { releaseGroupsToDisplay, releaseToDisplay } from 'frontend/utils/musicbrainz/transform';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease,
		MusicBrainzReleaseGroup,
		MusicBrainzRelease
	} from 'frontend/types/musicbrainz.type';
	import type { TorrentInfo } from 'frontend/types/torrent.type';
	import { formatDuration } from 'frontend/utils/musicbrainz/transform';
	import AlbumCard from 'ui-lib/components/music/AlbumCard.svelte';
	import { smartSearchService } from 'frontend/services/smart-search.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import classNames from 'classnames';

	const GENRES = [
		'rock',
		'pop',
		'electronic',
		'hip hop',
		'jazz',
		'classical',
		'r&b',
		'metal',
		'folk',
		'soul',
		'punk',
		'blues',
		'country',
		'ambient',
		'indie',
		'alternative'
	];

	let selectedGenre = $state('rock');
	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let selectedAlbum = $state<DisplayMusicBrainzReleaseGroup | null>(null);

	// Tracks for the selected album
	let selectedRelease = $state<DisplayMusicBrainzRelease | null>(null);
	let tracksLoading = $state(false);

	// Album → torrent infoHash mapping
	let albumTorrentMap = $state<Record<string, string>>({});

	// Torrent state from service
	const torrentState = torrentService.state;

	// Smart search store — capture mapping when a torrent is actually added
	const searchStore = smartSearchService.store;

	$effect(() => {
		const s = $searchStore;
		if (s.selection?.type === 'music' && s.downloadedHash) {
			albumTorrentMap[s.selection.musicbrainzId] = s.downloadedHash;
		}
	});

	// Reactive torrent lookup per album
	let albumTorrents = $derived.by(() => {
		const torrents = $torrentState.torrents;
		const result: Record<string, TorrentInfo> = {};
		for (const [albumId, infoHash] of Object.entries(albumTorrentMap)) {
			const torrent = torrents.find((t) => t.infoHash === infoHash);
			if (torrent) result[albumId] = torrent;
		}
		return result;
	});

	// Cache per genre
	let genreCache: Record<string, DisplayMusicBrainzReleaseGroup[]> = {};

	// Cache release data per release-group ID
	let releaseCache: Record<string, DisplayMusicBrainzRelease> = {};

	async function fetchPopularAlbums(genre: string) {
		if (genreCache[genre]) {
			albums = genreCache[genre];
			return;
		}

		loading = true;
		error = null;
		try {
			const res = await fetch(
				apiUrl(`/api/musicbrainz/popular?genre=${encodeURIComponent(genre)}`)
			);
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
		if (releaseCache[albumId]) {
			selectedRelease = releaseCache[albumId];
			return;
		}

		tracksLoading = true;
		selectedRelease = null;
		try {
			// Fetch release-group to get the list of releases
			const rgRes = await fetch(apiUrl(`/api/musicbrainz/release-group/${albumId}`));
			if (!rgRes.ok) throw new Error('Failed to fetch release group');
			const rgData = await rgRes.json();

			const releases: MusicBrainzRelease[] = rgData.releases ?? [];
			if (releases.length === 0) {
				tracksLoading = false;
				return;
			}

			// Pick the first official release, or just the first one
			const official = releases.find((r) => r.status === 'Official') ?? releases[0];

			// Fetch the release with tracks
			const relRes = await fetch(apiUrl(`/api/musicbrainz/release/${official.id}`));
			if (!relRes.ok) throw new Error('Failed to fetch release');
			const relData: MusicBrainzRelease = await relRes.json();

			const display = releaseToDisplay(relData);
			releaseCache[albumId] = display;
			selectedRelease = display;
		} catch {
			selectedRelease = null;
		}
		tracksLoading = false;
	}

	function handleGenreChange(genre: string) {
		selectedGenre = genre;
		selectedAlbum = null;
		selectedRelease = null;
		fetchPopularAlbums(genre);
	}

	function handleSelectAlbum(album: DisplayMusicBrainzReleaseGroup) {
		if (selectedAlbum?.id === album.id) {
			selectedAlbum = null;
			selectedRelease = null;
		} else {
			selectedAlbum = album;
			selectedRelease = null;
			fetchAlbumTracks(album.id);
		}
	}

	function handleDownloadAlbum(album: DisplayMusicBrainzReleaseGroup) {
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
		fetchPopularAlbums(selectedGenre);
	});
</script>

<div class="flex h-full w-full">
	<!-- Main content -->
	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- Header -->
		<div class="flex items-center gap-3 border-b border-base-300 px-4 py-3">
			<h2 class="text-lg font-bold">Popular Albums</h2>
			<span class="badge badge-ghost">{albums.length} albums</span>
		</div>

		<!-- Genre tabs -->
		<div class="flex flex-wrap gap-1.5 border-b border-base-300 px-4 py-2">
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
		</div>

		<!-- Grid -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if loading}
				<div class="flex items-center justify-center py-16">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else if error}
				<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
					<p class="text-lg">Failed to load albums</p>
					<p class="mt-1 text-sm">{error}</p>
					<button
						class="btn btn-primary btn-sm mt-4"
						onclick={() => fetchPopularAlbums(selectedGenre)}
					>
						Retry
					</button>
				</div>
			{:else if albums.length === 0}
				<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
					<p class="text-lg">No albums found</p>
				</div>
			{:else}
				<div
					class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each albums as album (album.id)}
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
					{/each}
				</div>
			{/if}
		</div>
	</div>

	<!-- Right sidebar: selected album detail -->
	{#if selectedAlbum}
		{@const torrent = albumTorrents[selectedAlbum.id]}
		<div class="flex w-80 flex-col gap-3 overflow-y-auto border-l border-base-300 bg-base-100 p-4">
			<div class="flex flex-col gap-2">
				{#if selectedAlbum.coverArtUrl}
					<img
						src={selectedAlbum.coverArtUrl}
						alt={selectedAlbum.title}
						class="aspect-square w-full rounded-lg object-cover"
					/>
				{:else}
					<div
						class="flex aspect-square w-full items-center justify-center rounded-lg bg-base-200"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-16 w-16 text-base-content/20"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="1.5"
								d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
							/>
						</svg>
					</div>
				{/if}

				<h3 class="text-sm font-bold">{selectedAlbum.title}</h3>
				<p class="text-xs opacity-60">{selectedAlbum.artistCredits}</p>
				{#if selectedAlbum.firstReleaseYear && selectedAlbum.firstReleaseYear !== 'Unknown'}
					<p class="text-xs opacity-40">{selectedAlbum.firstReleaseYear}</p>
				{/if}

				<div class="flex flex-wrap gap-1">
					{#if selectedAlbum.primaryType}
						<span class="badge badge-ghost badge-sm">{selectedAlbum.primaryType}</span>
					{/if}
					{#each selectedAlbum.secondaryTypes as type_}
						<span class="badge badge-ghost badge-sm">{type_}</span>
					{/each}
				</div>

				<!-- Download progress -->
				{#if torrent}
					<div class="rounded-lg bg-base-200 p-2">
						<div class="flex items-center justify-between text-xs">
							<span class="font-medium">
								{torrent.state === 'seeding' ? 'Complete' : `${Math.round(torrent.progress * 100)}%`}
							</span>
							<span class="badge badge-xs {torrent.state === 'seeding' ? 'badge-success' : 'badge-info'}">
								{torrent.state}
							</span>
						</div>
						{#if torrent.state !== 'seeding'}
							<progress
								class="progress progress-primary mt-1 w-full"
								value={Math.round(torrent.progress * 100)}
								max="100"
							></progress>
						{/if}
					</div>
				{/if}

				<button
					class="btn btn-primary btn-sm mt-2"
					onclick={() => handleDownloadAlbum(selectedAlbum!)}
					disabled={torrent?.state === 'downloading' || torrent?.state === 'seeding'}
				>
					{torrent?.state === 'seeding' ? 'Downloaded' : torrent ? 'Downloading...' : 'Download'}
				</button>
			</div>

			<!-- Tracklist -->
			{#if tracksLoading}
				<div class="flex items-center justify-center py-4">
					<span class="loading loading-sm loading-spinner"></span>
				</div>
			{:else if selectedRelease && selectedRelease.tracks.length > 0}
				<div class="flex flex-col gap-0.5">
					<div class="flex items-center justify-between">
						<h4 class="text-xs font-semibold opacity-50">Tracklist</h4>
						<span class="text-xs opacity-30">{selectedRelease.tracks.length} tracks</span>
					</div>
					{#each selectedRelease.tracks as track (track.id)}
						<div class="flex items-center gap-2 rounded px-1 py-0.5 hover:bg-base-200">
							<span class="w-5 text-right text-xs opacity-30">{track.number}</span>
							<span class="min-w-0 flex-1 truncate text-xs">{track.title}</span>
							{#if track.duration}
								<span class="text-xs opacity-30">{track.duration}</span>
							{/if}
						</div>
					{/each}
				</div>
			{:else if selectedRelease}
				<p class="text-xs opacity-30">No tracks available</p>
			{/if}
		</div>
	{/if}
</div>

<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryFile } from '$types/library.type';
	import type { DisplayMusicBrainzRecording, DisplayMusicBrainzArtist, DisplayMusicBrainzReleaseGroup } from 'musicbrainz/types';
	import { searchRecordings, searchArtists, searchReleaseGroups } from 'musicbrainz';
	import { recordingsToDisplay, artistsToDisplay, releaseGroupsToDisplay } from 'musicbrainz/transform';

	interface Props {
		file: LibraryFile;
		onlink: (musicbrainzId: string) => void;
		onclose: () => void;
	}

	let { file, onlink, onclose }: Props = $props();

	type SearchMode = 'track' | 'artist' | 'album';

	let searchMode: SearchMode = $state('track');
	let query = $state(cleanFilename(file.name));
	let searching = $state(false);
	let trackResults: DisplayMusicBrainzRecording[] = $state([]);
	let artistResults: DisplayMusicBrainzArtist[] = $state([]);
	let albumResults: DisplayMusicBrainzReleaseGroup[] = $state([]);
	let error: string | null = $state(null);

	function cleanFilename(name: string): string {
		return name
			.replace(/\.[^.]+$/, '')
			.replace(/[._]/g, ' ')
			.replace(/\s*[\[(].*?[\])]\s*/g, ' ')
			.replace(/\b(720|1080|2160|480)p?\b/gi, '')
			.replace(/\b(x264|x265|h264|h265|hevc|avc|bluray|bdrip|brrip|webrip|web-dl|hdtv|dvdrip|hdrip)\b/gi, '')
			.replace(/\b(aac|ac3|dts|mp3|flac|atmos|truehd)\b/gi, '')
			.replace(/\b(s\d{1,2}e\d{1,2})\b/gi, '')
			.replace(/\s{2,}/g, ' ')
			.trim();
	}

	function clearResults() {
		trackResults = [];
		artistResults = [];
		albumResults = [];
	}

	async function search() {
		if (!query.trim()) return;
		searching = true;
		error = null;
		clearResults();

		try {
			if (searchMode === 'track') {
				const response = await searchRecordings(query.trim());
				if (response?.recordings) {
					trackResults = recordingsToDisplay(response.recordings);
				}
			} else if (searchMode === 'artist') {
				const response = await searchArtists(query.trim());
				if (response?.artists) {
					artistResults = artistsToDisplay(response.artists);
				}
			} else {
				const response = await searchReleaseGroups(query.trim());
				if (response?.['release-groups']) {
					albumResults = releaseGroupsToDisplay(response['release-groups']);
				}
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			searching = false;
		}
	}

	function selectTrack(track: DisplayMusicBrainzRecording) {
		onlink(track.id);
	}

	function selectArtist(artist: DisplayMusicBrainzArtist) {
		onlink(artist.id);
	}

	function selectAlbum(album: DisplayMusicBrainzReleaseGroup) {
		onlink(album.id);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			search();
		}
	}

	function handleCoverError(e: Event) {
		const img = e.target as HTMLImageElement;
		img.style.display = 'none';
	}

	let hasResults = $derived(trackResults.length > 0 || artistResults.length > 0 || albumResults.length > 0);
</script>

<div class="modal modal-open">
	<div class="modal-box max-w-2xl">
		<button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Link MusicBrainz</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		<div class="mt-4 flex gap-2">
			<div class="join flex-1">
				<input
					type="text"
					class="input input-bordered input-sm join-item w-full"
					placeholder="Search MusicBrainz..."
					bind:value={query}
					onkeydown={handleKeydown}
				/>
				<button class="btn btn-sm btn-primary join-item" onclick={search} disabled={searching || !query.trim()}>
					{#if searching}
						<span class="loading loading-spinner loading-xs"></span>
					{:else}
						Search
					{/if}
				</button>
			</div>
			<div class="join">
				<button
					class={classNames('join-item btn btn-sm', { 'btn-active': searchMode === 'track' })}
					onclick={() => { searchMode = 'track'; clearResults(); }}
				>
					Track
				</button>
				<button
					class={classNames('join-item btn btn-sm', { 'btn-active': searchMode === 'artist' })}
					onclick={() => { searchMode = 'artist'; clearResults(); }}
				>
					Artist
				</button>
				<button
					class={classNames('join-item btn btn-sm', { 'btn-active': searchMode === 'album' })}
					onclick={() => { searchMode = 'album'; clearResults(); }}
				>
					Album
				</button>
			</div>
		</div>

		{#if error}
			<div class="mt-3 rounded-lg bg-error/10 px-3 py-2 text-sm text-error">{error}</div>
		{/if}

		<div class="mt-4 max-h-80 overflow-y-auto">
			{#if searching}
				<div class="flex justify-center py-8">
					<span class="loading loading-spinner loading-md"></span>
				</div>
			{:else if searchMode === 'track' && trackResults.length > 0}
				<div class="flex flex-col gap-2">
					{#each trackResults as track (track.id)}
						<button
							class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
							onclick={() => selectTrack(track)}
						>
							<div class="h-16 w-16 flex-shrink-0 overflow-hidden rounded bg-base-300">
								{#if track.coverArtUrl}
									<img
										src={track.coverArtUrl}
										alt={track.title}
										class="h-full w-full object-cover"
										onerror={handleCoverError}
									/>
								{/if}
							</div>
							<div class="flex-1 overflow-hidden">
								<p class="truncate text-sm font-medium">{track.title}</p>
								<p class="text-xs opacity-60">
									{track.artistCredits}
									{#if track.duration}
										&middot; {track.duration}
									{/if}
								</p>
								{#if track.disambiguation}
									<p class="mt-1 line-clamp-1 text-xs opacity-50">{track.disambiguation}</p>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{:else if searchMode === 'artist' && artistResults.length > 0}
				<div class="flex flex-col gap-2">
					{#each artistResults as artist (artist.id)}
						<button
							class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
							onclick={() => selectArtist(artist)}
						>
							<div class="flex-1 overflow-hidden">
								<p class="truncate text-sm font-medium">{artist.name}</p>
								<p class="text-xs opacity-60">
									{#if artist.type}
										{artist.type}
									{/if}
									{#if artist.country}
										&middot; {artist.country}
									{/if}
									{#if artist.beginYear}
										&middot; {artist.beginYear}{artist.endYear ? `\u2013${artist.endYear}` : ''}
									{/if}
								</p>
								{#if artist.disambiguation}
									<p class="mt-1 line-clamp-2 text-xs opacity-50">{artist.disambiguation}</p>
								{/if}
								{#if artist.tags.length > 0}
									<div class="mt-1 flex flex-wrap gap-1">
										{#each artist.tags.slice(0, 3) as tag}
											<span class="badge badge-ghost badge-xs">{tag}</span>
										{/each}
									</div>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{:else if searchMode === 'album' && albumResults.length > 0}
				<div class="flex flex-col gap-2">
					{#each albumResults as album (album.id)}
						<button
							class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
							onclick={() => selectAlbum(album)}
						>
							<div class="h-16 w-16 flex-shrink-0 overflow-hidden rounded bg-base-300">
								{#if album.coverArtUrl}
									<img
										src={album.coverArtUrl}
										alt={album.title}
										class="h-full w-full object-cover"
										onerror={handleCoverError}
									/>
								{/if}
							</div>
							<div class="flex-1 overflow-hidden">
								<p class="truncate text-sm font-medium">{album.title}</p>
								<p class="text-xs opacity-60">
									{album.firstReleaseYear}
									{#if album.primaryType}
										&middot; {album.primaryType}
									{/if}
								</p>
								{#if album.artistCredits !== 'Unknown Artist'}
									<p class="mt-1 line-clamp-1 text-xs opacity-50">{album.artistCredits}</p>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{:else if !hasResults && !searching && query.trim()}
				<div class="py-8 text-center">
					<p class="text-sm opacity-50">No results found</p>
				</div>
			{/if}
		</div>
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>

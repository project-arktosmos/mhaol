<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type {
		DisplayMusicBrainzArtist,
		DisplayMusicBrainzReleaseGroup
	} from '$types/musicbrainz.type';
	import { musicBrainzService } from '$services/musicbrainz.service';
	import { musicBrainzAdapter } from '$adapters/classes/musicbrainz.adapter';

	type SearchMode = 'artists' | 'albums';

	let searchMode = $state<SearchMode>('artists');
	let artists = $state<DisplayMusicBrainzArtist[]>([]);
	let albums = $state<DisplayMusicBrainzReleaseGroup[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let page = $state(1);
	let totalResults = $state(0);
	let searchQuery = $state('');
	let hasSearched = $state(false);

	// Artist image cache: mbid -> image URL
	let artistImages = $state<Record<string, string | null>>({});

	const limit = 25;
	let totalPages = $derived(Math.ceil(totalResults / limit));

	async function loadArtistImages(artistList: DisplayMusicBrainzArtist[]) {
		for (const artist of artistList) {
			if (artistImages[artist.id] !== undefined) continue;
			musicBrainzService.fetchArtistImage(artist.id).then((url) => {
				artistImages = { ...artistImages, [artist.id]: url };
			});
		}
	}

	async function searchArtists(p: number = 1) {
		if (!searchQuery.trim()) return;

		loading = true;
		error = null;
		hasSearched = true;

		try {
			const offset = (p - 1) * limit;
			const response = await musicBrainzService.searchArtists(searchQuery.trim(), limit, offset);
			if (response && response.artists) {
				artists = musicBrainzAdapter.artistsToDisplay(response.artists);
				totalResults = response.count;
				page = p;
				loadArtistImages(artists);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	async function searchAlbums(p: number = 1) {
		if (!searchQuery.trim()) return;

		loading = true;
		error = null;
		hasSearched = true;

		try {
			const offset = (p - 1) * limit;
			const response = await musicBrainzService.searchReleaseGroups(
				searchQuery.trim(),
				limit,
				offset
			);
			if (response && response['release-groups']) {
				albums = musicBrainzAdapter.releaseGroupsToDisplay(response['release-groups']);
				totalResults = response.count;
				page = p;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleSearch(p: number = 1) {
		if (searchMode === 'artists') {
			searchArtists(p);
		} else {
			searchAlbums(p);
		}
	}

	function handleSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			page = 1;
			handleSearch(1);
		}
	}

	function handleModeChange(mode: SearchMode) {
		searchMode = mode;
		artists = [];
		albums = [];
		totalResults = 0;
		page = 1;
		if (hasSearched && searchQuery.trim()) {
			handleSearch(1);
		}
	}

	function handlePageChange(newPage: number) {
		page = newPage;
		handleSearch(newPage);
	}

	function handleCoverError(e: Event) {
		const img = e.target as HTMLImageElement;
		img.style.display = 'none';
		const placeholder = img.nextElementSibling as HTMLElement | null;
		if (placeholder) {
			placeholder.style.display = 'flex';
		}
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Music</h1>
		<p class="text-sm opacity-70">Search artists and albums from MusicBrainz</p>
	</div>

	<!-- Search Mode Tabs -->
	<div class="mb-4 flex flex-wrap gap-2">
		{#each [['artists', 'Artists'], ['albums', 'Albums']] as [key, label]}
			<button
				class={classNames('btn btn-sm', {
					'btn-primary': searchMode === key,
					'btn-ghost': searchMode !== key
				})}
				onclick={() => handleModeChange(key as SearchMode)}
			>
				{label}
			</button>
		{/each}
	</div>

	<!-- Search Bar -->
	<div class="mb-4">
		<input
			type="text"
			placeholder={searchMode === 'artists' ? 'Search artists...' : 'Search albums...'}
			class="input input-bordered w-full"
			bind:value={searchQuery}
			onkeydown={handleSearchKeydown}
		/>
	</div>

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if !hasSearched}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">
				Search for {searchMode === 'artists' ? 'artists' : 'albums'} to get started.
			</p>
		</div>
	{:else if searchMode === 'artists' && artists.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No artists found.</p>
		</div>
	{:else if searchMode === 'albums' && albums.length === 0}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No albums found.</p>
		</div>
	{:else}
		<div class="mb-4 text-sm opacity-70">
			Page {page} of {totalPages} ({totalResults.toLocaleString()} results)
		</div>

		<!-- Artists Grid -->
		{#if searchMode === 'artists'}
			<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
				{#each artists as artist (artist.id)}
					<a
						href="/music/{artist.id}"
						class="card bg-base-200 shadow-md transition-transform hover:scale-105"
					>
						<figure class="aspect-square">
							{#if artistImages[artist.id]}
								<img
									src={artistImages[artist.id]}
									alt={artist.name}
									class="h-full w-full object-cover"
									onerror={handleCoverError}
								/>
							{/if}
							<div
								class={classNames(
									'h-full w-full items-center justify-center bg-base-300',
									{ 'flex': !artistImages[artist.id], 'hidden': artistImages[artist.id] }
								)}
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="1.5"
									stroke="currentColor"
									class="h-12 w-12 opacity-30"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
									/>
								</svg>
							</div>
						</figure>
						<div class="card-body p-3">
							<h3 class="card-title line-clamp-2 text-sm">{artist.name}</h3>
							<div class="flex items-center gap-2 text-xs opacity-70">
								{#if artist.type}
									<span>{artist.type}</span>
								{/if}
								{#if artist.country}
									<span>{artist.country}</span>
								{/if}
								{#if artist.beginYear}
									<span>{artist.beginYear}{artist.endYear ? `–${artist.endYear}` : ''}</span>
								{/if}
							</div>
							{#if artist.disambiguation}
								<div class="text-xs italic opacity-50 line-clamp-1">
									{artist.disambiguation}
								</div>
							{/if}
							{#if artist.tags.length > 0}
								<div class="flex flex-wrap gap-1">
									{#each artist.tags.slice(0, 2) as tag}
										<span class="badge badge-ghost badge-xs">{tag}</span>
									{/each}
								</div>
							{/if}
						</div>
					</a>
				{/each}
			</div>
		{/if}

		<!-- Albums Grid -->
		{#if searchMode === 'albums'}
			<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
				{#each albums as album (album.id)}
					<a
						href="/music/{album.id}?type=release-group"
						class="card bg-base-200 shadow-md transition-transform hover:scale-105"
					>
						<figure class="aspect-square">
							<img
								src={album.coverArtUrl}
								alt={album.title}
								class="h-full w-full object-cover"
								onerror={handleCoverError}
							/>
							<div
								class="hidden h-full w-full items-center justify-center bg-base-300"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="1.5"
									stroke="currentColor"
									class="h-12 w-12 opacity-30"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z"
									/>
								</svg>
							</div>
						</figure>
						<div class="card-body p-3">
							<h3 class="card-title line-clamp-2 text-sm">{album.title}</h3>
							<div class="flex items-center gap-2 text-xs opacity-70">
								<span>{album.firstReleaseYear}</span>
								{#if album.primaryType}
									<span class="badge badge-ghost badge-xs">{album.primaryType}</span>
								{/if}
							</div>
							{#if album.artistCredits !== 'Unknown Artist'}
								<div class="text-xs opacity-60 line-clamp-1">
									{album.artistCredits}
								</div>
							{/if}
						</div>
					</a>
				{/each}
			</div>
		{/if}

		{#if totalPages > 1}
			<div class="mt-6 flex justify-center gap-2">
				<button
					class="btn btn-sm"
					disabled={page <= 1}
					onclick={() => handlePageChange(page - 1)}
				>
					Previous
				</button>
				<span class="flex items-center px-4 text-sm">
					Page {page} of {totalPages}
				</span>
				<button
					class="btn btn-sm"
					disabled={page >= totalPages}
					onclick={() => handlePageChange(page + 1)}
				>
					Next
				</button>
			</div>
		{/if}
	{/if}
</div>

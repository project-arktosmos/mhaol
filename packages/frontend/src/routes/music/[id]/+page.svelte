<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type {
		DisplayMusicBrainzArtistDetails,
		DisplayMusicBrainzRelease
	} from 'musicbrainz/types';
	import { fetchArtist, fetchReleasesForReleaseGroup, fetchArtistImage } from 'musicbrainz';
	import { artistDetailsToDisplay, releaseToDisplay } from 'musicbrainz/transform';

	let artist = $state<DisplayMusicBrainzArtistDetails | null>(null);
	let artistImageUrl = $state<string | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Track expanded release groups and their loaded release details
	let expandedReleaseGroups = $state<Set<string>>(new Set());
	let releaseDetails = $state<Record<string, DisplayMusicBrainzRelease>>({}),
		releaseLoading = $state<Set<string>>(new Set());

	function handleCoverError(e: Event) {
		const img = e.target as HTMLImageElement;
		img.style.display = 'none';
		const placeholder = img.nextElementSibling as HTMLElement | null;
		if (placeholder) {
			placeholder.style.display = 'flex';
		}
	}

	async function toggleReleaseGroup(releaseGroupId: string) {
		const next = new Set(expandedReleaseGroups);
		if (next.has(releaseGroupId)) {
			next.delete(releaseGroupId);
			expandedReleaseGroups = next;
			return;
		}

		next.add(releaseGroupId);
		expandedReleaseGroups = next;

		if (releaseDetails[releaseGroupId]) return;

		const loadingNext = new Set(releaseLoading);
		loadingNext.add(releaseGroupId);
		releaseLoading = loadingNext;

		try {
			const response =
				await fetchReleasesForReleaseGroup(releaseGroupId);
			if (response && response.releases && response.releases.length > 0) {
				releaseDetails = {
					...releaseDetails,
					[releaseGroupId]: releaseToDisplay(response.releases[0])
				};
			}
		} catch {
			// Silently handle - the UI will just not show tracks
		} finally {
			const loadingDone = new Set(releaseLoading);
			loadingDone.delete(releaseGroupId);
			releaseLoading = loadingDone;
		}
	}

	onMount(async () => {
		const id = $page.params.id;
		if (!id) {
			error = 'Invalid ID';
			loading = false;
			return;
		}

		try {
			const data = await fetchArtist(id);
			if (data) {
				artist = artistDetailsToDisplay(data);
				fetchArtistImage(id).then((url) => {
					artistImageUrl = url;
				});
			} else {
				error = 'Artist not found';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="container mx-auto p-4">
	<div class="mb-4">
		<a href="/music" class="btn btn-ghost btn-sm gap-1">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="2"
				stroke="currentColor"
				class="h-4 w-4"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18"
				/>
			</svg>
			Back to Music
		</a>
	</div>

	{#if loading}
		<div class="flex justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else if error}
		<div class="alert alert-error">
			<span>{error}</span>
		</div>
	{:else if artist}
		<!-- Artist Header -->
		<div class="flex flex-col gap-6 md:flex-row">
			<!-- Artist Image -->
			<div class="flex-shrink-0">
				{#if artistImageUrl}
					<img
						src={artistImageUrl}
						alt={artist.name}
						class="mx-auto w-48 rounded-lg shadow-lg object-cover md:w-64"
					/>
				{:else}
					<div
						class="mx-auto flex h-48 w-48 items-center justify-center rounded-lg bg-base-300 md:h-64 md:w-64"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							stroke-width="1.5"
							stroke="currentColor"
							class="h-16 w-16 opacity-30"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z"
							/>
						</svg>
					</div>
				{/if}
			</div>

			<!-- Artist Details -->
			<div class="flex-1">
				<h1 class="text-3xl font-bold md:text-4xl">{artist.name}</h1>

				{#if artist.disambiguation}
					<p class="mt-1 text-lg italic opacity-70">{artist.disambiguation}</p>
				{/if}

				<div class="mt-3 flex flex-wrap items-center gap-3">
					{#if artist.type}
						<span class="badge badge-outline">{artist.type}</span>
					{/if}
					{#if artist.country}
						<span class="text-sm opacity-70">{artist.country}</span>
					{/if}
					{#if artist.beginYear}
						<span class="text-sm opacity-70">
							{artist.beginYear}{artist.endYear ? ` – ${artist.endYear}` : ' – present'}
						</span>
					{/if}
				</div>

				{#if artist.tags.length > 0}
					<div class="mt-3 flex flex-wrap gap-2">
						{#each artist.tags as tag}
							<span class="badge badge-primary badge-outline">{tag}</span>
						{/each}
					</div>
				{/if}

				<!-- MusicBrainz Link -->
				<div class="mt-4">
					<a
						href="https://musicbrainz.org/artist/{artist.id}"
						target="_blank"
						rel="noopener noreferrer"
						class="btn btn-outline btn-sm"
					>
						View on MusicBrainz
					</a>
				</div>
			</div>
		</div>

		<!-- Discography -->
		{#if artist.releaseGroups.length > 0}
			<div class="mt-8">
				<h2 class="mb-4 text-xl font-semibold">
					Discography ({artist.releaseGroups.length})
				</h2>
				<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
					{#each artist.releaseGroups as rg (rg.id)}
						<div class="card bg-base-200 shadow-md">
							<button
								class="w-full text-left"
								onclick={() => toggleReleaseGroup(rg.id)}
							>
								<figure class="aspect-square">
									<img
										src={rg.coverArtUrl}
										alt={rg.title}
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
									<h3 class="card-title line-clamp-2 text-sm">{rg.title}</h3>
									<div class="flex items-center gap-2 text-xs opacity-70">
										<span>{rg.firstReleaseYear}</span>
										{#if rg.primaryType}
											<span class="badge badge-ghost badge-xs">{rg.primaryType}</span>
										{/if}
									</div>
								</div>
							</button>

							<!-- Expanded Track Listing -->
							{#if expandedReleaseGroups.has(rg.id)}
								<div class="border-t border-base-300 px-3 pb-3 pt-2">
									{#if releaseLoading.has(rg.id)}
										<div class="flex justify-center py-2">
											<span class="loading loading-spinner loading-sm"></span>
										</div>
									{:else if releaseDetails[rg.id]}
										<div class="space-y-1">
											{#each releaseDetails[rg.id].tracks as track}
												<div class="flex items-center gap-2 text-xs">
													<span class="w-5 text-right opacity-50">{track.number}</span>
													<span class="flex-1 truncate">{track.title}</span>
													{#if track.duration}
														<span class="opacity-50">{track.duration}</span>
													{/if}
												</div>
											{/each}
										</div>
									{:else}
										<p class="text-xs opacity-50">No track listing available.</p>
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>

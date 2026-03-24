<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type {
		DisplayMusicBrainzArtist,
		DisplayMusicBrainzReleaseGroup
	} from 'addons/musicbrainz/types';

	interface Props {
		artist: DisplayMusicBrainzArtist;
		albums: DisplayMusicBrainzReleaseGroup[];
		fetching: boolean;
		fetched: boolean;
		fetchSteps: {
			terms: boolean;
			search: boolean;
			searching: boolean;
			eval: boolean;
			done: boolean;
		} | null;
		downloadStatus: { state: string; progress: number } | null;
		fetchedTorrent: { name: string; quality: string } | null;
		onfetch: () => void;
		ondownload: () => void;
		onshowsearch: () => void;
		onback: () => void;
		onalbumclick?: (albumId: string) => void;
	}

	let {
		artist,
		albums,
		fetching,
		fetched,
		fetchSteps,
		downloadStatus,
		fetchedTorrent,
		onfetch,
		ondownload,
		onshowsearch,
		onback,
		onalbumclick
	}: Props = $props();

	let lifeSpan = $derived.by(() => {
		if (!artist.beginYear) return null;
		if (artist.ended && artist.endYear) return `${artist.beginYear} - ${artist.endYear}`;
		if (artist.beginYear) return `${artist.beginYear} - present`;
		return null;
	});

	let dlState = $derived(downloadStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'completed' || dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
	let dlProgress = $derived(downloadStatus?.progress ?? 0);
	let dlPercent = $derived(Math.round(dlProgress * 100));
</script>

<DetailPageLayout>
	<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</button>

	<div class="flex items-center gap-4">
		<div
			class="flex h-24 w-24 shrink-0 items-center justify-center rounded-full bg-base-200 text-base-content/20"
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-12 w-12"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="1.5"
					d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
				/>
			</svg>
		</div>
		<div class="min-w-0">
			<h1 class="text-xl font-bold">{artist.name}</h1>
			{#if artist.disambiguation}
				<p class="text-sm opacity-60">{artist.disambiguation}</p>
			{/if}
		</div>
	</div>

	<div class="flex flex-col gap-1.5">
		{#if artist.type}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Type:</span><span>{artist.type}</span>
			</div>
		{/if}
		{#if artist.country}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Country:</span><span>{artist.country}</span>
			</div>
		{/if}
		{#if lifeSpan}
			<div class="flex items-center gap-1 text-sm">
				<span class="opacity-40">Active:</span><span>{lifeSpan}</span>
			</div>
		{/if}
	</div>

	{#if artist.tags.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each artist.tags as tag}
				<span class="badge badge-outline badge-sm">{tag}</span>
			{/each}
		</div>
	{/if}

	<div class="grid grid-cols-2 gap-2">
		<button
			class="btn col-span-2 btn-sm {fetched ? 'btn-ghost' : 'btn-info'}"
			onclick={onfetch}
			disabled={fetching}
		>
			{#if fetching}
				<span class="loading loading-xs loading-spinner"></span>
			{:else}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
					/>
				</svg>
			{/if}
			Smart Search Discography
		</button>
		{#if fetchSteps}
			<button
				class="col-span-2 cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
				onclick={onshowsearch}
			>
				<ul class="steps steps-horizontal w-full text-xs">
					<li class={classNames('step', { 'step-success': fetchSteps.terms })}>Terms</li>
					<li class={classNames('step', { 'step-success': fetchSteps.search })}>
						{fetchSteps.searching ? 'Searching...' : 'Search'}
					</li>
					<li class={classNames('step', { 'step-success': fetchSteps.eval })}>Analysis</li>
					<li class={classNames('step', { 'step-success': fetchSteps.done })}>
						{fetchSteps.done ? 'Done' : 'Candidate'}
					</li>
				</ul>
			</button>
		{/if}
		{#if fetchedTorrent}
			<div class="col-span-2 flex items-center gap-2">
				<p class="min-w-0 flex-1 truncate text-xs opacity-60" title={fetchedTorrent.name}>
					{fetchedTorrent.name}
				</p>
				{#if fetchedTorrent.quality}
					<span class="badge badge-xs badge-info">{fetchedTorrent.quality}</span>
				{/if}
			</div>
		{/if}
		<button
			class={classNames('btn col-span-2 btn-sm', {
				'btn-ghost': isDownloaded,
				'btn-success': !isDownloaded
			})}
			onclick={ondownload}
			disabled={downloadButtonDisabled}
		>
			{#if isDownloading}
				<span class="loading loading-xs loading-spinner"></span> Downloading
			{:else if isDownloaded}
				Downloaded
			{:else}
				Download
			{/if}
		</button>
		{#if isDownloading || isDownloaded}
			<div class="col-span-2 flex items-center gap-2">
				<progress
					class={classNames('progress flex-1', {
						'progress-info': isDownloading,
						'progress-success': isDownloaded
					})}
					value={dlPercent}
					max="100"
				></progress>
				<span class="text-xs font-medium opacity-60">{dlPercent}%</span>
			</div>
		{/if}
	</div>

	{#if albums.length > 0}
		<div class="flex flex-col gap-1">
			<h4 class="text-sm font-semibold opacity-50">Discography ({albums.length})</h4>
			{#each albums as albumItem (albumItem.id)}
				<button
					class="flex items-center gap-2 rounded px-1 py-1 text-left hover:bg-base-200"
					onclick={() => onalbumclick?.(albumItem.id)}
				>
					{#if albumItem.coverArtUrl}
						<img
							src={albumItem.coverArtUrl}
							alt={albumItem.title}
							class="h-8 w-8 rounded object-cover"
						/>
					{:else}
						<div class="flex h-8 w-8 items-center justify-center rounded bg-base-300">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="h-4 w-4 text-base-content/20"
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
					<div class="min-w-0 flex-1">
						<p class="truncate text-sm">{albumItem.title}</p>
						<p class="text-xs opacity-40">
							{albumItem.firstReleaseYear}
							{#if albumItem.primaryType}
								<span class="ml-1 badge badge-ghost badge-xs">{albumItem.primaryType}</span>
							{/if}
						</p>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</DetailPageLayout>

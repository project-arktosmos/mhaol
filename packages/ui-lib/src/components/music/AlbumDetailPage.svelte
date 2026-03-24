<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease
	} from 'addons/musicbrainz/types';

	interface Props {
		album: DisplayMusicBrainzReleaseGroup;
		release: DisplayMusicBrainzRelease | null;
		loading: boolean;
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
	}

	let {
		album,
		release,
		loading,
		fetching,
		fetched,
		fetchSteps,
		downloadStatus,
		fetchedTorrent,
		onfetch,
		ondownload,
		onshowsearch,
		onback
	}: Props = $props();

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

	<h1 class="text-xl font-bold">{album.title}</h1>

	{#if album.coverArtUrl}
		<img
			src={album.coverArtUrl}
			alt={album.title}
			class="aspect-square w-full max-w-sm rounded-lg object-cover"
		/>
	{:else}
		<div
			class="flex aspect-square w-full max-w-sm items-center justify-center rounded-lg bg-base-200"
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

	<p class="text-sm opacity-60">{album.artistCredits}</p>
	{#if album.firstReleaseYear && album.firstReleaseYear !== 'Unknown'}
		<p class="text-sm opacity-40">{album.firstReleaseYear}</p>
	{/if}

	<div class="flex flex-wrap gap-1">
		{#if album.primaryType}
			<span class="badge badge-ghost badge-sm">{album.primaryType}</span>
		{/if}
		{#each album.secondaryTypes as type_}
			<span class="badge badge-ghost badge-sm">{type_}</span>
		{/each}
	</div>

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
			Smart Search
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

	{#if loading}
		<div class="flex items-center justify-center py-4">
			<span class="loading loading-sm loading-spinner"></span>
		</div>
	{:else if release && release.tracks.length > 0}
		<div class="flex flex-col gap-0.5">
			<div class="flex items-center justify-between">
				<h4 class="text-sm font-semibold opacity-50">Tracklist</h4>
				<span class="text-xs opacity-30">{release.tracks.length} tracks</span>
			</div>
			{#each release.tracks as track (track.id)}
				<div class="flex items-center gap-2 rounded px-1 py-0.5 hover:bg-base-200">
					<span class="w-5 text-right text-xs opacity-30">{track.number}</span>
					<span class="min-w-0 flex-1 truncate text-sm">{track.title}</span>
					{#if track.duration}
						<span class="text-xs opacity-30">{track.duration}</span>
					{/if}
				</div>
			{/each}
		</div>
	{:else if release}
		<p class="text-sm opacity-30">No tracks available</p>
	{/if}
</DetailPageLayout>

<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type {
		DisplayMusicBrainzArtist,
		DisplayMusicBrainzReleaseGroup
	} from 'addons/musicbrainz/types';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateLabel,
		getStateColor
	} from 'ui-lib/types/torrent.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

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
		torrentStatus: {
			state: TorrentState;
			progress: number;
			size: number;
			downloadSpeed: number;
			uploadSpeed: number;
			peers: number;
			seeds: number;
			eta: number | null;
		} | null;
		fetchedTorrent: { name: string; quality: string; languages: string } | null;
		isFavorite?: boolean;
		togglingFavorite?: boolean;
		isPinned?: boolean;
		togglingPin?: boolean;
		onfetch: () => void;
		ondownload: () => void;
		onshowsearch: () => void;
		onback: () => void;
		onalbumclick?: (albumId: string) => void;
		ontogglefavorite?: () => void;
		ontogglepin?: () => void;
	}

	let {
		artist,
		albums,
		fetching,
		fetched,
		fetchSteps,
		torrentStatus,
		fetchedTorrent,
		isFavorite = false,
		togglingFavorite = false,
		isPinned = false,
		togglingPin = false,
		onfetch,
		ondownload,
		onshowsearch,
		onback,
		onalbumclick,
		ontogglefavorite,
		ontogglepin
	}: Props = $props();

	let lifeSpan = $derived.by(() => {
		if (!artist.beginYear) return null;
		if (artist.ended && artist.endYear) return `${artist.beginYear} - ${artist.endYear}`;
		if (artist.beginYear) return `${artist.beginYear} - present`;
		return null;
	});

	let dlState = $derived(torrentStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
	let dlPercent = $derived(Math.round((torrentStatus?.progress ?? 0) * 100));
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

	{#if artist.imageUrl}
		<img
			src={artist.imageUrl}
			alt={artist.name}
			class="h-24 w-24 shrink-0 rounded-full object-cover"
		/>
	{:else}
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
	{/if}

	{#snippet cellA()}
		<h1 class="text-xl font-bold">{artist.name}</h1>
		{#if artist.disambiguation}
			<p class="text-sm opacity-60">{artist.disambiguation}</p>
		{/if}

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
	{/snippet}

	{#snippet cellB()}
		{#if ontogglefavorite && ontogglepin}
			<div class="grid grid-cols-2 gap-2">
				<button
					class={classNames('btn btn-sm', { 'btn-error': isFavorite, 'btn-outline': !isFavorite })}
					onclick={ontogglefavorite}
					disabled={togglingFavorite}
				>
					{#if togglingFavorite}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill={isFavorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
						</svg>
					{/if}
					{isFavorite ? 'Unfavorite' : 'Favorite'}
				</button>
				<button
					class={classNames('btn btn-sm', { 'btn-info': isPinned, 'btn-outline': !isPinned })}
					onclick={ontogglepin}
					disabled={togglingPin}
				>
					{#if togglingPin}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill={isPinned ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
							<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
						</svg>
					{/if}
					{isPinned ? 'Unpin' : 'Pin'}
				</button>
			</div>
		{/if}

		<button
			class="btn w-full btn-sm {fetched ? 'btn-ghost' : 'btn-info'}"
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
				class="w-full cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
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

		{#if fetchedTorrent || torrentStatus}
			<table class="table table-xs">
				<tbody>
					{#if fetchedTorrent}
						<tr>
							<td class="font-medium opacity-60">File</td>
							<td class="break-all">{fetchedTorrent.name}</td>
						</tr>
						{#if fetchedTorrent.quality}
							<tr>
								<td class="font-medium opacity-60">Quality</td>
								<td><span class="badge badge-xs badge-info">{fetchedTorrent.quality}</span></td>
							</tr>
						{/if}
						{#if fetchedTorrent.languages}
							<tr>
								<td class="font-medium opacity-60">Languages</td>
								<td><span class="badge badge-ghost badge-xs">{fetchedTorrent.languages}</span></td>
							</tr>
						{/if}
					{/if}
					{#if torrentStatus}
						<tr>
							<td class="font-medium opacity-60">Status</td>
							<td>
								<span class="badge badge-xs badge-{getStateColor(torrentStatus.state)}">
									{getStateLabel(torrentStatus.state)}
								</span>
							</td>
						</tr>
						<tr>
							<td class="font-medium opacity-60">Size</td>
							<td>{formatBytes(torrentStatus.size)}</td>
						</tr>
						{#if isDownloading}
							<tr>
								<td class="font-medium opacity-60">Progress</td>
								<td>
									<div class="flex items-center gap-2">
										<progress class="progress flex-1 progress-info" value={dlPercent} max="100"
										></progress>
										<span class="text-xs font-medium">{dlPercent}%</span>
									</div>
								</td>
							</tr>
							<tr>
								<td class="font-medium opacity-60">Speed</td>
								<td>
									{formatSpeed(torrentStatus.downloadSpeed)} &darr;
									{formatSpeed(torrentStatus.uploadSpeed)} &uarr;
								</td>
							</tr>
							<tr>
								<td class="font-medium opacity-60">Peers</td>
								<td>{torrentStatus.seeds} seeds &middot; {torrentStatus.peers} peers</td>
							</tr>
							{#if torrentStatus.eta !== null}
								<tr>
									<td class="font-medium opacity-60">ETA</td>
									<td>{formatEta(torrentStatus.eta)}</td>
								</tr>
							{/if}
						{/if}
						{#if isDownloaded}
							<tr>
								<td class="font-medium opacity-60">Progress</td>
								<td>
									<div class="flex items-center gap-2">
										<progress class="progress flex-1 progress-success" value="100" max="100"
										></progress>
										<span class="text-xs font-medium">100%</span>
									</div>
								</td>
							</tr>
						{/if}
					{:else if fetchedTorrent}
						<tr>
							<td class="font-medium opacity-60">Status</td>
							<td><span class="badge badge-ghost badge-xs">Not started</span></td>
						</tr>
					{/if}
				</tbody>
			</table>
		{/if}

		<button
			class={classNames('btn w-full btn-sm', {
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
	{/snippet}
</DetailPageLayout>

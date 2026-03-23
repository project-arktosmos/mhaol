<script lang="ts">
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type {
		DisplayMusicBrainzReleaseGroup,
		DisplayMusicBrainzRelease
	} from 'addons/musicbrainz/types';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';

	interface Props {
		album: DisplayMusicBrainzReleaseGroup;
		release: DisplayMusicBrainzRelease | null;
		torrent: TorrentInfo | null;
		loading: boolean;
		ondownloadalbum: () => void;
		onback: () => void;
	}

	let { album, release, torrent, loading, ondownloadalbum, onback }: Props = $props();
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
					class="progress mt-1 w-full progress-primary"
					value={Math.round(torrent.progress * 100)}
					max="100"
				></progress>
			{/if}
		</div>
	{/if}

	<button
		class="btn btn-sm btn-primary"
		onclick={ondownloadalbum}
		disabled={torrent?.state === 'downloading' || torrent?.state === 'seeding'}
	>
		{torrent?.state === 'seeding' ? 'Downloaded' : torrent ? 'Downloading...' : 'Download'}
	</button>

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

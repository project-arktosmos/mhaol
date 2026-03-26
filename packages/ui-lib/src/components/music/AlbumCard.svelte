<script lang="ts">
	import classNames from 'classnames';
	import type { DisplayMusicBrainzReleaseGroup } from 'addons/musicbrainz/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';
	import TorrentProgressOverlay from 'ui-lib/components/torrent/TorrentProgressOverlay.svelte';

	interface Props {
		album: DisplayMusicBrainzReleaseGroup;
		selected?: boolean;
		favorited?: boolean;
		pinned?: boolean;
		torrentProgress?: number | null;
		torrentState?: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
		onselect?: (album: DisplayMusicBrainzReleaseGroup) => void;
	}

	let {
		album,
		selected = false,
		favorited = false,
		pinned = false,
		torrentProgress = null,
		torrentState = null,
		torrentSpeed = null,
		torrentEta = null,
		onselect
	}: Props = $props();

	let imgError = $state(false);

	let isDownloading = $derived(
		torrentProgress !== null && torrentState !== null && torrentState !== 'seeding'
	);
	let isCompleted = $derived(torrentState === 'seeding');
</script>

<div
	class={classNames('card-compact card bg-base-200 shadow-sm', {
		'ring-2 ring-primary': selected,
		'cursor-pointer transition-shadow hover:shadow-md': !!onselect
	})}
	onclick={() => onselect?.(album)}
	role={onselect ? 'button' : undefined}
	tabindex={onselect ? 0 : undefined}
	onkeydown={onselect
		? (e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onselect?.(album);
				}
			}
		: undefined}
>
	<figure class="relative aspect-square overflow-hidden bg-base-300">
		{#if album.coverArtUrl && !imgError}
			<img
				src={album.coverArtUrl}
				alt={album.title}
				class="h-full w-full object-cover"
				loading="lazy"
				onerror={() => (imgError = true)}
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-base-content/20">
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
						d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
					/>
				</svg>
			</div>
		{/if}
		{#if isCompleted}
			<div class="absolute top-1 right-1">
				<span class="badge badge-xs badge-success">Done</span>
			</div>
		{:else if isDownloading}
			<TorrentProgressOverlay {torrentProgress} {torrentState} {torrentSpeed} {torrentEta} />
		{/if}
		{#if favorited || pinned}
			<div class="absolute bottom-1.5 left-1.5 z-10 flex gap-1">
				{#if favorited}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4 text-red-500 drop-shadow"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
						/>
					</svg>
				{/if}
				{#if pinned}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-4 w-4 text-blue-400 drop-shadow"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							fill-rule="evenodd"
							d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
							clip-rule="evenodd"
						/>
					</svg>
				{/if}
			</div>
		{/if}
	</figure>
	<div class="card-body gap-0.5">
		<h3 class="card-title truncate text-sm" title={album.title}>{album.title}</h3>
		<p class="truncate text-xs opacity-60" title={album.artistCredits}>{album.artistCredits}</p>
		<div class="flex items-center gap-1">
			{#if album.firstReleaseYear && album.firstReleaseYear !== 'Unknown'}
				<span class="text-xs opacity-40">{album.firstReleaseYear}</span>
			{/if}
			{#if album.primaryType}
				<span class="badge badge-ghost badge-xs">{album.primaryType}</span>
			{/if}
		</div>
	</div>
</div>

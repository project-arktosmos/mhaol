<script lang="ts">
	import classNames from 'classnames';
	import type { DisplayMusicBrainzReleaseGroup } from 'ui-lib/types/musicbrainz.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';
	import { formatSpeed, formatEta } from 'ui-lib/types/torrent.type';

	interface Props {
		album: DisplayMusicBrainzReleaseGroup;
		selected?: boolean;
		torrentProgress?: number | null;
		torrentState?: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
		onselect?: (album: DisplayMusicBrainzReleaseGroup) => void;
	}

	let {
		album,
		selected = false,
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
	let progressPercent = $derived(torrentProgress !== null ? Math.round(torrentProgress * 100) : 0);
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
			<div class="absolute inset-x-0 bottom-0 bg-black/70 px-2 py-1.5">
				<div class="mb-1 flex items-center justify-between text-xs text-white">
					<span
						class={classNames('font-medium', {
							'text-info': torrentState === 'initializing' || torrentState === 'checking',
							'text-primary': torrentState === 'downloading',
							'text-warning': torrentState === 'paused',
							'text-error': torrentState === 'error'
						})}
					>
						{progressPercent}%
					</span>
					<span class="opacity-70">
						{#if torrentState === 'downloading' && torrentSpeed}
							{formatSpeed(torrentSpeed)}
						{:else if torrentState === 'initializing'}
							Starting...
						{:else if torrentState === 'paused'}
							Paused
						{:else if torrentState === 'error'}
							Error
						{/if}
					</span>
				</div>
				<div class="h-1 w-full overflow-hidden rounded-full bg-white/20">
					<div
						class={classNames('h-full rounded-full transition-all', {
							'bg-primary': torrentState === 'downloading',
							'bg-info': torrentState === 'initializing' || torrentState === 'checking',
							'bg-warning': torrentState === 'paused',
							'bg-error': torrentState === 'error'
						})}
						style="width: {progressPercent}%"
					></div>
				</div>
				{#if torrentState === 'downloading' && torrentEta}
					<div class="mt-0.5 text-right text-xs text-white/50">
						{formatEta(torrentEta)}
					</div>
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

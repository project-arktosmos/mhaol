<script lang="ts">
	import classNames from 'classnames';
	import type { DisplayMusicBrainzArtist } from 'addons/musicbrainz/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';
	import TorrentProgressOverlay from 'ui-lib/components/torrent/TorrentProgressOverlay.svelte';

	interface Props {
		artist: DisplayMusicBrainzArtist;
		selected?: boolean;
		torrentProgress?: number | null;
		torrentState?: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
		onselect?: (artist: DisplayMusicBrainzArtist) => void;
	}

	let {
		artist,
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

	let lifeSpan = $derived.by(() => {
		if (!artist.beginYear) return null;
		if (artist.ended && artist.endYear) return `${artist.beginYear} - ${artist.endYear}`;
		if (artist.beginYear) return `${artist.beginYear} - present`;
		return null;
	});
</script>

<div
	class={classNames('card-compact card bg-base-200 shadow-sm', {
		'ring-2 ring-primary': selected,
		'cursor-pointer transition-shadow hover:shadow-md': !!onselect
	})}
	onclick={() => onselect?.(artist)}
	role={onselect ? 'button' : undefined}
	tabindex={onselect ? 0 : undefined}
	onkeydown={onselect
		? (e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onselect?.(artist);
				}
			}
		: undefined}
>
	<figure class="relative aspect-square overflow-hidden bg-base-300">
		{#if artist.imageUrl && !imgError}
			<img
				src={artist.imageUrl}
				alt={artist.name}
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
						d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
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
	</figure>
	<div class="card-body gap-0.5">
		<h3 class="card-title truncate text-sm" title={artist.name}>{artist.name}</h3>
		{#if artist.disambiguation}
			<p class="truncate text-xs opacity-60" title={artist.disambiguation}>
				{artist.disambiguation}
			</p>
		{/if}
		<div class="flex flex-wrap items-center gap-1">
			{#if artist.type}
				<span class="badge badge-ghost badge-xs">{artist.type}</span>
			{/if}
			{#if artist.country}
				<span class="badge badge-ghost badge-xs">{artist.country}</span>
			{/if}
			{#if lifeSpan}
				<span class="text-xs opacity-40">{lifeSpan}</span>
			{/if}
		</div>
		{#if artist.tags.length > 0}
			<div class="mt-0.5 flex flex-wrap gap-0.5">
				{#each artist.tags.slice(0, 3) as tag}
					<span class="badge badge-outline badge-xs opacity-50">{tag}</span>
				{/each}
			</div>
		{/if}
	</div>
</div>

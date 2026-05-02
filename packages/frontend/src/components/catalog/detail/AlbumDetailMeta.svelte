<script lang="ts">
	import type { CatalogAlbum } from '$types/catalog.type';
	import { authorsByRole } from '$types/catalog.type';
	import type { SmartSearchTorrentResult } from '$types/smart-search.type';
	import type { TorrentInfo } from '$types/torrent.type';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateLabel,
		getStateColor
	} from '$types/torrent.type';
	import AuthorList from './AuthorList.svelte';

	interface TrackFile {
		path: string;
		name: string;
		infoHash: string;
	}

	interface Props {
		item: CatalogAlbum;
		albumCandidate?: SmartSearchTorrentResult | null;
		torrentByHash?: Record<string, TorrentInfo>;
		trackFiles?: Record<string, TrackFile>;
		onplaytrack?: (path: string, name: string, infoHash: string) => void;
	}

	let {
		item,
		albumCandidate = null,
		torrentByHash = {},
		trackFiles = {},
		onplaytrack
	}: Props = $props();

	let artists = $derived(authorsByRole(item.metadata.authors, 'artist'));
	let primaryType = $derived(item.metadata.primaryType);
	let releases = $derived(item.metadata.releases);
	let firstRelease = $derived(releases[0] ?? null);
	let tracks = $derived(firstRelease?.tracks ?? []);
	let albumTorrent = $derived(
		albumCandidate ? torrentByHash[albumCandidate.infoHash.toLowerCase()] : undefined
	);
</script>

{#snippet torrentProgress(torrent: TorrentInfo)}
	{@const percent = Math.round(torrent.progress * 100)}
	<div class="flex flex-col gap-1">
		<div class="flex items-center justify-between text-xs">
			<span class={getStateColor(torrent.state)}>{getStateLabel(torrent.state)}</span>
			<span class="font-mono">{percent}%</span>
		</div>
		<progress class="progress w-full progress-primary" value={torrent.progress} max="1"></progress>
		<div class="flex flex-wrap gap-3 text-xs opacity-60">
			<span>↓ {formatSpeed(torrent.downloadSpeed)}</span>
			<span>↑ {formatSpeed(torrent.uploadSpeed)}</span>
			<span>{formatBytes(torrent.size)}</span>
			{#if torrent.eta}<span>ETA {formatEta(torrent.eta)}</span>{/if}
		</div>
	</div>
{/snippet}

<div class="flex flex-col gap-3">
	{#if artists.length > 0}
		<AuthorList authors={artists} layout="inline" label="Artist" />
	{/if}

	{#if primaryType}
		<div class="text-sm">
			<span class="opacity-50">Type:</span>
			<span class="badge badge-ghost badge-sm">{primaryType}</span>
		</div>
	{/if}

	{#if firstRelease}
		<div class="grid grid-cols-2 gap-2 text-sm">
			{#if firstRelease.date}
				<div>
					<span class="opacity-50">Released:</span>
					<span class="font-medium">{firstRelease.date}</span>
				</div>
			{/if}
			{#if firstRelease.label}
				<div>
					<span class="opacity-50">Label:</span>
					<span class="font-medium">{firstRelease.label}</span>
				</div>
			{/if}
			{#if firstRelease.country}
				<div>
					<span class="opacity-50">Country:</span>
					<span class="font-medium">{firstRelease.country}</span>
				</div>
			{/if}
		</div>
	{/if}

	{#if albumCandidate}
		<div class="rounded-lg border border-base-300 p-2">
			<div class="flex items-center gap-2">
				<span class="badge badge-xs badge-primary">Album</span>
				<p class="flex-1 truncate text-xs">{albumCandidate.name}</p>
			</div>
			{#if albumTorrent}
				<div class="mt-2">
					{@render torrentProgress(albumTorrent)}
				</div>
			{:else}
				<p class="mt-1 text-xs opacity-50">Queued…</p>
			{/if}
		</div>
	{/if}

	{#if tracks.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
				Tracks ({tracks.length})
			</h3>
			<div class="flex flex-col">
				{#each tracks as track}
					{@const file = trackFiles?.[track.number]}
					<div
						class="flex items-center justify-between gap-2 border-b border-base-200 py-1.5 text-sm last:border-0"
					>
						<div class="flex min-w-0 items-center gap-2">
							<span class="w-6 text-right text-xs opacity-40">{track.number}</span>
							<span class="truncate">{track.title}</span>
						</div>
						<div class="flex items-center gap-2">
							{#if track.duration}
								<span class="text-xs opacity-50">{track.duration}</span>
							{/if}
							{#if file && onplaytrack}
								<button
									class="btn btn-xs btn-primary"
									onclick={() => onplaytrack(file.path, file.name, file.infoHash)}
								>
									Play
								</button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

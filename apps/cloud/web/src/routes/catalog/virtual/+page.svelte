<script lang="ts">
	import classNames from 'classnames';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import { cachedImageUrl } from '$lib/image-cache';
	import {
		firkinsService,
		type Firkin,
		type FirkinSource,
		type FirkinType,
		type ImageMeta
	} from '$lib/firkins.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type TorrentResultItem
	} from '$lib/search.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { page as pageStore } from '$app/state';

	function mapToFirkinType(addonId: string, typeId: string): FirkinType {
		if (addonId === 'tmdb') {
			if (typeId === 'tv') return 'tv show';
			if (typeId === 'tv_season') return 'tv season';
			if (typeId === 'tv_episode') return 'tv episode';
			if (typeId === 'image') return 'image';
			return 'movie';
		}
		if (addonId === 'musicbrainz') {
			if (typeId === 'track') return 'track';
			return 'album';
		}
		if (addonId === 'retroachievements') return 'game';
		if (addonId === 'youtube') {
			if (typeId === 'channel') return 'youtube channel';
			return 'youtube video';
		}
		if (addonId === 'lrclib') return 'track';
		if (addonId === 'openlibrary') return 'book';
		if (addonId === 'wyzie-subs') {
			if (typeId === 'tv_episode') return 'tv episode';
			return 'movie';
		}
		if (addonId === 'iptv') return 'iptv channel';
		if (addonId === 'radio') return 'radio station';
		return 'movie';
	}

	function mapToFirkinSource(addonId: string): FirkinSource {
		if (addonId === 'tmdb') return 'tmdb';
		if (addonId === 'musicbrainz') return 'musicbrainz';
		if (addonId === 'retroachievements') return 'retroachievements';
		if (addonId === 'youtube') return 'youtube';
		if (addonId === 'lrclib') return 'lrclib';
		if (addonId === 'openlibrary') return 'openlibrary';
		if (addonId === 'wyzie-subs') return 'wyzie-subs';
		if (addonId === 'iptv') return 'iptv';
		if (addonId === 'radio') return 'radio';
		return 'tmdb';
	}

	const params = $derived(pageStore.url.searchParams);
	const addon = $derived(params.get('addon') ?? '');
	const catalogType = $derived(params.get('type') ?? '');
	const itemId = $derived(params.get('id') ?? '');
	const title = $derived(params.get('title') ?? '');
	const yearParam = $derived(params.get('year'));
	const year = $derived(
		yearParam !== null && yearParam !== '' ? Number.parseInt(yearParam, 10) : null
	);
	const description = $derived(params.get('description') ?? '');
	const posterUrl = $derived(params.get('posterUrl'));
	const backdropUrl = $derived(params.get('backdropUrl'));

	const firkinType = $derived<FirkinType>(mapToFirkinType(addon, catalogType));
	const firkinSource = $derived<FirkinSource>(mapToFirkinSource(addon));

	const images = $derived<ImageMeta[]>(
		[posterUrl, backdropUrl]
			.filter((url): url is string => Boolean(url))
			.map((url) => ({ url, mimeType: 'image/jpeg', fileSize: 0, width: 0, height: 0 }))
	);

	const virtualFirkin = $derived<CloudFirkin>({
		id: `virtual:${addon}:${catalogType}:${itemId}`,
		title,
		artists: [],
		description,
		images,
		files: [],
		year,
		type: firkinType,
		source: firkinSource,
		created_at: '',
		updated_at: '',
		version: 0,
		version_hashes: []
	});

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentMatches = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;

	$effect(() => {
		if (!title) return;
		const t = title;
		const k = firkinType;
		const y = year;
		void runTorrentSearch(t, k, y);
	});

	async function runTorrentSearch(title: string, kind: FirkinType, year: number | null) {
		const myRun = ++searchRun;
		torrentStatus = 'searching';
		torrentError = null;
		torrentMatches = [];
		try {
			const torrents = await searchTorrents(kind, title);
			if (myRun !== searchRun) return;
			const matches = matchTorrentsForResult(
				{ title, description: '', artists: [], images: [], files: [], year, raw: null },
				torrents
			);
			torrentMatches = matches;
			torrentStatus = 'done';
		} catch (err) {
			if (myRun !== searchRun) return;
			torrentMatches = [];
			torrentError = err instanceof Error ? err.message : 'Unknown error';
			torrentStatus = 'error';
		}
	}

	async function startTorrentDownload(magnet: string): Promise<void> {
		const res = await fetch('/api/torrent/add', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ magnet })
		});
		if (!res.ok) {
			let message = `HTTP ${res.status}`;
			try {
				const body = await res.json();
				if (body && typeof body.error === 'string') message = body.error;
			} catch {
				// ignore
			}
			throw new Error(message);
		}
	}

	async function assignTorrent(torrent: TorrentResultItem) {
		if (!torrent.magnetLink || addingHash) return;
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const created: Firkin = await firkinsService.create({
				title,
				artists: [],
				description,
				images,
				files: [{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }],
				year,
				type: firkinType,
				source: firkinSource
			});
			await startTorrentDownload(torrent.magnetLink);
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '—';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		let value = bytes;
		let unit = 0;
		while (value >= 1024 && unit < units.length - 1) {
			value /= 1024;
			unit++;
		}
		return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {title || 'Catalog'}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{title}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-outline badge-sm">{firkinType}</span>
				<span class="badge badge-outline badge-sm">{firkinSource}</span>
				{#if year !== null && year !== undefined && Number.isFinite(year)}
					<span class="badge badge-outline badge-sm">{year}</span>
				{/if}
				<span class="badge badge-sm badge-warning">virtual</span>
			</p>
		</div>
	</header>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<FirkinCard firkin={virtualFirkin} />
		</aside>

		<section class="flex flex-col gap-6">
			{#if description}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
					<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{description}</p>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Status</h2>
				<p class="text-xs text-base-content/70">
					This item is virtual — no record exists in the database yet, and nothing is pinned to
					IPFS. Picking a torrent below will create the firkin, pin its files, and bring it into the
					catalog properly.
				</p>
			</div>

			{#if images.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Images ({images.length})
					</h2>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
						{#each images as image, i (i)}
							<figure
								class="flex flex-col gap-1 overflow-hidden rounded-box border border-base-content/10 bg-base-300"
							>
								<img
									src={cachedImageUrl(image.url)}
									alt={`Image ${i + 1}`}
									class="block h-auto w-full"
									loading="lazy"
								/>
								<figcaption class="px-2 py-1 text-[10px] text-base-content/70">
									{image.width || '?'}×{image.height || '?'}
									{#if image.fileSize}· {formatBytes(image.fileSize)}{/if}
									{#if image.mimeType}· {image.mimeType}{/if}
								</figcaption>
							</figure>
						{/each}
					</div>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<div class="mb-2 flex items-center justify-between gap-2">
					<h2 class="text-sm font-semibold text-base-content/70 uppercase">
						Torrent search{torrentMatches.length > 0 ? ` (${torrentMatches.length})` : ''}
					</h2>
					<button
						type="button"
						class="btn btn-outline btn-xs"
						onclick={() => runTorrentSearch(title, firkinType, year)}
						disabled={torrentStatus === 'searching'}
					>
						{torrentStatus === 'searching' ? 'Searching…' : 'Refresh'}
					</button>
				</div>
				{#if assignError}
					<div class="mb-2 alert alert-error">
						<span>{assignError}</span>
					</div>
				{/if}
				{#if torrentStatus === 'searching' && torrentMatches.length === 0}
					<p class="text-sm text-base-content/60">Searching…</p>
				{:else if torrentStatus === 'error'}
					<p class="text-sm text-error">{torrentError ?? 'Failed'}</p>
				{:else if torrentMatches.length === 0}
					<p class="text-sm text-base-content/60">No matching torrents.</p>
				{:else}
					<div class="flex flex-col gap-1">
						{#each torrentMatches as torrent (torrent.infoHash)}
							{@const adding = addingHash === torrent.magnetLink}
							<button
								type="button"
								class={classNames(
									'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-100',
									{ 'opacity-60': adding }
								)}
								onclick={() => assignTorrent(torrent)}
								disabled={addingHash !== null}
								title={torrent.title}
							>
								<span class="font-medium">{torrent.quality ?? '—'}</span>
								<span class="text-success">↑{torrent.seeders}</span>
								<span class="text-warning">↓{torrent.leechers}</span>
								<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
								<span class="truncate text-base-content/70"
									>{torrent.parsedTitle || torrent.title}</span
								>
								{#if adding}
									<span class="ml-auto">…</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</section>
	</div>
</div>

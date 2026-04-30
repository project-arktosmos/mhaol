<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import { firkinPlaybackService } from 'ui-lib/services/firkin-playback.service';
	import {
		firkinTorrentsService,
		infoHashFromMagnet
	} from 'ui-lib/services/firkin-torrents.service';
	import { playerService } from 'ui-lib/services/player.service';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import type { PlayableFile } from 'ui-lib/types/player.type';
	import { cachedImageUrl } from '$lib/image-cache';
	import { firkinsService, addonKind, type Firkin, type FirkinAddon } from '$lib/firkins.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type TorrentResultItem
	} from '$lib/search.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';

	interface Props {
		data: { firkin: Firkin };
	}

	let { data }: Props = $props();
	const firkin = $derived<Firkin>(data.firkin);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	const hasIpfsFiles = $derived(firkin.files.some((f) => f.type === 'ipfs'));
	const firstIpfsCid = $derived(firkin.files.find((f) => f.type === 'ipfs')?.value ?? null);
	const hasMagnetFiles = $derived(firkin.files.some((f) => f.type === 'torrent magnet'));
	const firkinKind = $derived(addonKind(firkin.addon));
	const isStreamUrlKind = $derived(firkinKind === 'iptv channel' || firkinKind === 'radio station');
	const firstStreamUrl = $derived(
		isStreamUrlKind ? (firkin.files.find((f) => f.type === 'url')?.value ?? null) : null
	);
	const hasStreamUrl = $derived(firstStreamUrl !== null);
	let ipfsStarting = $state(false);
	let ipfsError = $state<string | null>(null);

	async function startIpfsPlay(): Promise<void> {
		if (!firstIpfsCid || ipfsStarting) return;
		ipfsStarting = true;
		ipfsError = null;
		try {
			const res = await fetch('/api/ipfs-stream/sessions', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ cid: firstIpfsCid })
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
			const body = (await res.json()) as {
				sessionId: string;
				playlistUrl: string;
				durationSeconds?: number;
			};
			const durationSecs =
				typeof body.durationSeconds === 'number' && body.durationSeconds > 0
					? body.durationSeconds
					: null;
			const file: PlayableFile = {
				id: `firkin:${firkin.id}:ipfs:${firstIpfsCid}`,
				type: 'library',
				name: firkin.title,
				outputPath: '',
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: durationSecs,
				size: 0,
				completedAt: ''
			};
			// The rolling HLS playlist has no #EXT-X-ENDLIST until transcode
			// completes, so videoElement.duration stays Infinity — seed
			// durationSecs from the server-probed source duration instead.
			await playerService.playUrl(
				file,
				body.playlistUrl,
				'application/vnd.apple.mpegurl',
				'sidebar'
			);
		} catch (err) {
			ipfsError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			ipfsStarting = false;
		}
	}

	const torrentsState = firkinTorrentsService.state;
	onMount(() => firkinTorrentsService.start());

	const firstMagnet = $derived(
		firkin.files.find((f) => f.type === 'torrent magnet')?.value ?? null
	);

	let torrentStreamStarting = $state(false);
	let torrentStreamError = $state<string | null>(null);

	async function startTorrentStream(): Promise<void> {
		if (!firstMagnet || torrentStreamStarting) return;
		torrentStreamStarting = true;
		torrentStreamError = null;
		try {
			const res = await fetch('/api/torrent/stream', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ magnet: firstMagnet })
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
			const body = (await res.json()) as {
				infoHash: string;
				name: string;
				fileIndex: number;
				fileName: string;
				fileSize: number;
				mimeType: string | null;
				streamUrl: string;
			};
			const file: PlayableFile = {
				id: `firkin:${firkin.id}:torrent:${body.infoHash}:${body.fileIndex}`,
				type: 'library',
				name: body.fileName || firkin.title,
				outputPath: '',
				mode: 'video',
				format: null,
				videoFormat: null,
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: null,
				size: body.fileSize,
				completedAt: ''
			};
			await playerService.playUrl(file, body.streamUrl, body.mimeType ?? null, 'sidebar');
		} catch (err) {
			torrentStreamError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			torrentStreamStarting = false;
		}
	}

	const completedTorrents = $derived.by(() => {
		const out: { hash: string; title: string }[] = [];
		for (const f of firkin.files) {
			if (f.type !== 'torrent magnet' || !f.value) continue;
			const hash = infoHashFromMagnet(f.value);
			if (!hash) continue;
			const t = $torrentsState.byHash[hash];
			if (!t) continue;
			const finished = t.state === 'seeding' || t.progress >= 1;
			if (finished) out.push({ hash, title: f.title ?? t.name });
		}
		return out;
	});

	const canPlay = $derived(hasIpfsFiles || completedTorrents.length > 0 || hasStreamUrl);

	let finalizing = $state(false);
	let finalizeError = $state<string | null>(null);

	async function play() {
		if (hasStreamUrl && firstStreamUrl) {
			const mode: 'audio' | 'video' = firkinKind === 'radio station' ? 'audio' : 'video';
			const file: PlayableFile = {
				id: `firkin:${firkin.id}`,
				type: 'library',
				name: firkin.title,
				outputPath: '',
				mode,
				format: null,
				videoFormat: null,
				thumbnailUrl: firkin.images[0]?.url ?? null,
				durationSeconds: null,
				size: 0,
				completedAt: ''
			};
			const mime = firkinKind === 'iptv channel' ? 'application/vnd.apple.mpegurl' : null;
			await playerService.playUrl(file, firstStreamUrl, mime, 'sidebar');
			return;
		}
		if (hasIpfsFiles) {
			firkinPlaybackService.select(firkin as CloudFirkin);
			return;
		}
		if (finalizing) return;
		finalizeError = null;
		finalizing = true;
		try {
			const res = await fetch(`/api/firkins/${encodeURIComponent(firkin.id)}/finalize`, {
				method: 'POST'
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
			const next = (await res.json()) as Firkin;
			if (next.id !== firkin.id) {
				await goto(`${base}/catalog/${encodeURIComponent(next.id)}`);
			} else {
				data.firkin = next;
			}
			if (next.files.some((f) => f.type === 'ipfs')) {
				firkinPlaybackService.select(next as unknown as CloudFirkin);
			}
		} catch (err) {
			finalizeError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			finalizing = false;
		}
	}

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentMatches = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;
	let startedHashes = $state<Set<string>>(new Set());

	const existingHashes = $derived(
		new Set(firkin.files.filter((f) => f.type === 'torrent magnet' && f.value).map((f) => f.value))
	);

	$effect(() => {
		const id = firkin.id;
		const title = firkin.title;
		const addon = firkin.addon;
		const year = firkin.year;
		void runTorrentSearch(id, title, addon, year);
	});

	$effect(() => {
		if (!hasMagnetFiles || hasIpfsFiles) return;
		const id = firkin.id;
		let cancelled = false;
		const tick = async () => {
			if (cancelled) return;
			try {
				const res = await fetch(`/api/firkins/${encodeURIComponent(id)}`, {
					cache: 'no-store'
				});
				if (cancelled) return;
				if (res.status === 404) {
					const listRes = await fetch('/api/firkins', { cache: 'no-store' });
					if (!listRes.ok) return;
					const list = (await listRes.json()) as Firkin[];
					if (cancelled) return;
					const successor = list.find((d) => (d.version_hashes ?? []).includes(id));
					if (successor) {
						await goto(`${base}/catalog/${encodeURIComponent(successor.id)}`);
					}
					return;
				}
				if (!res.ok) return;
				const fresh = (await res.json()) as Firkin;
				if (cancelled) return;
				if (fresh.files.some((f) => f.type === 'ipfs')) {
					data.firkin = fresh;
				}
			} catch {
				// swallow — try again on next tick
			}
		};
		const timer = setInterval(tick, 4000);
		return () => {
			cancelled = true;
			clearInterval(timer);
		};
	});

	$effect(() => {
		const magnets = firkin.files
			.filter((f) => f.type === 'torrent magnet' && f.value)
			.map((f) => f.value);
		for (const magnet of magnets) {
			if (startedHashes.has(magnet)) continue;
			startedHashes = new Set(startedHashes).add(magnet);
			void startTorrentDownload(magnet).catch((err) => {
				console.warn('[catalog detail] auto-start failed for magnet:', err);
			});
		}
	});

	async function runTorrentSearch(_id: string, title: string, addon: string, year: number | null) {
		const myRun = ++searchRun;
		torrentStatus = 'searching';
		torrentError = null;
		torrentMatches = [];
		try {
			const torrents = await searchTorrents(addon, title);
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
		if (!torrent.magnetLink || addingHash || existingHashes.has(torrent.magnetLink)) {
			return;
		}
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const created = await firkinsService.create({
				title: firkin.title,
				artists: firkin.artists,
				description: firkin.description,
				images: firkin.images,
				files: [
					...firkin.files,
					{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }
				],
				year: firkin.year,
				addon: firkin.addon as FirkinAddon
			});
			await startTorrentDownload(torrent.magnetLink);
			if (created.id !== firkin.id) {
				await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
			}
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
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

	async function remove() {
		if (removing) return;
		removing = true;
		removeError = null;
		try {
			await firkinsService.remove(firkin.id);
			window.location.href = `${base}/catalog`;
		} catch (err) {
			removeError = err instanceof Error ? err.message : 'Unknown error';
			removing = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {firkin.title}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{firkin.title}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-outline badge-sm">{firkin.addon}</span>
				{#if firkinKind}
					<span class="badge badge-outline badge-sm">{firkinKind}</span>
				{/if}
				{#if firkin.year !== null && firkin.year !== undefined}
					<span class="badge badge-outline badge-sm">{firkin.year}</span>
				{/if}
			</p>
		</div>
		<div class="flex items-center gap-2">
			{#if canPlay}
				<button
					type="button"
					class="btn gap-2 btn-sm btn-primary"
					onclick={play}
					disabled={finalizing}
					aria-label="Play"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						stroke="none"
						class="h-4 w-4 shrink-0"
						aria-hidden="true"
					>
						<polygon points="6 4 20 12 6 20 6 4" />
					</svg>
					<span>{finalizing ? 'Pinning…' : 'Play'}</span>
				</button>
			{/if}
			<button
				type="button"
				class="btn gap-2 btn-sm btn-secondary"
				onclick={startIpfsPlay}
				disabled={!hasIpfsFiles || ipfsStarting}
				aria-label="IPFS Play"
				title={hasIpfsFiles
					? 'Stream over IPFS as HLS'
					: 'Available once at least one file is pinned to IPFS'}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="currentColor"
					stroke="none"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				>
					<polygon points="6 4 20 12 6 20 6 4" />
				</svg>
				<span>{ipfsStarting ? 'Starting…' : 'IPFS Play'}</span>
			</button>
			<button
				type="button"
				class="btn gap-2 btn-sm btn-accent"
				onclick={startTorrentStream}
				disabled={!firstMagnet || torrentStreamStarting}
				aria-label="Torrent Stream"
				title={firstMagnet
					? 'Resolve magnet metadata, pick the largest video file, and stream it as it downloads'
					: 'Available once a torrent magnet is attached'}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="currentColor"
					stroke="none"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				>
					<polygon points="6 4 20 12 6 20 6 4" />
				</svg>
				<span>{torrentStreamStarting ? 'Resolving…' : 'Torrent Stream'}</span>
			</button>
			<button
				type="button"
				class="btn btn-outline btn-sm btn-error"
				onclick={remove}
				disabled={removing}
			>
				{removing ? 'Deleting…' : 'Delete firkin'}
			</button>
		</div>
	</header>

	{#if removeError}
		<div class="alert alert-error">
			<span>{removeError}</span>
		</div>
	{/if}

	{#if finalizeError}
		<div class="alert alert-error">
			<span>{finalizeError}</span>
		</div>
	{/if}

	{#if ipfsError}
		<div class="alert alert-error">
			<span>{ipfsError}</span>
		</div>
	{/if}

	{#if torrentStreamError}
		<div class="alert alert-error">
			<span>{torrentStreamError}</span>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<FirkinCard firkin={firkin as CloudFirkin} />
		</aside>

		<section class="flex flex-col gap-6">
			{#if firkin.description}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
					<p class="text-sm [overflow-wrap:anywhere] whitespace-pre-wrap">{firkin.description}</p>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Identity</h2>
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-top">CID</th>
							<td class="font-mono text-xs break-all">{firkin.id}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Created</th>
							<td class="text-xs">{formatDate(firkin.created_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Updated</th>
							<td class="text-xs">{formatDate(firkin.updated_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Version</th>
							<td class="text-xs">{firkin.version ?? 0}</td>
						</tr>
					</tbody>
				</table>
			</div>

			{#if firkin.version_hashes && firkin.version_hashes.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Version history ({firkin.version_hashes.length})
					</h2>
					<ol class="list-decimal pl-6 text-xs">
						{#each firkin.version_hashes as cid, i (i)}
							<li class="font-mono break-all">
								<a class="link" href="{base}/catalog/{encodeURIComponent(cid)}">{cid}</a>
							</li>
						{/each}
					</ol>
				</div>
			{/if}

			{#if firkin.artists.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Artists ({firkin.artists.length})
					</h2>
					<ul class="flex flex-col gap-3">
						{#each firkin.artists as artist, i (i)}
							<li class="flex items-center gap-3">
								{#if artist.imageUrl}
									<img
										src={cachedImageUrl(artist.imageUrl)}
										alt={artist.name}
										class="h-12 w-12 rounded-full object-cover"
										loading="lazy"
									/>
								{/if}
								<div class="flex flex-col">
									<span class="text-sm font-medium">{artist.name}</span>
									{#if artist.url}
										<a
											class="link text-xs break-all link-primary"
											href={artist.url}
											target="_blank"
											rel="noopener noreferrer">{artist.url}</a
										>
									{/if}
								</div>
							</li>
						{/each}
					</ul>
				</div>
			{/if}

			{#if firkin.images.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Images ({firkin.images.length})
					</h2>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
						{#each firkin.images as image, i (i)}
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
						onclick={() => runTorrentSearch(firkin.id, firkin.title, firkin.addon, firkin.year)}
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
							{@const added = existingHashes.has(torrent.magnetLink)}
							{@const adding = addingHash === torrent.magnetLink}
							<button
								type="button"
								class={classNames(
									'flex flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs hover:bg-base-100',
									{ 'opacity-60': added || adding }
								)}
								onclick={() => assignTorrent(torrent)}
								disabled={addingHash !== null || added}
								title={torrent.title}
							>
								<span class="font-medium">{torrent.quality ?? '—'}</span>
								<span class="text-success">↑{torrent.seeders}</span>
								<span class="text-warning">↓{torrent.leechers}</span>
								<span class="text-base-content/60">{formatSizeBytes(torrent.sizeBytes)}</span>
								<span class="truncate text-base-content/70"
									>{torrent.parsedTitle || torrent.title}</span
								>
								{#if added}
									<span class="ml-auto">✓</span>
								{:else if adding}
									<span class="ml-auto">…</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
					Files ({firkin.files.length})
				</h2>
				{#if firkin.files.length === 0}
					<p class="text-sm text-base-content/60">No files attached.</p>
				{:else}
					<div class="overflow-x-auto rounded-box border border-base-content/10">
						<table class="table table-sm">
							<thead>
								<tr>
									<th class="w-24">Type</th>
									<th>Title</th>
									<th>Value</th>
								</tr>
							</thead>
							<tbody>
								{#each firkin.files as file, i (i)}
									<tr>
										<td class={classNames('text-xs font-semibold')}>
											<span class="badge badge-outline badge-sm">{file.type}</span>
										</td>
										<td class="text-xs [overflow-wrap:anywhere]">{file.title ?? ''}</td>
										<td class="font-mono text-xs break-all">{file.value}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		</section>
	</div>
</div>

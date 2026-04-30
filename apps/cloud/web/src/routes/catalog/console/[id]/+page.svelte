<script lang="ts">
	import classNames from 'classnames';
	import FirkinCard from 'ui-lib/components/firkins/FirkinCard.svelte';
	import type { CloudFirkin } from 'ui-lib/types/firkin.type';
	import { firkinsService, type Firkin } from '$lib/firkins.service';
	import { formatSizeBytes, searchTorrents, type TorrentResultItem } from '$lib/search.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';

	interface Props {
		data: { console: { id: string; name: string } };
	}

	let { data }: Props = $props();
	const consoleInfo = $derived(data.console);

	const virtualFirkin = $derived<CloudFirkin>({
		id: `console:${consoleInfo.id}`,
		title: consoleInfo.name,
		artists: [],
		description: 'RetroAchievements console',
		images: [],
		files: [],
		year: null,
		addon: 'retroachievements',
		created_at: '',
		updated_at: '',
		version: 0,
		version_hashes: []
	});

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentResults = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;

	$effect(() => {
		const name = consoleInfo.name;
		void runTorrentSearch(name);
	});

	async function runTorrentSearch(query: string) {
		const myRun = ++searchRun;
		torrentStatus = 'searching';
		torrentError = null;
		torrentResults = [];
		try {
			const torrents = await searchTorrents('retroachievements', query);
			if (myRun !== searchRun) return;
			const sorted = torrents.slice().sort((a, b) => b.seeders - a.seeders);
			torrentResults = sorted;
			torrentStatus = 'done';
		} catch (err) {
			if (myRun !== searchRun) return;
			torrentResults = [];
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
				title: consoleInfo.name,
				artists: [],
				description: 'RetroAchievements console',
				images: [],
				files: [{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }],
				year: null,
				addon: 'retroachievements'
			});
			await startTorrentDownload(torrent.magnetLink);
			await goto(`${base}/catalog/${encodeURIComponent(created.id)}`);
		} catch (err) {
			assignError = err instanceof Error ? err.message : 'Unknown error';
		} finally {
			addingHash = null;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {consoleInfo.name}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{consoleInfo.name}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-outline badge-sm">retroachievements</span>
				<span class="badge badge-outline badge-sm">console</span>
				<span class="badge badge-outline badge-sm">game</span>
			</p>
		</div>
	</header>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<FirkinCard firkin={virtualFirkin} />
		</aside>

		<section class="flex flex-col gap-6">
			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Status</h2>
				<p class="text-xs text-base-content/70">
					This is a console listing — no record exists in the database yet, and nothing is pinned to
					IPFS. Picking a torrent below will create a firkin for the console, attach the magnet, and
					start the download.
				</p>
			</div>

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<div class="mb-2 flex items-center justify-between gap-2">
					<h2 class="text-sm font-semibold text-base-content/70 uppercase">
						Torrent search{torrentResults.length > 0 ? ` (${torrentResults.length})` : ''}
					</h2>
					<button
						type="button"
						class="btn btn-outline btn-xs"
						onclick={() => runTorrentSearch(consoleInfo.name)}
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
				{#if torrentStatus === 'searching' && torrentResults.length === 0}
					<p class="text-sm text-base-content/60">Searching…</p>
				{:else if torrentStatus === 'error'}
					<p class="text-sm text-error">{torrentError ?? 'Failed'}</p>
				{:else if torrentResults.length === 0}
					<p class="text-sm text-base-content/60">No torrents found.</p>
				{:else}
					<div class="flex flex-col gap-1">
						{#each torrentResults as torrent (torrent.infoHash)}
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

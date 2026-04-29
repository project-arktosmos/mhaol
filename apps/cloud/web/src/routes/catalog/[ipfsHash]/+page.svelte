<script lang="ts">
	import classNames from 'classnames';
	import DocumentCard from 'ui-lib/components/documents/DocumentCard.svelte';
	import type { CloudDocument } from 'ui-lib/types/document.type';
	import { cachedImageUrl } from '$lib/image-cache';
	import {
		documentsService,
		type Document,
		type DocumentSource,
		type DocumentType
	} from '$lib/documents.service';
	import {
		formatSizeBytes,
		matchTorrentsForResult,
		searchTorrents,
		type TorrentResultItem
	} from '$lib/search.service';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';

	interface Props {
		data: { document: Document };
	}

	let { data }: Props = $props();
	const document = $derived<Document>(data.document);
	let removing = $state(false);
	let removeError = $state<string | null>(null);

	type TorrentStatus = 'idle' | 'searching' | 'done' | 'error';
	let torrentStatus = $state<TorrentStatus>('idle');
	let torrentError = $state<string | null>(null);
	let torrentMatches = $state<TorrentResultItem[]>([]);
	let addingHash = $state<string | null>(null);
	let assignError = $state<string | null>(null);
	let searchRun = 0;

	const existingHashes = $derived(
		new Set(
			document.files
				.filter((f) => f.type === 'torrent magnet' && f.value)
				.map((f) => f.value)
		)
	);

	$effect(() => {
		const id = document.id;
		const title = document.title;
		const kind = document.type as DocumentType;
		const year = document.year;
		void runTorrentSearch(id, title, kind, year);
	});

	async function runTorrentSearch(
		_id: string,
		title: string,
		kind: DocumentType,
		year: number | null
	) {
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

	async function assignTorrent(torrent: TorrentResultItem) {
		if (
			!torrent.magnetLink ||
			addingHash ||
			existingHashes.has(torrent.magnetLink)
		) {
			return;
		}
		assignError = null;
		addingHash = torrent.magnetLink;
		try {
			const created = await documentsService.create({
				title: document.title,
				artists: document.artists,
				description: document.description,
				images: document.images,
				files: [
					...document.files,
					{ type: 'torrent magnet', value: torrent.magnetLink, title: torrent.title }
				],
				year: document.year,
				type: document.type as DocumentType,
				source: document.source as DocumentSource
			});
			if (created.id !== document.id) {
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
			await documentsService.remove(document.id);
			window.location.href = `${base}/catalog`;
		} catch (err) {
			removeError = err instanceof Error ? err.message : 'Unknown error';
			removing = false;
		}
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — {document.title}</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex flex-wrap items-start justify-between gap-3">
		<div class="flex flex-col gap-1">
			<a class="text-xs text-base-content/60 hover:underline" href="{base}/catalog">← Catalog</a>
			<h1 class="text-2xl font-bold [overflow-wrap:anywhere]">{document.title}</h1>
			<p class="text-sm text-base-content/70">
				<span class="badge badge-sm badge-outline">{document.type}</span>
				<span class="badge badge-sm badge-outline">{document.source}</span>
				{#if document.year !== null && document.year !== undefined}
					<span class="badge badge-sm badge-outline">{document.year}</span>
				{/if}
			</p>
		</div>
		<button
			type="button"
			class="btn btn-error btn-sm btn-outline"
			onclick={remove}
			disabled={removing}
		>
			{removing ? 'Deleting…' : 'Delete document'}
		</button>
	</header>

	{#if removeError}
		<div class="alert alert-error">
			<span>{removeError}</span>
		</div>
	{/if}

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,_320px)_1fr]">
		<aside class="flex flex-col gap-4">
			<DocumentCard document={document as CloudDocument} />
		</aside>

		<section class="flex flex-col gap-6">
			{#if document.description}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Description</h2>
					<p class="text-sm whitespace-pre-wrap [overflow-wrap:anywhere]">{document.description}</p>
				</div>
			{/if}

			<div class="card border border-base-content/10 bg-base-200 p-4">
				<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">Identity</h2>
				<table class="table table-sm">
					<tbody>
						<tr>
							<th class="w-32 align-top">CID</th>
							<td class="font-mono text-xs break-all">{document.id}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Created</th>
							<td class="text-xs">{formatDate(document.created_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Updated</th>
							<td class="text-xs">{formatDate(document.updated_at)}</td>
						</tr>
						<tr>
							<th class="w-32 align-top">Version</th>
							<td class="text-xs">{document.version ?? 0}</td>
						</tr>
					</tbody>
				</table>
			</div>

			{#if document.version_hashes && document.version_hashes.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Version history ({document.version_hashes.length})
					</h2>
					<ol class="list-decimal pl-6 text-xs">
						{#each document.version_hashes as cid, i (i)}
							<li class="font-mono break-all">
								<a class="link" href="{base}/catalog/{encodeURIComponent(cid)}">{cid}</a>
							</li>
						{/each}
					</ol>
				</div>
			{/if}

			{#if document.artists.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Artists ({document.artists.length})
					</h2>
					<ul class="flex flex-col gap-3">
						{#each document.artists as artist, i (i)}
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
											class="text-xs link link-primary break-all"
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

			{#if document.images.length > 0}
				<div class="card border border-base-content/10 bg-base-200 p-4">
					<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">
						Images ({document.images.length})
					</h2>
					<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
						{#each document.images as image, i (i)}
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
						onclick={() => runTorrentSearch(document.id, document.title, document.type as DocumentType, document.year)}
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
								<span class="truncate text-base-content/70">{torrent.parsedTitle || torrent.title}</span>
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
					Files ({document.files.length})
				</h2>
				{#if document.files.length === 0}
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
								{#each document.files as file, i (i)}
									<tr>
										<td class={classNames('text-xs font-semibold')}>
											<span class="badge badge-sm badge-outline">{file.type}</span>
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

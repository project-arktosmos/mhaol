<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { ipfsService, type IpfsPin } from '$lib/ipfs.service';
	import { documentsService } from '$lib/documents.service';
	import { parseTorrentName } from '$lib/search.service';

	const pinsStore = ipfsService.state;
	const docsStore = documentsService.state;

	const usedCids = $derived(
		new Set(
			$docsStore.documents.flatMap((d) =>
				(d.files ?? []).filter((f) => f.type === 'ipfs' && f.value).map((f) => f.value)
			)
		)
	);

	onMount(() => {
		ipfsService.refresh();
		documentsService.refresh();
	});

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB'];
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[i]}`;
	}

	function formatDate(value: string): string {
		try {
			return new Date(value).toLocaleString();
		} catch {
			return value;
		}
	}

	function basename(path: string): string {
		const idx = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
		return idx >= 0 ? path.slice(idx + 1) : path;
	}

	function stripExt(name: string): string {
		const idx = name.lastIndexOf('.');
		return idx > 0 ? name.slice(0, idx) : name;
	}

	function presetForMime(mime: string): { source: string; type: string } {
		if (mime.startsWith('video/')) return { source: 'tmdb', type: 'movie' };
		if (mime.startsWith('audio/')) return { source: 'musicbrainz', type: 'album' };
		if (mime.startsWith('image/')) return { source: 'tmdb', type: 'image' };
		return { source: 'tmdb', type: 'movie' };
	}

	function addAsDocumentHref(pin: IpfsPin): string {
		const filename = basename(pin.path);
		const stripped = stripExt(filename);
		const parsed = parseTorrentName(stripped);
		const preset = presetForMime(pin.mime);
		const params = new URLSearchParams({
			cid: pin.cid,
			title: parsed.parsedTitle,
			source: preset.source,
			type: preset.type,
			filename
		});
		if (parsed.year) params.set('year', String(parsed.year));
		return `${base}/documents?${params.toString()}`;
	}
</script>

<svelte:head>
	<title>Mhaol Cloud — IPFS</title>
</svelte:head>

<div class="flex min-h-full flex-col gap-6 p-6">
	<header class="flex items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">IPFS</h1>
			<p class="text-sm text-base-content/60">
				Audio, video, and image files discovered while scanning libraries are pinned to the embedded
				IPFS node. This page lists every pin recorded by the cloud server.
			</p>
		</div>
		<button
			class="btn btn-outline btn-sm"
			onclick={() => {
				ipfsService.refresh();
				documentsService.refresh();
			}}
			disabled={$pinsStore.loading}
		>
			Refresh
		</button>
	</header>

	{#if $pinsStore.error}
		<div class="alert alert-error">
			<span>{$pinsStore.error}</span>
		</div>
	{/if}

	{#if $pinsStore.loading && $pinsStore.pins.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if $pinsStore.pins.length === 0}
		<p class="text-sm text-base-content/60">
			No pins yet. Run a library scan to pin its audio files.
		</p>
	{:else}
		<div class="overflow-x-auto rounded-box border border-base-content/10">
			<table class="table table-sm">
				<thead>
					<tr>
						<th>CID</th>
						<th>Path</th>
						<th class="w-32">MIME</th>
						<th class="w-24 text-right">Size</th>
						<th class="w-40">Pinned</th>
						<th class="w-40"></th>
					</tr>
				</thead>
				<tbody>
					{#each $pinsStore.pins as pin (pin.id)}
						<tr>
							<td class="font-mono text-xs break-all">{pin.cid}</td>
							<td class="font-mono text-xs break-all">{pin.path}</td>
							<td class="font-mono text-xs">{pin.mime}</td>
							<td class="text-right text-xs">{formatBytes(pin.size)}</td>
							<td class="text-xs text-base-content/60">{formatDate(pin.created_at)}</td>
							<td class="text-right">
								{#if usedCids.has(pin.cid)}
									<span class="text-xs text-base-content/40">In document</span>
								{:else}
									<a href={addAsDocumentHref(pin)} class="btn btn-outline btn-xs btn-primary">
										Add as document
									</a>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

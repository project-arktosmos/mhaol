<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import type { CloudDocument, DocumentFile } from 'ui-lib/types/document.type';
	import {
		documentTorrentsService,
		infoHashFromMagnet
	} from 'ui-lib/services/document-torrents.service';
	import { documentPlaybackService } from 'ui-lib/services/document-playback.service';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';

	interface Props {
		document: CloudDocument;
		classes?: string;
		onRemove?: (id: string) => void;
		removing?: boolean;
	}

	let { document, classes = '', onRemove, removing = false }: Props = $props();

	let coverImage = $derived(document.images?.[0] ?? null);

	let hasYear = $derived(document.year !== null && document.year !== undefined);

	let files = $derived(document.files ?? []);
	let magnetFiles = $derived(files.filter((f) => f.type === 'torrent magnet'));
	let tableFiles = $derived(files.filter((f) => f.type !== 'torrent magnet'));
	let hasIpfsFiles = $derived(files.some((f) => f.type === 'ipfs'));

	const torrentsState = documentTorrentsService.state;
	let pendingHashes = $state<Record<string, boolean>>({});

	onMount(() => {
		if (magnetFiles.length === 0) return;
		return documentTorrentsService.start();
	});

	function torrentFor(file: DocumentFile): TorrentInfo | null {
		const hash = infoHashFromMagnet(file.value);
		if (!hash) return null;
		return $torrentsState.byHash[hash] ?? null;
	}

	function isPending(file: DocumentFile): boolean {
		const hash = infoHashFromMagnet(file.value);
		return hash ? pendingHashes[hash] === true : false;
	}

	async function downloadMagnet(file: DocumentFile) {
		const hash = infoHashFromMagnet(file.value);
		if (!hash) return;
		pendingHashes = { ...pendingHashes, [hash]: true };
		try {
			await documentTorrentsService.add(file.value);
		} finally {
			pendingHashes = { ...pendingHashes, [hash]: false };
		}
	}

	function fileTooltip(file: DocumentFile): string {
		return file.title ? `${file.title}\n${file.value}` : file.value;
	}

	function progressPercent(t: TorrentInfo): number {
		return Math.round(Math.min(1, Math.max(0, t.progress)) * 100);
	}

	function progressLabel(t: TorrentInfo): string {
		const pct = progressPercent(t);
		switch (t.state) {
			case 'seeding':
				return 'Seeding · 100%';
			case 'paused':
				return `Paused · ${pct}%`;
			case 'error':
				return 'Error';
			case 'initializing':
				return pct > 0 ? `Starting · ${pct}%` : 'Starting…';
			case 'checking':
				return `Checking · ${pct}%`;
			default:
				return `${pct}%`;
		}
	}
</script>

<article class={classNames('group card bg-base-200 shadow-sm', classes)}>
	<header
		class="flex items-baseline justify-between gap-3 border-b border-base-content/10 px-4 py-3"
	>
		<span class="text-xs text-base-content/70">{document.type}</span>
		<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
			{document.title}
		</h3>
		<span class="text-xs text-base-content/70">{hasYear ? document.year : ''}</span>
		{#if onRemove}
			<button
				type="button"
				class="btn text-error btn-ghost btn-xs"
				onclick={() => onRemove?.(document.id)}
				disabled={removing}
				aria-label="Remove document"
			>
				{removing ? '…' : '×'}
			</button>
		{/if}
	</header>
	{#if coverImage}
		<figure class="relative overflow-hidden bg-base-300">
			<img
				src={coverImage.url}
				alt={document.title}
				width={coverImage.width || undefined}
				height={coverImage.height || undefined}
				class="block h-auto w-full"
				loading="lazy"
			/>
			{#if document.description}
				<figcaption
					class="pointer-events-none absolute inset-x-0 bottom-0 bg-black/50 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-white opacity-0 transition-opacity group-hover:opacity-100"
				>
					{document.description}
				</figcaption>
			{/if}
		</figure>
	{:else if document.description}
		<p
			class="border-b border-base-content/10 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-base-content/80"
		>
			{document.description}
		</p>
	{/if}
	{#if tableFiles.length > 0}
		<details class="group border-t border-base-content/10">
			<summary
				class="flex cursor-pointer items-center justify-between px-4 py-2 text-xs font-semibold text-base-content/70 hover:bg-base-300"
			>
				<span>Files ({tableFiles.length})</span>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="h-3 w-3 transition-transform group-open:rotate-180"
					aria-hidden="true"
				>
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</summary>
			<table class="table w-full table-fixed table-sm">
				<tbody>
					{#each tableFiles as file, i (i)}
						<tr>
							<th class="w-1/3 align-top text-xs font-semibold text-base-content/70">{file.type}</th
							>
							<td
								class="w-2/3 text-xs [overflow-wrap:anywhere] [word-break:break-word] whitespace-pre-wrap"
								title={fileTooltip(file)}>{file.title ?? file.value}</td
							>
						</tr>
					{/each}
				</tbody>
			</table>
		</details>
	{/if}
	{#if magnetFiles.length > 0 || hasIpfsFiles}
		<footer class="flex flex-col gap-2 border-t border-base-content/10 px-4 py-3">
			{#if hasIpfsFiles}
				<button
					type="button"
					class="btn justify-start gap-2 btn-sm btn-primary"
					onclick={() => documentPlaybackService.select(document)}
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
					<span>Play</span>
				</button>
			{:else}
				{#each magnetFiles as file, i (i)}
					{@const torrent = torrentFor(file)}
					{@const pending = isPending(file)}
					{#if torrent}
						<div class="flex flex-col gap-1" title={fileTooltip(file)}>
							<div class="flex items-center justify-between gap-2 text-xs">
								<span class="truncate text-base-content/80">{file.title ?? torrent.name}</span>
								<span class="shrink-0 font-mono text-base-content/70">{progressLabel(torrent)}</span
								>
							</div>
							<progress
								class={classNames('progress w-full', {
									'progress-primary': torrent.state === 'downloading',
									'progress-success': torrent.state === 'seeding',
									'progress-warning': torrent.state === 'paused',
									'progress-error': torrent.state === 'error',
									'progress-info': torrent.state === 'initializing' || torrent.state === 'checking'
								})}
								value={progressPercent(torrent)}
								max="100"
							></progress>
						</div>
					{:else}
						<button
							type="button"
							class={classNames('btn justify-start gap-2 btn-outline btn-sm', {
								'btn-disabled': pending
							})}
							onclick={() => downloadMagnet(file)}
							disabled={pending}
							title={fileTooltip(file)}
							aria-label={file.title ? `Download torrent: ${file.title}` : 'Download torrent'}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								class="h-4 w-4 shrink-0"
								aria-hidden="true"
							>
								<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
								<polyline points="7 10 12 15 17 10" />
								<line x1="12" y1="15" x2="12" y2="3" />
							</svg>
							<span class="truncate">
								{pending ? 'Adding…' : (file.title ?? 'Download torrent')}
							</span>
						</button>
					{/if}
				{/each}
			{/if}
		</footer>
	{/if}
</article>

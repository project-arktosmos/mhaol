<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { blo } from 'blo';
	import type { CloudFirkin, FirkinFile } from 'ui-lib/types/firkin.type';
	import {
		firkinTorrentsService,
		infoHashFromMagnet
	} from 'ui-lib/services/firkin-torrents.service';
	import { firkinPlaybackService } from 'ui-lib/services/firkin-playback.service';
	import { firkinsService } from 'ui-lib/services/firkins.service';
	import {
		firkinReactionsService,
		REACTION_EMOJIS,
		type ReactionEmoji
	} from 'ui-lib/services/firkin-reactions.service';
	import { getCachedImageUrl } from 'ui-lib/services/image-cache.service';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';

	interface Props {
		firkin: CloudFirkin;
		classes?: string;
	}

	let { firkin, classes = '' }: Props = $props();

	let coverImage = $derived(firkin.images?.[0] ?? null);
	let resolvedCoverUrl = $state<string | null>(null);

	$effect(() => {
		const url = coverImage?.url;
		if (!url) {
			resolvedCoverUrl = null;
			return;
		}
		let cancelled = false;
		getCachedImageUrl(url).then((u) => {
			if (!cancelled) resolvedCoverUrl = u;
		});
		return () => {
			cancelled = true;
		};
	});

	let files = $derived(firkin.files ?? []);
	let magnetFiles = $derived(files.filter((f) => f.type === 'torrent magnet'));
	let tableFiles = $derived(files.filter((f) => f.type !== 'torrent magnet'));
	let hasIpfsFiles = $derived(files.some((f) => f.type === 'ipfs'));

	const torrentsState = firkinTorrentsService.state;
	const reactionsState = firkinReactionsService.state;
	let pendingHashes = $state<Record<string, boolean>>({});

	let persistedRealIds = $state<Record<string, string>>({});
	let effectiveId = $derived(persistedRealIds[firkin.id] ?? firkin.id);
	let currentReaction = $derived<ReactionEmoji | null>($reactionsState[effectiveId] ?? null);
	let persistingReaction = $state(false);

	const REACTION_LABELS: Record<ReactionEmoji, string> = {
		'👎': 'Thumbs down',
		'⭐': 'Favorite',
		'👍': 'Thumbs up'
	};

	async function ensureRealId(): Promise<string | null> {
		const current = effectiveId;
		if (!current.startsWith('virtual:')) return current;
		try {
			const created = await firkinsService.create({
				title: firkin.title,
				artists: firkin.artists ?? [],
				description: firkin.description ?? '',
				images: firkin.images ?? [],
				files: firkin.files ?? [],
				year: firkin.year ?? null,
				addon: firkin.addon,
				creator: firkin.creator ?? ''
			});
			persistedRealIds = { ...persistedRealIds, [firkin.id]: created.id };
			return created.id;
		} catch (err) {
			console.error('Failed to persist virtual firkin', err);
			return null;
		}
	}

	let creatorAddress = $derived(firkin.creator ?? '');
	let creatorIdenticon = $derived(
		creatorAddress ? blo(creatorAddress as `0x${string}`) : null
	);

	async function reactWith(emoji: ReactionEmoji) {
		if (persistingReaction) return;
		persistingReaction = true;
		try {
			const id = await ensureRealId();
			if (!id) return;
			firkinReactionsService.set(id, emoji);
		} finally {
			persistingReaction = false;
		}
	}

	onMount(() => {
		if (magnetFiles.length === 0) return;
		return firkinTorrentsService.start();
	});

	function torrentFor(file: FirkinFile): TorrentInfo | null {
		const hash = infoHashFromMagnet(file.value);
		if (!hash) return null;
		return $torrentsState.byHash[hash] ?? null;
	}

	function isPending(file: FirkinFile): boolean {
		const hash = infoHashFromMagnet(file.value);
		return hash ? pendingHashes[hash] === true : false;
	}

	async function downloadMagnet(file: FirkinFile) {
		const hash = infoHashFromMagnet(file.value);
		if (!hash) return;
		pendingHashes = { ...pendingHashes, [hash]: true };
		try {
			await firkinTorrentsService.add(file.value);
		} finally {
			pendingHashes = { ...pendingHashes, [hash]: false };
		}
	}

	function fileTooltip(file: FirkinFile): string {
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
		<span class="text-xs text-base-content/70">{firkin.addon}</span>
		{#if !coverImage}
			<h3 class="flex-1 text-center text-base font-semibold [overflow-wrap:anywhere]">
				{firkin.title}
			</h3>
		{/if}
		{#if creatorIdenticon}
			<img
				src={creatorIdenticon}
				alt=""
				class="h-6 w-6 shrink-0 rounded-full"
				title={`Creator: ${creatorAddress}`}
				aria-label={`Creator: ${creatorAddress}`}
			/>
		{/if}
	</header>
	{#if coverImage}
		<figure class="relative overflow-hidden bg-base-300">
			<h3
				class="absolute inset-x-0 top-0 z-10 bg-black/60 px-4 py-2 text-center text-base font-semibold [overflow-wrap:anywhere] text-white"
			>
				{firkin.title}
			</h3>
			{#if resolvedCoverUrl}
				<img
					src={resolvedCoverUrl}
					alt={firkin.title}
					width={coverImage.width || undefined}
					height={coverImage.height || undefined}
					class="block h-auto w-full"
					loading="lazy"
				/>
			{/if}
			{#if firkin.description}
				<figcaption
					class="pointer-events-none absolute inset-x-0 bottom-0 bg-black/50 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-white opacity-0 transition-opacity group-hover:opacity-100"
				>
					{firkin.description}
				</figcaption>
			{/if}
		</figure>
	{:else if firkin.description}
		<p
			class="border-b border-base-content/10 px-4 py-3 text-xs [overflow-wrap:anywhere] whitespace-pre-wrap text-base-content/80"
		>
			{firkin.description}
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
					onclick={() => firkinPlaybackService.select(firkin)}
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
	<footer class="grid grid-cols-3 gap-1 border-t border-base-content/10 px-2 py-2">
		{#each REACTION_EMOJIS as emoji (emoji)}
			<button
				type="button"
				class={classNames('btn text-lg btn-ghost btn-sm', {
					grayscale: currentReaction !== emoji,
					'grayscale-0': currentReaction === emoji
				})}
				aria-label={REACTION_LABELS[emoji]}
				aria-pressed={currentReaction === emoji}
				disabled={persistingReaction}
				onclick={() => reactWith(emoji)}
			>
				{emoji}
			</button>
		{/each}
	</footer>
</article>

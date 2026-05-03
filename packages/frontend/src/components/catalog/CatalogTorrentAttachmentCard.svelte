<script lang="ts" module>
	export interface AttachmentInfo {
		title: string;
		seeders: number | null;
		leechers: number | null;
		sizeBytes: number | null;
		/** Quality bucket label (`4K`, `1080p`, …) of the matching torrent
		 * search row. Used by the Stream cell to default the quality
		 * selector to the currently-attached stream's quality. `null` when
		 * the attached magnet doesn't match any indexed search result. */
		quality?: string | null;
	}

	export interface DownloadAttachmentInfo extends AttachmentInfo {
		/** 0..1 fraction of bytes downloaded; `null` when the torrent client
		 * doesn't yet have a live record for this magnet (the auto-start
		 * tick hasn't picked it up, or the backend's torrent session is
		 * still warming up). */
		progress: number | null;
		/** Bytes/second from the live `TorrentInfo` snapshot. `null` when
		 * the torrent isn't actively downloading (paused, queued, finished,
		 * or no live record yet). */
		downloadSpeed: number | null;
		/** Seconds remaining at the current rate, or `null` when unknown. */
		etaSeconds: number | null;
		/** True once the torrent has either reached `Seeding` or hit 100%
		 * progress. Mutually exclusive with `progress < 1` semantically — the
		 * card flips from progress-bar to "Seeding" on this. */
		finished: boolean;
		/** CID of the IPFS pin the torrent-completion task wrote once the
		 * download finished. Set ⇒ the on-disk file is now in the local
		 * IPFS blockstore and the cell flips into a click-to-stream state
		 * that runs the IPFS HLS pipeline on it. `null` until the pin
		 * lands. */
		ipfsCid: string | null;
	}
</script>

<script lang="ts">
	import { Icon } from 'cloud-ui';
	import { formatSizeBytes, type TorrentResultItem } from '$lib/search.service';

	interface Props {
		/** The persisted `torrent magnet` (Download flow — long-lived in
		 * `<data_root>/downloads/torrents/`). `null` when nothing's attached. */
		download: DownloadAttachmentInfo | null;
		/** The persisted `torrent stream magnet` (Stream flow — ephemeral,
		 * `<data_root>/downloads/torrent-streams/`, wiped on next stream).
		 * `null` when nothing's attached. */
		stream: AttachmentInfo | null;
		/** Click handler for the Stream cell — re-runs the torrent-stream
		 * stack on the persisted stream magnet (start_stream wipes any
		 * prior stream, re-adds this one to `torrent-streams/`, and the
		 * inline player picks up the new bytes). Disabled while a stream
		 * is starting. */
		onStreamPlay?: () => void | Promise<void>;
		/** True while a stream-start round-trip is in flight. Renders the
		 * cell as `…` and disables clicks so the user can't double-fire. */
		streamPlaying?: boolean;
		/** Click handler for the Download cell once the torrent has been
		 * pinned to IPFS (`download.ipfsCid` set). Should kick the IPFS
		 * HLS stream on the picked CID. */
		onDownloadPlay?: () => void | Promise<void>;
		/** True while an IPFS-stream-start round-trip is in flight. */
		downloadPlaying?: boolean;
		/** Best non-attached torrent from the current search (highest quality
		 * first, most seeders within that quality). Surfaced as a faded
		 * preview in the Download cell when nothing's attached yet, so the
		 * user can attach the obvious pick in one click instead of opening
		 * the search table. `null` when no eligible row exists. */
		preferredDownload?: TorrentResultItem | null;
		/** One preferred row per quality bucket discovered by the torrent
		 * search, in quality priority order. `status: 'streamable'` means
		 * the eval probe came back positive — the row's Assign / Play
		 * button is enabled. `status: 'probing'` means a row in this
		 * quality is still being probed (`pending` / `evaluating`); the
		 * row appears in the table with a spinner action button so the
		 * user sees the quality discovered ASAP. Empty until the search
		 * has at least one row in any group. */
		streamPicksByQuality?: Array<{
			quality: string;
			torrent: TorrentResultItem;
			status: 'streamable' | 'probing';
		}>;
		/** One representative torrent per quality bucket from the torrent
		 * search. Powers the Download tab's per-quality picks table — every
		 * discovered quality gets a row with its own Assign / Play button.
		 * Empty until the search returns at least one row. */
		downloadPicksByQuality?: Array<{ quality: string; torrent: TorrentResultItem }>;
		/** Click handler for the faded Download preview — should run the
		 * same flow as the Download button on the search table row (attach
		 * the magnet to the firkin and start the torrent client). */
		onAttachDownload?: (torrent: TorrentResultItem) => void | Promise<void>;
		/** Click handler for the faded Stream preview — should run the same
		 * flow as the Stream button on the search table row (start the
		 * torrent stream and persist the picked magnet). */
		onAttachStream?: (torrent: TorrentResultItem) => void | Promise<void>;
		/** True while `preferredDownload` is being attached. */
		attachingDownload?: boolean;
		/** True while `preferredStream` is being attached. */
		attachingStream?: boolean;
		/** All available trailers (movie + per-season). When non-empty and
		 * `onTrailerPlay` is provided, the merged Stream / Download tabbed
		 * panel grows a Trailer tab as its first tab; the tab body is a
		 * picks table mirroring the Stream and Download tables — one row
		 * per trailer with its YouTube id and a Play button. */
		trailers?: Array<{ key: string; label: string | null; youtubeId: string }>;
		/** Click handler for a Trailer row. Receives the row's `key` so the
		 * trailer player can switch to that trailer before starting
		 * playback. */
		onTrailerPlay?: (key: string) => void | Promise<void>;
		/** True while a trailer-start round-trip is in flight. */
		trailerPlaying?: boolean;
	}

	let {
		download,
		stream,
		onStreamPlay,
		streamPlaying = false,
		onDownloadPlay,
		downloadPlaying = false,
		preferredDownload = null,
		streamPicksByQuality = [],
		downloadPicksByQuality = [],
		onAttachDownload,
		onAttachStream,
		attachingDownload = false,
		attachingStream = false,
		trailers = [],
		onTrailerPlay,
		trailerPlaying = false
	}: Props = $props();

	// Once the download has been pinned to IPFS the Download tab is the
	// strictly-better path (same bytes, faster start, no peer churn) so we
	// flip the merged tab panel to it by default; the user can still pick
	// Stream manually.
	const downloadActionable = $derived(Boolean(download && download.ipfsCid && onDownloadPlay));

	const hasTrailers = $derived(trailers.length > 0 && Boolean(onTrailerPlay));

	// Quality of the currently-attached stream, mapped to a bucket label
	// from `streamPicksByQuality`. Tries the bucket label first (most
	// indexer rows already use the bucket name), falls back to the raw
	// `torrent.quality` field. Used to highlight the matching row in the
	// stream picks table and disable that row's Assign button (it's
	// already the active stream).
	const attachedStreamQuality = $derived.by<string | null>(() => {
		if (!stream?.quality) return null;
		const byLabel = streamPicksByQuality.find((p) => p.quality === stream.quality);
		if (byLabel) return byLabel.quality;
		const byRaw = streamPicksByQuality.find((p) => p.torrent.quality === stream.quality);
		return byRaw?.quality ?? null;
	});

	// Same as `attachedStreamQuality` but for the Download tab's table.
	const attachedDownloadQuality = $derived.by<string | null>(() => {
		if (!download?.quality) return null;
		const byLabel = downloadPicksByQuality.find((p) => p.quality === download.quality);
		if (byLabel) return byLabel.quality;
		const byRaw = downloadPicksByQuality.find((p) => p.torrent.quality === download.quality);
		return byRaw?.quality ?? null;
	});

	function torrentToInfo(t: TorrentResultItem): AttachmentInfo {
		return {
			title: t.parsedTitle || t.title,
			seeders: t.seeders,
			leechers: t.leechers,
			sizeBytes: t.sizeBytes
		};
	}

	function shortCid(cid: string): string {
		if (cid.length <= 16) return cid;
		return `${cid.slice(0, 8)}…${cid.slice(-6)}`;
	}

	function formatSpeed(bytesPerSec: number): string {
		if (bytesPerSec <= 0) return '—';
		return `${formatSizeBytes(bytesPerSec)}/s`;
	}

	function formatEta(seconds: number): string {
		if (seconds < 60) return `${Math.round(seconds)}s`;
		if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
		return `${Math.round(seconds / 3600)}h`;
	}
</script>

{#snippet playIcon(klass: string)}
	<svg
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 24 24"
		fill="currentColor"
		class={klass}
		aria-hidden="true"
	>
		<polygon points="6 4 20 12 6 20 6 4" />
	</svg>
{/snippet}

{#snippet stats(info: AttachmentInfo)}
	<div class="flex items-center gap-1.5 text-[10px] text-base-content/60">
		<span class="text-success" title="Seeders">↑ {info.seeders ?? '—'}</span>
		<span class="text-warning" title="Leechers">↓ {info.leechers ?? '—'}</span>
		<span title="File size">· {info.sizeBytes != null ? formatSizeBytes(info.sizeBytes) : '—'}</span
		>
	</div>
{/snippet}

{#snippet trailerContent()}
	<table class="table table-xs w-full">
		<thead>
			<tr class="text-[10px] text-base-content/60 uppercase">
				<th class="text-left">Trailer</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each trailers as t (t.key)}
				<tr>
					<td class="text-left text-xs font-medium">
						{t.label ?? 'Trailer'}
					</td>
					<td>
						<button
							type="button"
							onclick={() => onTrailerPlay?.(t.key)}
							disabled={trailerPlaying}
							class="btn w-full btn-xs btn-primary"
							aria-label="Play trailer"
						>
							{#if trailerPlaying}
								<span class="loading loading-spinner loading-xs"></span>
							{:else}
								{@render playIcon('h-3 w-3 translate-x-0.5')}
							{/if}
						</button>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/snippet}

{#snippet streamPicksTable(activeQuality: string | null)}
	<table class="table w-full table-xs">
		<thead>
			<tr class="text-[10px] text-base-content/60 uppercase">
				<th class="text-left">Quality</th>
				<th class="text-success" title="Seeders">↑</th>
				<th class="text-warning" title="Leechers">↓</th>
				<th class="text-right">Size</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each streamPicksByQuality as pick (pick.quality)}
				{@const isActive = activeQuality === pick.quality}
				{@const isProbing = pick.status === 'probing'}
				<tr class={isActive ? 'bg-base-200' : ''}>
					<td class="text-left text-xs font-medium">{pick.quality}</td>
					<td class="text-success">{pick.torrent.seeders ?? '—'}</td>
					<td class="text-warning">{pick.torrent.leechers ?? '—'}</td>
					<td class="text-right text-[10px] text-base-content/70">
						{pick.torrent.sizeBytes != null ? formatSizeBytes(pick.torrent.sizeBytes) : '—'}
					</td>
					<td>
						{#if isActive}
							<button
								type="button"
								onclick={() => onStreamPlay?.()}
								disabled={streamPlaying}
								class="btn w-full btn-xs btn-primary"
								aria-label="Play stream"
							>
								{#if streamPlaying}
									<span class="loading loading-spinner loading-xs"></span>
								{:else}
									{@render playIcon('h-3 w-3 translate-x-0.5')}
								{/if}
							</button>
						{:else if isProbing}
							<button
								type="button"
								disabled
								class="btn w-full btn-xs btn-primary"
								aria-label="Probing streamability"
								title="Checking whether this row is streamable…"
							>
								<span class="loading loading-spinner loading-xs"></span>
							</button>
						{:else}
							<button
								type="button"
								onclick={() => onAttachStream?.(pick.torrent)}
								disabled={attachingStream}
								class="btn w-full btn-xs btn-primary"
								aria-label="Play stream"
							>
								{#if attachingStream}
									<span class="loading loading-spinner loading-xs"></span>
								{:else}
									{@render playIcon('h-3 w-3 translate-x-0.5')}
								{/if}
							</button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/snippet}

{#snippet streamContent()}
	{#if stream && onStreamPlay}
		<div class="flex flex-col items-stretch gap-2">
			{#if streamPicksByQuality.length > 0 && onAttachStream}
				{@render streamPicksTable(attachedStreamQuality)}
			{:else}
				<div class="flex flex-col items-center gap-1">
					{@render stats(stream)}
					<button
						type="button"
						onclick={() => onStreamPlay?.()}
						disabled={streamPlaying}
						class="btn btn-sm btn-primary"
						aria-label="Play stream"
					>
						{#if streamPlaying}
							<span class="loading loading-spinner loading-sm"></span>
						{:else}
							{@render playIcon('h-4 w-4 translate-x-0.5')}
						{/if}
					</button>
				</div>
			{/if}
		</div>
	{:else if stream}
		<div class="flex flex-col items-center gap-1">
			{@render stats(stream)}
		</div>
	{:else if streamPicksByQuality.length > 0 && onAttachStream}
		{@render streamPicksTable(null)}
	{:else}
		<div class="flex flex-col items-center gap-1">
			<span class="text-[10px] text-base-content/60">Not attached</span>
			<button type="button" disabled class="btn btn-sm btn-primary" aria-label="Play stream">
				{@render playIcon('h-4 w-4 translate-x-0.5')}
			</button>
		</div>
	{/if}
{/snippet}

{#snippet downloadPicksTable(activeQuality: string | null)}
	<table class="table table-xs w-full">
		<thead>
			<tr class="text-[10px] text-base-content/60 uppercase">
				<th class="text-left">Quality</th>
				<th class="text-success" title="Seeders">↑</th>
				<th class="text-warning" title="Leechers">↓</th>
				<th class="text-right">Size</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each downloadPicksByQuality as pick (pick.quality)}
				{@const isActive = activeQuality === pick.quality}
				<tr class={isActive ? 'bg-base-200' : ''}>
					<td class="text-left text-xs font-medium">{pick.quality}</td>
					<td class="text-success">{pick.torrent.seeders ?? '—'}</td>
					<td class="text-warning">{pick.torrent.leechers ?? '—'}</td>
					<td class="text-right text-[10px] text-base-content/70">
						{pick.torrent.sizeBytes != null ? formatSizeBytes(pick.torrent.sizeBytes) : '—'}
					</td>
					<td>
						{#if isActive && download?.ipfsCid && onDownloadPlay}
							<button
								type="button"
								onclick={() => onDownloadPlay?.()}
								disabled={downloadPlaying}
								class="btn w-full btn-xs btn-primary"
								aria-label="Play download"
							>
								{#if downloadPlaying}
									<span class="loading loading-spinner loading-xs"></span>
								{:else}
									{@render playIcon('h-3 w-3 translate-x-0.5')}
								{/if}
							</button>
						{:else if isActive && download}
							<button
								type="button"
								disabled
								class="btn w-full btn-xs btn-primary"
								aria-label="Downloading"
								title={download.finished
									? 'Seeding · pinning to IPFS…'
									: download.progress != null
										? `${Math.round(download.progress * 100)}%`
										: 'Queued'}
							>
								{#if download.finished}
									Seeding…
								{:else if download.progress != null}
									{Math.round(download.progress * 100)}%
								{:else}
									Queued
								{/if}
							</button>
						{:else}
							<button
								type="button"
								onclick={() => onAttachDownload?.(pick.torrent)}
								disabled={attachingDownload}
								class="btn w-full btn-xs btn-primary"
								aria-label="Download torrent"
							>
								{#if attachingDownload}
									<span class="loading loading-spinner loading-xs"></span>
								{:else}
									<Icon name="delapouite/plain-arrow" size="0.75rem" />
								{/if}
							</button>
						{/if}
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
{/snippet}

{#snippet downloadContent()}
	{#if download && !download.ipfsCid}
		<div class="flex flex-col items-stretch gap-1">
			{#if download.finished}
				<span class="text-center text-[10px] text-success">Seeding · pinning to IPFS…</span>
			{:else if download.progress != null}
				<progress class="progress h-1.5 w-full progress-primary" value={download.progress} max="1"
				></progress>
				<span class="text-center text-[10px] text-base-content/70">
					{Math.round(download.progress * 100)}%{download.downloadSpeed != null &&
					download.downloadSpeed > 0
						? ` · ${formatSpeed(download.downloadSpeed)}`
						: ''}{download.etaSeconds != null && download.etaSeconds > 0
						? ` · ETA ${formatEta(download.etaSeconds)}`
						: ''}
				</span>
			{:else}
				<span class="text-center text-[10px] text-base-content/50">Queued</span>
			{/if}
			<div class="flex justify-center">
				{@render stats(download)}
			</div>
		</div>
	{:else if downloadPicksByQuality.length > 0 && onAttachDownload}
		{@render downloadPicksTable(attachedDownloadQuality)}
	{:else if download && download.ipfsCid && onDownloadPlay}
		<div class="flex flex-col items-center gap-1">
			<span
				class="block max-w-full truncate font-mono text-[10px] text-base-content/60"
				title={download.ipfsCid}
			>
				{shortCid(download.ipfsCid)}
			</span>
			<button
				type="button"
				onclick={() => onDownloadPlay?.()}
				disabled={downloadPlaying}
				class="btn btn-sm btn-primary"
				aria-label="Play download"
			>
				{#if downloadPlaying}
					<span class="loading loading-spinner loading-sm"></span>
				{:else}
					{@render playIcon('h-4 w-4 translate-x-0.5')}
				{/if}
			</button>
		</div>
	{:else if preferredDownload && onAttachDownload}
		{@const info = torrentToInfo(preferredDownload)}
		<div class="flex flex-col items-center gap-1">
			{@render stats(info)}
			<button
				type="button"
				onclick={() => onAttachDownload?.(preferredDownload)}
				disabled={attachingDownload}
				class="btn btn-sm btn-primary"
				aria-label="Download torrent"
			>
				{#if attachingDownload}
					<span class="loading loading-spinner loading-sm"></span>
				{:else}
					<Icon name="delapouite/plain-arrow" size="1rem" />
				{/if}
			</button>
		</div>
	{:else}
		<div class="flex flex-col items-center gap-1">
			<span class="text-[10px] text-base-content/60">Not attached</span>
			<button type="button" disabled class="btn btn-sm btn-primary" aria-label="Download torrent">
				<Icon name="delapouite/plain-arrow" size="1rem" />
			</button>
		</div>
	{/if}
{/snippet}

<div class="grid w-full grid-cols-3 gap-3">
	{#if hasTrailers}
		<div class="flex flex-col rounded-md border border-base-content/10 bg-base-300">
			<div
				class="flex items-center justify-center gap-2 border-b border-base-content/10 px-3 py-2"
			>
				<Icon name="delapouite/film-strip" size={20} title="Trailer" />
				<span class="text-xs font-medium">Trailer</span>
			</div>
			<div class="p-3">
				{@render trailerContent()}
			</div>
		</div>
	{/if}
	<div class="flex flex-col rounded-md border border-base-content/10 bg-base-300">
		<div class="flex items-center justify-center gap-2 border-b border-base-content/10 px-3 py-2">
			<Icon name="lorc/magnet" size={20} title="Stream mode" />
			<span class="text-xs font-medium">Stream</span>
		</div>
		<div class="p-3">
			{@render streamContent()}
		</div>
	</div>
	<div class="flex flex-col rounded-md border border-base-content/10 bg-base-300">
		<div class="flex items-center justify-center gap-2 border-b border-base-content/10 px-3 py-2">
			<Icon name="delapouite/cloud-download" size={20} title="Download mode" />
			<span class="text-xs font-medium">Download</span>
		</div>
		<div class="p-3">
			{@render downloadContent()}
		</div>
	</div>
</div>

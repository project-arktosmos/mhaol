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
		/** One preferred `streamable`-probed torrent per quality bucket
		 * (4K → 2160p → 1080p → … → Other), sorted by quality priority then
		 * seeders within the bucket. Surfaced as a faded preview in the
		 * Stream cell when nothing's attached yet, with a quality selector
		 * that swaps the currently displayed pick. Empty when no row has
		 * been probed `streamable` yet. */
		streamPicksByQuality?: Array<{ quality: string; torrent: TorrentResultItem }>;
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
		/** Trailer cell payload. When set with `onTrailerPlay`, a "Trailer"
		 * button is rendered above Stream / Download (col-span-2 when both
		 * are visible, single column when paired only with the actionable
		 * Play cell). The YouTube video id is shown beneath the label in
		 * the same monospace style as the actionable Play cell's IPFS CID.
		 * `null` hides the cell. */
		trailer?: { youtubeId: string } | null;
		/** Click handler for the Trailer cell. */
		onTrailerPlay?: () => void | Promise<void>;
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
		onAttachDownload,
		onAttachStream,
		attachingDownload = false,
		attachingStream = false,
		trailer = null,
		onTrailerPlay,
		trailerPlaying = false
	}: Props = $props();

	// Once the download has been pinned to IPFS the Download cell becomes a
	// click-to-play button, which is strictly better than streaming the
	// torrent (same bytes, faster start, no peer churn). Hide the Stream
	// cell entirely in that state so the user lands on the obvious choice;
	// the Download cell promotes to `col-span-2` so the row stays balanced.
	const downloadActionable = $derived(Boolean(download && download.ipfsCid && onDownloadPlay));

	// When only two cells are visible (Trailer + the actionable Download),
	// drop the col-span-2 on both so they sit side-by-side in a single row
	// instead of stacking. The all-three case (Trailer + Stream + Download)
	// keeps Trailer as a full-width header above Stream / Download.
	const hasTrailer = $derived(Boolean(trailer && onTrailerPlay));
	const trailerSpansFull = $derived(hasTrailer && !downloadActionable);
	const downloadSpansFull = $derived(downloadActionable && !hasTrailer);

	// User's quality pick from the Stream cell's selector (suggestion mode).
	// `null` falls back to `streamPicksByQuality[0]` — the best available
	// quality. The pick is reset when the available qualities change so a
	// stale selection (e.g. user picked 1080p, results refreshed and 1080p
	// is no longer streamable) doesn't strand the cell.
	let selectedStreamQuality = $state<string | null>(null);
	$effect(() => {
		if (
			selectedStreamQuality !== null &&
			!streamPicksByQuality.some((p) => p.quality === selectedStreamQuality)
		) {
			selectedStreamQuality = null;
		}
	});
	const currentStreamPick = $derived.by(() => {
		if (streamPicksByQuality.length === 0) return null;
		if (selectedStreamQuality === null) return streamPicksByQuality[0];
		return (
			streamPicksByQuality.find((p) => p.quality === selectedStreamQuality) ??
			streamPicksByQuality[0]
		);
	});

	// Quality of the currently-attached stream, mapped to a bucket label
	// from `streamPicksByQuality`. Tries the bucket label first (most
	// indexer rows already use the bucket name), falls back to the raw
	// `torrent.quality` field. Used to default the actionable Stream
	// cell's quality selector so the attached row is the one shown
	// selected.
	const attachedStreamQuality = $derived.by<string | null>(() => {
		if (!stream?.quality) return null;
		const byLabel = streamPicksByQuality.find((p) => p.quality === stream.quality);
		if (byLabel) return byLabel.quality;
		const byRaw = streamPicksByQuality.find((p) => p.torrent.quality === stream.quality);
		return byRaw?.quality ?? null;
	});
	const actionableSelectedQuality = $derived(
		selectedStreamQuality ?? attachedStreamQuality ?? streamPicksByQuality[0]?.quality ?? null
	);

	function changeStreamQuality(quality: string, attach: boolean): void {
		selectedStreamQuality = quality;
		if (attach && onAttachStream) {
			const pick = streamPicksByQuality.find((p) => p.quality === quality);
			if (pick) void onAttachStream(pick.torrent);
		}
	}

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

{#snippet stats(info: AttachmentInfo)}
	<div class="flex items-center gap-1.5 text-[10px] text-base-content/60">
		<span class="text-success" title="Seeders">↑ {info.seeders ?? '—'}</span>
		<span class="text-warning" title="Leechers">↓ {info.leechers ?? '—'}</span>
		<span title="File size">· {info.sizeBytes != null ? formatSizeBytes(info.sizeBytes) : '—'}</span
		>
	</div>
{/snippet}

{#snippet downloadHeader()}
	<svg
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 24 24"
		fill="currentColor"
		class="h-8 w-8 translate-x-0.5"
		aria-hidden="true"
	>
		<polygon points="6 4 20 12 6 20 6 4" />
	</svg>
	<span class="text-xs font-medium">Play</span>
{/snippet}

{#snippet header(iconName: string, label: string, iconTitle: string)}
	<Icon name={iconName} size={32} title={iconTitle} />
	<span class="text-xs font-medium">{label}</span>
{/snippet}

<div class="flex flex-col gap-2">
	<div class="grid grid-cols-2 gap-3">
		{#if trailer && onTrailerPlay}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center"
				class:col-span-2={trailerSpansFull}
			>
				<Icon name="delapouite/film-strip" size={32} title="Trailer" />
				<span class="text-xs font-medium">Trailer</span>
				<span
					class="block max-w-full truncate font-mono text-[10px] text-base-content/60"
					title={trailer.youtubeId}
				>
					{trailer.youtubeId}
				</span>
				<button
					type="button"
					onclick={() => onTrailerPlay?.()}
					disabled={trailerPlaying}
					class="btn btn-sm btn-primary"
				>
					{trailerPlaying ? 'Starting…' : 'Play'}
				</button>
			</div>
		{/if}

		{#if !downloadActionable}
			{#if stream && onStreamPlay}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center text-base-content"
				>
					{@render header('lorc/magnet', 'Stream', 'Stream mode')}
					{#if streamPicksByQuality.length > 1 && onAttachStream}
						<select
							class="select-bordered select select-xs"
							value={actionableSelectedQuality ?? ''}
							onchange={(e) =>
								changeStreamQuality((e.currentTarget as HTMLSelectElement).value, true)}
							aria-label="Pick stream quality"
							title="Pick stream quality — switching re-attaches the stream"
						>
							{#each streamPicksByQuality as pick (pick.quality)}
								<option value={pick.quality}>{pick.quality}</option>
							{/each}
						</select>
					{/if}
					{@render stats(stream)}
					<button
						type="button"
						onclick={() => onStreamPlay?.()}
						disabled={streamPlaying}
						class="btn btn-sm btn-primary"
					>
						{streamPlaying ? 'Starting…' : 'Play'}
					</button>
				</div>
			{:else if stream}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center text-base-content"
				>
					{@render header('lorc/magnet', 'Stream', 'Stream mode')}
					{@render stats(stream)}
				</div>
			{:else if currentStreamPick && onAttachStream}
				{@const info = torrentToInfo(currentStreamPick.torrent)}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center opacity-60"
				>
					{@render header('lorc/magnet', 'Stream', 'Stream mode')}
					{#if streamPicksByQuality.length > 1}
						<select
							class="select-bordered select select-xs"
							value={currentStreamPick.quality}
							onchange={(e) =>
								changeStreamQuality((e.currentTarget as HTMLSelectElement).value, false)}
							aria-label="Pick stream quality"
							title="Pick stream quality"
						>
							{#each streamPicksByQuality as pick (pick.quality)}
								<option value={pick.quality}>{pick.quality}</option>
							{/each}
						</select>
					{:else}
						<span class="text-[10px] font-medium text-base-content/70">
							{currentStreamPick.quality}
						</span>
					{/if}
					{@render stats(info)}
					<button
						type="button"
						onclick={() => onAttachStream?.(currentStreamPick.torrent)}
						disabled={attachingStream}
						class="btn btn-sm btn-primary"
					>
						{attachingStream ? 'Starting…' : 'Assign'}
					</button>
				</div>
			{:else}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center text-base-content/40"
				>
					{@render header('lorc/magnet', 'Stream', 'Stream mode')}
					<span class="text-[10px] text-base-content/60">Not attached</span>
				</div>
			{/if}
		{/if}

		{#if download && download.ipfsCid && onDownloadPlay}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center text-base-content"
				class:col-span-2={downloadSpansFull}
			>
				{@render downloadHeader()}
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
				>
					{downloadPlaying ? 'Starting…' : 'Play'}
				</button>
			</div>
		{:else if download}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center text-base-content"
			>
				{@render header('delapouite/cloud-download', 'Download', 'Download mode')}
				{#if download.finished}
					<span class="text-[10px] text-success">Seeding · pinning to IPFS…</span>
				{:else if download.progress != null}
					<progress
						class="progress h-1.5 w-full progress-primary"
						value={download.progress}
						max="1"
					></progress>
					<span class="text-[10px] text-base-content/70">
						{Math.round(download.progress * 100)}%{download.downloadSpeed != null &&
						download.downloadSpeed > 0
							? ` · ${formatSpeed(download.downloadSpeed)}`
							: ''}{download.etaSeconds != null && download.etaSeconds > 0
							? ` · ETA ${formatEta(download.etaSeconds)}`
							: ''}
					</span>
				{:else}
					<span class="text-[10px] text-base-content/50">Queued</span>
				{/if}
				{@render stats(download)}
			</div>
		{:else if preferredDownload && onAttachDownload}
			{@const info = torrentToInfo(preferredDownload)}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300 p-3 text-center opacity-60"
			>
				{@render header('delapouite/cloud-download', 'Download', 'Download mode')}
				{@render stats(info)}
				<button
					type="button"
					onclick={() => onAttachDownload?.(preferredDownload)}
					disabled={attachingDownload}
					class="btn btn-sm btn-primary"
				>
					{attachingDownload ? 'Starting…' : 'Assign'}
				</button>
			</div>
		{:else}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center text-base-content/40"
			>
				{@render header('delapouite/cloud-download', 'Download', 'Download mode')}
				<span class="text-[10px] text-base-content/60">Not attached</span>
			</div>
		{/if}
	</div>
</div>

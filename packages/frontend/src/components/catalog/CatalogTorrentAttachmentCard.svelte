<script lang="ts" module>
	export interface AttachmentInfo {
		title: string;
		seeders: number | null;
		leechers: number | null;
		sizeBytes: number | null;
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
		/** Best `streamable`-probed torrent (highest quality first, most
		 * seeders within that quality). Surfaced as a faded preview in the
		 * Stream cell when nothing's attached yet. `null` when no row has
		 * been probed `streamable` yet. */
		preferredStream?: TorrentResultItem | null;
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
		 * button is rendered as a `col-span-2` row above Stream / Download
		 * and click runs the callback (typically the trailer player's
		 * `handleStart`). `null` hides the cell. */
		trailer?: { title: string } | null;
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
		preferredStream = null,
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

{#snippet downloadHeader(info: AttachmentInfo | null)}
	<svg
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 24 24"
		fill="currentColor"
		class="h-8 w-8 translate-x-0.5"
		aria-hidden="true"
	>
		<polygon points="6 4 20 12 6 20 6 4" />
	</svg>
	<span class="text-xs font-medium">Download</span>
	{#if info}
		<span class="block max-w-full truncate text-[10px] text-base-content/70" title={info.title}>
			{info.title}
		</span>
	{/if}
{/snippet}

{#snippet header(info: AttachmentInfo | null, iconName: string, label: string, iconTitle: string)}
	<Icon name={iconName} size={32} title={iconTitle} />
	<span class="text-xs font-medium">{label}</span>
	{#if info}
		<span class="block max-w-full truncate text-[10px] text-base-content/70" title={info.title}>
			{info.title}
		</span>
	{/if}
{/snippet}

<div class="flex flex-col gap-2">
	<h2 class="text-sm font-semibold text-base-content/70 uppercase">Torrent attachment</h2>
	<div class="grid grid-cols-2 gap-3">
		{#if trailer && onTrailerPlay}
			<button
				type="button"
				onclick={() => onTrailerPlay?.()}
				disabled={trailerPlaying}
				class="col-span-2 flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center transition-colors hover:border-success/50 hover:bg-base-300/70 disabled:cursor-progress disabled:opacity-60"
				aria-label="Play trailer"
			>
				<Icon name="delapouite/film-strip" size={32} title="Trailer" />
				<span class="text-xs font-medium">Trailer</span>
				<span
					class="block max-w-full truncate text-[10px] text-base-content/70"
					title={trailer.title}
				>
					{trailer.title}
				</span>
				<span class="text-[10px] font-medium text-success">
					{trailerPlaying ? 'Starting…' : 'Click to play'}
				</span>
			</button>
		{/if}

		{#if !downloadActionable}
			{#if stream && onStreamPlay}
				<button
					type="button"
					onclick={() => onStreamPlay?.()}
					disabled={streamPlaying}
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center transition-colors hover:border-success/50 hover:bg-base-300/70 disabled:cursor-progress disabled:opacity-60"
					aria-label="Play stream"
				>
					{@render header(stream, 'lorc/magnet', 'Stream', 'Stream mode')}
					{@render stats(stream)}
					<span class="text-[10px] font-medium text-success">
						{streamPlaying ? 'Starting…' : 'Click to play'}
					</span>
				</button>
			{:else if stream}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center text-base-content"
				>
					{@render header(stream, 'lorc/magnet', 'Stream', 'Stream mode')}
					{@render stats(stream)}
				</div>
			{:else if preferredStream && onAttachStream}
				{@const info = torrentToInfo(preferredStream)}
				<button
					type="button"
					onclick={() => onAttachStream?.(preferredStream)}
					disabled={attachingStream}
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center opacity-60 transition-opacity hover:border-success/50 hover:bg-base-300/70 hover:opacity-90 disabled:cursor-progress disabled:opacity-40"
					aria-label="Attach this torrent for streaming"
					title="Suggested pick from the torrent search — click to start streaming"
				>
					{@render header(info, 'lorc/magnet', 'Stream', 'Stream mode')}
					{@render stats(info)}
					<span class="text-[10px] font-medium text-base-content/70">
						{attachingStream ? 'Starting…' : 'Click to attach'}
					</span>
				</button>
			{:else}
				<div
					class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center text-base-content/40"
				>
					{@render header(null, 'lorc/magnet', 'Stream', 'Stream mode')}
					<span class="text-[10px] text-base-content/60">Not attached</span>
				</div>
			{/if}
		{/if}

		{#if download && download.ipfsCid && onDownloadPlay}
			<button
				type="button"
				onclick={() => onDownloadPlay?.()}
				disabled={downloadPlaying}
				class="col-span-2 flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center transition-colors hover:border-success/50 hover:bg-base-300/70 disabled:cursor-progress disabled:opacity-60"
				aria-label="Play via IPFS"
			>
				{@render downloadHeader(download)}
				<span
					class="block max-w-full truncate font-mono text-[10px] text-base-content/60"
					title={download.ipfsCid}
				>
					{shortCid(download.ipfsCid)}
				</span>
				<span class="text-[10px] font-medium text-success">
					{downloadPlaying ? 'Starting…' : 'Click to play'}
				</span>
			</button>
		{:else if download}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center text-base-content"
			>
				{@render downloadHeader(download)}
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
			<button
				type="button"
				onclick={() => onAttachDownload?.(preferredDownload)}
				disabled={attachingDownload}
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center opacity-60 transition-opacity hover:border-success/50 hover:bg-base-300/70 hover:opacity-90 disabled:cursor-progress disabled:opacity-40"
				aria-label="Attach this torrent for download"
				title="Suggested pick from the torrent search — click to attach and start downloading"
			>
				{@render downloadHeader(info)}
				{@render stats(info)}
				<span class="text-[10px] font-medium text-base-content/70">
					{attachingDownload ? 'Starting…' : 'Click to attach'}
				</span>
			</button>
		{:else}
			<div
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center text-base-content/40"
			>
				{@render downloadHeader(null)}
				<span class="text-[10px] text-base-content/60">Not attached</span>
			</div>
		{/if}
	</div>
</div>

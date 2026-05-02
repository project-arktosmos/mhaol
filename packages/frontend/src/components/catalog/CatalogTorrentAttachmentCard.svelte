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
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';
	import { formatSizeBytes } from '$lib/search.service';

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
	}

	let {
		download,
		stream,
		onStreamPlay,
		streamPlaying = false,
		onDownloadPlay,
		downloadPlaying = false
	}: Props = $props();

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

{#snippet header(info: AttachmentInfo | null, iconName: string, label: string, iconTitle: string)}
	<Icon name={iconName} size={32} title={iconTitle} />
	<span class="text-xs font-medium">{label}</span>
	{#if info}
		<span class="block max-w-full truncate text-[10px] text-base-content/70" title={info.title}>
			{info.title}
		</span>
	{/if}
{/snippet}

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-3 text-sm font-semibold text-base-content/70 uppercase">Torrent attachment</h2>
	<div class="grid grid-cols-2 gap-3">
		{#if download && download.ipfsCid && onDownloadPlay}
			<button
				type="button"
				onclick={() => onDownloadPlay?.()}
				disabled={downloadPlaying}
				class="flex flex-col items-center gap-1 rounded-md border border-base-content/10 bg-base-300/40 p-3 text-center transition-colors hover:border-success/50 hover:bg-base-300/70 disabled:cursor-progress disabled:opacity-60"
				aria-label="Play via IPFS"
			>
				{@render header(download, 'delapouite/cloud-download', 'Download', 'Download mode')}
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
		{:else}
			<div
				class={classNames(
					'flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center',
					download ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
				)}
			>
				{@render header(download, 'delapouite/cloud-download', 'Download', 'Download mode')}
				{#if download}
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
				{:else}
					<span class="text-[10px] text-base-content/60">Not attached</span>
				{/if}
			</div>
		{/if}

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
		{:else}
			<div
				class={classNames(
					'flex flex-col items-center gap-1 rounded-md border border-base-content/10 p-3 text-center',
					stream ? 'bg-base-300/40 text-base-content' : 'text-base-content/40'
				)}
			>
				{@render header(stream, 'lorc/magnet', 'Stream', 'Stream mode')}
				{#if stream}
					{@render stats(stream)}
				{:else}
					<span class="text-[10px] text-base-content/60">Not attached</span>
				{/if}
			</div>
		{/if}
	</div>
</div>

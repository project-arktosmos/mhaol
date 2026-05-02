<script lang="ts">
	import classNames from 'classnames';
	import { Icon } from 'cloud-ui';
	import type { TrackResolver } from '$services/catalog/track-resolver.svelte';

	interface Props {
		resolver: TrackResolver;
		thumb: string | null;
		albumTitle?: string;
		// CID of the firkin this tracklist belongs to. Forwarded to the
		// player so the navbar's per-track consumption tracker buckets
		// time against the right (firkin, track) pair. Omitted on the
		// virtual catalog page (no firkin yet); `preview` already disables
		// playback there.
		firkinId?: string;
		onRefresh?: () => void;
		// Preview mode hides per-track YouTube/lyrics state and disables
		// playback. Used by `/catalog/virtual` where nothing has been
		// resolved yet — bookmarking is what kicks off the server-side
		// per-track YouTube + LRCLIB resolution.
		preview?: boolean;
		// Triggered by the "Download album" button. The detail page kicks
		// off the backend's `POST /api/firkins/:id/download-album` task
		// and starts polling `/download-progress` so per-track download
		// status updates live.
		onDownloadAlbum?: () => void;
		downloadInFlight?: boolean;
		downloadError?: string | null;
	}
	let {
		resolver,
		thumb,
		albumTitle,
		firkinId,
		onRefresh,
		preview = false,
		onDownloadAlbum,
		downloadInFlight = false,
		downloadError = null
	}: Props = $props();

	let expandedLyricsIdx = $state<number | null>(null);

	function formatDuration(ms: number | null): string {
		if (!ms || !Number.isFinite(ms) || ms <= 0) return '—';
		const total = Math.round(ms / 1000);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	const canDownload = $derived(
		!preview &&
			Boolean(onDownloadAlbum) &&
			resolver.tracks.some((t) => Boolean(t.youtubeUrl) && !t.localCid)
	);
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="mb-2 flex items-center justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
			Tracks{resolver.tracks.length > 0 ? ` (${resolver.tracks.length})` : ''}
		</h2>
		<div class="flex items-center gap-2">
			{#if !preview && onDownloadAlbum}
				<button
					type="button"
					class="btn btn-xs btn-primary"
					onclick={() => onDownloadAlbum?.()}
					disabled={!canDownload || downloadInFlight}
					title="Download every track and store it locally"
				>
					<Icon name="delapouite/cloud-download" size={14} />
					{downloadInFlight ? 'Downloading…' : 'Download album'}
				</button>
			{/if}
			{#if onRefresh}
				<button
					type="button"
					class="btn btn-outline btn-xs"
					onclick={() => onRefresh?.()}
					disabled={resolver.status === 'loading'}
				>
					{resolver.status === 'loading' ? 'Loading…' : 'Refresh'}
				</button>
			{/if}
		</div>
	</div>
	{#if downloadError}
		<div class="mb-2 alert alert-error">
			<span>{downloadError}</span>
		</div>
	{/if}
	{#if resolver.status === 'loading' && resolver.tracks.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if resolver.status === 'error'}
		<p class="text-sm text-error">{resolver.error ?? 'Failed'}</p>
	{:else if resolver.tracks.length === 0}
		<p class="text-sm text-base-content/60">No tracks found.</p>
	{:else}
		{#if resolver.playError}
			<div class="mb-2 alert alert-error">
				<span>{resolver.playError}</span>
			</div>
		{/if}
		<ol class="flex flex-col gap-1">
			{#each resolver.tracks as track, idx (track.id || `${track.position}-${track.title}`)}
				{@const streamable =
					!preview &&
					(track.youtubeStatus === 'found' || track.youtubeStatus === 'idle') &&
					!!track.youtubeUrl}
				{@const playable = !preview && Boolean(track.localCid)}
				{@const isPlaying = resolver.playingIndex === idx}
				{@const lyricsExpanded = !preview && expandedLyricsIdx === idx && Boolean(track.lyrics)}
				{@const lyricsKindLabel = track.lyrics
					? (track.lyrics.syncedLyrics?.length ?? 0) > 0
						? 'synced'
						: track.lyrics.instrumental
							? 'instrumental'
							: track.lyrics.plainLyrics
								? 'plain'
								: 'lyrics'
					: null}
				<li class="flex flex-col gap-1">
					<div
						class="flex w-full flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-xs"
					>
						<div class="flex min-w-0 flex-1 items-center gap-2 text-left">
							<span class="w-6 shrink-0 text-right font-mono text-base-content/60"
								>{track.position}</span
							>
							<span class="flex-1 truncate" title={track.title}>{track.title}</span>
							<span class="text-base-content/60">{formatDuration(track.lengthMs)}</span>
						</div>

						{#if !preview}
							{#if playable}
								{#if isPlaying}
									<span class="badge badge-xs badge-primary">starting…</span>
								{:else}
									<button
										type="button"
										class="btn btn-xs btn-primary"
										onclick={() => resolver.playLocal(idx, { thumb, albumTitle, firkinId })}
										disabled={resolver.playingIndex !== null}
										title={`Play "${track.title}" from local file`}
									>
										<Icon name="guard13007/play-button" size={12} />
										Play
									</button>
								{/if}
							{/if}
							{#if streamable}
								{#if isPlaying && !playable}
									<span class="badge badge-xs badge-primary">starting…</span>
								{:else}
									<button
										type="button"
										class={classNames('btn btn-xs', {
											'btn-outline': playable,
											'btn-secondary': !playable
										})}
										onclick={() => resolver.play(idx, { thumb, albumTitle, firkinId })}
										disabled={resolver.playingIndex !== null}
										title={`Stream "${track.title}" from YouTube`}
									>
										Stream
									</button>
								{/if}
							{/if}

							{#if track.downloadStatus === 'downloading'}
								<span class="badge badge-xs badge-info">
									DL {Math.round((track.downloadProgress ?? 0) * 100)}%
								</span>
							{:else if track.downloadStatus === 'pending'}
								<span class="badge badge-ghost badge-xs">DL queued</span>
							{:else if track.downloadStatus === 'failed'}
								<span
									class="badge badge-xs badge-error"
									title={track.downloadError ?? 'Download failed'}
								>
									DL err
								</span>
							{:else if track.downloadStatus === 'completed' && !track.localCid}
								<span class="badge badge-xs badge-success">DL ✓</span>
							{/if}

							{#if track.youtubeStatus === 'pending'}
								<span class="badge badge-ghost badge-xs">YT queued</span>
							{:else if track.youtubeStatus === 'searching'}
								<span class="badge badge-ghost badge-xs">YT…</span>
							{:else if !streamable && track.youtubeStatus === 'missing'}
								<span class="badge badge-xs badge-warning">no YT</span>
							{:else if track.youtubeStatus === 'error'}
								<span class="badge badge-xs badge-error">YT err</span>
							{/if}

							{#if track.lyricsStatus === 'pending'}
								<span class="badge badge-ghost badge-xs">Lyrics queued</span>
							{:else if track.lyricsStatus === 'searching'}
								<span class="badge badge-ghost badge-xs">Lyrics…</span>
							{:else if track.lyrics}
								<button
									type="button"
									class={classNames('badge badge-xs', {
										'badge-info': !lyricsExpanded,
										'badge-primary': lyricsExpanded
									})}
									onclick={() => (expandedLyricsIdx = lyricsExpanded ? null : idx)}
									title={lyricsExpanded ? 'Hide lyrics' : `Show ${lyricsKindLabel} lyrics`}
								>
									{lyricsExpanded ? 'hide' : lyricsKindLabel}
								</button>
							{:else if track.lyricsStatus === 'missing'}
								<span class="badge badge-xs badge-warning">no lyrics</span>
							{:else if track.lyricsStatus === 'error'}
								<span class="badge badge-xs badge-error">lyrics err</span>
							{/if}
						{/if}
					</div>

					{#if lyricsExpanded && track.lyrics}
						{@const lyrics = track.lyrics}
						<div class="flex flex-col gap-1 rounded border border-base-content/10 bg-base-100 p-2">
							{#if lyrics.syncedLyrics && lyrics.syncedLyrics.length > 0}
								<div class="flex max-h-64 flex-col gap-0.5 overflow-y-auto text-xs leading-tight">
									{#each lyrics.syncedLyrics as line, lineIdx (lineIdx)}
										<span class="text-base-content/80">{line.text || '…'}</span>
									{/each}
								</div>
							{:else if lyrics.plainLyrics}
								<pre
									class="max-h-64 overflow-y-auto text-xs whitespace-pre-wrap text-base-content/80">{lyrics.plainLyrics}</pre>
							{:else if lyrics.instrumental}
								<span class="text-xs text-base-content/60">Instrumental.</span>
							{:else}
								<span class="text-xs text-base-content/60">No lyrics in this entry.</span>
							{/if}
						</div>
					{/if}
				</li>
			{/each}
		</ol>
	{/if}
</div>

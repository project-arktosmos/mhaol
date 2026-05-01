<script lang="ts">
	import classNames from 'classnames';
	import type { TrackResolver } from '$services/catalog/track-resolver.svelte';

	interface Props {
		resolver: TrackResolver;
		thumb: string | null;
		onRefresh?: () => void;
	}
	let { resolver, thumb, onRefresh }: Props = $props();

	function formatDuration(ms: number | null): string {
		if (!ms || !Number.isFinite(ms) || ms <= 0) return '—';
		const total = Math.round(ms / 1000);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="mb-2 flex items-center justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
			Tracks{resolver.tracks.length > 0 ? ` (${resolver.tracks.length})` : ''}
		</h2>
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
				{@const playable =
					(track.youtubeStatus === 'found' || track.youtubeStatus === 'idle') && !!track.youtubeUrl}
				{@const isPlaying = resolver.playingIndex === idx}
				<li>
					<button
						type="button"
						class={classNames(
							'flex w-full flex-wrap items-center gap-2 rounded border border-base-content/10 px-2 py-1 text-left text-xs',
							{
								'cursor-pointer hover:bg-base-100': playable && !isPlaying,
								'opacity-60': isPlaying,
								'cursor-default': !playable
							}
						)}
						disabled={!playable || resolver.playingIndex !== null}
						onclick={() => resolver.play(idx, { thumb })}
						title={playable ? `Play "${track.title}"` : track.title}
					>
						<span class="w-6 shrink-0 text-right font-mono text-base-content/60"
							>{track.position}</span
						>
						<span class="flex-1 truncate">{track.title}</span>
						<span class="text-base-content/60">{formatDuration(track.lengthMs)}</span>
						{#if track.youtubeStatus === 'pending'}
							<span class="badge badge-ghost badge-xs">YT queued</span>
						{:else if track.youtubeStatus === 'searching'}
							<span class="badge badge-ghost badge-xs">YT…</span>
						{:else if playable}
							{#if isPlaying}
								<span class="badge badge-xs badge-primary">starting…</span>
							{:else}
								<span class="badge badge-xs badge-primary">▶ Play</span>
							{/if}
						{:else if track.youtubeStatus === 'missing'}
							<span class="badge badge-xs badge-warning">no match</span>
						{:else if track.youtubeStatus === 'error'}
							<span class="badge badge-xs badge-error">error</span>
						{/if}
					</button>
				</li>
			{/each}
		</ol>
	{/if}
</div>

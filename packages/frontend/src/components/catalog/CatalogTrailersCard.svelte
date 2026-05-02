<script lang="ts">
	import classNames from 'classnames';
	import type { TrailerResolver } from '$services/catalog/trailer-resolver.svelte';
	import type { TrailerEntry } from '$services/catalog/types';

	interface Props {
		resolver: TrailerResolver;
		firkinTitle: string;
		thumb: string | null;
	}
	let { resolver, firkinTitle, thumb }: Props = $props();

	function onPlay(entry: TrailerEntry): void {
		void resolver.play(entry, { firkinTitle, thumb });
	}
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<div class="mb-2 flex items-center justify-between gap-2">
		<h2 class="text-sm font-semibold text-base-content/70 uppercase">
			Trailers{resolver.trailers.length > 0 ? ` (${resolver.trailers.length})` : ''}
		</h2>
	</div>
	{#if resolver.playError}
		<div class="mb-2 alert alert-error">
			<span>{resolver.playError}</span>
		</div>
	{/if}
	{#if resolver.status === 'loading' && resolver.trailers.length === 0}
		<p class="text-sm text-base-content/60">Loading…</p>
	{:else if resolver.status === 'error' && resolver.trailers.length === 0}
		<p class="text-sm text-error">{resolver.error ?? 'Failed'}</p>
	{:else if resolver.trailers.length === 0}
		<p class="text-sm text-base-content/60">No trailers found.</p>
	{:else}
		<ol class="flex flex-col gap-1">
			{#each resolver.trailers as trailer (trailer.key)}
				{@const playable =
					(trailer.status === 'found' || trailer.status === 'idle') && !!trailer.youtubeUrl}
				{@const isPlaying = resolver.playingKey === trailer.key}
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
						disabled={!playable || resolver.playingKey !== null}
						onclick={() => onPlay(trailer)}
						title={playable ? `Play ${trailer.label ?? 'trailer'}` : (trailer.label ?? 'Trailer')}
					>
						<span class="flex-1 truncate">
							{trailer.label ?? 'Trailer'}
						</span>
						{#if trailer.status === 'pending'}
							<span class="badge badge-ghost badge-xs">YT queued</span>
						{:else if trailer.status === 'searching'}
							<span class="badge badge-ghost badge-xs">YT…</span>
						{:else if playable}
							{#if isPlaying}
								<span class="badge badge-xs badge-primary">starting…</span>
							{:else}
								<span class="badge badge-xs badge-primary">▶ Play</span>
							{/if}
						{:else if trailer.status === 'missing'}
							<span class="badge badge-xs badge-warning">no match</span>
						{:else if trailer.status === 'error'}
							<span class="badge badge-xs badge-error">error</span>
						{/if}
					</button>
				</li>
			{/each}
		</ol>
	{/if}
</div>

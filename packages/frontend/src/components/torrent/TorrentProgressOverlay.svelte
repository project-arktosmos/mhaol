<script lang="ts">
	import classNames from 'classnames';
	import type { TorrentState } from '$types/torrent.type';
	import { formatSpeed, formatEta } from '$types/torrent.type';

	interface Props {
		torrentProgress: number | null;
		torrentState: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
	}

	let { torrentProgress, torrentState, torrentSpeed = null, torrentEta = null }: Props = $props();

	let progressPercent = $derived(torrentProgress !== null ? Math.round(torrentProgress * 100) : 0);
</script>

<div class="absolute inset-x-0 bottom-0 bg-black/70 px-2 py-1.5">
	<div class="mb-1 flex items-center justify-between text-xs text-white">
		<span
			class={classNames('font-medium', {
				'text-info': torrentState === 'initializing' || torrentState === 'checking',
				'text-primary': torrentState === 'downloading',
				'text-warning': torrentState === 'paused',
				'text-error': torrentState === 'error'
			})}
		>
			{progressPercent}%
		</span>
		<span class="opacity-70">
			{#if torrentState === 'downloading' && torrentSpeed}
				{formatSpeed(torrentSpeed)}
			{:else if torrentState === 'initializing'}
				Starting...
			{:else if torrentState === 'paused'}
				Paused
			{:else if torrentState === 'error'}
				Error
			{:else if torrentState === 'checking'}
				Checking...
			{/if}
		</span>
	</div>
	<div class="h-1 w-full overflow-hidden rounded-full bg-white/20">
		<div
			class={classNames('h-full rounded-full transition-all', {
				'bg-primary': torrentState === 'downloading',
				'bg-info': torrentState === 'initializing' || torrentState === 'checking',
				'bg-warning': torrentState === 'paused',
				'bg-error': torrentState === 'error'
			})}
			style="width: {progressPercent}%"
		></div>
	</div>
	{#if torrentState === 'downloading' && torrentEta}
		<div class="mt-0.5 text-right text-xs text-white/50">
			{formatEta(torrentEta)}
		</div>
	{/if}
</div>

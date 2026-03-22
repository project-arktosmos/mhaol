<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateColor,
		getStateLabel
	} from 'ui-lib/types/torrent.type';
	import type { TorrentInfo } from 'ui-lib/types/torrent.type';

	export let torrent: TorrentInfo;

	const dispatch = createEventDispatcher<{
		pause: { infoHash: string };
		resume: { infoHash: string };
		remove: { infoHash: string };
		stream: { infoHash: string };
	}>();

	$: progressPercent = Math.round(torrent.progress * 100);
	$: isPaused = torrent.state === 'paused';
	$: isActive = torrent.state === 'downloading' || torrent.state === 'initializing';
	$: isSeeding = torrent.state === 'seeding';
	$: isStreamable = (torrent.state === 'downloading' && torrent.progress >= 0.02) || isSeeding;
	$: stateColor = getStateColor(torrent.state);
</script>

<div class="rounded-lg bg-base-100 p-4">
	<div class="flex items-start justify-between gap-4">
		<div class="flex-1 overflow-hidden">
			<h3 class="truncate font-medium" title={torrent.name}>
				{torrent.name}
			</h3>
			<div class="mt-1 flex flex-wrap items-center gap-2">
				<span
					class={classNames('badge badge-sm', {
						'badge-info': stateColor === 'info',
						'badge-primary': stateColor === 'primary',
						'badge-success': stateColor === 'success',
						'badge-warning': stateColor === 'warning',
						'badge-error': stateColor === 'error',
						'badge-neutral': stateColor === 'neutral'
					})}
				>
					{getStateLabel(torrent.state)}
				</span>

				{#if torrent.size > 0}
					<span class="text-xs text-base-content/60">
						{formatBytes(torrent.size)}
					</span>
				{/if}

				{#if isActive || isSeeding}
					<span class="text-xs text-base-content/60">
						{torrent.peers} peers / {torrent.seeds} seeds
					</span>
				{/if}
			</div>

			{#if torrent.state === 'downloading'}
				<div class="mt-2 flex items-center gap-2">
					<progress class="progress flex-1 progress-primary" value={progressPercent} max="100"
					></progress>
					<span class="text-sm font-medium">{progressPercent}%</span>
				</div>
				<div class="mt-1 flex items-center gap-4 text-xs text-base-content/60">
					<span>DL: {formatSpeed(torrent.downloadSpeed)}</span>
					<span>UL: {formatSpeed(torrent.uploadSpeed)}</span>
					<span>ETA: {formatEta(torrent.eta)}</span>
				</div>
			{:else if isSeeding}
				<div class="mt-2 flex items-center gap-2">
					<progress class="progress flex-1 progress-success" value="100" max="100"></progress>
					<span class="text-sm font-medium">100%</span>
				</div>
				<div class="mt-1 text-xs text-base-content/60">
					UL: {formatSpeed(torrent.uploadSpeed)}
				</div>
			{:else if torrent.state === 'initializing'}
				<div class="mt-2">
					<progress class="progress w-full"></progress>
				</div>
			{/if}

			{#if torrent.outputPath && isSeeding}
				<p class="mt-1 truncate text-xs text-base-content/50" title={torrent.outputPath}>
					{torrent.outputPath}
				</p>
			{/if}
		</div>

		<div class="flex items-center gap-1">
			{#if isStreamable}
				<button
					class="btn text-primary btn-ghost btn-sm"
					on:click={() => dispatch('stream', { infoHash: torrent.infoHash })}
					title="Stream"
					aria-label="Stream torrent"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="currentColor"
						viewBox="0 0 24 24"
					>
						<path d="M8 5v14l11-7z" />
					</svg>
				</button>
			{/if}

			{#if isActive || isSeeding}
				<button
					class="btn btn-ghost btn-sm"
					on:click={() => dispatch('pause', { infoHash: torrent.infoHash })}
					title="Pause"
					aria-label="Pause torrent"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</button>
			{:else if isPaused}
				<button
					class="btn btn-ghost btn-sm"
					on:click={() => dispatch('resume', { infoHash: torrent.infoHash })}
					title="Resume"
					aria-label="Resume torrent"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
						/>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</button>
			{/if}

			<button
				class="btn btn-ghost btn-sm"
				on:click={() => dispatch('remove', { infoHash: torrent.infoHash })}
				title="Remove"
				aria-label="Remove torrent"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-5 w-5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
					/>
				</svg>
			</button>
		</div>
	</div>
</div>

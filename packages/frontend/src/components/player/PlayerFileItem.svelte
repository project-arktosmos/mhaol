<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import type { PlayableFile } from '$types/player.type';
	import { playerAdapter } from '$adapters/classes/player.adapter';

	export let file: PlayableFile;
	export let isPlaying: boolean = false;

	const dispatch = createEventDispatcher<{ play: void }>();
</script>

<button
	class={classNames(
		'flex w-full items-center gap-3 rounded-lg p-3 text-left transition-colors',
		{
			'bg-primary/10 ring-1 ring-primary': isPlaying,
			'bg-base-100 hover:bg-base-300': !isPlaying
		}
	)}
	on:click={() => dispatch('play')}
>
	<div class="flex h-12 w-16 flex-shrink-0 items-center justify-center rounded bg-base-300">
		{#if file.thumbnailUrl}
			<img
				src={file.thumbnailUrl}
				alt={file.name}
				class="h-full w-full rounded object-cover"
			/>
		{:else}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6 opacity-40"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				{#if file.mode === 'audio'}
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
					/>
				{:else}
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
				{/if}
			</svg>
		{/if}
	</div>

	<div class="flex-1 overflow-hidden">
		<p class="truncate text-sm font-medium" title={file.name}>{file.name}</p>
		<div class="flex items-center gap-2">
			<span class={classNames('badge badge-xs', playerAdapter.getSourceBadgeClass(file.type))}>
				{file.type}
			</span>
			<span class="text-xs opacity-60">{playerAdapter.getFormatLabel(file)}</span>
			{#if file.durationSeconds}
				<span class="text-xs opacity-60">
					{playerAdapter.formatDuration(file.durationSeconds)}
				</span>
			{/if}
			<span class="text-xs opacity-60">{playerAdapter.formatSize(file.size)}</span>
		</div>
	</div>

	{#if isPlaying}
		<div class="flex-shrink-0">
			<span class="loading loading-bars loading-xs text-primary"></span>
		</div>
	{/if}
</button>

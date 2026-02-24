<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import type { PlayableFile } from '$types/player.type';
	import PlayerFileItem from './PlayerFileItem.svelte';

	export let files: PlayableFile[] = [];
	export let currentFileId: string | null = null;
	export let loading: boolean = false;
	export let filter: 'all' | 'audio' | 'video' = 'all';

	const dispatch = createEventDispatcher<{
		play: { file: PlayableFile };
		refresh: void;
		filterChange: { filter: 'all' | 'audio' | 'video' };
	}>();

	$: filteredFiles = filter === 'all' ? files : files.filter((f) => f.mode === filter);
</script>

<div class="card bg-base-200">
	<div class="card-body p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-lg">Library</h2>
			<div class="flex items-center gap-2">
				<div class="join">
					<button
						class={classNames('join-item btn btn-xs', {
							'btn-active': filter === 'all'
						})}
						on:click={() => dispatch('filterChange', { filter: 'all' })}
					>
						All
					</button>
					<button
						class={classNames('join-item btn btn-xs', {
							'btn-active': filter === 'video'
						})}
						on:click={() => dispatch('filterChange', { filter: 'video' })}
					>
						Video
					</button>
					<button
						class={classNames('join-item btn btn-xs', {
							'btn-active': filter === 'audio'
						})}
						on:click={() => dispatch('filterChange', { filter: 'audio' })}
					>
						Audio
					</button>
				</div>
				<button
					class="btn btn-ghost btn-xs"
					on:click={() => dispatch('refresh')}
					disabled={loading}
				>
					{#if loading}
						<span class="loading loading-spinner loading-xs"></span>
					{:else}
						Refresh
					{/if}
				</button>
			</div>
		</div>

		{#if filteredFiles.length === 0}
			<div class="rounded-lg bg-base-300 p-6 text-center">
				<p class="opacity-50">No playable files found.</p>
			</div>
		{:else}
			<div class="flex max-h-96 flex-col gap-2 overflow-y-auto">
				{#each filteredFiles as file (file.id)}
					<PlayerFileItem
						{file}
						isPlaying={currentFileId === file.id}
						on:play={() => dispatch('play', { file })}
					/>
				{/each}
			</div>
		{/if}

		<p class="text-right text-xs opacity-50">
			{filteredFiles.length} file{filteredFiles.length !== 1 ? 's' : ''}
		</p>
	</div>
</div>

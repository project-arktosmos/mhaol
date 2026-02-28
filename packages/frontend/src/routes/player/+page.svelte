<script lang="ts">
	import { playerService } from '$services/player.service';
	import type { PlayableFile } from '$types/player.type';
	import PlayerFileList from '$components/player/PlayerFileList.svelte';
	import PlayerStatus from '$components/player/PlayerStatus.svelte';

	const state = playerService.state;

	let filter: 'all' | 'audio' | 'video' = 'all';

	function handlePlay(event: CustomEvent<{ file: PlayableFile }>) {
		playerService.play(event.detail.file);
	}

	function handleRefresh() {
		playerService.refreshFiles();
	}

	function handleFilterChange(event: CustomEvent<{ filter: 'all' | 'audio' | 'video' }>) {
		filter = event.detail.filter;
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Player</h1>
			<p class="text-sm text-base-content/60">Stream completed downloads via WebRTC</p>
		</div>
		<div class="flex items-center gap-4">
			<PlayerStatus
				serverAvailable={$state.streamServerAvailable}
				connectionState={$state.connectionState}
			/>
			{#if !$state.initialized && $state.loading}
				<span class="loading loading-spinner loading-md"></span>
			{/if}
		</div>
	</div>

	{#if $state.error}
		<div class="alert alert-error">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6 shrink-0 stroke-current"
				fill="none"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
				/>
			</svg>
			<span>{$state.error}</span>
			<button
				class="btn btn-sm btn-ghost"
				on:click={() => playerService.state.update((s) => ({ ...s, error: null }))}
			>
				Dismiss
			</button>
		</div>
	{/if}

	<PlayerFileList
		files={$state.files}
		currentFileId={$state.currentFile?.id ?? null}
		loading={$state.loading}
		{filter}
		on:play={handlePlay}
		on:refresh={handleRefresh}
		on:filterChange={handleFilterChange}
	/>
</div>

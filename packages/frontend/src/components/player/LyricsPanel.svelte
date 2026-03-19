<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher, tick } from 'svelte';
	import { lyricsService } from 'frontend/services/lyrics.service';
	import type { PlayableFile } from 'frontend/types/player.type';

	export let currentFile: PlayableFile | null = null;
	export let positionSecs: number = 0;

	const dispatch = createEventDispatcher<{
		seek: { positionSecs: number };
	}>();

	const lyricsState = lyricsService.store;

	let lyricsContainer: HTMLDivElement | null = null;
	let lastFetchedFileId: string | null = null;

	$: lyrics = $lyricsState;

	$: if (currentFile) {
		if (currentFile.id !== lastFetchedFileId) {
			lastFetchedFileId = currentFile.id;
			lyricsService.fetchForFile(currentFile);
		}
	} else if (!currentFile && lastFetchedFileId) {
		lastFetchedFileId = null;
		lyricsService.clear();
	}

	$: currentLineIndex = lyrics.lyrics?.syncedLyrics
		? lyricsService.getCurrentLineIndex(positionSecs)
		: -1;

	$: if (currentLineIndex >= 0 && lyricsContainer) {
		scrollToCurrentLine(currentLineIndex);
	}

	async function scrollToCurrentLine(index: number) {
		await tick();
		if (!lyricsContainer) return;

		const lineElement = lyricsContainer.querySelector(`[data-line-index="${index}"]`);
		if (lineElement) {
			lineElement.scrollIntoView({
				behavior: 'smooth',
				block: 'center'
			});
		}
	}

	function handleLineClick(time: number) {
		dispatch('seek', { positionSecs: time });
	}
</script>

<div class="flex flex-col overflow-hidden rounded-lg bg-base-200">
	<div class="flex items-center justify-between border-b border-base-300 bg-base-200/50 px-3 py-2">
		<h4 class="text-sm font-semibold text-base-content/70">Lyrics</h4>
		{#if lyrics.status === 'success' && lyrics.lyrics?.syncedLyrics}
			<span class="badge badge-xs badge-primary">Synced</span>
		{/if}
	</div>

	<div bind:this={lyricsContainer} class="max-h-64 overflow-y-auto scroll-smooth px-3 py-2">
		{#if lyrics.status === 'loading'}
			<div class="flex flex-col items-center justify-center py-8">
				<span class="loading loading-md loading-spinner text-primary"></span>
				<span class="mt-2 text-sm text-base-content/60">Fetching lyrics...</span>
			</div>
		{:else if lyrics.status === 'not_found'}
			<div class="flex flex-col items-center justify-center py-8 text-base-content/40">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mb-2 h-8 w-8"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
				<span class="text-sm">No lyrics found</span>
			</div>
		{:else if lyrics.status === 'error'}
			<div class="flex flex-col items-center justify-center py-8 text-error/60">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mb-2 h-8 w-8"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
					/>
				</svg>
				<span class="text-sm">Failed to load lyrics</span>
				<span class="mt-1 text-xs">{lyrics.error}</span>
			</div>
		{:else if lyrics.status === 'success' && lyrics.lyrics}
			{#if lyrics.lyrics.instrumental}
				<div class="flex flex-col items-center justify-center py-8 text-base-content/40">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="mb-2 h-8 w-8"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
						/>
					</svg>
					<span class="text-sm">Instrumental</span>
				</div>
			{:else if lyrics.lyrics.syncedLyrics && lyrics.lyrics.syncedLyrics.length > 0}
				<div class="space-y-1 py-4">
					{#each lyrics.lyrics.syncedLyrics as line, index}
						<button
							data-line-index={index}
							class={classNames(
								'w-full cursor-pointer rounded px-2 py-1 text-left text-sm transition-all duration-200',
								{
									'bg-primary font-semibold text-primary-content': index === currentLineIndex,
									'text-base-content/60 hover:bg-base-300/50': index !== currentLineIndex
								}
							)}
							on:click={() => handleLineClick(line.time)}
						>
							{#if line.text}
								{line.text}
							{:else}
								<span class="text-base-content/20">...</span>
							{/if}
						</button>
					{/each}
				</div>
			{:else if lyrics.lyrics.plainLyrics}
				<div class="py-4 text-sm leading-relaxed whitespace-pre-wrap text-base-content/80">
					{lyrics.lyrics.plainLyrics}
				</div>
			{:else}
				<div class="flex flex-col items-center justify-center py-8 text-base-content/40">
					<span class="text-sm">No lyrics available</span>
				</div>
			{/if}
		{/if}
	</div>
</div>

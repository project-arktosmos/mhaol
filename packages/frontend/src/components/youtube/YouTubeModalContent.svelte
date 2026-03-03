<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { youtubeService } from '$services/youtube.service';
	import { libraryService } from '$services/library.service';
	import YouTubeUrlInput from '$components/youtube/YouTubeUrlInput.svelte';
	import YouTubeVideoPreview from '$components/youtube/YouTubeVideoPreview.svelte';
	import YouTubePlaylistPreview from '$components/youtube/YouTubePlaylistPreview.svelte';
	import YouTubeDownloadSettings from '$components/youtube/YouTubeDownloadSettings.svelte';
	import YouTubeDownloadQueue from '$components/youtube/YouTubeDownloadQueue.svelte';

	const state = youtubeService.state;

	onMount(() => {
		Promise.all([youtubeService.initialize(), libraryService.initialize()]);
	});

	onDestroy(() => {
		youtubeService.destroy();
	});

	async function handleDownload() {
		if ($state.currentPlaylistInfo) {
			await youtubeService.downloadPlaylist();
		} else {
			await youtubeService.download();
		}
	}
</script>

<!-- Header -->
<div class="flex items-center justify-between pr-8">
	<div>
		<h3 class="text-lg font-bold">YouTube Download</h3>
		<p class="text-sm text-base-content/60">
			Download audio or video from YouTube videos and playlists
		</p>
	</div>
	{#if !$state.initialized && $state.loading}
		<span class="loading loading-md loading-spinner"></span>
	{/if}
</div>

<!-- Error display -->
{#if $state.error}
	<div class="mt-4 alert alert-error">
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
			class="btn btn-ghost btn-sm"
			onclick={() => youtubeService.state.update((s) => ({ ...s, error: null }))}
		>
			Dismiss
		</button>
	</div>
{/if}

<div class="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-3">
	<!-- Left column: URL input, settings, and download action -->
	<div class="flex flex-col gap-4 lg:col-span-1">
		<YouTubeUrlInput />
		<YouTubeDownloadSettings />

		{#if $state.currentVideoInfo || $state.currentPlaylistInfo}
			<button class="btn w-full btn-primary" onclick={handleDownload}>
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
						d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
					/>
				</svg>
				{#if $state.currentPlaylistInfo}
					Download All ({$state.currentPlaylistInfo.videoCount} videos)
				{:else}
					Download
				{/if}
			</button>
		{/if}
	</div>

	<!-- Right column: Preview and queue -->
	<div class="flex flex-col gap-4 lg:col-span-2">
		{#if $state.currentVideoInfo}
			<YouTubeVideoPreview />
		{:else if $state.currentPlaylistInfo}
			<YouTubePlaylistPreview />
		{/if}
		<YouTubeDownloadQueue />
	</div>
</div>

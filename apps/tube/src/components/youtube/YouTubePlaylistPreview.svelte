<script lang="ts">
	import { youtubeService } from '$services/youtube.service';
	import { formatDuration } from '$types/youtube.type';

	const ytState = youtubeService.state;
	const settings = youtubeService.store;

	let downloadingAll = $state(false);
	let downloadingIds = $state(new Set<string>());

	async function handleDownloadAll() {
		downloadingAll = true;
		try {
			await youtubeService.downloadPlaylist();
		} finally {
			downloadingAll = false;
		}
	}

	async function handleDownloadSingle(videoId: string, title: string) {
		downloadingIds.add(videoId);
		downloadingIds = downloadingIds;

		try {
			const url = `https://www.youtube.com/watch?v=${videoId}`;
			await youtubeService.queueSingleDownload(url, videoId, title);
		} finally {
			downloadingIds.delete(videoId);
			downloadingIds = downloadingIds;
		}
	}

	function handleClear() {
		youtubeService.clearCurrentVideo();
	}

	function getTotalDuration(videos: { duration: number }[]): string {
		const total = videos.reduce((sum, v) => sum + v.duration, 0);
		const hours = Math.floor(total / 3600);
		const minutes = Math.floor((total % 3600) / 60);
		if (hours > 0) {
			return `${hours}h ${minutes}m`;
		}
		return `${minutes}m`;
	}
</script>

{#if $ytState.currentPlaylistInfo}
	<div class="card bg-base-200">
		<div class="card-body gap-4">
			<!-- Header -->
			<div class="flex items-start justify-between gap-4">
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<span class="badge badge-primary">Playlist</span>
						<span class="badge badge-ghost">{$ytState.currentPlaylistInfo.videoCount} videos</span>
						<span class="badge badge-ghost"
							>{getTotalDuration($ytState.currentPlaylistInfo.videos)}</span
						>
					</div>
					<h3 class="mt-2 text-lg font-semibold">
						{$ytState.currentPlaylistInfo.title}
					</h3>
					{#if $ytState.currentPlaylistInfo.author}
						<p class="text-sm text-base-content/60">
							{$ytState.currentPlaylistInfo.author}
						</p>
					{/if}
				</div>

				<div class="flex flex-col gap-2">
					<button class="btn btn-primary" onclick={handleDownloadAll} disabled={downloadingAll}>
						{#if downloadingAll}
							<span class="loading loading-sm loading-spinner"></span>
						{:else}
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
						{/if}
						Download All
					</button>
					<button class="btn btn-ghost btn-sm" onclick={handleClear}>Cancel</button>
				</div>
			</div>

			<!-- Format/Quality info -->
			<div class="flex items-center gap-2 text-sm text-base-content/60">
				<span class="badge badge-outline badge-sm">
					{$settings.downloadMode === 'video' ? 'Video' : 'Audio'}
				</span>
				<span>
					Format: <span class="font-medium"
						>{$settings.downloadMode === 'video'
							? $settings.defaultVideoFormat.toUpperCase()
							: $settings.defaultFormat.toUpperCase()}</span
					>
				</span>
				<span class="text-base-content/30">|</span>
				<span>
					Quality: <span class="font-medium capitalize"
						>{$settings.downloadMode === 'video'
							? $settings.defaultVideoQuality
							: $settings.defaultQuality}</span
					>
				</span>
			</div>

			<!-- Video list -->
			<div class="max-h-96 overflow-y-auto">
				<div class="flex flex-col gap-2">
					{#each $ytState.currentPlaylistInfo.videos as video (video.index)}
						<div class="flex items-center gap-3 rounded-lg bg-base-300 p-2">
							<!-- Index -->
							<span class="w-8 text-center text-sm text-base-content/50">
								{video.index + 1}
							</span>

							<!-- Thumbnail -->
							{#if video.thumbnailUrl}
								<img
									src={video.thumbnailUrl}
									alt={video.title}
									class="h-12 w-20 flex-shrink-0 rounded object-cover"
								/>
							{:else}
								<div
									class="flex h-12 w-20 flex-shrink-0 items-center justify-center rounded bg-base-100"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-6 w-6 text-base-content/30"
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
								</div>
							{/if}

							<!-- Info -->
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm font-medium">{video.title}</p>
								<p class="text-xs text-base-content/50">
									{formatDuration(video.duration)}
								</p>
							</div>

							<!-- Download button -->
							<button
								class="btn btn-square btn-ghost btn-sm"
								onclick={() => handleDownloadSingle(video.videoId, video.title)}
								disabled={downloadingIds.has(video.videoId)}
								title="Download this video"
							>
								{#if downloadingIds.has(video.videoId)}
									<span class="loading loading-xs loading-spinner"></span>
								{:else}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-4 w-4"
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
								{/if}
							</button>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
{/if}

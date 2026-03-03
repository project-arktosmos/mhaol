<script lang="ts">
	import { youtubeService } from '$services/youtube.service';
	import { formatDuration } from '$types/youtube.type';

	const state = youtubeService.state;
	const settings = youtubeService.store;

	async function handleDownload() {
		await youtubeService.download();
	}

	function handleClear() {
		youtubeService.clearCurrentVideo();
	}
</script>

{#if $state.currentVideoInfo}
	<div class="card bg-base-200">
		<div class="card-body">
			<div class="flex gap-4">
				<!-- Thumbnail -->
				{#if $state.currentVideoInfo.thumbnailUrl}
					<div class="flex-shrink-0">
						<img
							src={$state.currentVideoInfo.thumbnailUrl}
							alt={$state.currentVideoInfo.title}
							class="h-24 w-40 rounded-lg object-cover"
						/>
					</div>
				{/if}

				<!-- Info -->
				<div class="flex flex-1 flex-col gap-2">
					<h3 class="line-clamp-2 text-lg font-semibold">
						{$state.currentVideoInfo.title}
					</h3>

					<div class="flex flex-wrap gap-2">
						{#if $state.currentVideoInfo.uploader}
							<span class="badge badge-ghost">{$state.currentVideoInfo.uploader}</span>
						{/if}
						<span class="badge badge-ghost">
							{formatDuration($state.currentVideoInfo.duration)}
						</span>
					</div>

					<div class="mt-2 flex items-center gap-2">
						<span class="badge badge-outline badge-sm">
							{$settings.downloadMode === 'video' ? 'Video' : 'Audio'}
						</span>
						<span class="text-sm text-base-content/60">
							Format: <span class="font-medium"
								>{$settings.downloadMode === 'video'
									? $settings.defaultVideoFormat.toUpperCase()
									: $settings.defaultFormat.toUpperCase()}</span
							>
						</span>
						<span class="text-base-content/30">|</span>
						<span class="text-sm text-base-content/60">
							Quality: <span class="font-medium capitalize"
								>{$settings.downloadMode === 'video'
									? $settings.defaultVideoQuality
									: $settings.defaultQuality}</span
							>
						</span>
					</div>
				</div>

				<!-- Actions -->
				<div class="flex flex-shrink-0 flex-col gap-2">
					<button class="btn btn-primary" on:click={handleDownload}>
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
						Download
					</button>
					<button class="btn btn-ghost btn-sm" on:click={handleClear}> Cancel </button>
				</div>
			</div>
		</div>
	</div>
{/if}

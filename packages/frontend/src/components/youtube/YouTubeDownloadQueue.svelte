<script lang="ts">
	import classNames from 'classnames';
	import { youtubeService } from '$services/youtube.service';
	import { getStateLabel } from '$types/youtube.type';
	import type { YouTubeDownloadProgress } from '$types/youtube.type';

	const state = youtubeService.state;

	function handleCancel(downloadId: string) {
		youtubeService.cancelDownload(downloadId);
	}

	function handleClearCompleted() {
		youtubeService.clearCompleted();
	}

	function getProgressPercent(download: YouTubeDownloadProgress): number {
		return Math.round(download.progress * 100);
	}

	function handleCopyPath(path: string) {
		navigator.clipboard.writeText(path);
	}

	$: hasCompletedOrFailed = $state.downloads.some(
		(d) => d.state === 'completed' || d.state === 'failed' || d.state === 'cancelled'
	);
</script>

<div class="card bg-base-200">
	<div class="card-body">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-lg">Downloads</h2>
			{#if hasCompletedOrFailed}
				<button class="btn btn-ghost btn-sm" on:click={handleClearCompleted}>
					Clear Finished
				</button>
			{/if}
		</div>

		{#if $state.downloads.length === 0}
			<div class="py-8 text-center text-base-content/50">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="mx-auto h-12 w-12 opacity-50"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
					/>
				</svg>
				<p class="mt-2">No downloads yet</p>
				<p class="text-sm">Paste a YouTube URL to get started</p>
			</div>
		{:else}
			<div class="flex flex-col gap-3">
				{#each $state.downloads as download (download.downloadId)}
					<div class="rounded-lg bg-base-100 p-4">
						<div class="flex items-start justify-between gap-4">
							<div class="flex-1 overflow-hidden">
								<h3 class="truncate font-medium" title={download.title}>
									{download.title}
								</h3>
								<div class="mt-1 flex items-center gap-2">
									<span
										class={classNames('badge badge-sm', {
											'badge-neutral': download.state === 'pending',
											'badge-info': download.state === 'fetching' || download.state === 'muxing',
											'badge-primary': download.state === 'downloading',
											'badge-success': download.state === 'completed',
											'badge-error': download.state === 'failed',
											'badge-warning': download.state === 'cancelled'
										})}
									>
										{getStateLabel(download.state)}
									</span>
									{#if download.state === 'downloading' || download.state === 'muxing'}
										<span class="text-sm text-base-content/60">
											{getProgressPercent(download)}%
										</span>
									{/if}
								</div>

								{#if download.state === 'downloading' || download.state === 'muxing'}
									<progress
										class="progress mt-2 w-full progress-primary"
										value={getProgressPercent(download)}
										max="100"
									></progress>
								{/if}

								{#if download.error}
									<p class="mt-1 text-sm text-error">{download.error}</p>
								{/if}

								{#if download.outputPath && download.state === 'completed'}
									<p class="mt-1 truncate text-xs text-base-content/50" title={download.outputPath}>
										{download.outputPath}
									</p>
								{/if}
							</div>

							<div class="flex items-center gap-1">
								{#if download.state === 'completed' && download.outputPath}
									<!-- Copy path button -->
									<button
										class="btn btn-ghost btn-sm"
										on:click={() => download.outputPath && handleCopyPath(download.outputPath)}
										title="Copy file path"
										aria-label="Copy file path"
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
												d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
											/>
										</svg>
									</button>
								{/if}

								{#if download.state === 'downloading' || download.state === 'fetching' || download.state === 'pending' || download.state === 'muxing'}
									<button
										class="btn btn-ghost btn-sm"
										on:click={() => handleCancel(download.downloadId)}
										title="Cancel download"
										aria-label="Cancel download"
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
												d="M6 18L18 6M6 6l12 12"
											/>
										</svg>
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

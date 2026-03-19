<script lang="ts">
	import classNames from 'classnames';
	import { rightPanelService } from '$services/right-panel.service';
	import { libraryService } from '$services/library.service';
	import { youtubeService } from '$services/youtube.service';
	import { mediaModeService } from '$services/media-mode.service';
	import { getStateLabel, getStateColor } from '$types/youtube.type';
	import type { YouTubeChannelMeta } from '$types/youtube.type';
	import { apiUrl } from '$lib/api-base';
	import MediaPlayer from '$components/core/MediaPlayer.svelte';
	import YouTubeChannelCard from '$components/youtube-search/YouTubeChannelCard.svelte';
	import type { YouTubeSearchChannelItem } from '$types/youtube-search.type';

	const panelStore = rightPanelService.store;
	const ytState = youtubeService.state;
	const libState = libraryService.state;
	const mediaModeStore = mediaModeService.store;
	let mediaMode = $derived($mediaModeStore);

	let video = $derived($panelStore.video);
	let liveContent = $derived(
		video ? ($libState.content.find((c) => c.youtubeId === video!.videoId) ?? null) : null
	);

	let videoDownloads = $derived(
		video ? $ytState.downloads.filter((d) => d.videoId === video!.videoId) : []
	);
	let activeDownload = $derived(
		videoDownloads.find((d) =>
			['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		) ??
			videoDownloads.at(-1) ??
			null
	);

	let hasVideo = $derived(liveContent?.hasVideo ?? false);
	let hasAudio = $derived(liveContent?.hasAudio ?? false);

	let videoSrc = $derived(hasVideo ? libraryService.streamVideoUrl(video!.videoId) : null);

	let downloadingAudio = $state(false);
	let downloadingVideo = $state(false);
	let togglingFavorite = $state(false);
	let deletingAudio = $state(false);
	let deletingVideo = $state(false);

	let streamUrl = $state<string | null>(null);
	let streamMimeType = $state<string | null>(null);
	let streamLoading = $state(false);
	let streamError = $state(false);

	$effect(() => {
		const v = video;
		const hasLocalVideo = hasVideo;
		const hasLocalAudio = hasAudio;
		const mode = mediaMode;

		const needsStream =
			v && !((mode === 'video' && hasLocalVideo) || (mode === 'audio' && hasLocalAudio));

		if (needsStream && v) {
			streamLoading = true;
			streamError = false;
			streamUrl = null;

			youtubeService.fetchStreamUrls(v.videoId).then((result) => {
				if (!result) {
					streamError = true;
					streamLoading = false;
					return;
				}
				const best = youtubeService.selectBestMuxedFormat(result);
				if (best) {
					streamUrl = best.url;
					streamMimeType = best.mimeType;
				} else {
					streamError = true;
				}
				streamLoading = false;
			});
		} else {
			streamUrl = null;
			streamLoading = false;
			streamError = false;
		}
	});

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
	}

	let isFavorite = $derived(liveContent?.isFavorite ?? false);

	let channelMeta = $state<YouTubeChannelMeta | null>(null);

	$effect(() => {
		const url = video?.uploaderUrl;
		channelMeta = null;
		if (!url) return;
		const handle = url.split('/').pop();
		if (!handle) return;

		fetch(apiUrl(`/api/youtube/channel-meta?handle=${handle}`))
			.then((res) => (res.ok ? res.json() : null))
			.then((data: YouTubeChannelMeta | null) => {
				channelMeta = data;
			})
			.catch(() => {});
	});

	let channelItem = $derived<YouTubeSearchChannelItem | null>(
		video?.uploaderName
			? {
					type: 'channel',
					channelId: channelMeta?.channelId ?? video.uploaderUrl?.split('/').pop() ?? '',
					name: video.uploaderName,
					thumbnail: channelMeta?.avatar ?? video.uploaderAvatar ?? '',
					url: video.uploaderUrl ?? '',
					subscriberText: channelMeta?.subscriberText ?? '',
					videoCountText: '',
					description: channelMeta?.description ?? '',
					verified: video.uploaderVerified ?? false
				}
			: null
	);

	async function handleToggleFavorite() {
		if (!video || togglingFavorite) return;
		togglingFavorite = true;
		await libraryService.toggleFavorite(video.videoId);
		togglingFavorite = false;
	}

	async function handleDeleteAudio() {
		if (!video) return;
		deletingAudio = true;
		await libraryService.deleteAudio(video.videoId);
		deletingAudio = false;
	}

	async function handleDeleteVideo() {
		if (!video) return;
		deletingVideo = true;
		await libraryService.deleteVideo(video.videoId);
		deletingVideo = false;
	}

	const activeStates = ['pending', 'fetching', 'downloading', 'muxing'];

	let audioInProgress = $derived(
		videoDownloads.some(
			(d) => (d.mode === 'audio' || d.mode === 'both') && activeStates.includes(d.state)
		)
	);
	let videoInProgress = $derived(
		videoDownloads.some(
			(d) => (d.mode === 'video' || d.mode === 'both') && activeStates.includes(d.state)
		)
	);

	let compact = $state(false);

	let wrapperClasses = $derived(
		classNames(
			'flex flex-col bg-base-200 overflow-y-auto overflow-x-hidden',
			'fixed inset-0 top-16 z-30 w-full',
			'md:static md:z-auto md:border-l md:border-base-300',
			compact ? 'md:w-[13.375rem]' : 'md:w-[26.75rem]'
		)
	);

	async function handleDownload(mode: 'audio' | 'video') {
		if (!video) return;

		if (mode === 'audio') downloadingAudio = true;
		else downloadingVideo = true;

		await youtubeService.queueDownloadWithMode(video.videoId, video.title, video.thumbnail, mode);

		if (mode === 'audio') downloadingAudio = false;
		else downloadingVideo = false;
	}
</script>

{#if video}
	<aside class={wrapperClasses}>
		<div
			class={classNames(
				'flex flex-col gap-4 p-4',
				compact ? 'md:min-w-[13.375rem]' : 'md:min-w-[26.75rem]'
			)}
		>
			<div class="flex items-center justify-between">
				<button
					class="btn hidden btn-ghost btn-xs md:flex"
					onclick={() => (compact = !compact)}
					aria-label={compact ? 'Expand panel' : 'Collapse panel'}
				>
					{#if compact}
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
								d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
							/>
						</svg>
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
								d="M13 5l7 7-7 7M5 5l7 7-7 7"
							/>
						</svg>
					{/if}
				</button>
				<button
					class="btn btn-circle btn-ghost btn-xs"
					onclick={() => rightPanelService.close()}
					aria-label="Close panel"
				>
					✕
				</button>
			</div>

			{#key video.videoId}
				{#if mediaMode === 'video' && videoSrc}
					<MediaPlayer source={{ type: 'video', src: videoSrc }} />
				{:else if mediaMode === 'audio' && hasAudio}
					<MediaPlayer
						source={{
							type: 'audio',
							src: libraryService.streamAudioUrl(video.videoId),
							thumbnail: video.thumbnail
						}}
					/>
				{:else if streamLoading}
					<div class="flex aspect-video w-full items-center justify-center rounded-lg bg-base-300">
						<span class="loading loading-md loading-spinner"></span>
					</div>
				{:else if streamUrl}
					<MediaPlayer
						source={{ type: 'video', src: streamUrl, mimeType: streamMimeType ?? 'video/mp4' }}
					/>
				{:else}
					<MediaPlayer source={{ type: 'youtube', videoId: video.videoId, title: video.title }} />
				{/if}
			{/key}

			<div class="flex flex-col gap-1">
				<div class="flex items-start justify-between gap-2">
					<p class="leading-snug font-medium">{video.title}</p>
					<button
						class={classNames(
							'btn btn-circle shrink-0 btn-ghost btn-sm',
							isFavorite ? 'text-error' : 'text-base-content/30'
						)}
						disabled={togglingFavorite}
						onclick={handleToggleFavorite}
						aria-label={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
					>
						{#if togglingFavorite}
							<span class="loading loading-xs loading-spinner"></span>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill={isFavorite ? 'currentColor' : 'none'}
								stroke="currentColor"
								stroke-width="2"
								class="h-5 w-5"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
								/>
							</svg>
						{/if}
					</button>
				</div>
				{#if channelItem}
					<YouTubeChannelCard channel={channelItem} />
				{/if}
				{#if video.viewsText}
					<p class="mt-1 text-sm text-base-content/60">{video.viewsText}</p>
				{/if}
				{#if video.publishedText}
					<p class="text-sm text-base-content/60">{video.publishedText}</p>
				{/if}
			</div>

			{#if hasAudio || hasVideo}
				<div class="divider my-0 text-xs opacity-50">Files</div>
				<div class="flex flex-col gap-2">
					{#if hasAudio}
						<div class="flex items-center justify-between gap-2">
							<div class="flex items-center gap-2">
								<span class="badge badge-xs badge-neutral">Audio</span>
								{#if liveContent?.audioSize}
									<span class="text-xs text-base-content/60"
										>{formatBytes(liveContent.audioSize)}</span
									>
								{/if}
							</div>
							<button
								class="btn text-error btn-ghost btn-xs"
								disabled={deletingAudio}
								onclick={handleDeleteAudio}
								aria-label="Delete audio"
							>
								{#if deletingAudio}
									<span class="loading loading-xs loading-spinner"></span>
								{:else}
									Delete
								{/if}
							</button>
						</div>
					{/if}
					{#if hasVideo}
						<div class="flex items-center justify-between gap-2">
							<div class="flex items-center gap-2">
								<span class="badge badge-xs badge-neutral">Video</span>
								{#if liveContent?.videoSize}
									<span class="text-xs text-base-content/60"
										>{formatBytes(liveContent.videoSize)}</span
									>
								{/if}
							</div>
							<button
								class="btn text-error btn-ghost btn-xs"
								disabled={deletingVideo}
								onclick={handleDeleteVideo}
								aria-label="Delete video"
							>
								{#if deletingVideo}
									<span class="loading loading-xs loading-spinner"></span>
								{:else}
									Delete
								{/if}
							</button>
						</div>
					{/if}
				</div>
			{/if}

			{#if activeDownload && activeStates.includes(activeDownload.state)}
				<div class="divider my-0 text-xs opacity-50">Download Progress</div>
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<span class="text-sm font-medium">{getStateLabel(activeDownload.state)}</span>
						<span class="badge badge-xs badge-{getStateColor(activeDownload.state)}">
							{activeDownload.mode}
						</span>
					</div>
					{#if activeDownload.state === 'downloading' || activeDownload.state === 'muxing'}
						<progress
							class="progress w-full progress-primary"
							value={activeDownload.progress}
							max="1"
						></progress>
						<p class="text-right text-xs text-base-content/50">
							{Math.round(activeDownload.progress * 100)}%
						</p>
					{:else}
						<progress class="progress w-full progress-primary"></progress>
					{/if}
				</div>
			{/if}

			{#if !hasAudio || !hasVideo}
				<div class="divider my-0 text-xs opacity-50">Download</div>
				<div class="grid grid-cols-2 gap-2">
					{#if !hasAudio}
						<button
							class="btn w-full gap-2 btn-sm btn-error"
							disabled={audioInProgress || downloadingAudio}
							onclick={() => handleDownload('audio')}
						>
							{#if downloadingAudio || audioInProgress}
								<span class="loading loading-xs loading-spinner"></span>
							{/if}
							Audio
						</button>
					{/if}
					{#if !hasVideo}
						<button
							class="btn w-full gap-2 btn-sm btn-error"
							disabled={videoInProgress || downloadingVideo}
							onclick={() => handleDownload('video')}
						>
							{#if downloadingVideo || videoInProgress}
								<span class="loading loading-xs loading-spinner"></span>
							{/if}
							Video
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</aside>
{/if}

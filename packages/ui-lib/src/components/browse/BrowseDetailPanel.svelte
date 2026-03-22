<script lang="ts">
	import classNames from 'classnames';
	import { browseDetailService } from 'frontend/services/browse-detail.service';
	import { playerService } from 'frontend/services/player.service';
	import { youtubeService } from 'frontend/services/youtube.service';
	import { youtubeLibraryService } from 'frontend/services/youtube-library.service';
	import { mediaModeService } from 'frontend/services/media-mode.service';
	import { apiUrl } from 'frontend/lib/api-base';
	import { formatDuration } from 'frontend/utils/musicbrainz/transform';
	import { getStateLabel, getStateColor } from 'frontend/types/youtube.type';
	import type { YouTubeChannelMeta } from 'frontend/types/youtube.type';
	import type { DisplayTMDBImage } from 'addons/tmdb/types';
	import type { YouTubeSearchChannelItem } from 'frontend/types/youtube-search.type';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import MediaPlayer from 'ui-lib/components/player/MediaPlayer.svelte';
	import TagPill from 'ui-lib/components/images/TagPill.svelte';
	import YouTubeChannelCard from 'ui-lib/components/youtube-search/YouTubeChannelCard.svelte';

	const browseDetail = browseDetailService.state;
	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	function handleAction(action: string, ...args: unknown[]) {
		const cbs = browseDetailService.getCallbacks();
		const cb = cbs[action as keyof typeof cbs];
		if (typeof cb === 'function') {
			(cb as (...a: unknown[]) => void)(...args);
		}
	}

	function handleClose() {
		handleAction('onclose');
	}

	// ===== Movie/TV derived state =====
	let title = $derived(
		$browseDetail.movie?.title ?? $browseDetail.tvShow?.name ?? ''
	);
	let year = $derived(
		$browseDetail.movie?.releaseYear ?? $browseDetail.tvShow?.firstAirYear ?? ''
	);
	let posterUrl = $derived(
		$browseDetail.movie?.posterUrl ?? $browseDetail.tvShow?.posterUrl ?? null
	);
	let backdropUrl = $derived(
		$browseDetail.movie?.backdropUrl ?? $browseDetail.tvShow?.backdropUrl ?? null
	);
	let voteAverage = $derived(
		$browseDetail.movie?.voteAverage ?? $browseDetail.tvShow?.voteAverage ?? 0
	);
	let voteCount = $derived(
		$browseDetail.movie?.voteCount ?? $browseDetail.tvShow?.voteCount ?? 0
	);
	let overview = $derived(
		$browseDetail.movie?.overview ?? $browseDetail.tvShow?.overview ?? ''
	);
	let genres = $derived(
		$browseDetail.movie?.genres ?? $browseDetail.tvShow?.genres ?? []
	);
	let tagline = $derived(
		$browseDetail.movieDetails?.tagline ?? $browseDetail.tvShowDetails?.tagline ?? null
	);
	let runtime = $derived($browseDetail.movieDetails?.runtime ?? null);
	let director = $derived($browseDetail.movieDetails?.director ?? null);
	let cast = $derived(
		$browseDetail.movieDetails?.cast ?? $browseDetail.tvShowDetails?.cast ?? []
	);
	let createdBy = $derived($browseDetail.tvShowDetails?.createdBy ?? []);
	let networks = $derived($browseDetail.tvShowDetails?.networks ?? []);
	let status = $derived($browseDetail.tvShowDetails?.status ?? null);
	let numberOfSeasons = $derived(
		$browseDetail.tvShow?.numberOfSeasons ?? $browseDetail.tvShowDetails?.numberOfSeasons ?? null
	);
	let numberOfEpisodes = $derived(
		$browseDetail.tvShow?.numberOfEpisodes ?? $browseDetail.tvShowDetails?.numberOfEpisodes ?? null
	);
	let lastAirYear = $derived(
		$browseDetail.tvShow?.lastAirYear ?? $browseDetail.tvShowDetails?.lastAirYear ?? null
	);
	let images: DisplayTMDBImage[] = $derived(
		$browseDetail.movieDetails?.images ?? $browseDetail.tvShowDetails?.images ?? []
	);
	let heroImageUrl = $derived(images.length > 0 ? images[0].thumbnailUrl : backdropUrl);

	let dlState = $derived($browseDetail.downloadStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' || dlState === 'initializing' || dlState === 'paused' || dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'completed' || dlState === 'seeding');
	let downloadButtonDisabled = $derived(!$browseDetail.fetched || isDownloading || isDownloaded);

	// ===== YouTube state =====
	const ytState = youtubeService.state;
	const libState = youtubeLibraryService.state;
	const mediaModeStore = mediaModeService.store;
	let mediaMode = $derived($mediaModeStore);

	let ytVideo = $derived($browseDetail.youtubeVideo);
	let ytLiveContent = $derived(
		ytVideo ? ($libState.content.find((c) => c.youtubeId === ytVideo!.videoId) ?? null) : null
	);
	let ytVideoDownloads = $derived(
		ytVideo ? $ytState.downloads.filter((d) => d.videoId === ytVideo!.videoId) : []
	);
	let ytActiveDownload = $derived(
		ytVideoDownloads.find((d) =>
			['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		) ?? ytVideoDownloads.at(-1) ?? null
	);
	let ytHasVideo = $derived(ytLiveContent?.hasVideo ?? false);
	let ytHasAudio = $derived(ytLiveContent?.hasAudio ?? false);
	let ytVideoSrc = $derived(
		ytHasVideo && ytVideo ? youtubeLibraryService.streamVideoUrl(ytVideo.videoId) : null
	);
	let ytIsFavorite = $derived(ytLiveContent?.isFavorite ?? false);
	let ytDownloadingAudio = $state(false);
	let ytDownloadingVideo = $state(false);
	let ytTogglingFavorite = $state(false);
	let ytDeletingAudio = $state(false);
	let ytDeletingVideo = $state(false);
	let ytStreamUrl = $state<string | null>(null);
	let ytStreamMimeType = $state<string | null>(null);
	let ytStreamLoading = $state(false);
	let ytStreamError = $state(false);
	let ytChannelMeta = $state<YouTubeChannelMeta | null>(null);

	$effect(() => {
		const v = ytVideo;
		const hasLocalVideo = ytHasVideo;
		const hasLocalAudio = ytHasAudio;
		const mode = mediaMode;
		const needsStream = v && !((mode === 'video' && hasLocalVideo) || (mode === 'audio' && hasLocalAudio));
		if (needsStream && v) {
			ytStreamLoading = true;
			ytStreamError = false;
			ytStreamUrl = null;
			youtubeService.fetchStreamUrls(v.videoId).then((result) => {
				if (!result) { ytStreamError = true; ytStreamLoading = false; return; }
				const best = youtubeService.selectBestMuxedFormat(result);
				if (best) { ytStreamUrl = best.url; ytStreamMimeType = best.mimeType; }
				else { ytStreamError = true; }
				ytStreamLoading = false;
			});
		} else {
			ytStreamUrl = null;
			ytStreamLoading = false;
			ytStreamError = false;
		}
	});

	$effect(() => {
		const url = ytVideo?.uploaderUrl;
		ytChannelMeta = null;
		if (!url) return;
		const handle = url.split('/').pop();
		if (!handle) return;
		fetch(apiUrl(`/api/youtube/channel-meta?handle=${handle}`))
			.then((res) => (res.ok ? res.json() : null))
			.then((data: YouTubeChannelMeta | null) => { ytChannelMeta = data; })
			.catch(() => {});
	});

	let ytChannelItem = $derived<YouTubeSearchChannelItem | null>(
		ytVideo?.uploaderName
			? {
					type: 'channel',
					channelId: ytChannelMeta?.channelId ?? ytVideo.uploaderUrl?.split('/').pop() ?? '',
					name: ytVideo.uploaderName,
					thumbnail: ytChannelMeta?.avatar ?? ytVideo.uploaderAvatar ?? '',
					url: ytVideo.uploaderUrl ?? '',
					subscriberText: ytChannelMeta?.subscriberText ?? '',
					videoCountText: '',
					description: ytChannelMeta?.description ?? '',
					verified: ytVideo.uploaderVerified ?? false
				}
			: null
	);

	let ytAudioInProgress = $derived(
		ytVideoDownloads.some(
			(d) => (d.mode === 'audio' || d.mode === 'both') && ['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		)
	);
	let ytVideoInProgress = $derived(
		ytVideoDownloads.some(
			(d) => (d.mode === 'video' || d.mode === 'both') && ['pending', 'fetching', 'downloading', 'muxing'].includes(d.state)
		)
	);

	async function handleYtDownload(mode: 'audio' | 'video') {
		if (!ytVideo) return;
		if (mode === 'audio') ytDownloadingAudio = true;
		else ytDownloadingVideo = true;
		await youtubeService.queueDownloadWithMode(ytVideo.videoId, ytVideo.title, ytVideo.thumbnail, mode);
		if (mode === 'audio') ytDownloadingAudio = false;
		else ytDownloadingVideo = false;
	}

	async function handleYtToggleFavorite() {
		if (!ytVideo || ytTogglingFavorite) return;
		ytTogglingFavorite = true;
		await youtubeLibraryService.toggleFavorite(ytVideo.videoId);
		ytTogglingFavorite = false;
	}

	async function handleYtDeleteAudio() {
		if (!ytVideo) return;
		ytDeletingAudio = true;
		await youtubeLibraryService.deleteAudio(ytVideo.videoId);
		ytDeletingAudio = false;
	}

	async function handleYtDeleteVideo() {
		if (!ytVideo) return;
		ytDeletingVideo = true;
		await youtubeLibraryService.deleteVideo(ytVideo.videoId);
		ytDeletingVideo = false;
	}

	// ===== Photo state =====
	let newPhotoTag = $state('');

	function handleAddTag() {
		if (!newPhotoTag.trim()) return;
		handleAction('onaddtag', newPhotoTag.trim());
		newPhotoTag = '';
	}

	// ===== Helpers =====
	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
	}

	function formatProgress(p: number): string {
		return `${(p * 100).toFixed(1)}%`;
	}

	let hasAnySelection = $derived($browseDetail.domain !== null);
</script>

{#if hasAnySelection || $playerState.currentFile}
	<div class="flex h-full flex-col overflow-y-auto">
		<!-- Header -->
		<div class="flex items-center justify-between p-3">
			{#if $browseDetail.domain === 'music' && $browseDetail.musicAlbum}
				<h2 class="min-w-0 truncate text-sm font-semibold">{$browseDetail.musicAlbum.title}</h2>
			{:else if $browseDetail.domain === 'photo' && $browseDetail.photoImage}
				<h2 class="min-w-0 truncate text-sm font-semibold">{$browseDetail.photoImage.name}</h2>
			{:else if $browseDetail.domain === 'videogame' && $browseDetail.videogame}
				<h2 class="min-w-0 truncate text-sm font-semibold">{$browseDetail.videogame.title}</h2>
			{:else if $browseDetail.domain === 'youtube' && ytVideo}
				<h2 class="min-w-0 truncate text-sm font-semibold">{ytVideo.title}</h2>
			{:else}
				<h2 class="min-w-0 truncate text-sm font-semibold" title={title}>{title}</h2>
			{/if}
			<button class="btn btn-square shrink-0 btn-ghost btn-xs" onclick={handleClose} aria-label="Close">
				&times;
			</button>
		</div>

		<!-- ===== MOVIE / TV ===== -->
		{#if $browseDetail.domain === 'movie' || $browseDetail.domain === 'tv' || ($browseDetail.domain === null && ($browseDetail.movie || $browseDetail.tvShow))}
			{#if $playerState.currentFile}
				<div class={classNames('bg-black', { 'fixed inset-0 z-50 flex flex-col': $playerDisplayMode === 'fullscreen' })}>
					<div class={classNames('flex items-center justify-between px-2 py-1', { 'p-3': $playerDisplayMode === 'fullscreen' })}>
						<p class={classNames('min-w-0 truncate font-semibold text-white', { 'text-xs': $playerDisplayMode !== 'fullscreen', 'text-sm': $playerDisplayMode === 'fullscreen' })} title={$playerState.currentFile.name}>
							{$playerState.currentFile.name}
						</p>
						<div class="flex shrink-0 items-center gap-1">
							{#if $playerDisplayMode === 'fullscreen'}
								<button class="btn btn-square text-white btn-ghost btn-sm" onclick={() => playerService.setDisplayMode('sidebar')} aria-label="Move to sidebar" title="Move to sidebar">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M18 8L14 12L18 16" /><rect x="3" y="3" width="18" height="18" rx="2" /><line x1="14" y1="3" x2="14" y2="21" />
									</svg>
								</button>
							{:else}
								<button class="btn btn-square text-white btn-ghost btn-xs" onclick={() => playerService.setDisplayMode('fullscreen')} aria-label="Fullscreen player" title="Fullscreen player">
									<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
										<path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4" />
									</svg>
								</button>
							{/if}
							<button class={classNames('btn btn-square text-white btn-ghost', { 'btn-sm': $playerDisplayMode === 'fullscreen', 'btn-xs': $playerDisplayMode !== 'fullscreen' })} onclick={() => playerService.stop()} aria-label="Close player">
								&times;
							</button>
						</div>
					</div>
					<div class={$playerDisplayMode === 'fullscreen' ? 'min-h-0 flex-1' : ''}>
						<PlayerVideo
							file={$playerState.currentFile}
							connectionState={$playerState.connectionState}
							positionSecs={$playerState.positionSecs}
							durationSecs={$playerState.durationSecs}
							streamUrl={$playerState.streamUrl}
							buffering={$playerState.buffering}
							fullscreen={$playerDisplayMode === 'fullscreen'}
						/>
					</div>
				</div>
			{:else if heroImageUrl}
				<div class="relative">
					<img src={heroImageUrl} alt={title} class="h-40 w-full object-cover" />
					<div class="absolute inset-0 bg-gradient-to-t from-base-200 to-transparent"></div>
				</div>
			{:else if posterUrl}
				<div class="flex justify-center bg-base-300 p-4">
					<img src={posterUrl} alt={title} class="h-48 rounded-lg object-cover" />
				</div>
			{/if}

			<div class="flex flex-col gap-3 p-3">
				{#if tagline}
					<p class="text-sm italic opacity-60">{tagline}</p>
				{/if}

				<div class="flex flex-wrap items-center gap-2 text-sm">
					<span class="font-medium">{year}{lastAirYear ? `\u2013${lastAirYear}` : ''}</span>
					{#if voteAverage > 0}
						<span class="flex items-center gap-1">
							<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="h-4 w-4 text-yellow-500">
								<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
							</svg>
							<span class="font-semibold">{voteAverage.toFixed(1)}</span>
							<span class="text-xs opacity-50">({voteCount})</span>
						</span>
					{/if}
					{#if $browseDetail.tvShow}
						<span class="badge badge-sm badge-info">TV</span>
					{:else}
						<span class="badge badge-sm badge-primary">Movie</span>
					{/if}
				</div>

				{#if runtime}
					<p class="text-xs opacity-60">{runtime}</p>
				{/if}

				{#if numberOfSeasons != null}
					<p class="text-xs opacity-60">
						{numberOfSeasons} season{numberOfSeasons !== 1 ? 's' : ''}{numberOfEpisodes != null ? `, ${numberOfEpisodes} episodes` : ''}
						{#if status}&middot; {status}{/if}
					</p>
				{/if}

				{#if genres.length > 0}
					<div class="flex flex-wrap gap-1">
						{#each genres as genre}
							<span class="badge badge-outline badge-sm">{genre}</span>
						{/each}
					</div>
				{/if}

				<div class="grid grid-cols-2 gap-2">
					<button class="btn col-span-2 btn-sm {$browseDetail.fetched ? 'btn-ghost' : 'btn-info'}" onclick={() => handleAction('onfetch')} disabled={$browseDetail.fetching}>
						{#if $browseDetail.fetching}
							<span class="loading loading-xs loading-spinner"></span>
						{:else}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
							</svg>
						{/if}
						Smart Search
					</button>
					{#if $browseDetail.fetchSteps}
						<button class="col-span-2 cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300" onclick={() => handleAction('onshowsearch')}>
							<ul class="steps steps-horizontal w-full text-xs">
								<li class={classNames('step', { 'step-success': $browseDetail.fetchSteps.terms })}>Terms</li>
								<li class={classNames('step', { 'step-success': $browseDetail.fetchSteps.search })}>{$browseDetail.fetchSteps.searching ? 'Searching...' : 'Search'}</li>
								<li class={classNames('step', { 'step-success': $browseDetail.fetchSteps.eval })}>Analysis</li>
								<li class={classNames('step', { 'step-success': $browseDetail.fetchSteps.done })}>{$browseDetail.fetchSteps.done ? 'Done' : 'Candidate'}</li>
							</ul>
						</button>
					{/if}
					<button class={classNames('btn btn-sm', { 'btn-ghost': isDownloaded, 'btn-success': !isDownloaded })} onclick={() => handleAction('ondownload')} disabled={downloadButtonDisabled}>
						{#if isDownloading}
							<span class="loading loading-xs loading-spinner"></span> Downloading
						{:else if isDownloaded}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg> Downloaded
						{:else}
							<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg> Download
						{/if}
					</button>
					<button class="btn btn-sm btn-primary" onclick={() => handleAction('onstream')} disabled={!$browseDetail.fetched}>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
						Torrent
					</button>
					<button class="btn btn-sm btn-secondary" onclick={() => handleAction('onp2pstream')} disabled={!$browseDetail.fetched}>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.14 0M1.394 9.393c5.857-5.858 15.355-5.858 21.213 0" /></svg>
						P2P
					</button>
				</div>

				{#if overview}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Overview</h3>
						<p class="text-sm leading-relaxed opacity-80">{overview}</p>
					</div>
				{/if}

				{#if $browseDetail.loading}
					<div class="flex justify-center py-4"><span class="loading loading-sm loading-spinner"></span></div>
				{/if}

				{#if director}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Director</h3>
						<p class="text-sm">{director}</p>
					</div>
				{/if}

				{#if createdBy.length > 0}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Created by</h3>
						<p class="text-sm">{createdBy.join(', ')}</p>
					</div>
				{/if}

				{#if networks.length > 0}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Networks</h3>
						<p class="text-sm">{networks.join(', ')}</p>
					</div>
				{/if}

				{#if cast.length > 0}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Cast</h3>
						<div class="flex flex-col gap-1">
							{#each cast.slice(0, 8) as member}
								<div class="flex items-center gap-2">
									{#if member.profileUrl}
										<img src={member.profileUrl} alt={member.name} class="h-8 w-8 rounded-full object-cover" loading="lazy" />
									{:else}
										<div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs opacity-40">?</div>
									{/if}
									<div class="min-w-0">
										<p class="truncate text-sm font-medium">{member.name}</p>
										<p class="truncate text-xs opacity-50">{member.character}</p>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if backdropUrl || posterUrl}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Primary Images</h3>
						<div class="flex flex-col gap-2">
							{#if backdropUrl}
								<div>
									<p class="mb-0.5 text-xs font-mono opacity-40">backdrop_path</p>
									<img src={backdropUrl} alt="{title} backdrop" class="aspect-video w-full rounded object-cover" loading="lazy" />
								</div>
							{/if}
							{#if posterUrl}
								<div>
									<p class="mb-0.5 text-xs font-mono opacity-40">poster_path</p>
									<img src={posterUrl} alt="{title} poster" class="aspect-[2/3] w-32 rounded object-cover" loading="lazy" />
								</div>
							{/if}
						</div>
					</div>
				{/if}

				{#if images.length > 0}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">All Images</h3>
						<div class="grid grid-cols-3 gap-1">
							{#each images as image}
								<a href={image.fullUrl} target="_blank" rel="noopener noreferrer">
									<img src={image.thumbnailUrl} alt="{title} image" class={classNames('w-full rounded object-cover transition-opacity hover:opacity-80', { 'aspect-video': image.width > image.height, 'aspect-[2/3]': image.width <= image.height })} loading="lazy" />
								</a>
							{/each}
						</div>
					</div>
				{/if}

				{#if $browseDetail.libraryItem}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Library Item</h3>
						<table class="table table-xs">
							<tbody>
								<tr><td class="font-medium opacity-60">ID</td><td class="break-all">{$browseDetail.libraryItem.id}</td></tr>
								<tr><td class="font-medium opacity-60">Name</td><td class="break-all">{$browseDetail.libraryItem.name}</td></tr>
								<tr><td class="font-medium opacity-60">Path</td><td class="break-all">{$browseDetail.libraryItem.path}</td></tr>
								<tr><td class="font-medium opacity-60">Extension</td><td>{$browseDetail.libraryItem.extension}</td></tr>
								<tr><td class="font-medium opacity-60">Media Type</td><td>{$browseDetail.libraryItem.mediaTypeId}</td></tr>
								<tr><td class="font-medium opacity-60">Category</td><td>{$browseDetail.libraryItem.categoryId ?? '\u2014'}</td></tr>
								<tr><td class="font-medium opacity-60">Created</td><td>{$browseDetail.libraryItem.createdAt}</td></tr>
								{#each Object.entries($browseDetail.libraryItem.links) as [service, link]}
									<tr><td class="font-medium opacity-60">Link: {service}</td><td class="break-all">{link.serviceId}</td></tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}

				{#if $browseDetail.relatedData?.library}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Library</h3>
						<table class="table table-xs">
							<tbody>
								<tr><td class="font-medium opacity-60">Name</td><td class="break-all">{$browseDetail.relatedData.library.name}</td></tr>
								<tr><td class="font-medium opacity-60">Path</td><td class="break-all">{$browseDetail.relatedData.library.path}</td></tr>
								<tr><td class="font-medium opacity-60">Media Types</td><td>{$browseDetail.relatedData.library.mediaTypes}</td></tr>
								<tr><td class="font-medium opacity-60">Created</td><td>{$browseDetail.relatedData.library.createdAt}</td></tr>
							</tbody>
						</table>
					</div>
				{/if}

				{#if $browseDetail.relatedData?.links && $browseDetail.relatedData.links.length > 0}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Item Links</h3>
						<table class="table table-xs">
							<tbody>
								{#each $browseDetail.relatedData.links as link}
									<tr><td class="font-medium opacity-60">{link.service}</td><td class="break-all">{link.serviceId}</td></tr>
									{#if link.seasonNumber != null}
										<tr><td class="pl-4 font-medium opacity-60">Season</td><td>{link.seasonNumber}</td></tr>
									{/if}
									{#if link.episodeNumber != null}
										<tr><td class="pl-4 font-medium opacity-60">Episode</td><td>{link.episodeNumber}</td></tr>
									{/if}
									<tr><td class="pl-4 font-medium opacity-60">Linked</td><td>{link.createdAt}</td></tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}

				{#if $browseDetail.relatedData?.fetchCache}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Fetch Cache</h3>
						<table class="table table-xs">
							<tbody>
								<tr><td class="font-medium opacity-60">TMDB ID</td><td>{$browseDetail.relatedData.fetchCache.tmdbId}</td></tr>
								<tr><td class="font-medium opacity-60">Media Type</td><td>{$browseDetail.relatedData.fetchCache.mediaType}</td></tr>
								<tr><td class="font-medium opacity-60">Created</td><td>{$browseDetail.relatedData.fetchCache.createdAt}</td></tr>
								{#if $browseDetail.relatedData.fetchCache.candidate.name}
									<tr><td class="font-medium opacity-60">Torrent</td><td class="break-all">{$browseDetail.relatedData.fetchCache.candidate.name}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.fetchCache.candidate.infoHash}
									<tr><td class="font-medium opacity-60">Info Hash</td><td class="break-all font-mono text-[10px]">{$browseDetail.relatedData.fetchCache.candidate.infoHash}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.fetchCache.candidate.size}
									<tr><td class="font-medium opacity-60">Size</td><td>{formatBytes(Number($browseDetail.relatedData.fetchCache.candidate.size))}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.fetchCache.candidate.seeds != null}
									<tr><td class="font-medium opacity-60">Seeds</td><td>{$browseDetail.relatedData.fetchCache.candidate.seeds}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.fetchCache.candidate.peers != null}
									<tr><td class="font-medium opacity-60">Peers</td><td>{$browseDetail.relatedData.fetchCache.candidate.peers}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.fetchCache.candidate.magnetUrl}
									<tr><td class="font-medium opacity-60">Magnet</td><td class="break-all font-mono text-[10px]">{$browseDetail.relatedData.fetchCache.candidate.magnetUrl}</td></tr>
								{/if}
							</tbody>
						</table>
					</div>
				{/if}

				{#if $browseDetail.relatedData?.torrentDownload}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Torrent Download</h3>
						<table class="table table-xs">
							<tbody>
								<tr><td class="font-medium opacity-60">Name</td><td class="break-all">{$browseDetail.relatedData.torrentDownload.name}</td></tr>
								<tr><td class="font-medium opacity-60">Info Hash</td><td class="break-all font-mono text-[10px]">{$browseDetail.relatedData.torrentDownload.infoHash}</td></tr>
								<tr><td class="font-medium opacity-60">State</td><td>{$browseDetail.relatedData.torrentDownload.state}</td></tr>
								<tr><td class="font-medium opacity-60">Progress</td><td>{formatProgress($browseDetail.relatedData.torrentDownload.progress)}</td></tr>
								<tr><td class="font-medium opacity-60">Size</td><td>{formatBytes($browseDetail.relatedData.torrentDownload.size)}</td></tr>
								<tr><td class="font-medium opacity-60">DL Speed</td><td>{formatBytes($browseDetail.relatedData.torrentDownload.downloadSpeed)}/s</td></tr>
								<tr><td class="font-medium opacity-60">UL Speed</td><td>{formatBytes($browseDetail.relatedData.torrentDownload.uploadSpeed)}/s</td></tr>
								<tr><td class="font-medium opacity-60">Peers</td><td>{$browseDetail.relatedData.torrentDownload.peers}</td></tr>
								<tr><td class="font-medium opacity-60">Seeds</td><td>{$browseDetail.relatedData.torrentDownload.seeds}</td></tr>
								<tr><td class="font-medium opacity-60">Source</td><td>{$browseDetail.relatedData.torrentDownload.source}</td></tr>
								{#if $browseDetail.relatedData.torrentDownload.outputPath}
									<tr><td class="font-medium opacity-60">Output</td><td class="break-all">{$browseDetail.relatedData.torrentDownload.outputPath}</td></tr>
								{/if}
								<tr><td class="font-medium opacity-60">Added</td><td>{new Date($browseDetail.relatedData.torrentDownload.addedAt * 1000).toLocaleString()}</td></tr>
								<tr><td class="font-medium opacity-60">Created</td><td>{$browseDetail.relatedData.torrentDownload.createdAt}</td></tr>
								<tr><td class="font-medium opacity-60">Updated</td><td>{$browseDetail.relatedData.torrentDownload.updatedAt}</td></tr>
							</tbody>
						</table>
					</div>
				{/if}

				{#if $browseDetail.relatedData?.tmdbCache}
					<div>
						<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">TMDB Cache</h3>
						<table class="table table-xs">
							<tbody>
								<tr><td class="font-medium opacity-60">TMDB ID</td><td>{$browseDetail.relatedData.tmdbCache.tmdbId}</td></tr>
								<tr><td class="font-medium opacity-60">Fetched</td><td>{$browseDetail.relatedData.tmdbCache.fetchedAt}</td></tr>
								<tr><td class="font-medium opacity-60">Title</td><td class="break-all">{$browseDetail.relatedData.tmdbCache.data.title ?? $browseDetail.relatedData.tmdbCache.data.name ?? '\u2014'}</td></tr>
								{#if $browseDetail.relatedData.tmdbCache.data.release_date}
									<tr><td class="font-medium opacity-60">Release</td><td>{$browseDetail.relatedData.tmdbCache.data.release_date}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.tmdbCache.data.runtime}
									<tr><td class="font-medium opacity-60">Runtime</td><td>{$browseDetail.relatedData.tmdbCache.data.runtime} min</td></tr>
								{/if}
								{#if $browseDetail.relatedData.tmdbCache.data.status}
									<tr><td class="font-medium opacity-60">Status</td><td>{$browseDetail.relatedData.tmdbCache.data.status}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.tmdbCache.data.original_language}
									<tr><td class="font-medium opacity-60">Language</td><td>{$browseDetail.relatedData.tmdbCache.data.original_language}</td></tr>
								{/if}
								{#if $browseDetail.relatedData.tmdbCache.data.imdb_id}
									<tr><td class="font-medium opacity-60">IMDB ID</td><td>{$browseDetail.relatedData.tmdbCache.data.imdb_id}</td></tr>
								{/if}
							</tbody>
						</table>
					</div>
				{/if}
			</div>

		<!-- ===== MUSIC ===== -->
		{:else if $browseDetail.domain === 'music' && $browseDetail.musicAlbum}
			{@const album = $browseDetail.musicAlbum}
			{@const release = $browseDetail.musicRelease}
			{@const torrent = $browseDetail.musicTorrent}
			<div class="flex flex-col gap-3 p-3">
				{#if album.coverArtUrl}
					<img src={album.coverArtUrl} alt={album.title} class="aspect-square w-full rounded-lg object-cover" />
				{:else}
					<div class="flex aspect-square w-full items-center justify-center rounded-lg bg-base-200">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 text-base-content/20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
						</svg>
					</div>
				{/if}

				<p class="text-xs opacity-60">{album.artistCredits}</p>
				{#if album.firstReleaseYear && album.firstReleaseYear !== 'Unknown'}
					<p class="text-xs opacity-40">{album.firstReleaseYear}</p>
				{/if}

				<div class="flex flex-wrap gap-1">
					{#if album.primaryType}
						<span class="badge badge-ghost badge-sm">{album.primaryType}</span>
					{/if}
					{#each album.secondaryTypes as type_}
						<span class="badge badge-ghost badge-sm">{type_}</span>
					{/each}
				</div>

				{#if torrent}
					<div class="rounded-lg bg-base-200 p-2">
						<div class="flex items-center justify-between text-xs">
							<span class="font-medium">
								{torrent.state === 'seeding' ? 'Complete' : `${Math.round(torrent.progress * 100)}%`}
							</span>
							<span class="badge badge-xs {torrent.state === 'seeding' ? 'badge-success' : 'badge-info'}">
								{torrent.state}
							</span>
						</div>
						{#if torrent.state !== 'seeding'}
							<progress class="progress progress-primary mt-1 w-full" value={Math.round(torrent.progress * 100)} max="100"></progress>
						{/if}
					</div>
				{/if}

				<button class="btn btn-primary btn-sm" onclick={() => handleAction('ondownloadalbum')} disabled={torrent?.state === 'downloading' || torrent?.state === 'seeding'}>
					{torrent?.state === 'seeding' ? 'Downloaded' : torrent ? 'Downloading...' : 'Download'}
				</button>

				{#if $browseDetail.loading}
					<div class="flex items-center justify-center py-4"><span class="loading loading-sm loading-spinner"></span></div>
				{:else if release && release.tracks.length > 0}
					<div class="flex flex-col gap-0.5">
						<div class="flex items-center justify-between">
							<h4 class="text-xs font-semibold opacity-50">Tracklist</h4>
							<span class="text-xs opacity-30">{release.tracks.length} tracks</span>
						</div>
						{#each release.tracks as track (track.id)}
							<div class="flex items-center gap-2 rounded px-1 py-0.5 hover:bg-base-200">
								<span class="w-5 text-right text-xs opacity-30">{track.number}</span>
								<span class="min-w-0 flex-1 truncate text-xs">{track.title}</span>
								{#if track.duration}
									<span class="text-xs opacity-30">{track.duration}</span>
								{/if}
							</div>
						{/each}
					</div>
				{:else if release}
					<p class="text-xs opacity-30">No tracks available</p>
				{/if}
			</div>

		<!-- ===== PHOTO ===== -->
		{:else if $browseDetail.domain === 'photo' && $browseDetail.photoImage}
			{@const img = $browseDetail.photoImage}
			<div class="flex flex-col gap-3 p-3">
				<!-- Tag browser -->
				{#if $browseDetail.photoTags.length > 0}
					<div>
						<h3 class="mb-2 text-sm font-semibold text-base-content/70">Tags</h3>
						<div class="flex flex-wrap gap-1">
							{#each $browseDetail.photoTags as tag}
								<span class="badge badge-ghost badge-sm">{tag}</span>
							{/each}
						</div>
					</div>
					<div class="divider my-0"></div>
				{/if}

				<img
					src={apiUrl(`/api/images/serve?path=${encodeURIComponent(img.path)}`)}
					alt={img.name}
					class="w-full rounded-lg object-cover"
				/>

				<div class="flex flex-wrap gap-1">
					{#each img.tags as tag (tag.tag)}
						<TagPill tag={tag.tag} score={tag.score} onremove={() => handleAction('onremovetag', tag.tag)} />
					{/each}
				</div>

				<div class="flex gap-1">
					<input
						type="text"
						placeholder="Add tag..."
						class="input input-xs input-bordered flex-1"
						bind:value={newPhotoTag}
						onkeydown={(e) => e.key === 'Enter' && handleAddTag()}
					/>
					<button class="btn btn-ghost btn-xs" onclick={handleAddTag}>+</button>
				</div>

				<button
					class="btn btn-sm btn-outline"
					disabled={$browseDetail.photoTagging}
					onclick={() => handleAction('onautotag')}
				>
					{#if $browseDetail.photoTagging}
						<span class="loading loading-xs loading-spinner"></span> Tagging...
					{:else}
						Auto-tag
					{/if}
				</button>
			</div>

		<!-- ===== VIDEOGAME ===== -->
		{:else if $browseDetail.domain === 'videogame' && $browseDetail.videogame}
			{@const game = $browseDetail.videogame}
			{@const details = $browseDetail.videogameDetails}
			<div class="flex flex-col gap-3 p-3">
				{#if game.imageIconUrl}
					<img src={game.imageIconUrl} alt={game.title} class="aspect-square w-full rounded-lg object-cover" />
				{:else}
					<div class="flex aspect-square w-full items-center justify-center rounded-lg bg-base-200">
						<svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 text-base-content/20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 01-.657.643 48.39 48.39 0 01-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 01-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 00-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 01-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 00.657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 01-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.4.604-.4.959v0c0 .333.277.599.61.58a48.1 48.1 0 005.427-.63 48.05 48.05 0 00.582-4.717.532.532 0 00-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.959.401v0a.656.656 0 00.658-.663 48.422 48.422 0 00-.37-5.36c-1.886.342-3.81.574-5.766.689a.578.578 0 01-.61-.58v0z" />
						</svg>
					</div>
				{/if}

				<p class="text-xs opacity-60">{game.consoleName}</p>

				{#if $browseDetail.videogameDetailsLoading}
					<div class="flex items-center justify-center py-4"><span class="loading loading-sm loading-spinner"></span></div>
				{:else if details}
					<div class="flex flex-col gap-1.5">
						{#if details.developer}
							<div class="flex items-center gap-1 text-xs"><span class="opacity-40">Developer:</span><span>{details.developer}</span></div>
						{/if}
						{#if details.publisher}
							<div class="flex items-center gap-1 text-xs"><span class="opacity-40">Publisher:</span><span>{details.publisher}</span></div>
						{/if}
						{#if details.genre}
							<div class="flex items-center gap-1 text-xs"><span class="opacity-40">Genre:</span><span>{details.genre}</span></div>
						{/if}
						{#if details.released}
							<div class="flex items-center gap-1 text-xs"><span class="opacity-40">Released:</span><span>{details.released}</span></div>
						{/if}
						{#if details.numDistinctPlayers}
							<div class="flex items-center gap-1 text-xs"><span class="opacity-40">Players:</span><span>{details.numDistinctPlayers.toLocaleString()}</span></div>
						{/if}

						<div class="flex flex-wrap gap-1 pt-1">
							{#if details.numAchievements > 0}
								<span class="badge badge-info badge-sm">{details.numAchievements} achievements</span>
							{/if}
							{#if details.points > 0}
								<span class="badge badge-ghost badge-sm">{details.points} points</span>
							{/if}
						</div>

						{#if details.imageBoxArtUrl}
							<img src={details.imageBoxArtUrl} alt="Box art" class="mt-2 w-full rounded-lg" loading="lazy" />
						{/if}
						{#if details.imageIngameUrl}
							<img src={details.imageIngameUrl} alt="In-game screenshot" class="w-full rounded-lg" loading="lazy" />
						{/if}
						{#if details.imageTitleUrl}
							<img src={details.imageTitleUrl} alt="Title screen" class="w-full rounded-lg" loading="lazy" />
						{/if}
					</div>

					{#if details.achievements && details.achievements.length > 0}
						<div class="flex flex-col gap-0.5 pt-2">
							<div class="flex items-center justify-between">
								<h4 class="text-xs font-semibold opacity-50">Achievements</h4>
								<span class="text-xs opacity-30">{details.achievements.length} total</span>
							</div>
							{#each details.achievements as achievement (achievement.id)}
								<div class="flex items-center gap-2 rounded px-1 py-1 hover:bg-base-200">
									{#if achievement.badgeUrl}
										<img src={achievement.badgeUrl} alt={achievement.title} class="h-8 w-8 rounded" loading="lazy" />
									{/if}
									<div class="min-w-0 flex-1">
										<p class="truncate text-xs font-medium">{achievement.title}</p>
										<p class="truncate text-xs opacity-40">{achievement.description}</p>
									</div>
									<span class="text-xs opacity-30">{achievement.points}pts</span>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</div>

		<!-- ===== YOUTUBE ===== -->
		{:else if $browseDetail.domain === 'youtube' && ytVideo}
			<div class="flex flex-col gap-4 p-4">
				{#key ytVideo.videoId}
					{#if mediaMode === 'video' && ytVideoSrc}
						<MediaPlayer source={{ type: 'video', src: ytVideoSrc }} />
					{:else if mediaMode === 'audio' && ytHasAudio}
						<MediaPlayer source={{ type: 'audio', src: youtubeLibraryService.streamAudioUrl(ytVideo.videoId), thumbnail: ytVideo.thumbnail }} />
					{:else if ytStreamLoading}
						<div class="flex aspect-video w-full items-center justify-center rounded-lg bg-base-300">
							<span class="loading loading-md loading-spinner"></span>
						</div>
					{:else if ytStreamUrl}
						<MediaPlayer source={{ type: 'video', src: ytStreamUrl, mimeType: ytStreamMimeType ?? 'video/mp4' }} />
					{:else}
						<MediaPlayer source={{ type: 'youtube', videoId: ytVideo.videoId, title: ytVideo.title }} />
					{/if}
				{/key}

				<div class="flex flex-col gap-1">
					<div class="flex items-start justify-between gap-2">
						<p class="leading-snug font-medium">{ytVideo.title}</p>
						<button
							class={classNames('btn btn-circle shrink-0 btn-ghost btn-sm', ytIsFavorite ? 'text-error' : 'text-base-content/30')}
							disabled={ytTogglingFavorite}
							onclick={handleYtToggleFavorite}
							aria-label={ytIsFavorite ? 'Remove from favorites' : 'Add to favorites'}
						>
							{#if ytTogglingFavorite}
								<span class="loading loading-xs loading-spinner"></span>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill={ytIsFavorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2" class="h-5 w-5">
									<path stroke-linecap="round" stroke-linejoin="round" d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z" />
								</svg>
							{/if}
						</button>
					</div>
					{#if ytChannelItem}
						<YouTubeChannelCard channel={ytChannelItem} />
					{/if}
					{#if ytVideo.viewsText}
						<p class="mt-1 text-sm text-base-content/60">{ytVideo.viewsText}</p>
					{/if}
					{#if ytVideo.publishedText}
						<p class="text-sm text-base-content/60">{ytVideo.publishedText}</p>
					{/if}
				</div>

				{#if ytHasAudio || ytHasVideo}
					<div class="divider my-0 text-xs opacity-50">Files</div>
					<div class="flex flex-col gap-2">
						{#if ytHasAudio}
							<div class="flex items-center justify-between gap-2">
								<div class="flex items-center gap-2">
									<span class="badge badge-xs badge-neutral">Audio</span>
									{#if ytLiveContent?.audioSize}
										<span class="text-xs text-base-content/60">{formatBytes(ytLiveContent.audioSize)}</span>
									{/if}
								</div>
								<button class="btn text-error btn-ghost btn-xs" disabled={ytDeletingAudio} onclick={handleYtDeleteAudio} aria-label="Delete audio">
									{#if ytDeletingAudio}<span class="loading loading-xs loading-spinner"></span>{:else}Delete{/if}
								</button>
							</div>
						{/if}
						{#if ytHasVideo}
							<div class="flex items-center justify-between gap-2">
								<div class="flex items-center gap-2">
									<span class="badge badge-xs badge-neutral">Video</span>
									{#if ytLiveContent?.videoSize}
										<span class="text-xs text-base-content/60">{formatBytes(ytLiveContent.videoSize)}</span>
									{/if}
								</div>
								<button class="btn text-error btn-ghost btn-xs" disabled={ytDeletingVideo} onclick={handleYtDeleteVideo} aria-label="Delete video">
									{#if ytDeletingVideo}<span class="loading loading-xs loading-spinner"></span>{:else}Delete{/if}
								</button>
							</div>
						{/if}
					</div>
				{/if}

				{#if ytActiveDownload && ['pending', 'fetching', 'downloading', 'muxing'].includes(ytActiveDownload.state)}
					<div class="divider my-0 text-xs opacity-50">Download Progress</div>
					<div class="flex flex-col gap-2">
						<div class="flex items-center justify-between">
							<span class="text-sm font-medium">{getStateLabel(ytActiveDownload.state)}</span>
							<span class="badge badge-xs badge-{getStateColor(ytActiveDownload.state)}">{ytActiveDownload.mode}</span>
						</div>
						{#if ytActiveDownload.state === 'downloading' || ytActiveDownload.state === 'muxing'}
							<progress class="progress w-full progress-primary" value={ytActiveDownload.progress} max="1"></progress>
							<p class="text-right text-xs text-base-content/50">{Math.round(ytActiveDownload.progress * 100)}%</p>
						{:else}
							<progress class="progress w-full progress-primary"></progress>
						{/if}
					</div>
				{/if}

				{#if !ytHasAudio || !ytHasVideo}
					<div class="divider my-0 text-xs opacity-50">Download</div>
					<div class="grid grid-cols-2 gap-2">
						{#if !ytHasAudio}
							<button class="btn w-full gap-2 btn-sm btn-error" disabled={ytAudioInProgress || ytDownloadingAudio} onclick={() => handleYtDownload('audio')}>
								{#if ytDownloadingAudio || ytAudioInProgress}<span class="loading loading-xs loading-spinner"></span>{/if}
								Audio
							</button>
						{/if}
						{#if !ytHasVideo}
							<button class="btn w-full gap-2 btn-sm btn-error" disabled={ytVideoInProgress || ytDownloadingVideo} onclick={() => handleYtDownload('video')}>
								{#if ytDownloadingVideo || ytVideoInProgress}<span class="loading loading-xs loading-spinner"></span>{/if}
								Video
							</button>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}

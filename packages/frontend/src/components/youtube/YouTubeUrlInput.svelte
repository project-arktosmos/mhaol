<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { youtubeService } from '$services/youtube.service';
	import { extractVideoId, extractPlaylistId } from '$types/youtube.type';

	export let initialUrl: string = '';

	const state = youtubeService.state;

	let urlInput = initialUrl;

	// Auto-fetch when initialUrl is provided
	onMount(() => {
		if (initialUrl && $state.initialized) {
			handleFetchInfo();
		}
	});

	// Also watch for when initialized becomes true with an initial URL
	$: if (
		initialUrl &&
		$state.initialized &&
		urlInput === initialUrl &&
		!$state.currentVideoInfo &&
		!$state.fetchingInfo
	) {
		handleFetchInfo();
	}

	// YouTube URL validation regex (supports videos and playlists)
	const youtubeRegex =
		/^(https?:\/\/)?(www\.)?(youtube\.com\/(watch\?v=|embed\/|v\/|playlist\?list=)|youtu\.be\/)[\w-]+/;

	$: isValidUrl =
		youtubeRegex.test(urlInput) ||
		(urlInput.includes('youtube.com') && urlInput.includes('list='));
	$: videoId = extractVideoId(urlInput);
	$: playlistId = extractPlaylistId(urlInput);
	$: hasVideoId = videoId !== null;
	$: hasPlaylistId = playlistId !== null;
	$: isPurePlaylist = !hasVideoId && hasPlaylistId;
	$: canFetch = isValidUrl && !$state.fetchingInfo && $state.initialized;
	$: canFetchPlaylist =
		isValidUrl && hasPlaylistId && !$state.fetchingPlaylistInfo && $state.initialized;

	async function handleFetchInfo() {
		if (!canFetch) return;

		if (isPurePlaylist) {
			await youtubeService.fetchPlaylistInfo(urlInput);
		} else {
			await youtubeService.fetchVideoInfo(urlInput);
		}
	}

	async function handleFetchAsPlaylist() {
		if (!canFetch || !hasPlaylistId) return;
		await youtubeService.fetchPlaylistInfo(urlInput);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && canFetch) {
			handleFetchInfo();
		}
	}

	function handlePaste() {
		setTimeout(() => {
			if (isValidUrl && $state.initialized) {
				handleFetchInfo();
			}
		}, 100);
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">YouTube URL</h2>

		<div class="form-control">
			<div class="join w-full">
				<input
					type="text"
					bind:value={urlInput}
					on:keydown={handleKeydown}
					on:paste={handlePaste}
					placeholder="https://youtube.com/watch?v=... or playlist link"
					class={classNames('input join-item input-bordered flex-1', {
						'input-error': urlInput && !isValidUrl,
						'input-success': isValidUrl
					})}
					disabled={!$state.initialized}
				/>
				<button
					class="btn btn-primary join-item"
					on:click={handleFetchInfo}
					disabled={!canFetch}
				>
					{#if $state.fetchingInfo}
						<span class="loading loading-spinner loading-sm"></span>
					{:else}
						Fetch
					{/if}
				</button>
			</div>
			{#if urlInput && !isValidUrl}
				<span class="label">
					<span class="label-text-alt text-error">Please enter a valid YouTube URL</span>
				</span>
			{:else if hasVideoId && hasPlaylistId}
				<div class="mt-2 flex items-center justify-between">
					<span class="text-xs text-info">This video is part of a playlist</span>
					<button
						class="btn btn-xs btn-ghost text-info"
						on:click={handleFetchAsPlaylist}
						disabled={!canFetchPlaylist}
					>
						{#if $state.fetchingPlaylistInfo}
							<span class="loading loading-spinner loading-xs"></span>
						{:else}
							Fetch full playlist
						{/if}
					</button>
				</div>
			{:else if isPurePlaylist}
				<span class="label">
					<span class="label-text-alt text-info"
						>Playlist URL - will fetch all videos</span
					>
				</span>
			{/if}
		</div>

		<!-- YouTube Embed Preview -->
		{#if hasVideoId && videoId}
			<div class="aspect-video w-full overflow-hidden rounded-lg">
				<iframe
					src="https://www.youtube.com/embed/{videoId}"
					title="YouTube video preview"
					class="h-full w-full"
					frameborder="0"
					allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
					allowfullscreen
				></iframe>
			</div>
		{/if}

		<p class="text-xs text-base-content/50">
			Paste a YouTube video or playlist URL to fetch info and download.
		</p>
	</div>
</div>

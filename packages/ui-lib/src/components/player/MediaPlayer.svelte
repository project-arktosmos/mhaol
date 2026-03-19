<script lang="ts">
	type MediaSource =
		| { type: 'video'; src: string; mimeType?: string }
		| { type: 'audio'; src: string; mimeType?: string; thumbnail?: string }
		| { type: 'youtube'; videoId: string; title?: string };

	let {
		source,
		autoplay = true
	}: {
		source: MediaSource;
		autoplay?: boolean;
	} = $props();

	let mediaEl = $state<HTMLVideoElement | HTMLAudioElement | null>(null);
	let duration = $state(0);
	let currentTime = $state(0);
	let paused = $state(true);
	let volume = $state(1);
	let muted = $state(false);
	let seeking = $state(false);
	let fullscreen = $state(false);

	let isMediaSource = $derived(source.type === 'video' || source.type === 'audio');
	let progressPercent = $derived(duration > 0 ? (currentTime / duration) * 100 : 0);

	function formatTime(seconds: number): string {
		if (!isFinite(seconds) || seconds < 0) return '0:00';
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = Math.floor(seconds % 60);
		if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
		return `${m}:${String(s).padStart(2, '0')}`;
	}

	function handleTimeUpdate() {
		if (mediaEl && !seeking) {
			currentTime = mediaEl.currentTime;
		}
	}

	function handleLoadedMetadata() {
		if (mediaEl) {
			duration = mediaEl.duration;
			paused = mediaEl.paused;
			volume = mediaEl.volume;
			muted = mediaEl.muted;
		}
	}

	function handlePlay() {
		paused = false;
	}

	function handlePause() {
		paused = true;
	}

	function handleVolumeChange() {
		if (mediaEl) {
			volume = mediaEl.volume;
			muted = mediaEl.muted;
		}
	}

	function togglePlayPause() {
		if (!mediaEl) return;
		if (mediaEl.paused) {
			mediaEl.play().catch(() => {});
		} else {
			mediaEl.pause();
		}
	}

	function handleSeek(e: MouseEvent) {
		if (!mediaEl || !duration) return;
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		mediaEl.currentTime = ratio * duration;
		currentTime = mediaEl.currentTime;
	}

	function handleSeekStart(e: MouseEvent) {
		seeking = true;
		handleSeek(e);

		function onMove(ev: MouseEvent) {
			handleSeek(ev);
		}
		function onUp() {
			seeking = false;
			document.removeEventListener('mousemove', onMove);
			document.removeEventListener('mouseup', onUp);
		}
		document.addEventListener('mousemove', onMove);
		document.addEventListener('mouseup', onUp);
	}

	function handleVolumeSeek(e: MouseEvent) {
		if (!mediaEl) return;
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		mediaEl.volume = ratio;
		mediaEl.muted = ratio === 0;
	}

	function handleVolumeKeydown(e: KeyboardEvent) {
		if (!mediaEl) return;
		if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
			mediaEl.volume = Math.min(1, mediaEl.volume + 0.05);
		} else if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
			mediaEl.volume = Math.max(0, mediaEl.volume - 0.05);
		}
	}

	function toggleMute() {
		if (!mediaEl) return;
		mediaEl.muted = !mediaEl.muted;
	}

	interface WebKitVideoElement extends HTMLVideoElement {
		webkitEnterFullscreen?: () => void;
		webkitExitFullscreen?: () => void;
		webkitDisplayingFullscreen?: boolean;
	}

	function toggleFullscreen() {
		if (!mediaEl || !(mediaEl instanceof HTMLVideoElement)) return;
		const vid = mediaEl as WebKitVideoElement;

		if (vid.webkitDisplayingFullscreen) {
			vid.webkitExitFullscreen?.();
		} else if (vid.webkitEnterFullscreen) {
			vid.webkitEnterFullscreen();
		} else if (vid.requestFullscreen) {
			vid.requestFullscreen().catch(() => {});
		}
	}

	function handleFullscreenChange() {
		if (!mediaEl || !(mediaEl instanceof HTMLVideoElement)) {
			fullscreen = false;
			return;
		}
		const vid = mediaEl as WebKitVideoElement;
		fullscreen =
			vid.webkitDisplayingFullscreen ?? (document.fullscreenElement === mediaEl || false);
	}

	$effect(() => {
		document.addEventListener('fullscreenchange', handleFullscreenChange);
		document.addEventListener('webkitfullscreenchange', handleFullscreenChange);
		const vid = mediaEl;
		if (vid) {
			vid.addEventListener('webkitbeginfullscreen', handleFullscreenChange);
			vid.addEventListener('webkitendfullscreen', handleFullscreenChange);
		}
		return () => {
			document.removeEventListener('fullscreenchange', handleFullscreenChange);
			document.removeEventListener('webkitfullscreenchange', handleFullscreenChange);
			if (vid) {
				vid.removeEventListener('webkitbeginfullscreen', handleFullscreenChange);
				vid.removeEventListener('webkitendfullscreen', handleFullscreenChange);
			}
		};
	});

	$effect(() => {
		if (mediaEl && autoplay) {
			mediaEl.play().catch(() => {});
		}
	});

	function skip(seconds: number) {
		if (!mediaEl) return;
		mediaEl.currentTime = Math.max(0, Math.min(duration, mediaEl.currentTime + seconds));
	}

	let volumeIcon = $derived(muted || volume === 0 ? 'muted' : volume < 0.5 ? 'low' : 'high');
</script>

{#if source.type === 'youtube'}
	<div data-media-player>
		<iframe
			src="https://www.youtube.com/embed/{source.videoId}?autoplay={autoplay ? 1 : 0}"
			title={source.title ?? ''}
			class="aspect-video w-full rounded-lg"
			frameborder="0"
			allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
			allowfullscreen
		></iframe>
	</div>
{:else}
	<div data-media-player class="flex flex-col gap-0 overflow-hidden rounded-lg bg-black">
		{#if source.type === 'video'}
			<!-- svelte-ignore a11y_media_has_caption -->
			<video
				bind:this={mediaEl}
				class="w-full cursor-pointer"
				src={source.src}
				ontimeupdate={handleTimeUpdate}
				onloadedmetadata={handleLoadedMetadata}
				onplay={handlePlay}
				onpause={handlePause}
				onvolumechange={handleVolumeChange}
				onclick={togglePlayPause}
			>
				<source src={source.src} type={source.mimeType ?? 'video/mp4'} />
			</video>
		{:else}
			{#if source.thumbnail}
				<img src={source.thumbnail} alt="" class="w-full rounded-t-lg object-cover" />
			{/if}
			<audio
				bind:this={mediaEl}
				src={source.src}
				ontimeupdate={handleTimeUpdate}
				onloadedmetadata={handleLoadedMetadata}
				onplay={handlePlay}
				onpause={handlePause}
				onvolumechange={handleVolumeChange}
			>
				<source src={source.src} type={source.mimeType ?? 'audio/x-m4a'} />
			</audio>
		{/if}

		<div class="flex flex-col gap-1 bg-base-300 px-3 py-2">
			<div
				class="group relative h-1.5 w-full cursor-pointer rounded-full bg-base-content/20 transition-all hover:h-2.5"
				onmousedown={handleSeekStart}
				role="slider"
				aria-valuenow={currentTime}
				aria-valuemin={0}
				aria-valuemax={duration}
				tabindex={0}
			>
				<div
					class="pointer-events-none absolute inset-y-0 left-0 rounded-full bg-primary transition-all"
					style:width="{progressPercent}%"
				></div>
				<div
					class="pointer-events-none absolute top-1/2 h-3 w-3 -translate-y-1/2 rounded-full bg-primary opacity-0 shadow transition-opacity group-hover:opacity-100"
					style:left="{progressPercent}%"
				></div>
			</div>

			<div class="flex items-center gap-2">
				<button
					class="btn btn-circle text-base-content btn-ghost btn-xs"
					onclick={() => skip(-10)}
					aria-label="Rewind 10 seconds"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						class="h-3.5 w-3.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3"
						/>
					</svg>
				</button>

				<button
					class="btn btn-circle text-base-content btn-ghost btn-sm"
					onclick={togglePlayPause}
					aria-label={paused ? 'Play' : 'Pause'}
				>
					{#if paused}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="currentColor"
							class="h-5 w-5"
						>
							<path
								fill-rule="evenodd"
								d="M4.5 5.653c0-1.427 1.529-2.33 2.779-1.643l11.54 6.347c1.295.712 1.295 2.573 0 3.286L7.28 19.99c-1.25.687-2.779-.217-2.779-1.643V5.653Z"
								clip-rule="evenodd"
							/>
						</svg>
					{:else}
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="currentColor"
							class="h-5 w-5"
						>
							<path
								fill-rule="evenodd"
								d="M6.75 5.25a.75.75 0 0 1 .75-.75H9a.75.75 0 0 1 .75.75v13.5a.75.75 0 0 1-.75.75H7.5a.75.75 0 0 1-.75-.75V5.25Zm7.5 0A.75.75 0 0 1 15 4.5h1.5a.75.75 0 0 1 .75.75v13.5a.75.75 0 0 1-.75.75H15a.75.75 0 0 1-.75-.75V5.25Z"
								clip-rule="evenodd"
							/>
						</svg>
					{/if}
				</button>

				<button
					class="btn btn-circle text-base-content btn-ghost btn-xs"
					onclick={() => skip(10)}
					aria-label="Forward 10 seconds"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						class="h-3.5 w-3.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="m15 15 6-6m0 0-6-6m6 6H9a6 6 0 0 0 0 12h3"
						/>
					</svg>
				</button>

				<span class="min-w-0 flex-shrink-0 text-xs text-base-content/70 tabular-nums">
					{formatTime(currentTime)} / {formatTime(duration)}
				</span>

				<div class="flex-1"></div>

				<div class="flex items-center gap-1">
					<button
						class="btn btn-circle text-base-content btn-ghost btn-xs"
						onclick={toggleMute}
						aria-label={muted ? 'Unmute' : 'Mute'}
					>
						{#if volumeIcon === 'muted'}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="currentColor"
								class="h-3.5 w-3.5"
							>
								<path
									d="M13.5 4.06c0-1.336-1.616-2.005-2.56-1.06l-4.5 4.5H4.508c-1.141 0-2.318.664-2.66 1.905A9.76 9.76 0 0 0 1.5 12c0 .898.121 1.768.35 2.595.341 1.24 1.518 1.905 2.659 1.905h1.93l4.5 4.5c.945.945 2.561.276 2.561-1.06V4.06ZM17.78 9.22a.75.75 0 1 0-1.06 1.06L18.44 12l-1.72 1.72a.75.75 0 1 0 1.06 1.06l1.72-1.72 1.72 1.72a.75.75 0 1 0 1.06-1.06L20.56 12l1.72-1.72a.75.75 0 1 0-1.06-1.06l-1.72 1.72-1.72-1.72Z"
								/>
							</svg>
						{:else if volumeIcon === 'low'}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="currentColor"
								class="h-3.5 w-3.5"
							>
								<path
									d="M13.5 4.06c0-1.336-1.616-2.005-2.56-1.06l-4.5 4.5H4.508c-1.141 0-2.318.664-2.66 1.905A9.76 9.76 0 0 0 1.5 12c0 .898.121 1.768.35 2.595.341 1.24 1.518 1.905 2.659 1.905h1.93l4.5 4.5c.945.945 2.561.276 2.561-1.06V4.06ZM18.584 5.106a.75.75 0 0 1 1.06 0c3.808 3.807 3.808 9.98 0 13.788a.75.75 0 0 1-1.06-1.06 8.25 8.25 0 0 0 0-11.668.75.75 0 0 1 0-1.06Z"
								/>
							</svg>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="currentColor"
								class="h-3.5 w-3.5"
							>
								<path
									d="M13.5 4.06c0-1.336-1.616-2.005-2.56-1.06l-4.5 4.5H4.508c-1.141 0-2.318.664-2.66 1.905A9.76 9.76 0 0 0 1.5 12c0 .898.121 1.768.35 2.595.341 1.24 1.518 1.905 2.659 1.905h1.93l4.5 4.5c.945.945 2.561.276 2.561-1.06V4.06ZM18.584 5.106a.75.75 0 0 1 1.06 0c3.808 3.807 3.808 9.98 0 13.788a.75.75 0 0 1-1.06-1.06 8.25 8.25 0 0 0 0-11.668.75.75 0 0 1 0-1.06Z"
								/>
								<path
									d="M15.932 7.757a.75.75 0 0 1 1.061 0 6 6 0 0 1 0 8.486.75.75 0 0 1-1.06-1.061 4.5 4.5 0 0 0 0-6.364.75.75 0 0 1 0-1.06Z"
								/>
							</svg>
						{/if}
					</button>
					<div
						class="h-1 w-14 cursor-pointer rounded-full bg-base-content/20"
						onclick={handleVolumeSeek}
						onkeydown={handleVolumeKeydown}
						role="slider"
						aria-valuenow={Math.round(volume * 100)}
						aria-valuemin={0}
						aria-valuemax={100}
						aria-label="Volume"
						tabindex={0}
					>
						<div
							class="pointer-events-none h-full rounded-full bg-base-content/60"
							style:width="{muted ? 0 : volume * 100}%"
						></div>
					</div>
				</div>

				{#if source.type === 'video'}
					<button
						class="btn btn-circle text-base-content btn-ghost btn-xs"
						onclick={toggleFullscreen}
						aria-label={fullscreen ? 'Exit fullscreen' : 'Fullscreen'}
					>
						{#if fullscreen}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								class="h-3.5 w-3.5"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M9 9V4.5M9 9H4.5M9 9 3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5 5.25 5.25"
								/>
							</svg>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								class="h-3.5 w-3.5"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15"
								/>
							</svg>
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}

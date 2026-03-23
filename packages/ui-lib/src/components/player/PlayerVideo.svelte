<script lang="ts">
	import { onDestroy, tick } from 'svelte';
	import { playerService } from 'ui-lib/services/player.service';
	import type {
		PlayableFile,
		PlayableFileSubtitle,
		PlayerConnectionState
	} from 'ui-lib/types/player.type';
	import PlayerControls from './PlayerControls.svelte';

	export let file: PlayableFile | null = null;
	export let connectionState: PlayerConnectionState = 'idle';
	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;
	export let buffering: boolean = false;
	export let fullscreen: boolean = false;

	let videoElement: HTMLVideoElement | null = null;
	let audioElement: HTMLAudioElement | null = null;
	let containerElement: HTMLElement | null = null;
	let streamAttached = false;

	$: isVideo = file?.mode !== 'audio';
	$: isStreaming = connectionState === 'streaming';
	$: activeMediaElement = (isVideo ? videoElement : audioElement) as HTMLMediaElement | null;

	$: if (isStreaming && !streamAttached) {
		tryAttachStream();
	}

	$: if (connectionState === 'idle') {
		streamAttached = false;
	}

	async function tryAttachStream(): Promise<void> {
		// Wait for the DOM to settle after branch switches ({#if isVideo})
		for (let attempt = 0; attempt < 10; attempt++) {
			await tick();
			const stream = playerService.getMediaStream();
			if (!stream) return;

			const element = file?.mode === 'audio' ? audioElement : videoElement;
			if (element) {
				element.srcObject = stream;
				element.play().catch((err: Error) => {
					console.error('[Player] play() failed:', err);
					if (err.name === 'NotAllowedError') {
						playerService.state.update((s) => ({
							...s,
							error: 'Playback blocked by browser. Click play to start.'
						}));
					}
				});
				streamAttached = true;
				return;
			}

			// Element not bound yet — wait a frame and retry
			await new Promise((r) => requestAnimationFrame(r));
		}
	}

	function handleStop(): void {
		playerService.stop();
		streamAttached = false;
	}

	function handleSeek(event: CustomEvent<{ positionSecs: number }>): void {
		playerService.seek(event.detail.positionSecs);
	}

	function handleSeekStart(): void {
		playerService.setSeeking(true);
	}

	function handleVideoClick(): void {
		if (!activeMediaElement || !isStreaming) return;
		if (activeMediaElement.paused) {
			activeMediaElement.play().catch(console.error);
		} else {
			activeMediaElement.pause();
		}
	}

	function handleWaiting(): void {
		playerService.setBuffering(true);
	}

	function handlePlaying(): void {
		playerService.setBuffering(false);
		playerService.setPaused(false);
	}

	function getStatusLabel(state: PlayerConnectionState): string {
		switch (state) {
			case 'idle':
				return '';
			case 'waiting-for-stream':
				return 'Finding stream...';
			case 'connecting':
				return 'Connecting to stream server...';
			case 'signaling':
				return 'Negotiating WebRTC connection...';
			case 'streaming':
				return '';
			case 'error':
				return 'Connection failed';
			case 'closed':
				return 'Stream ended';
		}
	}

	onDestroy(() => {
		streamAttached = false;
	});

	$: subtitles = file?.subtitles ?? [];
	$: statusLabel = getStatusLabel(connectionState);
</script>

<div class={fullscreen ? 'flex h-full flex-col' : ''}>
	<div class={fullscreen ? 'relative min-h-0 flex-1' : 'relative'} bind:this={containerElement}>
		{#if isVideo}
			<video
				bind:this={videoElement}
				class={fullscreen
					? 'h-full w-full cursor-pointer bg-black object-contain'
					: 'w-full cursor-pointer rounded-lg bg-black'}
				playsinline
				on:click={handleVideoClick}
				on:waiting={handleWaiting}
				on:playing={handlePlaying}
			>
				{#each subtitles as sub}
					<track
						kind="subtitles"
						src={sub.url}
						srclang={sub.languageCode}
						label={sub.languageName}
					/>
				{/each}
			</video>
			{#if buffering}
				<div class="absolute inset-0 flex items-center justify-center rounded-lg bg-black/40">
					<span class="loading loading-lg loading-spinner text-primary"></span>
				</div>
			{/if}
		{:else}
			<div class="flex h-20 items-center justify-center rounded-lg bg-base-300">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-10 w-10 opacity-30"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
					/>
				</svg>
			</div>
			<audio bind:this={audioElement} class="absolute h-0 w-0 overflow-hidden"></audio>
		{/if}

		{#if !isStreaming && connectionState !== 'idle'}
			<div class="absolute inset-0 flex items-center justify-center rounded-lg bg-base-300/80">
				{#if connectionState === 'waiting-for-stream' || connectionState === 'connecting' || connectionState === 'signaling'}
					<div class="text-center">
						<span class="loading loading-sm loading-spinner"></span>
						<p class="mt-1 text-xs">{statusLabel}</p>
					</div>
				{:else if connectionState === 'error'}
					<div class="text-center text-error">
						<p class="text-xs font-medium">Connection failed</p>
						<button class="btn mt-1 btn-xs btn-error" on:click={handleStop}> Close </button>
					</div>
				{:else if connectionState === 'closed'}
					<div class="text-center">
						<p class="text-xs opacity-70">Stream ended</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	{#if isStreaming}
		<div class="mt-1">
			<PlayerControls
				mediaElement={activeMediaElement}
				{isVideo}
				{positionSecs}
				{durationSecs}
				{connectionState}
				{containerElement}
				on:seek={handleSeek}
				on:seekstart={handleSeekStart}
				on:seekend
				on:stop={handleStop}
			/>
		</div>
	{/if}
</div>

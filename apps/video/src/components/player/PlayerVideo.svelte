<script lang="ts">
	import { onDestroy, tick } from 'svelte';
	import { playerService } from '$services/player.service';
	import type { PlayableFile, PlayerConnectionState } from '$types/player.type';
	import PlayerControls from './PlayerControls.svelte';

	export let file: PlayableFile | null = null;
	export let connectionState: PlayerConnectionState = 'idle';
	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;

	let videoElement: HTMLVideoElement | null = null;
	let audioElement: HTMLAudioElement | null = null;
	let containerElement: HTMLElement | null = null;
	let streamAttached = false;

	$: isVideo = file?.mode !== 'audio';
	$: activeMediaElement = (isVideo ? videoElement : audioElement) as HTMLMediaElement | null;

	$: if (connectionState === 'streaming' && !streamAttached) {
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
		if (!activeMediaElement || connectionState !== 'streaming') return;
		if (activeMediaElement.paused) {
			activeMediaElement.play().catch(console.error);
		} else {
			activeMediaElement.pause();
		}
	}

	function getStatusLabel(state: PlayerConnectionState): string {
		switch (state) {
			case 'idle':
				return '';
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

	$: statusLabel = getStatusLabel(connectionState);
</script>

<div>
	<div class="relative" bind:this={containerElement}>
		{#if isVideo}
			<video
				bind:this={videoElement}
				class="w-full cursor-pointer rounded-lg bg-black"
				playsinline
				on:click={handleVideoClick}
			>
				<track kind="captions" />
			</video>
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

		{#if connectionState !== 'streaming' && connectionState !== 'idle'}
			<div class="absolute inset-0 flex items-center justify-center rounded-lg bg-base-300/80">
				{#if connectionState === 'connecting' || connectionState === 'signaling'}
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

	{#if connectionState === 'streaming'}
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

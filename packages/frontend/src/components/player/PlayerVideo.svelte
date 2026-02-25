<script lang="ts">
	import { onDestroy } from 'svelte';
	import { playerService } from '$services/player.service';
	import type { PlayableFile, PlayerConnectionState } from '$types/player.type';
	import PlayerSeekBar from './PlayerSeekBar.svelte';

	export let file: PlayableFile | null = null;
	export let connectionState: PlayerConnectionState = 'idle';
	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;

	let videoElement: HTMLVideoElement | null = null;
	let audioElement: HTMLAudioElement | null = null;
	let streamAttached = false;

	$: if (connectionState === 'streaming' && !streamAttached) {
		attachStream();
	}

	$: if (connectionState === 'idle') {
		streamAttached = false;
	}

	function attachStream(): void {
		const stream = playerService.getMediaStream();
		if (!stream) return;

		if (file?.mode === 'audio' && audioElement) {
			audioElement.srcObject = stream;
			audioElement.play().catch(console.error);
			streamAttached = true;
		} else if (videoElement) {
			videoElement.srcObject = stream;
			videoElement.play().catch(console.error);
			streamAttached = true;
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

	function handleSeekEnd(): void {
		// isSeeking is cleared by playerService.seek() via a timeout
		// to absorb any in-flight position updates from before the seek
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

	$: isVideo = file?.mode !== 'audio';
	$: statusLabel = getStatusLabel(connectionState);
</script>

<div class="card bg-base-200">
	<div class="card-body p-4">
		{#if !file}
			<div class="flex h-64 items-center justify-center rounded-lg bg-base-300">
				<p class="opacity-50">Select a file to play</p>
			</div>
		{:else}
			<div class="relative">
				{#if isVideo}
					<video bind:this={videoElement} class="w-full rounded-lg bg-black" controls playsinline>
						<track kind="captions" />
					</video>
				{:else}
					<div class="flex h-32 items-center justify-center rounded-lg bg-base-300">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-16 w-16 opacity-30"
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
					<audio bind:this={audioElement} class="mt-2 w-full" controls></audio>
				{/if}

				{#if connectionState !== 'streaming' && connectionState !== 'idle'}
					<div
						class="absolute inset-0 flex items-center justify-center rounded-lg bg-base-300/80"
					>
						{#if connectionState === 'connecting' || connectionState === 'signaling'}
							<div class="text-center">
								<span class="loading loading-spinner loading-lg"></span>
								<p class="mt-2 text-sm">{statusLabel}</p>
							</div>
						{:else if connectionState === 'error'}
							<div class="text-center text-error">
								<p class="font-medium">Connection failed</p>
								<button class="btn btn-sm btn-error mt-2" on:click={handleStop}>
									Close
								</button>
							</div>
						{:else if connectionState === 'closed'}
							<div class="text-center">
								<p class="opacity-70">Stream ended</p>
							</div>
						{/if}
					</div>
				{/if}
			</div>

			{#if connectionState === 'streaming'}
				<div class="mt-2">
					<PlayerSeekBar
						{positionSecs}
						{durationSecs}
						disabled={connectionState !== 'streaming'}
						on:seek={handleSeek}
						on:seekstart={handleSeekStart}
						on:seekend={handleSeekEnd}
					/>
				</div>
			{/if}

			<div class="mt-2 flex items-center justify-between">
				<div class="overflow-hidden">
					<p class="truncate font-medium">{file.name}</p>
				</div>
				{#if connectionState === 'streaming'}
					<button class="btn btn-ghost btn-sm" on:click={handleStop}> Stop </button>
				{/if}
			</div>
		{/if}
	</div>
</div>

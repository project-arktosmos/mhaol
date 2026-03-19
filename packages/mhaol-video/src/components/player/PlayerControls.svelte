<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher, onDestroy } from 'svelte';
	import { playerService } from '$services/player.service';
	import type { PlayerConnectionState } from '$types/player.type';
	import PlayerSeekBar from './PlayerSeekBar.svelte';

	export let mediaElement: HTMLMediaElement | null = null;
	export let isVideo: boolean = false;
	export let positionSecs: number = 0;
	export let durationSecs: number | null = null;
	export let connectionState: PlayerConnectionState = 'idle';
	export let containerElement: HTMLElement | null = null;

	const dispatch = createEventDispatcher<{
		seek: { positionSecs: number };
		seekstart: void;
		seekend: void;
		stop: void;
	}>();

	let isPaused = true;
	let volume = playerService.getVolume();
	let isMuted = false;
	let isFullscreen = false;
	let volumeBeforeMute = volume;

	function onPlay(): void {
		isPaused = false;
		playerService.setPaused(false);
	}

	function onPause(): void {
		isPaused = true;
		playerService.setPaused(true);
	}

	function onVolumeChange(): void {
		if (!mediaElement) return;
		volume = mediaElement.volume;
		isMuted = mediaElement.muted;
	}

	function onFullscreenChange(): void {
		isFullscreen = !!document.fullscreenElement;
	}

	let currentElement: HTMLMediaElement | null = null;

	$: if (mediaElement !== currentElement) {
		if (currentElement) {
			currentElement.removeEventListener('play', onPlay);
			currentElement.removeEventListener('pause', onPause);
			currentElement.removeEventListener('volumechange', onVolumeChange);
		}
		currentElement = mediaElement;
		if (mediaElement) {
			mediaElement.addEventListener('play', onPlay);
			mediaElement.addEventListener('pause', onPause);
			mediaElement.addEventListener('volumechange', onVolumeChange);
			mediaElement.volume = volume;
			isPaused = mediaElement.paused;
			isMuted = mediaElement.muted;
		}
	}

	function togglePlayPause(): void {
		if (!mediaElement) return;
		if (mediaElement.paused) {
			mediaElement.play().catch(console.error);
		} else {
			mediaElement.pause();
		}
	}

	function toggleMute(): void {
		if (!mediaElement) return;
		if (isMuted || volume === 0) {
			mediaElement.muted = false;
			if (volumeBeforeMute === 0) volumeBeforeMute = 0.5;
			mediaElement.volume = volumeBeforeMute;
			volume = volumeBeforeMute;
			isMuted = false;
		} else {
			volumeBeforeMute = volume;
			mediaElement.muted = true;
			isMuted = true;
		}
		playerService.setVolume(mediaElement.muted ? 0 : mediaElement.volume);
	}

	function handleVolumeInput(event: Event): void {
		if (!mediaElement) return;
		const target = event.target as HTMLInputElement;
		const newVolume = parseFloat(target.value);
		mediaElement.volume = newVolume;
		mediaElement.muted = newVolume === 0;
		volume = newVolume;
		isMuted = newVolume === 0;
		playerService.setVolume(newVolume);
	}

	function toggleFullscreen(): void {
		if (!containerElement) return;
		if (document.fullscreenElement) {
			document.exitFullscreen().catch(console.error);
		} else {
			containerElement.requestFullscreen().catch(console.error);
		}
	}

	$: disabled = connectionState !== 'streaming';

	onDestroy(() => {
		if (currentElement) {
			currentElement.removeEventListener('play', onPlay);
			currentElement.removeEventListener('pause', onPause);
			currentElement.removeEventListener('volumechange', onVolumeChange);
		}
		document.removeEventListener('fullscreenchange', onFullscreenChange);
	});

	if (typeof document !== 'undefined') {
		document.addEventListener('fullscreenchange', onFullscreenChange);
	}

	$: volumeDisplay = isMuted || volume === 0 ? 'muted' : volume < 0.5 ? 'low' : 'high';
</script>

<div class={classNames('flex flex-col gap-1', { 'pointer-events-none opacity-50': disabled })}>
	<PlayerSeekBar {positionSecs} {durationSecs} {disabled} on:seek on:seekstart on:seekend />

	<div class="flex items-center gap-0.5">
		<!-- Play/Pause -->
		<button
			class="btn btn-square btn-ghost btn-xs"
			on:click={togglePlayPause}
			aria-label={isPaused ? 'Play' : 'Pause'}
		>
			{#if isPaused}
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
						d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
					/>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
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
						d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
			{/if}
		</button>

		<!-- Volume -->
		<button
			class="btn btn-square btn-ghost btn-xs"
			on:click={toggleMute}
			aria-label={isMuted ? 'Unmute' : 'Mute'}
		>
			{#if volumeDisplay === 'muted'}
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
						d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
					/>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"
					/>
				</svg>
			{:else if volumeDisplay === 'low'}
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
						d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
					/>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15.536 8.464a5 5 0 010 7.072"
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
						d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
					/>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15.536 8.464a5 5 0 010 7.072M18.364 5.636a9 9 0 010 12.728"
					/>
				</svg>
			{/if}
		</button>

		<div class="flex-1"></div>

		<!-- Fullscreen (video only) -->
		{#if isVideo}
			<button
				class="btn btn-square btn-ghost btn-xs"
				on:click={toggleFullscreen}
				aria-label={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
			>
				{#if isFullscreen}
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
							d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25"
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
							d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15"
						/>
					</svg>
				{/if}
			</button>
		{/if}

		<!-- Stop -->
		<button
			class="btn btn-square btn-ghost btn-xs"
			on:click={() => dispatch('stop')}
			aria-label="Stop"
		>
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
					d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
				/>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"
				/>
			</svg>
		</button>
	</div>
</div>

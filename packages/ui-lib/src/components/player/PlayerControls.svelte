<script lang="ts">
	import classNames from 'classnames';
	import { playerService } from 'ui-lib/services/player.service';
	import type { PlayerConnectionState } from 'ui-lib/types/player.type';
	import PlayerSeekBar from './PlayerSeekBar.svelte';

	let {
		mediaElement = null,
		isVideo = false,
		positionSecs = 0,
		durationSecs = null,
		connectionState = 'idle',
		containerElement = null,
		onseek,
		onseekstart,
		onseekend,
		onstop
	}: {
		mediaElement?: HTMLMediaElement | null;
		isVideo?: boolean;
		positionSecs?: number;
		durationSecs?: number | null;
		connectionState?: PlayerConnectionState;
		containerElement?: HTMLElement | null;
		onseek?: (positionSecs: number) => void;
		onseekstart?: () => void;
		onseekend?: () => void;
		onstop?: () => void;
	} = $props();

	let isPaused = $state(true);
	let volume = $state(playerService.getVolume());
	let isMuted = $state(false);
	let isFullscreen = $state(false);
	let volumeBeforeMute = $state(playerService.getVolume());

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

	$effect(() => {
		if (mediaElement !== currentElement) {
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

		return () => {
			if (currentElement) {
				currentElement.removeEventListener('play', onPlay);
				currentElement.removeEventListener('pause', onPause);
				currentElement.removeEventListener('volumechange', onVolumeChange);
			}
		};
	});

	$effect(() => {
		if (typeof document !== 'undefined') {
			document.addEventListener('fullscreenchange', onFullscreenChange);
		}
		return () => {
			document.removeEventListener('fullscreenchange', onFullscreenChange);
		};
	});

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

	let disabled = $derived(connectionState !== 'streaming');

	let volumeDisplay = $derived(isMuted || volume === 0 ? 'muted' : volume < 0.5 ? 'low' : 'high');
</script>

<div class={classNames('flex flex-col gap-1', { 'pointer-events-none opacity-50': disabled })}>
	<PlayerSeekBar
		{positionSecs}
		{durationSecs}
		{disabled}
		onseek={(pos) => onseek?.(pos)}
		onseekstart={() => onseekstart?.()}
		onseekend={() => onseekend?.()}
	/>

	<div class="flex items-center gap-1">
		<button class="btn" onclick={togglePlayPause}>
			{isPaused ? 'Play' : 'Pause'}
		</button>

		<button class="btn" onclick={toggleMute}>
			{volumeDisplay === 'muted' ? 'Unmute' : 'Mute'}
		</button>

		<div class="flex-1"></div>

		{#if isVideo}
			<button class="btn" onclick={toggleFullscreen}>
				{isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
			</button>
		{/if}

		<button class="btn" onclick={() => onstop?.()}>Stop</button>
	</div>
</div>

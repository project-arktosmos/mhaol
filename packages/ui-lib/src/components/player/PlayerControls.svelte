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
		isFullscreen = false,
		onseek,
		onseekstart,
		onseekend,
		onstop,
		onfullscreentoggle,
		onprev,
		onnext
	}: {
		mediaElement?: HTMLMediaElement | null;
		isVideo?: boolean;
		positionSecs?: number;
		durationSecs?: number | null;
		connectionState?: PlayerConnectionState;
		isFullscreen?: boolean;
		onseek?: (positionSecs: number) => void;
		onseekstart?: () => void;
		onseekend?: () => void;
		onstop?: () => void;
		onfullscreentoggle?: () => void;
		onprev?: () => void;
		onnext?: () => void;
	} = $props();

	let isPaused = $state(true);
	let volume = $state(playerService.getVolume());
	let isMuted = $state(false);
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
		{#if onprev}
			<button class="btn" onclick={() => onprev?.()}>Previous</button>
		{/if}

		<button class="btn" onclick={togglePlayPause}>
			{isPaused ? 'Play' : 'Pause'}
		</button>

		{#if onnext}
			<button class="btn" onclick={() => onnext?.()}>Next</button>
		{/if}

		<button class="btn" onclick={toggleMute}>
			{volumeDisplay === 'muted' ? 'Unmute' : 'Mute'}
		</button>

		<div class="flex-1"></div>

		{#if isVideo}
			<button class="btn" onclick={() => onfullscreentoggle?.()}>
				{isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
			</button>
		{/if}

		<button class="btn" onclick={() => onstop?.()}>Stop</button>
	</div>
</div>

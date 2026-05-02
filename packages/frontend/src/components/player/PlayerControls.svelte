<script lang="ts">
	import classNames from 'classnames';
	import { untrack, type Snippet } from 'svelte';
	import type { PlayerConnectionState } from '$types/player.type';
	import PlayerSeekBar from './PlayerSeekBar.svelte';

	let {
		mediaElement = null,
		isVideo = false,
		positionSecs = 0,
		durationSecs = null,
		bufferedRanges = [],
		connectionState = 'idle',
		isFullscreen = false,
		initialVolume = 0.5,
		onseek,
		onseekstart,
		onseekend,
		onstop,
		onfullscreentoggle,
		onprev,
		onnext,
		onpaused,
		onvolumechange,
		extraControls
	}: {
		mediaElement?: HTMLMediaElement | null;
		isVideo?: boolean;
		positionSecs?: number;
		durationSecs?: number | null;
		bufferedRanges?: { start: number; end: number }[];
		connectionState?: PlayerConnectionState;
		isFullscreen?: boolean;
		/**
		 * Volume to apply to the media element on first attachment. The
		 * component otherwise stays decoupled from any external volume
		 * store; emit `onvolumechange` to persist user changes.
		 */
		initialVolume?: number;
		onseek?: (positionSecs: number) => void;
		onseekstart?: () => void;
		onseekend?: () => void;
		onstop?: () => void;
		onfullscreentoggle?: () => void;
		onprev?: () => void;
		onnext?: () => void;
		/** Fires whenever the underlying media element pauses or resumes. */
		onpaused?: (paused: boolean) => void;
		/** Fires whenever the user mutes / unmutes / changes volume. */
		onvolumechange?: (volume: number) => void;
		/**
		 * Optional snippet rendered between the mute and fullscreen buttons.
		 * Used by the catalog detail page to surface its source-picker
		 * buttons inline in the player controls.
		 */
		extraControls?: Snippet;
	} = $props();

	let isPaused = $state(true);
	// `initialVolume` is read once on mount via `untrack` (below). Tracking
	// it reactively in the $state initialiser triggers Svelte's
	// `state_referenced_locally` warning since the value would be captured
	// only on first render anyway.
	let volume = $state(0);
	let isMuted = $state(false);
	let volumeBeforeMute = $state(0);

	function onPlay(): void {
		isPaused = false;
		onpaused?.(false);
	}

	function onPause(): void {
		isPaused = true;
		onpaused?.(true);
	}

	function onVolumeChange(): void {
		if (!mediaElement) return;
		volume = mediaElement.volume;
		isMuted = mediaElement.muted;
	}

	let currentElement: HTMLMediaElement | null = null;
	let volumeInitialized = false;

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
				if (!volumeInitialized) {
					const seed = untrack(() => initialVolume);
					volume = seed;
					volumeBeforeMute = seed;
					volumeInitialized = true;
				}
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
		onvolumechange?.(mediaElement.muted ? 0 : mediaElement.volume);
	}

	let disabled = $derived(connectionState !== 'streaming');

	let volumeDisplay = $derived(isMuted || volume === 0 ? 'muted' : volume < 0.5 ? 'low' : 'high');
</script>

<div class="flex flex-col gap-1">
	<div class={classNames({ 'pointer-events-none opacity-50': disabled })}>
		<PlayerSeekBar
			{positionSecs}
			{durationSecs}
			{bufferedRanges}
			{disabled}
			onseek={(pos) => onseek?.(pos)}
			onseekstart={() => onseekstart?.()}
			onseekend={() => onseekend?.()}
		/>
	</div>

	<div class="flex items-center gap-1">
		<div
			class={classNames('flex items-center gap-1', {
				'pointer-events-none opacity-50': disabled
			})}
		>
			{#if onprev}
				<button class="btn" onclick={() => onprev?.()}>Previous</button>
			{/if}

			<button
				class="btn"
				onclick={togglePlayPause}
				aria-label={isPaused ? 'Play' : 'Pause'}
				title={isPaused ? 'Play' : 'Pause'}
			>
				{#if isPaused}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-5 w-5 translate-x-0.5"
						aria-hidden="true"
					>
						<polygon points="6 4 20 12 6 20 6 4" />
					</svg>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-5 w-5"
						aria-hidden="true"
					>
						<rect x="6" y="4" width="4" height="16" />
						<rect x="14" y="4" width="4" height="16" />
					</svg>
				{/if}
			</button>

			{#if onnext}
				<button class="btn" onclick={() => onnext?.()}>Next</button>
			{/if}

			<button class="btn" onclick={toggleMute}>
				{volumeDisplay === 'muted' ? 'Unmute' : 'Mute'}
			</button>
		</div>

		{#if extraControls}
			<div class="flex flex-1 flex-wrap items-center justify-center gap-1">
				{@render extraControls()}
			</div>
		{:else}
			<div class="flex-1"></div>
		{/if}

		<div
			class={classNames('flex items-center gap-1', {
				'pointer-events-none opacity-50': disabled
			})}
		>
			{#if isVideo}
				<button class="btn" onclick={() => onfullscreentoggle?.()}>
					{isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
				</button>
			{/if}

			<button class="btn" onclick={() => onstop?.()}>Stop</button>
		</div>
	</div>
</div>

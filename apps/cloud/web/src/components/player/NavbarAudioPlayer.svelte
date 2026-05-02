<script lang="ts">
	import { get } from 'svelte/store';
	import classNames from 'classnames';
	import { playerService } from '$services/player.service';

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;

	// Hidden `<video>` element drives the audio. Same rationale as
	// `PlayerVideo` — `<audio>` is fussier with the muxed/fragmented MP4
	// shape googlevideo hands us for `pickAudioFormat` results.
	let mediaElement = $state<HTMLVideoElement | null>(null);
	let attachedUrl: string | null = null;
	let mediaError = $state<string | null>(null);

	let visible = $derived(
		$playerDisplayMode === 'navbar' && $playerState.currentFile !== null
	);
	let isLoading = $derived(visible && !$playerState.directStreamUrl);

	$effect(() => {
		const el = mediaElement;
		if (!el) return;
		if (!visible) {
			if (attachedUrl) {
				el.removeAttribute('src');
				el.load();
				attachedUrl = null;
			}
			return;
		}
		const url = $playerState.directStreamUrl;
		if (!url || attachedUrl === url) return;
		mediaError = null;
		attachedUrl = url;
		el.src = url;
		el.volume = playerService.getVolume();
		el.load();
		el.play().catch((err: Error) => {
			mediaError =
				err.name === 'NotAllowedError'
					? 'Playback blocked. Click play to start.'
					: err.message || 'Playback failed';
			playerService.state.update((s) => ({ ...s, error: mediaError }));
		});
	});

	$effect(() => {
		const el = mediaElement;
		if (!el) return;
		const onTime = () => {
			if (get(playerService.state).isSeeking) return;
			const t = el.currentTime;
			playerService.state.update((s) =>
				s.positionSecs === t ? s : { ...s, positionSecs: t, buffering: false }
			);
		};
		const onMeta = () => {
			if (Number.isFinite(el.duration) && el.duration > 0) {
				const d = el.duration;
				playerService.state.update((s) => (s.durationSecs === d ? s : { ...s, durationSecs: d }));
			}
		};
		const onPlay = () => {
			playerService.state.update((s) => (s.isPaused ? { ...s, isPaused: false } : s));
		};
		const onPause = () => {
			playerService.state.update((s) => (!s.isPaused ? s : { ...s, isPaused: true }));
		};
		const onWaiting = () => playerService.setBuffering(true);
		const onPlaying = () => playerService.setBuffering(false);
		const onError = () => {
			const err = el.error;
			mediaError = err ? `Audio error (code ${err.code})` : 'Audio error';
			playerService.state.update((s) => ({ ...s, error: mediaError }));
		};
		const onVolume = () => {
			playerService.setVolume(el.muted ? 0 : el.volume);
		};
		el.addEventListener('timeupdate', onTime);
		el.addEventListener('loadedmetadata', onMeta);
		el.addEventListener('durationchange', onMeta);
		el.addEventListener('play', onPlay);
		el.addEventListener('pause', onPause);
		el.addEventListener('waiting', onWaiting);
		el.addEventListener('playing', onPlaying);
		el.addEventListener('error', onError);
		el.addEventListener('volumechange', onVolume);
		return () => {
			el.removeEventListener('timeupdate', onTime);
			el.removeEventListener('loadedmetadata', onMeta);
			el.removeEventListener('durationchange', onMeta);
			el.removeEventListener('play', onPlay);
			el.removeEventListener('pause', onPause);
			el.removeEventListener('waiting', onWaiting);
			el.removeEventListener('playing', onPlaying);
			el.removeEventListener('error', onError);
			el.removeEventListener('volumechange', onVolume);
		};
	});

	function togglePlay(): void {
		const el = mediaElement;
		if (!el) return;
		if (el.paused) {
			el.play().catch(console.error);
		} else {
			el.pause();
		}
	}

	function handleSeek(event: Event): void {
		const target = event.currentTarget as HTMLInputElement;
		const pos = parseFloat(target.value);
		const el = mediaElement;
		if (!Number.isFinite(pos) || !el) return;
		el.currentTime = pos;
		playerService.state.update((s) => ({ ...s, positionSecs: pos, isSeeking: false }));
	}

	function handleStop(): void {
		void playerService.stop();
	}

	function formatTime(secs: number | null): string {
		if (secs === null || !Number.isFinite(secs) || secs < 0) return '0:00';
		const total = Math.floor(secs);
		const m = Math.floor(total / 60);
		const s = total % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	let progress = $derived($playerState.positionSecs);
	let total = $derived($playerState.durationSecs ?? 0);
	let isPaused = $derived($playerState.isPaused);
</script>

<div
	class={classNames('flex items-center gap-2 p-2', { hidden: !visible })}
	aria-label="Audio player"
>
	<!-- Hidden media element kept mounted so attach/detach effects can run. -->
	<!-- svelte-ignore a11y_media_has_caption -->
	<video bind:this={mediaElement} class="hidden" playsinline></video>

	{#if visible && $playerState.currentFile}
		{#if $playerState.currentFile.thumbnailUrl}
			<img
				src={$playerState.currentFile.thumbnailUrl}
				alt=""
				class="h-8 w-8 shrink-0 rounded object-cover"
			/>
		{/if}
		<div class="flex min-w-0 flex-1 flex-col leading-tight">
			<span class="truncate text-xs font-medium" title={$playerState.currentFile.name}>
				{$playerState.currentFile.name}
			</span>
			{#if mediaError}
				<span class="truncate text-[10px] text-error" title={mediaError}>{mediaError}</span>
			{:else if $playerState.buffering}
				<span class="truncate text-[10px] opacity-60">Buffering…</span>
			{/if}
			<input
				type="range"
				class="range w-full range-xs"
				min="0"
				max={total > 0 ? total : 0}
				step="0.1"
				value={progress}
				disabled={total <= 0}
				oninput={() => playerService.setSeeking(true)}
				onchange={handleSeek}
				aria-label="Seek"
			/>
			<div class="flex items-center justify-between font-mono text-[10px] tabular-nums opacity-70">
				<span>{formatTime(progress)}</span>
				<span>{formatTime(total > 0 ? total : null)}</span>
			</div>
		</div>
		{#if isLoading}
			<span
				class="loading loading-sm loading-spinner text-primary"
				aria-label="Loading"
				title="Loading…"
			></span>
		{:else}
			<button
				type="button"
				class="btn btn-ghost btn-sm"
				onclick={togglePlay}
				aria-label={isPaused ? 'Play' : 'Pause'}
				title={isPaused ? 'Play' : 'Pause'}
			>
				{isPaused ? '▶' : '⏸'}
			</button>
		{/if}
		<button
			type="button"
			class="btn btn-ghost btn-sm"
			onclick={handleStop}
			aria-label="Stop"
			title="Stop"
		>
			✕
		</button>
	{/if}
</div>

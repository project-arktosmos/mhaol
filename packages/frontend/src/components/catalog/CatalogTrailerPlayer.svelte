<script lang="ts">
	import classNames from 'classnames';
	import type { Snippet } from 'svelte';
	import { resolveYouTubeStreamUrl } from '$lib/youtube-match.service';
	import PlayerControls from '$components/player/PlayerControls.svelte';

	interface Props {
		posterUrl: string | null;
		youtubeUrl: string | null;
		title: string;
		/// Cached Innertube client (`web`, `web_embedded`, `tv`, `android`,
		/// `ios`) that resolved this video on a previous visit. Passed to the
		/// backend as a hint so the failing-candidate iteration is skipped on
		/// the happy path. A stale hint just falls through to the regular
		/// browser priority list.
		preferredClient?: string | null;
		/// Fired once a stream URL has been resolved successfully, with the
		/// Innertube client that produced it. The parent persists this back
		/// to the firkin (only when it differs from the current cached
		/// value) so the next visit lands on the right client first.
		onResolved?: (clientName: string) => void;
		/// Full list of playable trailers. When length > 1 and `onTrailerSelect`
		/// is provided, the player renders a top-right select to switch between
		/// them. The active `youtubeUrl` prop is still the source of truth for
		/// what is being streamed.
		trailerOptions?: { key: string; label: string | null; youtubeUrl: string }[];
		selectedTrailerKey?: string | null;
		onTrailerSelect?: (key: string) => void;
		/// Forwarded to <PlayerControls> — rendered between the mute and
		/// fullscreen buttons.
		extraControls?: Snippet;
		/// Optional snippet rendered absolutely inside the player container,
		/// in place of the circular play button. The snippet receives a
		/// `playByKey(key)` callback so its content can offer per-trailer
		/// play actions (e.g. a Trailer tab in the torrent attachment
		/// panel that lists every season/version with its own Play
		/// button). Calling `playByKey` switches `selectedTrailerKey` to
		/// that key and starts playback once the stream URL has resolved.
		/// When omitted, the legacy circular play overlay is shown.
		playOverlay?: Snippet<[(key: string) => void]>;
	}

	let {
		posterUrl,
		youtubeUrl,
		title,
		preferredClient = null,
		onResolved,
		trailerOptions = [],
		selectedTrailerKey = null,
		onTrailerSelect,
		extraControls,
		playOverlay
	}: Props = $props();

	let containerElement = $state<HTMLDivElement | null>(null);
	let videoElement = $state<HTMLVideoElement | null>(null);
	let streamUrl = $state<string | null>(null);
	let started = $state(false);
	let starting = $state(false);
	let error = $state<string | null>(null);
	let resolvedYoutubeUrl: string | null = null;

	let positionSecs = $state(0);
	let durationSecs = $state<number | null>(null);
	let isFullscreen = $state(false);

	$effect(() => {
		if (!youtubeUrl) return;
		if (resolvedYoutubeUrl === youtubeUrl) return;
		resolvedYoutubeUrl = youtubeUrl;
		streamUrl = null;
		started = false;
		error = null;
		positionSecs = 0;
		durationSecs = null;
		void resolveStream(youtubeUrl);
	});

	async function resolveStream(url: string): Promise<void> {
		const resolved = await resolveYouTubeStreamUrl(url, preferredClient);
		if (resolvedYoutubeUrl !== url) return;
		if (!resolved) {
			error = 'No playable trailer format';
			return;
		}
		streamUrl = resolved.url;
		if (resolved.clientName) onResolved?.(resolved.clientName);
	}

	$effect(() => {
		const element = videoElement;
		if (!element) return;
		const onTime = () => {
			positionSecs = element.currentTime;
		};
		const onMeta = () => {
			if (Number.isFinite(element.duration) && element.duration > 0) {
				durationSecs = element.duration;
			}
		};
		element.addEventListener('timeupdate', onTime);
		element.addEventListener('loadedmetadata', onMeta);
		element.addEventListener('durationchange', onMeta);
		return () => {
			element.removeEventListener('timeupdate', onTime);
			element.removeEventListener('loadedmetadata', onMeta);
			element.removeEventListener('durationchange', onMeta);
		};
	});

	function handleStart(): void {
		if (!videoElement || !streamUrl || starting || started) return;
		starting = true;
		error = null;
		videoElement
			.play()
			.then(() => {
				started = true;
				starting = false;
			})
			.catch((err: Error) => {
				error = `Playback failed: ${err.message}`;
				starting = false;
			});
	}

	function togglePlayPause(): void {
		if (!videoElement) return;
		if (!started) {
			handleStart();
			return;
		}
		if (videoElement.paused) videoElement.play().catch(console.error);
		else videoElement.pause();
	}

	// Switch to a different trailer key and start playback once the
	// stream URL for it has resolved. The picks table in the attachment
	// card calls this with each row's key. If the requested key is
	// already the active one and the URL is ready, plays immediately.
	let pendingPlayKey = $state<string | null>(null);

	function playByKey(key: string): void {
		const target = trailerOptions.find((t) => t.key === key);
		if (!target) return;
		if (target.youtubeUrl === youtubeUrl && streamUrl) {
			handleStart();
			return;
		}
		pendingPlayKey = key;
		onTrailerSelect?.(key);
	}

	$effect(() => {
		if (!pendingPlayKey) return;
		const target = trailerOptions.find((t) => t.key === pendingPlayKey);
		if (!target) {
			pendingPlayKey = null;
			return;
		}
		if (target.youtubeUrl === youtubeUrl && streamUrl) {
			pendingPlayKey = null;
			handleStart();
		}
	});

	function handleSeek(pos: number): void {
		if (!videoElement) return;
		videoElement.currentTime = pos;
		positionSecs = pos;
	}

	function handleStop(): void {
		if (!videoElement) return;
		videoElement.pause();
		videoElement.currentTime = 0;
		positionSecs = 0;
		started = false;
	}

	function toggleFullscreen(): void {
		const container = containerElement;
		if (!container) return;
		if (document.fullscreenElement === container) {
			void document.exitFullscreen();
		} else {
			void container.requestFullscreen();
		}
	}

	$effect(() => {
		const onChange = () => {
			isFullscreen = document.fullscreenElement === containerElement;
		};
		document.addEventListener('fullscreenchange', onChange);
		return () => document.removeEventListener('fullscreenchange', onChange);
	});

	// PlayerControls disables itself when `connectionState !== 'streaming'`.
	// We use 'streaming' once the YouTube stream URL is resolved and the
	// `<video>` element has it attached; before that we report 'idle' so the
	// seek bar / buttons stay greyed out.
	const synthConnectionState = $derived<'idle' | 'streaming'>(streamUrl ? 'streaming' : 'idle');
</script>

<div class="flex flex-col gap-1">
	<div
		bind:this={containerElement}
		class={classNames(
			'relative aspect-video w-full overflow-hidden rounded-md bg-black',
			isFullscreen && 'aspect-auto'
		)}
	>
		{#if streamUrl}
			<!-- svelte-ignore a11y_media_has_caption -->
			<video
				bind:this={videoElement}
				src={streamUrl}
				class="absolute inset-0 h-full w-full cursor-pointer bg-black"
				playsinline
				preload="auto"
				aria-label={title}
				onclick={togglePlayPause}
			></video>
		{/if}

		{#if posterUrl}
			<div
				class={classNames(
					'pointer-events-none absolute inset-0 bg-cover bg-center transition-opacity duration-500',
					started ? 'opacity-0' : 'opacity-100'
				)}
				style:background-image={`url(${posterUrl})`}
				aria-hidden="true"
			></div>
		{/if}

		{#if trailerOptions.length > 1 && onTrailerSelect}
			<select
				class="select-bordered select absolute top-2 right-2 z-30 max-w-[60%] select-sm"
				value={selectedTrailerKey ?? trailerOptions[0]?.key ?? ''}
				onchange={(e) => onTrailerSelect((e.currentTarget as HTMLSelectElement).value)}
				aria-label="Pick trailer"
				title="Pick trailer"
			>
				{#each trailerOptions as opt (opt.key)}
					<option value={opt.key}>{opt.label ?? 'Trailer'}</option>
				{/each}
			</select>
		{/if}

		{#if !started && (streamUrl || posterUrl)}
			{#if playOverlay}
				<div
					class="absolute inset-0 z-20 flex items-center justify-center bg-black/40 p-4 backdrop-blur-sm"
				>
					<div class="w-full max-w-md">
						{@render playOverlay(playByKey)}
					</div>
				</div>
			{:else}
				<button
					type="button"
					class="absolute inset-0 z-20 flex items-center justify-center bg-black/30 transition-colors hover:bg-black/40 disabled:cursor-wait"
					aria-label="Play trailer"
					onclick={handleStart}
					disabled={!streamUrl || starting}
				>
					<span
						class={classNames(
							'flex h-20 w-20 items-center justify-center rounded-full bg-primary text-primary-content shadow-lg transition-transform',
							streamUrl && !starting ? 'hover:scale-110' : 'opacity-70'
						)}
					>
						{#if starting || !streamUrl}
							<span class="loading loading-md loading-spinner"></span>
						{:else}
							<svg
								xmlns="http://www.w3.org/2000/svg"
								viewBox="0 0 24 24"
								fill="currentColor"
								class="h-10 w-10 translate-x-0.5"
								aria-hidden="true"
							>
								<polygon points="6 4 20 12 6 20 6 4" />
							</svg>
						{/if}
					</span>
				</button>
			{/if}
		{/if}

		{#if error}
			<div
				class="absolute inset-x-2 bottom-2 z-30 rounded bg-error/90 px-2 py-1 text-xs text-error-content"
			>
				{error}
			</div>
		{/if}
	</div>

	<PlayerControls
		mediaElement={videoElement}
		isVideo={true}
		{positionSecs}
		{durationSecs}
		connectionState={synthConnectionState}
		{isFullscreen}
		onseek={handleSeek}
		onstop={handleStop}
		onfullscreentoggle={toggleFullscreen}
		{extraControls}
	/>
</div>

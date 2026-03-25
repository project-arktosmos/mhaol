<script lang="ts">
	import { onMount } from 'svelte';
	import Hls from 'hls.js';

	let {
		src,
		poster = null
	}: {
		src: string;
		poster?: string | null;
	} = $props();

	let videoElement = $state<HTMLVideoElement | null>(null);
	let hls = $state<Hls | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(true);

	$effect(() => {
		if (!videoElement || !src) return;
		destroyHls();
		loading = true;
		error = null;
		attachSource(src);
	});

	function attachSource(url: string): void {
		if (!videoElement) return;

		// Most IPTV streams are HLS — always try hls.js first
		if (Hls.isSupported()) {
			const instance = new Hls({
				enableWorker: true,
				lowLatencyMode: true
			});
			instance.loadSource(url);
			instance.attachMedia(videoElement);
			instance.on(Hls.Events.MANIFEST_PARSED, () => {
				loading = false;
				videoElement?.play().catch(() => {});
			});
			instance.on(Hls.Events.ERROR, (_event, data) => {
				if (data.fatal) {
					// If HLS fails, try as direct video source
					instance.destroy();
					hls = null;
					tryDirectSource(url);
				}
			});
			hls = instance;
		} else if (videoElement.canPlayType('application/vnd.apple.mpegurl')) {
			// Safari native HLS
			videoElement.src = url;
			videoElement.addEventListener(
				'loadedmetadata',
				() => {
					loading = false;
					videoElement?.play().catch(() => {});
				},
				{ once: true }
			);
			videoElement.addEventListener(
				'error',
				() => {
					error = 'Failed to load stream';
					loading = false;
				},
				{ once: true }
			);
		} else {
			tryDirectSource(url);
		}
	}

	function tryDirectSource(url: string): void {
		if (!videoElement) return;
		videoElement.src = url;
		videoElement.addEventListener(
			'loadedmetadata',
			() => {
				loading = false;
				videoElement?.play().catch(() => {});
			},
			{ once: true }
		);
		videoElement.addEventListener(
			'error',
			() => {
				error = 'Failed to load stream';
				loading = false;
			},
			{ once: true }
		);
	}

	function destroyHls(): void {
		if (hls) {
			hls.destroy();
			hls = null;
		}
		if (videoElement) {
			videoElement.removeAttribute('src');
			videoElement.load();
		}
	}

	onMount(() => {
		return () => destroyHls();
	});
</script>

<div class="relative overflow-hidden rounded-lg bg-black">
	<video
		bind:this={videoElement}
		class="aspect-video w-full bg-black"
		controls
		playsinline
		poster={poster ?? undefined}
	></video>

	{#if loading}
		<div class="absolute inset-0 flex items-center justify-center bg-black/60">
			<span class="loading loading-lg loading-spinner text-primary"></span>
		</div>
	{/if}

	{#if error}
		<div class="absolute inset-0 flex items-center justify-center bg-black/80">
			<div class="text-center text-error">
				<p class="text-sm font-medium">{error}</p>
				<button
					class="btn mt-2 btn-sm btn-error"
					onclick={() => {
						error = null;
						loading = true;
						if (videoElement && src) attachSource(src);
					}}
				>
					Retry
				</button>
			</div>
		</div>
	{/if}
</div>

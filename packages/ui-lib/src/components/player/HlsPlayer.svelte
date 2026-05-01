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
	let error = $state<string | null>(null);
	let loading = $state(true);

	let hlsInstance: Hls | null = null;

	$effect(() => {
		const el = videoElement;
		const url = src;
		if (!el || !url) return;

		destroyHls();
		loading = true;
		error = null;
		attachSource(el, url);

		return () => destroyHls();
	});

	function attachSource(el: HTMLVideoElement, url: string): void {
		if (Hls.isSupported()) {
			const instance = new Hls({
				enableWorker: true,
				lowLatencyMode: true
			});
			instance.loadSource(url);
			instance.attachMedia(el);
			instance.on(Hls.Events.MANIFEST_PARSED, () => {
				loading = false;
				el.play().catch(() => {});
			});
			instance.on(Hls.Events.ERROR, (_event, data) => {
				if (data.fatal) {
					instance.destroy();
					hlsInstance = null;
					tryDirectSource(el, url);
				}
			});
			hlsInstance = instance;
		} else if (el.canPlayType('application/vnd.apple.mpegurl')) {
			el.src = url;
			el.addEventListener(
				'loadedmetadata',
				() => {
					loading = false;
					el.play().catch(() => {});
				},
				{ once: true }
			);
			el.addEventListener(
				'error',
				() => {
					error = 'Failed to load stream';
					loading = false;
				},
				{ once: true }
			);
		} else {
			tryDirectSource(el, url);
		}
	}

	function tryDirectSource(el: HTMLVideoElement, url: string): void {
		el.src = url;
		el.addEventListener(
			'loadedmetadata',
			() => {
				loading = false;
				el.play().catch(() => {});
			},
			{ once: true }
		);
		el.addEventListener(
			'error',
			() => {
				error = 'Failed to load stream';
				loading = false;
			},
			{ once: true }
		);
	}

	function destroyHls(): void {
		if (hlsInstance) {
			hlsInstance.destroy();
			hlsInstance = null;
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
						if (videoElement && src) attachSource(videoElement, src);
					}}
				>
					Retry
				</button>
			</div>
		</div>
	{/if}
</div>

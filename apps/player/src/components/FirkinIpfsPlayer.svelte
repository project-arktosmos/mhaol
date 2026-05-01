<script lang="ts">
	import type { Firkin, FirkinFile } from 'cloud-ui';
	import { catAsBlob, type PlayerIpfsClient } from '$ipfs/client';

	interface Props {
		firkin: Firkin;
		client: PlayerIpfsClient;
	}
	let { firkin, client }: Props = $props();

	const ipfsFiles = $derived<FirkinFile[]>(firkin.files.filter((f) => f.type === 'ipfs'));

	let selectedIndex = $state<number>(0);
	let videoSrc = $state<string | null>(null);
	let videoMime = $state<string | null>(null);
	let loading = $state(false);
	let progress = $state(0);
	let totalBytes = $state(0);
	let error = $state<string | null>(null);
	let abortCtrl: AbortController | null = null;

	$effect(() => {
		// reset selection when the firkin changes
		const firkinId = firkin.id;
		selectedIndex = 0;
		void firkinId;
		clearVideo();
	});

	function clearVideo() {
		if (videoSrc) {
			URL.revokeObjectURL(videoSrc);
			videoSrc = null;
		}
		videoMime = null;
		progress = 0;
		totalBytes = 0;
		error = null;
		if (abortCtrl) {
			abortCtrl.abort();
			abortCtrl = null;
		}
	}

	function guessMime(file: FirkinFile): string {
		const title = (file.title ?? '').toLowerCase();
		if (title.endsWith('.mp4') || title.endsWith('.m4v')) return 'video/mp4';
		if (title.endsWith('.webm')) return 'video/webm';
		if (title.endsWith('.mov')) return 'video/quicktime';
		if (title.endsWith('.mkv')) return 'video/x-matroska';
		if (title.endsWith('.mp3')) return 'audio/mpeg';
		if (title.endsWith('.flac')) return 'audio/flac';
		if (title.endsWith('.ogg')) return 'audio/ogg';
		if (title.endsWith('.m4a')) return 'audio/mp4';
		if (title.endsWith('.opus')) return 'audio/opus';
		// Fall back to a generic video type — most browsers accept a Blob URL
		// and inspect bytes regardless of the typed MIME.
		return 'video/mp4';
	}

	async function play() {
		const file = ipfsFiles[selectedIndex];
		if (!file) return;
		clearVideo();
		loading = true;
		error = null;
		progress = 0;
		const ctrl = new AbortController();
		abortCtrl = ctrl;
		const mime = guessMime(file);
		try {
			const blob = await catAsBlob(client, file.value, mime, {
				signal: ctrl.signal,
				onProgress: (n) => {
					progress = n;
				}
			});
			if (ctrl.signal.aborted) return;
			totalBytes = blob.size;
			videoSrc = URL.createObjectURL(blob);
			videoMime = mime;
		} catch (err) {
			if (!ctrl.signal.aborted) {
				error = err instanceof Error ? err.message : String(err);
			}
		} finally {
			if (abortCtrl === ctrl) abortCtrl = null;
			loading = false;
		}
	}

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '—';
		const units = ['B', 'KB', 'MB', 'GB'];
		let v = bytes;
		let u = 0;
		while (v >= 1024 && u < units.length - 1) {
			v /= 1024;
			u++;
		}
		return `${v.toFixed(v >= 10 || u === 0 ? 0 : 1)} ${units[u]}`;
	}

	const isAudio = $derived(videoMime?.startsWith('audio/') ?? false);
</script>

<div class="card border border-base-content/10 bg-base-200 p-4">
	<h2 class="mb-2 text-sm font-semibold text-base-content/70 uppercase">IPFS playback</h2>

	{#if ipfsFiles.length === 0}
		<p class="text-sm text-base-content/60">
			This firkin has no <code>ipfs</code> file entries — there is nothing to stream.
		</p>
	{:else}
		<div class="mb-3 flex flex-wrap items-center gap-2">
			<label class="form-control min-w-[280px] flex-1">
				<select class="select-bordered select select-sm" bind:value={selectedIndex}>
					{#each ipfsFiles as file, i (i)}
						<option value={i}>{file.title ?? file.value}</option>
					{/each}
				</select>
			</label>
			<button type="button" class="btn btn-sm btn-primary" disabled={loading} onclick={play}>
				{loading ? 'Loading from IPFS…' : 'Play over IPFS'}
			</button>
			{#if videoSrc}
				<button type="button" class="btn btn-ghost btn-sm" onclick={clearVideo}>Clear</button>
			{/if}
		</div>

		{#if loading}
			<p class="mb-2 text-xs text-base-content/70">
				Streamed {formatBytes(progress)} so far…
			</p>
		{/if}

		{#if error}
			<div class="my-2 alert alert-error"><span>{error}</span></div>
		{/if}

		{#if videoSrc}
			<p class="mb-2 text-xs text-base-content/70">
				Loaded {formatBytes(totalBytes)} via IPFS — {videoMime}
			</p>
			{#if isAudio}
				<audio controls class="w-full" src={videoSrc}>
					<track kind="captions" />
				</audio>
			{:else}
				<video controls playsinline class="aspect-video w-full rounded bg-black" src={videoSrc}>
					<track kind="captions" />
				</video>
			{/if}
		{/if}
	{/if}
</div>

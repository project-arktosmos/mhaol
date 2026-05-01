<script lang="ts">
	import type { Firkin, FirkinFile } from 'cloud-ui';
	import type { PlayerIpfsClient } from '$ipfs/client';
	import {
		startStream,
		pickStreamMode,
		type StreamPlayerHandle,
		type StreamPlayerKind
	} from '$ipfs/stream-player';

	interface Props {
		firkin: Firkin;
		client: PlayerIpfsClient;
	}
	let { firkin, client }: Props = $props();

	const ipfsFiles = $derived<FirkinFile[]>(firkin.files.filter((f) => f.type === 'ipfs'));

	let selectedIndex = $state<number>(0);
	let mediaSrc = $state<string | null>(null);
	let mediaMode = $state<StreamPlayerKind | null>(null);
	let starting = $state(false);
	let progress = $state(0);
	let error = $state<string | null>(null);
	let handle = $state<StreamPlayerHandle | null>(null);

	$effect(() => {
		// Reset playback when the firkin changes.
		const firkinId = firkin.id;
		void firkinId;
		selectedIndex = 0;
		clearMedia();
	});

	function clearMedia() {
		if (handle) {
			handle.cancel();
			handle = null;
		}
		mediaSrc = null;
		mediaMode = null;
		progress = 0;
		error = null;
		starting = false;
	}

	async function play() {
		const file = ipfsFiles[selectedIndex];
		if (!file) return;
		clearMedia();
		starting = true;
		error = null;
		try {
			handle = await startStream({
				client,
				cid: file.value,
				title: file.title,
				onProgress: (p) => {
					progress = p.bytesReceived;
					mediaMode = p.mode;
				}
			});
			mediaSrc = handle.src;
			mediaMode = handle.mode;
			handle.done.catch((err) => {
				if (err instanceof DOMException && err.name === 'AbortError') return;
				error = err instanceof Error ? err.message : String(err);
			});
		} catch (err) {
			error = err instanceof Error ? err.message : String(err);
		} finally {
			starting = false;
		}
	}

	function formatBytes(bytes: number): string {
		if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB'];
		let v = bytes;
		let u = 0;
		while (v >= 1024 && u < units.length - 1) {
			v /= 1024;
			u++;
		}
		return `${v.toFixed(v >= 10 || u === 0 ? 0 : 1)} ${units[u]}`;
	}

	const isAudio = $derived.by(() => {
		const f = ipfsFiles[selectedIndex];
		const t = (f?.title ?? '').toLowerCase();
		return (
			t.endsWith('.mp3') ||
			t.endsWith('.flac') ||
			t.endsWith('.ogg') ||
			t.endsWith('.m4a') ||
			t.endsWith('.opus')
		);
	});

	const plannedMode = $derived(pickStreamMode(ipfsFiles[selectedIndex]?.title));
	const modeLabel = $derived.by(() => {
		switch (mediaMode ?? plannedMode) {
			case 'mse-mp4':
				return 'MSE / mp4 (fragmented on the fly)';
			case 'mse-webm':
				return 'MSE / webm';
			case 'blob':
				return 'Buffered Blob (full download before playback)';
		}
	});
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
			<button type="button" class="btn btn-sm btn-primary" disabled={starting} onclick={play}>
				{starting ? 'Opening stream…' : 'Play over IPFS'}
			</button>
			{#if mediaSrc || handle}
				<button type="button" class="btn btn-ghost btn-sm" onclick={clearMedia}>Clear</button>
			{/if}
		</div>

		<p class="mb-2 text-xs text-base-content/60">
			Mode: <span class="font-mono">{modeLabel}</span>
		</p>

		{#if progress > 0 || starting}
			<p class="mb-2 text-xs text-base-content/70">
				Streamed {formatBytes(progress)} from IPFS
				{#if mediaMode === 'mse-mp4' || mediaMode === 'mse-webm'}
					— playback starts as soon as the first segment is decodable
				{/if}
			</p>
		{/if}

		{#if error}
			<div class="my-2 alert alert-error"><span>{error}</span></div>
		{/if}

		{#if mediaSrc}
			{#if isAudio}
				<audio controls class="w-full" src={mediaSrc} autoplay>
					<track kind="captions" />
				</audio>
			{:else}
				<video
					controls
					playsinline
					autoplay
					class="aspect-video w-full rounded bg-black"
					src={mediaSrc}
				>
					<track kind="captions" />
				</video>
			{/if}
		{/if}
	{/if}
</div>

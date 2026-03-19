<script lang="ts">
	import type { LibraryFile } from '$types/library.type';
	import type { YouTubeOEmbedResponse } from 'youtube/oembed';
	import { getThumbnailUrl } from 'youtube/embed';
	import { apiUrl } from '$lib/api-base';

	interface Props {
		file: LibraryFile;
		onlink: (youtubeId: string) => void;
		onclose: () => void;
	}

	let { file, onlink, onclose }: Props = $props();

	let videoId = $state(extractVideoId(file.name));
	let loading = $state(false);
	let metadata: YouTubeOEmbedResponse | null = $state(null);
	let error: string | null = $state(null);

	function extractVideoId(name: string): string {
		return name.replace(/\.[^.]+$/, '').trim();
	}

	async function fetchMetadata() {
		const id = videoId.trim();
		if (!id) return;

		loading = true;
		error = null;
		metadata = null;

		try {
			const res = await fetch(apiUrl(`/api/youtube/oembed?videoId=${encodeURIComponent(id)}`));
			if (res.ok) {
				metadata = await res.json();
			} else {
				const data = await res.json().catch(() => null);
				error = data?.error ?? `Failed to fetch metadata (${res.status})`;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			fetchMetadata();
		}
	}

	$effect(() => {
		if (videoId.trim()) {
			fetchMetadata();
		}
	});
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-2xl">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Link YouTube</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		<div class="mt-4 flex gap-2">
			<div class="join flex-1">
				<input
					type="text"
					class="input-bordered input input-sm join-item w-full"
					placeholder="YouTube video ID..."
					bind:value={videoId}
					onkeydown={handleKeydown}
				/>
				<button
					class="btn join-item btn-sm btn-primary"
					onclick={fetchMetadata}
					disabled={loading || !videoId.trim()}
				>
					{#if loading}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						Fetch
					{/if}
				</button>
			</div>
		</div>

		{#if error}
			<div class="mt-3 rounded-lg bg-error/10 px-3 py-2 text-sm text-error">{error}</div>
		{/if}

		{#if metadata}
			<div class="mt-4 flex gap-4 rounded-lg bg-base-200 p-3">
				<div class="h-20 w-28 flex-shrink-0 overflow-hidden rounded bg-base-300">
					<img
						src={getThumbnailUrl(videoId.trim())}
						alt={metadata.title}
						class="h-full w-full object-cover"
					/>
				</div>
				<div class="flex flex-1 flex-col justify-center overflow-hidden">
					<p class="truncate text-sm font-medium">{metadata.title}</p>
					<p class="text-xs opacity-60">{metadata.author_name}</p>
				</div>
			</div>

			<div class="mt-4 flex justify-end">
				<button class="btn btn-sm btn-info" onclick={() => onlink(videoId.trim())}> Link </button>
			</div>
		{/if}
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>

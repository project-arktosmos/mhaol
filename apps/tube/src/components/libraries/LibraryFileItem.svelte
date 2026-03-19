<script lang="ts">
	import type { YouTubeContent } from '$types/youtube.type';
	import { formatDuration } from '$types/youtube.type';
	import { libraryService } from '$services/library.service';

	interface Props {
		item: YouTubeContent;
	}

	let { item }: Props = $props();

	let videoUrl = $derived(libraryService.streamVideoUrl(item.youtubeId));
	let audioUrl = $derived(libraryService.streamAudioUrl(item.youtubeId));
</script>

<tr class="hover">
	<td class="w-12">
		{#if item.thumbnailUrl}
			<img
				src={item.thumbnailUrl}
				alt={item.title}
				class="h-8 w-12 rounded object-cover"
				loading="lazy"
			/>
		{:else}
			<div class="h-8 w-12 rounded bg-base-300"></div>
		{/if}
	</td>
	<td class="max-w-xs">
		<span class="block truncate text-sm font-medium" title={item.title}>{item.title}</span>
		{#if item.channelName}
			<span class="block truncate text-xs opacity-50">{item.channelName}</span>
		{/if}
	</td>
	<td class="w-20 text-xs opacity-60">
		{#if item.durationSeconds}
			{formatDuration(item.durationSeconds)}
		{:else}
			—
		{/if}
	</td>
	<td class="w-24">
		<div class="flex gap-1">
			{#if item.hasVideo}
				<a href={videoUrl} target="_blank" class="btn btn-ghost btn-xs" title="Play video">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M15 10l4.553-2.277A1 1 0 0121 8.677v6.646a1 1 0 01-1.447.894L15 14M4 6h9a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V8a2 2 0 012-2z"
						/>
					</svg>
				</a>
			{/if}
			{#if item.hasAudio}
				<a href={audioUrl} target="_blank" class="btn btn-ghost btn-xs" title="Play audio">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
						/>
					</svg>
				</a>
			{/if}
		</div>
	</td>
</tr>

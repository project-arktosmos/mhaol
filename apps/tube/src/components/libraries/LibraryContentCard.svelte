<script lang="ts">
	import type { LibraryCardItem } from '$types/library.type';
	import type { YouTubeDownloadProgress } from '$types/youtube.type';
	import { formatDuration, getStateColor, getStateLabel } from '$types/youtube.type';

	let {
		item,
		download,
		onclick
	}: {
		item: LibraryCardItem;
		download?: YouTubeDownloadProgress;
		onclick: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card cursor-pointer bg-base-100 shadow-sm transition-shadow hover:shadow-md" {onclick}>
	<figure class="relative aspect-video bg-base-300">
		{#if item.thumbnailUrl}
			<img
				src={item.thumbnailUrl}
				alt={item.title}
				class="h-full w-full object-cover"
				loading="lazy"
			/>
		{/if}
		{#if download}
			<div class="absolute inset-x-0 bottom-0 flex flex-col gap-0.5 bg-base-300/80 px-1.5 py-1">
				<div class="flex items-center justify-between gap-1">
					<span class="badge badge-xs badge-{getStateColor(download.state)}">
						{getStateLabel(download.state)}
					</span>
					<span class="badge badge-ghost badge-xs opacity-70">{download.mode}</span>
				</div>
				{#if download.state === 'downloading'}
					<progress class="progress h-1 w-full progress-primary" value={download.progress} max="1"
					></progress>
				{:else}
					<progress class="progress h-1 w-full progress-primary"></progress>
				{/if}
			</div>
		{/if}
	</figure>
	<div class="card-body p-2">
		<p class="line-clamp-2 text-xs leading-tight font-medium" title={item.title}>
			{item.title}
		</p>
		{#if item.channelName}
			<p class="truncate text-xs opacity-50">{item.channelName}</p>
		{/if}
		<div class="mt-1 flex items-center gap-1">
			{#if item.durationSeconds}
				<span class="text-xs opacity-40">{formatDuration(item.durationSeconds)}</span>
				<span class="flex-1"></span>
			{/if}
			{#if item.hasVideo}
				<span class="badge badge-xs badge-secondary">▶</span>
			{/if}
			{#if item.hasAudio}
				<span class="badge badge-xs badge-primary">♪</span>
			{/if}
		</div>
	</div>
</div>

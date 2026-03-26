<script lang="ts">
	import type { LibraryCardItem } from 'ui-lib/types/library.type';
	import type { YouTubeDownloadProgress } from 'ui-lib/types/youtube.type';
	import { formatDuration, getStateColor, getStateLabel } from 'ui-lib/types/youtube.type';

	let {
		item,
		download,
		favorited = false,
		pinned = false,
		onclick
	}: {
		item: LibraryCardItem;
		download?: YouTubeDownloadProgress;
		favorited?: boolean;
		pinned?: boolean;
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
		{#if favorited || pinned}
			<div class="absolute top-1.5 left-1.5 z-10 flex gap-1">
				{#if favorited}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-red-500 drop-shadow" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
					</svg>
				{/if}
				{#if pinned}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-blue-400 drop-shadow" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
						<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
					</svg>
				{/if}
			</div>
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

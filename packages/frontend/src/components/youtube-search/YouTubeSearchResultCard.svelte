<script lang="ts">
	import classNames from 'classnames';
	import type { YouTubeSearchItem } from '$types/youtube-search.type';

	let {
		item,
		onselect
	}: {
		item: YouTubeSearchItem;
		onselect?: (item: YouTubeSearchItem) => void;
	} = $props();

	let isLive = $derived(item.duration === -1 || item.duration === 0);

	let durationBadgeClasses = $derived(
		classNames('badge badge-sm absolute bottom-1 right-1', {
			'badge-error': isLive,
			'bg-black/80 text-white': !isLive
		})
	);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="card-compact card cursor-pointer bg-base-200 transition-colors hover:bg-base-300"
	onclick={() => onselect?.(item)}
>
	<figure class="relative aspect-video">
		<img src={item.thumbnail} alt={item.title} class="h-full w-full object-cover" loading="lazy" />
		{#if item.durationText}
			<span class={durationBadgeClasses}>
				{isLive ? 'LIVE' : item.durationText}
			</span>
		{/if}
	</figure>
	<div class="card-body gap-1 p-3">
		<h3 class="line-clamp-2 text-sm font-medium">{item.title}</h3>
		<div class="flex items-center gap-2 text-xs text-base-content/60">
			{#if item.uploaderAvatar}
				<img src={item.uploaderAvatar} alt={item.uploaderName} class="h-4 w-4 rounded-full" />
			{/if}
			<span class="truncate">{item.uploaderName}</span>
			{#if item.uploaderVerified}
				<span class="badge badge-xs badge-info">Verified</span>
			{/if}
		</div>
		<div class="flex items-center gap-2 text-xs text-base-content/50">
			{#if item.viewsText}
				<span>{item.viewsText}</span>
			{/if}
			{#if item.uploadedDate}
				<span>&middot;</span>
				<span>{item.uploadedDate}</span>
			{/if}
		</div>
	</div>
</div>

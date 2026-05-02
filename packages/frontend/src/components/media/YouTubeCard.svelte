<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import { getThumbnailUrl } from 'addons/youtube/helpers';
	import type { MediaItem } from '$types/media-card.type';
	import type { YouTubeOEmbedResponse } from 'addons/youtube/types';

	interface Props {
		item: MediaItem;
		metadata?: YouTubeOEmbedResponse | null;
		loading?: boolean;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let { item, metadata = null, loading = false, selected = false, onselect }: Props = $props();

	let videoId = $derived(item.links.youtube?.serviceId ?? '');
	let thumbnailUrl = $derived(videoId ? getThumbnailUrl(videoId) : null);
</script>

<MediaCardBase
	{item}
	imageUrl={thumbnailUrl}
	imageAlt={metadata?.title ?? item.name}
	{loading}
	{selected}
	onclick={() => onselect?.(item)}
>
	{#if metadata}
		<p class="truncate text-xs font-semibold">{metadata.title}</p>
		<p class="text-xs opacity-60">{metadata.author_name}</p>
	{/if}
</MediaCardBase>

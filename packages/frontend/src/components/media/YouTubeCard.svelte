<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import { getThumbnailUrl } from 'youtube/embed';
	import type { MediaItem } from '$types/media-card.type';
	import type { YouTubeOEmbedResponse } from 'youtube/oembed';

	interface Props {
		item: MediaItem;
		metadata?: YouTubeOEmbedResponse | null;
		loading?: boolean;
		onplay?: (item: MediaItem) => void;
		onunlink?: (item: MediaItem, service: string) => void;
	}

	let { item, metadata = null, loading = false, onplay, onunlink }: Props = $props();

	let videoId = $derived(item.links.youtube?.serviceId ?? '');
	let thumbnailUrl = $derived(videoId ? getThumbnailUrl(videoId) : null);
</script>

<MediaCardBase
	{item}
	imageUrl={thumbnailUrl}
	imageAlt={metadata?.title ?? item.name}
	{loading}
>
	{#if metadata}
		<p class="truncate text-xs font-semibold">{metadata.title}</p>
		<p class="text-xs opacity-60">{metadata.author_name}</p>
	{/if}
	{#snippet actions()}
		<button class="btn btn-accent btn-xs" onclick={() => onplay?.(item)}>Play</button>
		<button class="btn btn-ghost btn-xs" onclick={() => onunlink?.(item, 'youtube')}>Unlink</button>
	{/snippet}
</MediaCardBase>

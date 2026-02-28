<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import type { MediaItem } from '$types/media-card.type';

	interface Props {
		item: MediaItem;
		onplay?: (item: MediaItem) => void;
		onlink?: (item: MediaItem, service: string) => void;
	}

	let { item, onplay, onlink }: Props = $props();
</script>

<MediaCardBase {item} imageAlt={item.name}>
	<p class="truncate text-xs opacity-60" title={item.path}>{item.path}</p>
	{#snippet actions()}
		<button class="btn btn-accent btn-xs" onclick={() => onplay?.(item)}>Play</button>
		<button class="btn btn-primary btn-xs" onclick={() => onlink?.(item, 'tmdb')}>Link metadata</button>
	{/snippet}
</MediaCardBase>

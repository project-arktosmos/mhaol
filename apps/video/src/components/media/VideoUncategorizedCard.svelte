<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import type { MediaItem } from '$types/media-card.type';

	interface Props {
		item: MediaItem;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let { item, selected = false, onselect }: Props = $props();
</script>

<MediaCardBase {item} imageAlt={item.name} {selected} onclick={() => onselect?.(item)}>
	<p class="truncate text-xs opacity-60" title={item.path}>{item.path}</p>
</MediaCardBase>

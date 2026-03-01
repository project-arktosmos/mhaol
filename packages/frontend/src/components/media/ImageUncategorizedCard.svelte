<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import TagPill from '$components/images/TagPill.svelte';
	import type { MediaItem } from '$types/media-card.type';
	import type { ImageTag } from '$types/image-tagger.type';

	interface Props {
		item: MediaItem;
		tags?: ImageTag[];
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let { item, tags = [], selected = false, onselect }: Props = $props();

	let imageUrl = $derived(`/api/images/serve?path=${encodeURIComponent(item.path)}`);
</script>

<MediaCardBase {item} {imageUrl} imageAlt={item.name} {selected} onclick={() => onselect?.(item)}>
	<p class="truncate text-xs opacity-60" title={item.path}>{item.path}</p>
	{#if tags.length > 0}
		<div class="flex flex-wrap gap-1">
			{#each tags as tag (tag.tag)}
				<TagPill tag={tag.tag} score={tag.score} readonly />
			{/each}
		</div>
	{/if}
</MediaCardBase>

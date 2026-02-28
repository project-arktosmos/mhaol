<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { ImageItem } from '$types/image-tagger.type';
	import ImageCard from './ImageCard.svelte';

	interface Props {
		images: ImageItem[];
		taggingItemIds: string[];
		filter: string;
	}

	let { images, taggingItemIds, filter }: Props = $props();

	const dispatch = createEventDispatcher<{
		tag: { id: string };
		addTag: { id: string; tag: string };
		removeTag: { id: string; tag: string };
	}>();

	let filteredImages = $derived(
		filter
			? images.filter(
					(img) =>
						img.tags.some((t) => t.tag.toLowerCase().includes(filter.toLowerCase())) ||
						img.name.toLowerCase().includes(filter.toLowerCase())
				)
			: images
	);
</script>

{#if images.length === 0}
	<div class="flex flex-col items-center justify-center py-12 text-base-content/50">
		<p class="text-lg">No images found</p>
		<p class="text-sm">Scan your libraries to discover image files</p>
	</div>
{:else if filteredImages.length === 0}
	<div class="flex flex-col items-center justify-center py-12 text-base-content/50">
		<p class="text-lg">No images match filter</p>
		<p class="text-sm">Try a different search term</p>
	</div>
{:else}
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
		{#each filteredImages as image (image.id)}
			<ImageCard
				{image}
				isTagging={taggingItemIds.includes(image.id)}
				on:tag={(e) => dispatch('tag', e.detail)}
				on:addTag={(e) => dispatch('addTag', e.detail)}
				on:removeTag={(e) => dispatch('removeTag', e.detail)}
			/>
		{/each}
	</div>
{/if}

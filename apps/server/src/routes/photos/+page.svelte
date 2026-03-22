<script lang="ts">
	import { onDestroy } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from 'frontend/lib/api-base';
	import { imageTaggerService } from 'frontend/services/image-tagger.service';
	import { browseDetailService } from 'frontend/services/browse-detail.service';
	import type { PhotoImageData } from 'frontend/types/browse-detail.type';
	import ImageUncategorizedCard from 'ui-lib/components/media/ImageUncategorizedCard.svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import type { MediaItem, MediaCategory, MediaLinkSource } from 'frontend/types/media-card.type';
	import type { ImageTag } from 'frontend/types/image-tagger.type';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			libraries: Record<string, string>;
			images: PhotoImageData[];
		};
	}

	let { data }: Props = $props();

	let imageItems = $derived(data.images ?? []);
	let allTags = $derived(
		[...new Set(imageItems.flatMap((img) => img.tags.map((t) => t.tag)))].sort()
	);

	let searchQuery = $state('');
	let selectedTag = $state<string | null>(null);
	let showUntaggedOnly = $state(false);

	let filteredItems = $derived(() => {
		let items = imageItems;
		if (showUntaggedOnly) items = items.filter((img) => img.tags.length === 0);
		if (selectedTag) items = items.filter((img) => img.tags.some((t) => t.tag === selectedTag));
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			items = items.filter(
				(img) => img.name.toLowerCase().includes(q) || img.tags.some((t) => t.tag.toLowerCase().includes(q))
			);
		}
		return items;
	});

	let selectedImage = $state<PhotoImageData | null>(null);
	let taggingItems = $state<Set<string>>(new Set());

	async function handleAutoTagAll() {
		const untagged = imageItems.filter((img) => img.tags.length === 0);
		if (untagged.length === 0) return;
		for (const img of untagged) {
			taggingItems.add(img.id);
			taggingItems = new Set(taggingItems);
			try { await imageTaggerService.tagImage(img.id); } catch { /* continue */ }
			taggingItems.delete(img.id);
			taggingItems = new Set(taggingItems);
		}
	}

	async function handleTagSingle() {
		if (!selectedImage) return;
		browseDetailService.update(() => ({ photoTagging: true }));
		try { await imageTaggerService.tagImage(selectedImage.id); } catch { /* ignore */ }
		browseDetailService.update(() => ({ photoTagging: false }));
	}

	async function handleAddTag(tag: string) {
		if (!selectedImage || !tag.trim()) return;
		await imageTaggerService.addTag(selectedImage.id, tag.trim());
		selectedImage.tags = [...selectedImage.tags, { tag: tag.trim(), confidence: 1.0, score: 1.0 }];
		syncToService();
	}

	async function handleRemoveTag(tag: string) {
		if (!selectedImage) return;
		await imageTaggerService.removeTag(selectedImage.id, tag);
		selectedImage.tags = selectedImage.tags.filter((t) => t.tag !== tag);
		syncToService();
	}

	function handleSelectImage(img: PhotoImageData) {
		selectedImage = img;
		syncToService();
	}

	function syncToService() {
		if (!selectedImage) return;
		browseDetailService.set({
			domain: 'photo',
			photoImage: selectedImage,
			photoTags: allTags,
			photoTagging: false
		});
		browseDetailService.registerCallbacks({
			onclose: () => { selectedImage = null; browseDetailService.close(); },
			onaddtag: handleAddTag,
			onremovetag: handleRemoveTag,
			onautotag: handleTagSingle
		});
	}

	function toMediaItem(img: PhotoImageData): MediaItem {
		return {
			id: img.id,
			libraryId: img.libraryId,
			name: img.name,
			extension: img.extension,
			path: img.path,
			categoryId: null,
			mediaTypeId: 'image',
			createdAt: '',
			links: {}
		};
	}

	onDestroy(() => { browseDetailService.close(); });
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-hidden">
	<BrowseHeader title="Photo Gallery" count={imageItems.length} countLabel="photos">
		{#snippet controls()}
			<div class="flex items-center gap-2">
				<button
					class={classNames('btn btn-xs', { 'btn-active': showUntaggedOnly })}
					onclick={() => { showUntaggedOnly = !showUntaggedOnly; selectedTag = null; }}
				>
					Untagged
				</button>
				{#if selectedTag}
					<span class="badge badge-primary gap-1">
						{selectedTag}
						<button class="btn btn-ghost btn-xs" onclick={() => (selectedTag = null)}>x</button>
					</span>
				{/if}
			</div>
			<div class="ml-auto flex items-center gap-2">
				<input
					type="text"
					placeholder="Search photos..."
					class="input input-sm input-bordered w-48"
					bind:value={searchQuery}
				/>
				<button class="btn btn-primary btn-sm" onclick={handleAutoTagAll}>Auto-tag All</button>
			</div>
		{/snippet}
	</BrowseHeader>

	<BrowseGrid
		items={filteredItems()}
		emptyTitle={imageItems.length === 0 ? 'No images found' : 'No matching photos'}
		emptySubtitle={imageItems.length === 0 ? 'Add a library with image files to get started' : ''}
	>
		{#snippet card(item)}
			{@const img = item as PhotoImageData}
			<ImageUncategorizedCard
				item={toMediaItem(img)}
				tags={img.tags}
				tagging={taggingItems.has(img.id)}
				selected={selectedImage?.id === img.id}
				onselect={() => handleSelectImage(img)}
			/>
		{/snippet}
	</BrowseGrid>
</div>

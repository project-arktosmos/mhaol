<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { imageTaggerService } from 'ui-lib/services/image-tagger.service';
	import type { ImageTag } from 'ui-lib/types/image-tagger.type';

	interface PhotoImageData {
		id: string;
		libraryId: string;
		name: string;
		path: string;
		extension: string;
		tags: ImageTag[];
	}
	import ImageUncategorizedCard from 'ui-lib/components/media/ImageUncategorizedCard.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import Portal from 'ui-lib/components/core/Portal.svelte';
	import Modal from 'ui-lib/components/core/Modal.svelte';
	import TagPill from 'ui-lib/components/images/TagPill.svelte';
	import type { MediaItem, MediaCategory, MediaLinkSource } from 'ui-lib/types/media-card.type';
	import { MEDIA_BAR_KEY, type MediaBarContext } from 'ui-lib/types/media-bar.type';

	const mediaBar = getContext<MediaBarContext>(MEDIA_BAR_KEY);

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

	$effect(() => {
		mediaBar.configure({ title: 'Photo Gallery', count: imageItems.length, countLabel: 'photos' });
	});

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
	let photoTagging = $state(false);
	let newPhotoTag = $state('');

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
		photoTagging = true;
		try { await imageTaggerService.tagImage(selectedImage.id); } catch { /* ignore */ }
		photoTagging = false;
	}

	async function handleAddTag() {
		if (!selectedImage || !newPhotoTag.trim()) return;
		const tag = newPhotoTag.trim();
		await imageTaggerService.addTag(selectedImage.id, tag);
		selectedImage.tags = [...selectedImage.tags, { tag, confidence: 1.0, score: 1.0 }];
		selectedImage = selectedImage;
		newPhotoTag = '';
	}

	async function handleRemoveTag(tag: string) {
		if (!selectedImage) return;
		await imageTaggerService.removeTag(selectedImage.id, tag);
		selectedImage.tags = selectedImage.tags.filter((t) => t.tag !== tag);
		selectedImage = selectedImage;
	}

	function handleSelectImage(img: PhotoImageData) {
		selectedImage = img;
		newPhotoTag = '';
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


</script>

<Portal target={mediaBar.controlsTarget}>
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
</Portal>

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

<Modal open={!!selectedImage} maxWidth="max-w-lg" onclose={() => (selectedImage = null)}>
	{#if selectedImage}
		<div class="flex flex-col gap-3 p-4">
			<h2 class="truncate text-sm font-semibold">{selectedImage.name}</h2>

			<img
				src={apiUrl(`/api/images/serve?path=${encodeURIComponent(selectedImage.path)}`)}
				alt={selectedImage.name}
				class="w-full rounded-lg object-cover"
			/>

			<div class="flex flex-wrap gap-1">
				{#each selectedImage.tags as tag (tag.tag)}
					<TagPill
						tag={tag.tag}
						score={tag.score}
						onremove={() => handleRemoveTag(tag.tag)}
					/>
				{/each}
			</div>

			<div class="flex gap-1">
				<input
					type="text"
					placeholder="Add tag..."
					class="input-bordered input input-sm flex-1"
					bind:value={newPhotoTag}
					onkeydown={(e) => e.key === 'Enter' && handleAddTag()}
				/>
				<button class="btn btn-ghost btn-sm" onclick={handleAddTag}>+</button>
			</div>

			<button
				class="btn btn-outline btn-sm"
				disabled={photoTagging}
				onclick={handleTagSingle}
			>
				{#if photoTagging}
					<span class="loading loading-xs loading-spinner"></span> Tagging...
				{:else}
					Auto-tag
				{/if}
			</button>
		</div>
	{/if}
</Modal>

<script lang="ts">
	import classNames from 'classnames';
	import { apiUrl } from 'frontend/lib/api-base';
	import { imageTaggerService } from 'frontend/services/image-tagger.service';
	import ImageUncategorizedCard from 'ui-lib/components/media/ImageUncategorizedCard.svelte';
	import TagPill from 'ui-lib/components/images/TagPill.svelte';
	import type { MediaItem, MediaCategory, MediaLinkSource } from 'frontend/types/media-card.type';
	import type { ImageTag } from 'frontend/types/image-tagger.type';

	interface ImageData {
		id: string;
		libraryId: string;
		name: string;
		path: string;
		extension: string;
		tags: ImageTag[];
	}

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			itemsByCategory: Record<string, MediaItem[]>;
			itemsByType: Record<string, MediaItem[]>;
			libraries: Record<string, string>;
			images: ImageData[];
		};
	}

	let { data }: Props = $props();

	const taggerState = imageTaggerService.state;

	// Image items from the /api/images endpoint (includes tags)
	let imageItems = $derived(data.images ?? []);

	// Tag index for filtering
	let allTags = $derived(
		[...new Set(imageItems.flatMap((img) => img.tags.map((t) => t.tag)))].sort()
	);

	// Filter state
	let searchQuery = $state('');
	let selectedTag = $state<string | null>(null);
	let showUntaggedOnly = $state(false);

	let filteredItems = $derived(() => {
		let items = imageItems;
		if (showUntaggedOnly) {
			items = items.filter((img) => img.tags.length === 0);
		}
		if (selectedTag) {
			items = items.filter((img) => img.tags.some((t) => t.tag === selectedTag));
		}
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			items = items.filter(
				(img) =>
					img.name.toLowerCase().includes(q) ||
					img.tags.some((t) => t.tag.toLowerCase().includes(q))
			);
		}
		return items;
	});

	// Selected image for detail view
	let selectedImage = $state<ImageData | null>(null);
	let taggingItems = $state<Set<string>>(new Set());
	let newTag = $state('');

	// Tagger status
	let taggerStatus = $derived($taggerState);

	async function handleAutoTagAll() {
		const untagged = imageItems.filter((img) => img.tags.length === 0);
		if (untagged.length === 0) return;

		for (const img of untagged) {
			taggingItems.add(img.id);
			taggingItems = new Set(taggingItems);
			try {
				await imageTaggerService.tagImage(img.id);
			} catch {
				// continue with next
			}
			taggingItems.delete(img.id);
			taggingItems = new Set(taggingItems);
		}
	}

	async function handleTagSingle(imageId: string) {
		taggingItems.add(imageId);
		taggingItems = new Set(taggingItems);
		try {
			await imageTaggerService.tagImage(imageId);
		} catch {
			// ignore
		}
		taggingItems.delete(imageId);
		taggingItems = new Set(taggingItems);
	}

	async function handleAddTag() {
		if (!selectedImage || !newTag.trim()) return;
		await imageTaggerService.addTag(selectedImage.id, newTag.trim());
		selectedImage.tags = [...selectedImage.tags, { tag: newTag.trim(), confidence: 1.0, score: 1.0 }];
		newTag = '';
	}

	async function handleRemoveTag(tag: string) {
		if (!selectedImage) return;
		await imageTaggerService.removeTag(selectedImage.id, tag);
		selectedImage.tags = selectedImage.tags.filter((t) => t.tag !== tag);
	}

	// Build MediaItem for the card component
	function toMediaItem(img: ImageData): MediaItem {
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

<div class="flex h-full w-full">
	<!-- Main content -->
	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- Header -->
		<div class="flex flex-wrap items-center gap-3 border-b border-base-300 px-4 py-3">
			<h2 class="text-lg font-bold">Photo Gallery</h2>
			<span class="badge badge-ghost">{imageItems.length} photos</span>

			<!-- Filter controls -->
			<div class="flex items-center gap-2">
				<button
					class={classNames('btn btn-xs', { 'btn-active': showUntaggedOnly })}
					onclick={() => {
						showUntaggedOnly = !showUntaggedOnly;
						selectedTag = null;
					}}
				>
					Untagged
				</button>
				{#if selectedTag}
					<span class="badge badge-primary gap-1">
						{selectedTag}
						<button class="btn btn-ghost btn-xs" onclick={() => (selectedTag = null)}>
							x
						</button>
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
				<button class="btn btn-primary btn-sm" onclick={handleAutoTagAll}>
					Auto-tag All
				</button>
			</div>
		</div>

		<!-- Grid -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if filteredItems().length === 0}
				<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
					{#if imageItems.length === 0}
						<p class="text-lg">No images found</p>
						<p class="mt-1 text-sm">Add a library with image files to get started</p>
					{:else}
						<p class="text-lg">No matching photos</p>
					{/if}
				</div>
			{:else}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
					{#each filteredItems() as img (img.id)}
						<ImageUncategorizedCard
							item={toMediaItem(img)}
							tags={img.tags}
							tagging={taggingItems.has(img.id)}
							selected={selectedImage?.id === img.id}
							onselect={() => (selectedImage = img)}
						/>
					{/each}
				</div>
			{/if}
		</div>
	</div>

	<!-- Right sidebar: tags browser + selected image detail -->
	<div class="hidden w-72 flex-col gap-3 overflow-y-auto border-l border-base-300 bg-base-100 p-4 lg:flex">
		<!-- Tags list -->
		<div>
			<h3 class="mb-2 text-sm font-semibold text-base-content/70">Tags</h3>
			{#if allTags.length === 0}
				<p class="text-xs text-base-content/40">No tags yet</p>
			{:else}
				<div class="flex flex-wrap gap-1">
					{#each allTags as tag}
						<button
							class={classNames('badge badge-sm cursor-pointer', {
								'badge-primary': selectedTag === tag,
								'badge-ghost': selectedTag !== tag
							})}
							onclick={() => {
								selectedTag = selectedTag === tag ? null : tag;
								showUntaggedOnly = false;
							}}
						>
							{tag}
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="divider my-0"></div>

		<!-- Selected image detail -->
		{#if selectedImage}
			<div class="flex flex-col gap-2">
				<img
					src={apiUrl(`/api/images/serve?path=${encodeURIComponent(selectedImage.path)}`)}
					alt={selectedImage.name}
					class="w-full rounded-lg object-cover"
				/>
				<h3 class="truncate text-sm font-bold" title={selectedImage.name}>
					{selectedImage.name}
				</h3>

				<!-- Tags -->
				<div class="flex flex-wrap gap-1">
					{#each selectedImage.tags as tag (tag.tag)}
						<TagPill tag={tag.tag} score={tag.score} onremove={() => handleRemoveTag(tag.tag)} />
					{/each}
				</div>

				<!-- Add tag -->
				<div class="flex gap-1">
					<input
						type="text"
						placeholder="Add tag..."
						class="input input-xs input-bordered flex-1"
						bind:value={newTag}
						onkeydown={(e) => e.key === 'Enter' && handleAddTag()}
					/>
					<button class="btn btn-ghost btn-xs" onclick={handleAddTag}>+</button>
				</div>

				<!-- Actions -->
				<button
					class="btn btn-sm btn-outline"
					disabled={taggingItems.has(selectedImage.id)}
					onclick={() => handleTagSingle(selectedImage!.id)}
				>
					{#if taggingItems.has(selectedImage.id)}
						<span class="loading loading-xs loading-spinner"></span>
						Tagging...
					{:else}
						Auto-tag
					{/if}
				</button>
			</div>
		{:else}
			<div class="flex flex-col items-center justify-center py-8 text-base-content/30">
				<p class="text-sm">Select a photo</p>
			</div>
		{/if}
	</div>
</div>

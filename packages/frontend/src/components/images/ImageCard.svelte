<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import type { ImageItem } from '$types/image-tagger.type';
	import TagPill from './TagPill.svelte';

	interface Props {
		image: ImageItem;
		isTagging: boolean;
	}

	let { image, isTagging }: Props = $props();

	const dispatch = createEventDispatcher<{
		tag: { id: string };
		addTag: { id: string; tag: string };
		removeTag: { id: string; tag: string };
	}>();

	let imageUrl = $derived(`/api/images/serve?path=${encodeURIComponent(image.path)}`);
	let newTag = $state('');

	function handleAddTag() {
		const trimmed = newTag.trim();
		if (!trimmed) return;
		dispatch('addTag', { id: image.id, tag: trimmed });
		newTag = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleAddTag();
		}
	}
</script>

<div class="card card-compact bg-base-200 shadow-sm">
	<figure class="relative h-48 overflow-hidden bg-base-300">
		<img
			src={imageUrl}
			alt={image.name}
			class="h-full w-full object-cover"
			loading="lazy"
		/>
		{#if isTagging}
			<div class="absolute inset-0 flex items-center justify-center bg-base-300/60">
				<span class="loading loading-spinner loading-md"></span>
			</div>
		{/if}
	</figure>
	<div class="card-body gap-2">
		<h3 class="card-title text-sm truncate" title={image.name}>
			{image.name}
		</h3>
		<p class="text-xs opacity-50">{image.libraryName}</p>

		{#if image.tags.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each image.tags as tag (tag.tag)}
					<TagPill
						tag={tag.tag}
						score={tag.score}
						on:remove={(e) => dispatch('removeTag', { id: image.id, tag: e.detail.tag })}
					/>
				{/each}
			</div>
		{/if}

		<div class="flex items-center gap-1">
			<input
				type="text"
				placeholder="Add tag..."
				class="input input-bordered input-xs flex-1 min-w-0"
				bind:value={newTag}
				onkeydown={handleKeydown}
			/>
			<button class="btn btn-xs btn-ghost" onclick={handleAddTag} disabled={!newTag.trim()}>
				+
			</button>
		</div>

		<div class="card-actions justify-end mt-1">
			<button
				class={classNames('btn btn-xs', {
					'btn-ghost': image.tags.length > 0,
					'btn-primary': image.tags.length === 0
				})}
				disabled={isTagging}
				onclick={() => dispatch('tag', { id: image.id })}
			>
				{#if isTagging}
					<span class="loading loading-spinner loading-xs"></span>
				{:else if image.tags.length > 0}
					Re-tag
				{:else}
					Tag
				{/if}
			</button>
		</div>
	</div>
</div>

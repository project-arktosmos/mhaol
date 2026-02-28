<script lang="ts">
	import { onMount } from 'svelte';
	import { imageTaggerService } from '$services/image-tagger.service';
	import ImageGrid from '$components/images/ImageGrid.svelte';
	import TagCloud from '$components/images/TagCloud.svelte';
	import TaggerControls from '$components/images/TaggerControls.svelte';

	const state = imageTaggerService.state;
	const images = imageTaggerService.store;

	let untaggedCount = $derived($images.filter((img) => img.tags.length === 0).length);

	onMount(() => {
		imageTaggerService.initialize();
		imageTaggerService.checkTaggerStatus();
	});

	function handleTagImage(e: CustomEvent<{ id: string }>) {
		imageTaggerService.tagImage(e.detail.id);
	}

	function handleTagUntagged() {
		const untaggedIds = $images.filter((img) => img.tags.length === 0).map((img) => img.id);
		imageTaggerService.tagBatch(untaggedIds);
	}

	function handleTagAll() {
		const allIds = $images.map((img) => img.id);
		imageTaggerService.tagBatch(allIds);
	}

	function handleAddTag(e: CustomEvent<{ id: string; tag: string }>) {
		imageTaggerService.addTag(e.detail.id, e.detail.tag);
	}

	function handleRemoveTag(e: CustomEvent<{ id: string; tag: string }>) {
		imageTaggerService.removeTag(e.detail.id, e.detail.tag);
	}

	function handleTagCloudSelect(tag: string) {
		imageTaggerService.setFilter(tag);
	}

	function handleFilterInput(e: Event) {
		imageTaggerService.setFilter((e.target as HTMLInputElement).value);
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex flex-col gap-4">
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Image Tagger</h1>
				<p class="text-sm text-base-content/60">
					Auto-tag images from your libraries using SigLIP
				</p>
			</div>
		</div>

		<TaggerControls
			taggerReady={$state.taggerReady}
			taggerInitializing={$state.taggerInitializing}
			taggerStatus={$state.taggerStatus}
			taggerProgress={$state.taggerProgress}
			taggerError={$state.taggerError}
			taggingCount={$state.taggingItemIds.length}
			totalImages={$images.length}
			{untaggedCount}
			on:tagUntagged={handleTagUntagged}
			on:tagAll={handleTagAll}
		/>

		{#if $images.length > 0}
			<div class="form-control w-full max-w-xs">
				<input
					type="text"
					placeholder="Filter by tag or filename..."
					class="input input-bordered input-sm"
					value={$state.filter}
					oninput={handleFilterInput}
				/>
			</div>

			<TagCloud
				images={$images}
				activeFilter={$state.filter}
				onselect={handleTagCloudSelect}
			/>
		{/if}
	</div>

	{#if $state.error}
		<div class="alert alert-error">
			<span>{$state.error}</span>
		</div>
	{/if}

	{#if $state.loading}
		<div class="flex items-center justify-center py-12">
			<span class="loading loading-spinner loading-lg"></span>
		</div>
	{:else}
		<ImageGrid
			images={$images}
			taggingItemIds={$state.taggingItemIds}
			filter={$state.filter}
			on:tag={handleTagImage}
			on:addTag={handleAddTag}
			on:removeTag={handleRemoveTag}
		/>
	{/if}
</div>

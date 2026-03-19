<script lang="ts">
	import classNames from 'classnames';
	import type { LibraryFile, MediaTypeOption, CategoryOption } from '$types/library.type';
	import { libraryService } from '$services/library.service';

	interface Props {
		file: LibraryFile;
		onsave: (mediaType: string, categoryId: string | null) => void;
		onclose: () => void;
	}

	let { file, onsave, onclose }: Props = $props();

	let mediaTypes = $state<MediaTypeOption[]>([]);
	let categories = $state<CategoryOption[]>([]);
	let selectedMediaType = $state<string>(file.mediaType);
	let selectedCategory = $state<string | null>(file.categoryId);
	let loading = $state(true);

	async function loadData() {
		loading = true;
		try {
			mediaTypes = await libraryService.fetchMediaTypes();
			if (selectedMediaType) {
				categories = await libraryService.fetchCategories(selectedMediaType);
			}
		} catch (e) {
			console.error('[modal] Failed to load data:', e);
		} finally {
			loading = false;
		}
	}

	async function handleMediaTypeChange(mediaTypeId: string) {
		selectedMediaType = mediaTypeId;
		selectedCategory = null;
		try {
			categories = await libraryService.fetchCategories(mediaTypeId);
		} catch {
			categories = [];
		}
	}

	function handleSave() {
		onsave(selectedMediaType, selectedCategory);
	}

	$effect(() => {
		loadData();
	});
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-md">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Edit Type & Category</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		{#if loading}
			<div class="flex justify-center py-8">
				<span class="loading loading-md loading-spinner"></span>
			</div>
		{:else}
			<div class="mt-4">
				<label class="label">
					<span class="label-text font-medium">Media Type</span>
				</label>
				<div class="flex flex-wrap gap-2">
					{#each mediaTypes as mt (mt.id)}
						<button
							class={classNames('btn btn-sm', {
								'btn-primary': selectedMediaType === mt.id,
								'btn-ghost': selectedMediaType !== mt.id
							})}
							onclick={() => handleMediaTypeChange(mt.id)}
						>
							{mt.label}
						</button>
					{/each}
				</div>
			</div>

			{#if categories.length > 0}
				<div class="mt-4">
					<label class="label">
						<span class="label-text font-medium">Category</span>
					</label>
					<div class="flex flex-wrap gap-2">
						<button
							class={classNames('btn btn-sm', {
								'btn-primary': selectedCategory === null,
								'btn-ghost': selectedCategory !== null
							})}
							onclick={() => (selectedCategory = null)}
						>
							None
						</button>
						{#each categories as cat (cat.id)}
							<button
								class={classNames('btn btn-sm', {
									'btn-primary': selectedCategory === cat.id,
									'btn-ghost': selectedCategory !== cat.id
								})}
								onclick={() => (selectedCategory = cat.id)}
							>
								{cat.label}
							</button>
						{/each}
					</div>
				</div>
			{/if}

			<div class="modal-action">
				<button class="btn btn-ghost btn-sm" onclick={onclose}>Cancel</button>
				<button class="btn btn-sm btn-primary" onclick={handleSave}>Save</button>
			</div>
		{/if}
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>

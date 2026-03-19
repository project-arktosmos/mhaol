<script lang="ts">
	import classNames from 'classnames';
	import { libraryService } from '$services/library.service';
	import { MEDIA_TYPE_OPTIONS, type MediaType } from '$types/library.type';
	import DirectoryBrowser from './DirectoryBrowser.svelte';

	const state = libraryService.state;

	function handleDirectorySelect(path: string, name: string) {
		libraryService.selectDirectory(path, name);
	}

	function handleNameInput(event: Event) {
		const target = event.target as HTMLInputElement;
		libraryService.setSelectedName(target.value);
	}

	function handleToggleMediaType(mediaType: MediaType) {
		libraryService.toggleMediaType(mediaType);
	}

	function handleAdd() {
		if (canAdd) {
			libraryService.addLibrary(
				$state.selectedName.trim(),
				$state.selectedPath,
				$state.selectedMediaTypes
			);
		}
	}

	function handleCancel() {
		libraryService.closeAddForm();
	}

	let canAdd = $derived(
		$state.selectedPath.length > 0 &&
			$state.selectedName.trim().length > 0 &&
			$state.selectedMediaTypes.length > 0
	);
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Add Library</h2>

		<!-- Directory Browser -->
		<div>
			<div class="label">
				<span class="label-text font-medium">Browse Directories</span>
			</div>
			<DirectoryBrowser onselect={handleDirectorySelect} />
		</div>

		<!-- Selected path display -->
		{#if $state.selectedPath}
			<div class="flex items-center gap-2 rounded-lg bg-success/10 px-3 py-2 text-sm">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4 text-success"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
				</svg>
				<span class="truncate font-mono">{$state.selectedPath}</span>
			</div>
		{/if}

		<!-- Library name -->
		<div class="form-control">
			<label class="label" for="library-name">
				<span class="label-text font-medium">Library Name</span>
			</label>
			<input
				id="library-name"
				type="text"
				placeholder="Enter a name for this library"
				class="input-bordered input"
				value={$state.selectedName}
				oninput={handleNameInput}
			/>
		</div>

		<!-- Media types -->
		<div class="form-control">
			<div class="label">
				<span class="label-text font-medium">Media Types</span>
			</div>
			<div class="flex flex-wrap gap-3">
				{#each MEDIA_TYPE_OPTIONS as option (option.value)}
					<label class="label cursor-pointer gap-2">
						<input
							type="checkbox"
							class="checkbox checkbox-sm checkbox-primary"
							checked={$state.selectedMediaTypes.includes(option.value)}
							onchange={() => handleToggleMediaType(option.value)}
						/>
						<span class="label-text">{option.label}</span>
					</label>
				{/each}
			</div>
		</div>

		<!-- Actions -->
		<div class="flex justify-end gap-2">
			<button class="btn btn-ghost" onclick={handleCancel}> Cancel </button>
			<button
				class={classNames('btn btn-primary', { 'btn-disabled': !canAdd })}
				disabled={!canAdd}
				onclick={handleAdd}
			>
				Add Library
			</button>
		</div>
	</div>
</div>

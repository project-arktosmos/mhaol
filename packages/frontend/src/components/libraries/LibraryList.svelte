<script lang="ts">
	import { libraryService } from '$services/library.service';
	import type { Library } from '$types/library.type';
	import LibraryListItem from './LibraryListItem.svelte';

	const store = libraryService.store;
	const state = libraryService.state;

	function handleRemove(library: Library) {
		libraryService.removeLibrary(library);
	}

	function handleToggle(library: Library) {
		libraryService.toggleLibraryFiles(library.id as string);
	}

	function handleRefresh(library: Library) {
		libraryService.fetchLibraryFiles(library.id as string);
	}
</script>

<div class="card bg-base-200">
	<div class="card-body">
		<div class="flex items-center gap-2">
			<h2 class="card-title text-lg">Libraries</h2>
			{#if $store.length > 0}
				<span class="badge badge-neutral badge-sm">{$store.length}</span>
			{/if}
		</div>

		{#if $store.length === 0}
			<div class="flex flex-col items-center gap-2 py-8 text-base-content/50">
				<svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
					<path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
				</svg>
				<p class="text-sm">No libraries configured</p>
				<p class="text-xs">Add a library to start organizing your media</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				{#each $store as library (library.id)}
					<LibraryListItem
						{library}
						expanded={$state.expandedLibraryId === library.id}
						files={$state.libraryFiles[library.id] ?? []}
						filesLoading={$state.libraryFilesLoading[library.id] ?? false}
						filesError={$state.libraryFilesError[library.id] ?? null}
						onremove={handleRemove}
						ontoggle={handleToggle}
						onrefresh={handleRefresh}
					/>
				{/each}
			</div>
		{/if}
	</div>
</div>

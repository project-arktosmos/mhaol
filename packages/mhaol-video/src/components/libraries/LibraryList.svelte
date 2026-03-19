<script lang="ts">
	import { libraryService } from '$services/library.service';
	import type { Library, LibraryFile } from '$types/library.type';
	import LibraryListItem from './LibraryListItem.svelte';

	const store = libraryService.store;
	const state = libraryService.state;

	function handleRemove(library: Library) {
		libraryService.removeLibrary(library);
	}

	function handleScan(library: Library) {
		libraryService.scanLibraryFiles(library.id as string);
	}

	async function handleLink(
		library: Library,
		file: LibraryFile,
		tmdbId: number,
		seasonNumber: number | null,
		episodeNumber: number | null,
		type: 'movie' | 'tv'
	) {
		const libraryId = library.id as string;
		await libraryService.linkTmdb(libraryId, file.id, tmdbId, seasonNumber, episodeNumber);
		const categoryId = type === 'movie' ? 'movies' : 'tv';
		if (file.categoryId !== categoryId) {
			await libraryService.updateCategory(libraryId, file.id, categoryId);
		}
	}

	function handleUnlink(library: Library, file: LibraryFile) {
		libraryService.unlinkTmdb(library.id as string, file.id);
	}

	async function handleEditType(
		library: Library,
		file: LibraryFile,
		mediaType: string,
		categoryId: string | null
	) {
		const libraryId = library.id as string;
		if (mediaType !== file.mediaType) {
			await libraryService.updateMediaType(libraryId, file.id, mediaType);
		}
		if (categoryId !== file.categoryId) {
			if (categoryId) {
				await libraryService.updateCategory(libraryId, file.id, categoryId);
			} else {
				await libraryService.clearCategory(libraryId, file.id);
			}
		}
	}
</script>

<div class="card bg-base-200">
	<div class="card-body">
		<div class="flex items-center gap-2">
			<h2 class="card-title text-lg">Libraries</h2>
			{#if $store.length > 0}
				<span class="badge badge-sm badge-neutral">{$store.length}</span>
			{/if}
		</div>

		{#if $store.length === 0}
			<div class="flex flex-col items-center gap-2 py-8 text-base-content/50">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="1"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
					/>
				</svg>
				<p class="text-sm">No libraries configured</p>
				<p class="text-xs">Add a library to start organizing your media</p>
			</div>
		{:else}
			<div class="flex flex-col gap-2">
				{#each $store as library (library.id)}
					<LibraryListItem
						{library}
						files={$state.libraryFiles[library.id] ?? []}
						filesLoading={$state.libraryFilesLoading[library.id] ?? false}
						filesError={$state.libraryFilesError[library.id] ?? null}
						onremove={handleRemove}
						onscan={handleScan}
						onlink={(file, tmdbId, season, episode, type) =>
							handleLink(library, file, tmdbId, season, episode, type)}
						onunlink={(file) => handleUnlink(library, file)}
						onedittype={(file, mediaType, categoryId) =>
							handleEditType(library, file, mediaType, categoryId)}
					/>
				{/each}
			</div>
		{/if}
	</div>
</div>

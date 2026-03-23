<script lang="ts">
	import { libraryService } from 'ui-lib/services/library.service';
	import type { Library } from 'ui-lib/types/library.type';
	import LibraryAddForm from 'ui-lib/components/libraries/LibraryAddForm.svelte';

	let {
		selectedLibraryId = '',
		currentLibraryId = undefined,
		onselect
	}: {
		selectedLibraryId?: string;
		currentLibraryId?: string;
		onselect: (libraryId: string) => void;
	} = $props();

	const libraries = libraryService.store;
	const libraryState = libraryService.state;

	let showInlineAddForm = $state(false);
	let previousLibraryCount = $state(0);

	// Auto-select library matching currentLibraryId, or first library if none matches
	$effect(() => {
		if ($libraries.length > 0) {
			if (currentLibraryId) {
				const match = $libraries.find((lib: Library) => String(lib.id) === currentLibraryId);
				if (match) {
					selectedLibraryId = String(match.id);
				}
			}
			if (!selectedLibraryId) {
				const first = $libraries[0];
				selectedLibraryId = String(first.id);
				onselect(String(first.id));
			}
		}
	});

	// Detect when inline add form closes (cancel or successful add)
	$effect(() => {
		if (showInlineAddForm && !$libraryState.showAddForm) {
			showInlineAddForm = false;
			// If a new library was added, auto-select it
			if ($libraries.length > previousLibraryCount) {
				const newest = $libraries[$libraries.length - 1];
				selectedLibraryId = String(newest.id);
				onselect(String(newest.id));
			}
		}
	});

	$effect(() => {
		previousLibraryCount = $libraries.length;
	});

	function handleLibrarySelect(event: Event) {
		const target = event.target as HTMLSelectElement;
		const libraryId = target.value;
		if (!libraryId) return;

		const library = $libraries.find((lib: Library) => String(lib.id) === libraryId);
		if (library) {
			selectedLibraryId = String(library.id);
			onselect(String(library.id));
		}
	}

	function handleShowAddForm() {
		showInlineAddForm = true;
		libraryService.openAddForm();
	}
</script>

<div class="form-control">
	<label class="label" for="library-select">
		<span class="label-text">Download Library</span>
	</label>

	{#if $libraries.length > 0}
		<div class="flex items-center gap-2">
			<select
				id="library-select"
				class="select-bordered select flex-1"
				value={selectedLibraryId}
				onchange={handleLibrarySelect}
			>
				<option value="" disabled>Select a library...</option>
				{#each $libraries as library (library.id)}
					<option value={String(library.id)}>
						{library.name}
					</option>
				{/each}
			</select>
			<button class="btn btn-ghost btn-sm" onclick={handleShowAddForm} title="Add new library">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
				</svg>
			</button>
		</div>
	{:else}
		<div class="rounded-lg bg-base-300 p-4 text-center">
			<p class="mb-2 text-sm text-base-content/60">No libraries configured</p>
			<button class="btn btn-sm btn-primary" onclick={handleShowAddForm}> Create Library </button>
		</div>
	{/if}
</div>

{#if showInlineAddForm && $libraryState.showAddForm}
	<LibraryAddForm />
{/if}

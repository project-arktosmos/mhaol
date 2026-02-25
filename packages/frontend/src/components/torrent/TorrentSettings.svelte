<script lang="ts">
	import classNames from 'classnames';
	import { torrentService } from '$services/torrent.service';
	import { libraryService } from '$services/library.service';
	import type { Library } from '$types/library.type';
	import LibraryAddForm from '$components/libraries/LibraryAddForm.svelte';

	const state = torrentService.state;
	const libraries = libraryService.store;
	const libraryState = libraryService.state;

	// Library selection
	let selectedLibraryId: string = '';
	let showInlineAddForm = false;
	let previousLibraryCount = 0;

	// Debug info
	let showDebug = false;
	let debugLogs: string[] = [];
	let loadingDebug = false;

	// Clear storage confirmation
	let confirmClear = false;

	// Auto-select library matching current libraryId, or first library if none matches
	$: if ($libraries.length > 0) {
		if ($state.libraryId) {
			const match = $libraries.find((lib: Library) => String(lib.id) === $state.libraryId);
			if (match) {
				selectedLibraryId = String(match.id);
			}
		}
		if (!selectedLibraryId) {
			const first = $libraries[0];
			selectedLibraryId = String(first.id);
			torrentService.setLibrary(String(first.id));
		}
	}

	// Detect when inline add form closes (cancel or successful add)
	$: if (showInlineAddForm && !$libraryState.showAddForm) {
		showInlineAddForm = false;
		// If a new library was added, auto-select it
		if ($libraries.length > previousLibraryCount) {
			const newest = $libraries[$libraries.length - 1];
			selectedLibraryId = String(newest.id);
			torrentService.setLibrary(String(newest.id));
		}
	}

	$: previousLibraryCount = $libraries.length;

	async function handleLibrarySelect(event: Event) {
		const target = event.target as HTMLSelectElement;
		const libraryId = target.value;
		if (!libraryId) return;

		const library = $libraries.find((lib: Library) => String(lib.id) === libraryId);
		if (library) {
			selectedLibraryId = String(library.id);
			await torrentService.setLibrary(String(library.id));
		}
	}

	function handleShowAddForm() {
		showInlineAddForm = true;
		libraryService.openAddForm();
	}

	async function handleFetchDebug() {
		loadingDebug = true;
		debugLogs = await torrentService.getDebugInfo();
		loadingDebug = false;
	}

	async function handleClearStorage() {
		if (!confirmClear) {
			confirmClear = true;
			return;
		}
		await torrentService.clearStorage();
		confirmClear = false;
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<h2 class="card-title text-lg">Settings</h2>

		<!-- Connection Status -->
		<div
			class={classNames('rounded-lg p-3', {
				'bg-success/10': $state.initialized,
				'bg-warning/10': !$state.initialized
			})}
		>
			<div class="flex items-center gap-2">
				<div
					class={classNames('h-2 w-2 rounded-full', {
						'bg-success': $state.initialized,
						'bg-warning': !$state.initialized
					})}
				></div>
				<span class="text-sm font-medium">
					{#if $state.initialized}
						Server Connected
					{:else}
						Server Disconnected
					{/if}
				</span>
			</div>
		</div>

		<!-- Download Library -->
		<div class="form-control">
			<label class="label" for="library-select">
				<span class="label-text">Download Library</span>
			</label>

			{#if $libraries.length > 0}
				<div class="flex items-center gap-2">
					<select
						id="library-select"
						class="select select-bordered flex-1"
						value={selectedLibraryId}
						on:change={handleLibrarySelect}
					>
						<option value="" disabled>Select a library...</option>
						{#each $libraries as library (library.id)}
							<option value={String(library.id)}>
								{library.name}
							</option>
						{/each}
					</select>
					<button
						class="btn btn-ghost btn-sm"
						on:click={handleShowAddForm}
						title="Add new library"
					>
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
					<button class="btn btn-primary btn-sm" on:click={handleShowAddForm}>
						Create Library
					</button>
				</div>
			{/if}
		</div>

		<!-- Inline Library Add Form -->
		{#if showInlineAddForm && $libraryState.showAddForm}
			<LibraryAddForm />
		{/if}

		<!-- Clear Storage -->
		<div class="form-control">
			<button
				class={classNames('btn btn-sm', {
					'btn-error': confirmClear,
					'btn-outline': !confirmClear
				})}
				on:click={handleClearStorage}
			>
				{#if confirmClear}
					Confirm Clear Storage
				{:else}
					Clear Storage
				{/if}
			</button>
			{#if confirmClear}
				<span class="label">
					<span class="label-text-alt text-warning"
						>This will delete all downloaded files and persistence data</span
					>
				</span>
			{/if}
		</div>

		<!-- Debug Info (Collapsible) -->
		<div class="divider my-1"></div>
		<button
			class="flex w-full items-center justify-between text-sm text-base-content/70 hover:text-base-content"
			on:click={() => {
				showDebug = !showDebug;
				if (showDebug && debugLogs.length === 0) {
					handleFetchDebug();
				}
			}}
		>
			<span>Debug Info</span>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4 transition-transform"
				class:rotate-180={showDebug}
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 9l-7 7-7-7"
				/>
			</svg>
		</button>

		{#if showDebug}
			<div class="mt-2 flex flex-col gap-2">
				<button
					class="btn btn-ghost btn-xs self-end"
					on:click={handleFetchDebug}
					disabled={loadingDebug}
				>
					{#if loadingDebug}
						<span class="loading loading-spinner loading-xs"></span>
					{:else}
						Refresh
					{/if}
				</button>
				<div
					class="max-h-64 overflow-auto rounded-lg bg-base-300 p-3 font-mono text-xs"
				>
					{#if debugLogs.length === 0}
						<p class="text-base-content/50">No debug info available</p>
					{:else}
						{#each debugLogs as line}
							<p class="whitespace-pre-wrap">{line}</p>
						{/each}
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

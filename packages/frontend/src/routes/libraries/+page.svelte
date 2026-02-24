<script lang="ts">
	import { libraryService } from '$services/library.service';
	import LibraryList from '$components/libraries/LibraryList.svelte';
	import LibraryAddForm from '$components/libraries/LibraryAddForm.svelte';

	const state = libraryService.state;
</script>

<div class="flex flex-col gap-6 p-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Libraries</h1>
			<p class="text-sm text-base-content/60">
				Manage media library locations on your server
			</p>
		</div>
		{#if !$state.showAddForm}
			<button
				class="btn btn-primary btn-sm"
				onclick={() => libraryService.openAddForm()}
			>
				Add Library
			</button>
		{/if}
	</div>

	<!-- Add Form -->
	{#if $state.showAddForm}
		<LibraryAddForm />
	{/if}

	<!-- Library List -->
	<LibraryList />
</div>

<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { libraryService } from '$services/library.service';
	import LibraryAddForm from './LibraryAddForm.svelte';
	import LibraryList from './LibraryList.svelte';

	let activeTab: 'add' | 'libraries' = $state('libraries');

	onMount(() => {
		libraryService.initialize();
	});
</script>

<div class="flex items-center justify-between pr-8">
	<div>
		<h3 class="text-lg font-bold">Libraries</h3>
		<p class="text-sm text-base-content/60">
			Manage media library locations on your server
		</p>
	</div>
</div>

<div class="mt-4 flex gap-2">
	<div class="join">
		<button
			class={classNames('join-item btn btn-sm', {
				'btn-active': activeTab === 'libraries'
			})}
			onclick={() => (activeTab = 'libraries')}
		>
			Libraries
		</button>
		<button
			class={classNames('join-item btn btn-sm', {
				'btn-active': activeTab === 'add'
			})}
			onclick={() => (activeTab = 'add')}
		>
			Add Library
		</button>
	</div>
</div>

<div class="mt-4">
	{#if activeTab === 'add'}
		<LibraryAddForm />
	{:else}
		<LibraryList />
	{/if}
</div>

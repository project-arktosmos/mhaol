<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { libraryService } from 'ui-lib/services/library.service';
	import type { LibraryType } from 'ui-lib/types/library.type';
	import LibraryAddForm from './LibraryAddForm.svelte';
	import LibraryList from './LibraryList.svelte';

	let {
		fixedCategory = null
	}: {
		fixedCategory?: LibraryType | null;
	} = $props();

	let activeTab: 'add' | 'libraries' = $state('libraries');

	onMount(() => {
		libraryService.initialize();
	});

	function switchTab(tab: 'add' | 'libraries') {
		activeTab = tab;
		if (tab === 'add') {
			libraryService.openAddForm();
		}
	}
</script>

<div class="flex gap-2">
	<div class="join">
		<button
			class={classNames('btn join-item btn-sm', {
				'btn-active': activeTab === 'libraries'
			})}
			onclick={() => switchTab('libraries')}
		>
			Libraries
		</button>
		<button
			class={classNames('btn join-item btn-sm', {
				'btn-active': activeTab === 'add'
			})}
			onclick={() => switchTab('add')}
		>
			Add Library
		</button>
	</div>
</div>

<div class="mt-4">
	{#if activeTab === 'add'}
		<LibraryAddForm {fixedCategory} />
	{:else}
		<LibraryList />
	{/if}
</div>

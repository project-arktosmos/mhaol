<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import { TorrentCategory, TORRENT_CATEGORY_LABELS } from 'addons/torrent-search-thepiratebay/types';

	export let query: string = '';
	export let category: TorrentCategory = TorrentCategory.All;
	export let searching: boolean = false;

	const dispatch = createEventDispatcher<{
		search: { query: string; category: TorrentCategory };
	}>();

	const categories = Object.entries(TORRENT_CATEGORY_LABELS) as [TorrentCategory, string][];

	$: canSearch = query.trim().length > 0 && !searching;

	function handleSubmit() {
		if (canSearch) {
			dispatch('search', { query: query.trim(), category });
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSubmit();
		}
	}
</script>

<div class="join w-full">
	<select class="select-bordered select join-item" bind:value={category}>
		{#each categories as [value, label]}
			<option {value}>{label}</option>
		{/each}
	</select>
	<input
		type="text"
		bind:value={query}
		on:keydown={handleKeydown}
		placeholder="Search torrents..."
		class="input-bordered input join-item flex-1"
	/>
	<button
		class={classNames('btn join-item btn-primary', { 'btn-disabled': !canSearch })}
		on:click={handleSubmit}
		disabled={!canSearch}
	>
		{#if searching}
			<span class="loading loading-sm loading-spinner"></span>
		{:else}
			Search
		{/if}
	</button>
</div>

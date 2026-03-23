<script lang="ts">
	import classNames from 'classnames';
	import {
		TorrentCategory,
		TORRENT_CATEGORY_LABELS
	} from 'addons/torrent-search-thepiratebay/types';

	let {
		query = $bindable(''),
		category = $bindable(TorrentCategory.All),
		searching = false,
		onsearch
	}: {
		query?: string;
		category?: TorrentCategory;
		searching?: boolean;
		onsearch?: (detail: { query: string; category: TorrentCategory }) => void;
	} = $props();

	const categories = Object.entries(TORRENT_CATEGORY_LABELS) as [TorrentCategory, string][];

	let canSearch = $derived(query.trim().length > 0 && !searching);

	function handleSubmit() {
		if (canSearch) {
			onsearch?.({ query: query.trim(), category });
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
		onkeydown={handleKeydown}
		placeholder="Search torrents..."
		class="input-bordered input join-item flex-1"
	/>
	<button
		class={classNames('btn join-item btn-primary', { 'btn-disabled': !canSearch })}
		onclick={handleSubmit}
		disabled={!canSearch}
	>
		{#if searching}
			<span class="loading loading-sm loading-spinner"></span>
		{:else}
			Search
		{/if}
	</button>
</div>

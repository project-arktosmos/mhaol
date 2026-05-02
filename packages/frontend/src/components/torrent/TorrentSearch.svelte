<script lang="ts">
	import { torrentSearchService } from '$services/torrent-search.service';
	import { torrentService } from '$services/torrent.service';
	import TorrentSearchInput from './TorrentSearchInput.svelte';
	import TorrentSearchResults from './TorrentSearchResults.svelte';
	import type {
		TorrentCategory,
		TorrentSearchSortField
	} from 'addons/torrent-search-thepiratebay/types';

	const searchState = torrentSearchService.state;
	const torrentState = torrentService.state;

	let { onadded }: { onadded?: () => void } = $props();

	let canAddTorrents = $derived($torrentState.initialized);

	let hasResults = $derived($searchState.results.length > 0);
	let showResults = $derived(hasResults || $searchState.error);

	async function handleSearch(detail: { query: string; category: TorrentCategory }) {
		await torrentSearchService.search(detail.query, detail.category);
	}

	function handleSort(detail: { field: TorrentSearchSortField }) {
		torrentSearchService.toggleSort(detail.field);
	}

	async function handleAdd(detail: { magnetLink: string; infoHash: string; name: string }) {
		const { magnetLink, infoHash } = detail;
		torrentSearchService.markAdding(infoHash);
		const result = await torrentService.addTorrent(magnetLink);
		torrentSearchService.unmarkAdding(infoHash);
		if (result) onadded?.();
	}
</script>

<div class="flex flex-col gap-4">
	{#if hasResults}
		<div class="flex justify-end">
			<button class="btn btn-ghost btn-sm" onclick={() => torrentSearchService.clearResults()}>
				Clear
			</button>
		</div>
	{/if}

	<TorrentSearchInput
		bind:query={$searchState.query}
		bind:category={$searchState.category}
		searching={$searchState.searching}
		onsearch={handleSearch}
	/>

	{#if $searchState.error}
		<div class="alert-sm alert alert-error">
			<span>{$searchState.error}</span>
		</div>
	{/if}

	{#if $searchState.searching}
		<div class="flex justify-center py-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if showResults}
		<TorrentSearchResults
			results={$searchState.results}
			sort={$searchState.sort}
			addingTorrents={$searchState.addingTorrents}
			disableAdd={!canAddTorrents}
			onadd={handleAdd}
			onsort={handleSort}
		/>
	{/if}
</div>

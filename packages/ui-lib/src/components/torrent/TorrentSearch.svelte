<script lang="ts">
	import { torrentSearchService } from 'ui-lib/services/torrent-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import TorrentSearchInput from './TorrentSearchInput.svelte';
	import TorrentSearchResults from './TorrentSearchResults.svelte';
	import type {
		TorrentCategory,
		TorrentSearchSortField
	} from 'addons/torrent-search-thepiratebay/types';

	const searchState = torrentSearchService.state;
	const torrentState = torrentService.state;

	export let onadded: (() => void) | undefined = undefined;

	$: canAddTorrents = $torrentState.initialized;

	$: hasResults = $searchState.results.length > 0;
	$: showResults = hasResults || $searchState.error;

	async function handleSearch(event: CustomEvent<{ query: string; category: TorrentCategory }>) {
		await torrentSearchService.search(event.detail.query, event.detail.category);
	}

	function handleSort(event: CustomEvent<{ field: TorrentSearchSortField }>) {
		torrentSearchService.toggleSort(event.detail.field);
	}

	async function handleAdd(
		event: CustomEvent<{ magnetLink: string; infoHash: string; name: string }>
	) {
		const { magnetLink, infoHash } = event.detail;
		torrentSearchService.markAdding(infoHash);
		const result = await torrentService.addTorrent(magnetLink);
		torrentSearchService.unmarkAdding(infoHash);
		if (result) onadded?.();
	}
</script>

<div class="flex flex-col gap-4">
	{#if hasResults}
		<div class="flex justify-end">
			<button class="btn btn-ghost btn-sm" on:click={() => torrentSearchService.clearResults()}>
				Clear
			</button>
		</div>
	{/if}

	<TorrentSearchInput
		bind:query={$searchState.query}
		bind:category={$searchState.category}
		searching={$searchState.searching}
		on:search={handleSearch}
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
			on:add={handleAdd}
			on:sort={handleSort}
		/>
	{/if}
</div>

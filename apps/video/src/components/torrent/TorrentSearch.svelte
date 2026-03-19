<script lang="ts">
	import { torrentSearchService } from '$services/torrent-search.service';
	import { torrentService } from '$services/torrent.service';
	import TorrentSearchInput from './TorrentSearchInput.svelte';
	import TorrentSearchResults from './TorrentSearchResults.svelte';
	import type { TorrentCategory, TorrentSearchSortField } from 'addons/torrent-search-thepiratebay/types';

	const searchState = torrentSearchService.state;
	const torrentState = torrentService.state;

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
		await torrentService.addTorrent(magnetLink);
		torrentSearchService.unmarkAdding(infoHash);
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-lg">Search Torrents</h2>
			{#if hasResults}
				<button class="btn btn-ghost btn-sm" on:click={() => torrentSearchService.clearResults()}>
					Clear
				</button>
			{/if}
		</div>

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
</div>

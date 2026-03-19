<script lang="ts">
	import classNames from 'classnames';
	import { onMount } from 'svelte';
	import {
		jackettSearchService,
		JackettCategory,
		JACKETT_CATEGORY_LABELS,
		type JackettSearchResult,
		type JackettSortField
	} from 'frontend/services/jackett-search.service';
	import { torrentService } from 'frontend/services/torrent.service';
	import {
		formatSearchSize,
		formatSeeders,
		getSeedersColor,
		formatUploadDate
	} from 'frontend/utils/torrent-search/format';

	const searchState = jackettSearchService.state;
	const torrentState = torrentService.state;

	let query = $state('');
	let category = $state<JackettCategory>(JackettCategory.All);
	let tracker = $state('');

	let canAddTorrents = $derived($torrentState.initialized);
	let hasResults = $derived($searchState.results.length > 0);
	let sortedResults = $derived(sortResults($searchState.results, $searchState.sort));

	const categories = Object.entries(JACKETT_CATEGORY_LABELS) as [JackettCategory, string][];

	function sortResults(
		results: JackettSearchResult[],
		sort: { field: JackettSortField; direction: 'asc' | 'desc' }
	): JackettSearchResult[] {
		const sorted = [...results].sort((a, b) => {
			const aVal = a[sort.field];
			const bVal = b[sort.field];
			if (typeof aVal === 'string' && typeof bVal === 'string') {
				return aVal.localeCompare(bVal);
			}
			return (aVal as number) - (bVal as number);
		});
		return sort.direction === 'desc' ? sorted.reverse() : sorted;
	}

	function handleSubmit() {
		if (query.trim()) {
			jackettSearchService.search(query, { category, tracker });
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSubmit();
		}
	}

	function handleSort(field: JackettSortField) {
		jackettSearchService.toggleSort(field);
	}

	async function handleAdd(result: JackettSearchResult) {
		jackettSearchService.markAdding(result.infoHash);
		await torrentService.addTorrent(result.magnetLink);
		jackettSearchService.unmarkAdding(result.infoHash);
	}

	function getSortIndicator(field: JackettSortField): string {
		if ($searchState.sort.field !== field) return '';
		return $searchState.sort.direction === 'asc' ? ' \u25B2' : ' \u25BC';
	}

	onMount(() => {
		torrentService.initialize();
	});
</script>

<div class="flex items-center justify-between pr-8">
	<div>
		<h3 class="text-lg font-bold">Jackett Search</h3>
		<p class="text-sm text-base-content/60">Search torrents across multiple indexers via Jackett</p>
	</div>
</div>

<div class="mt-4 flex flex-col gap-4">
	<!-- Search input -->
	<div class="join w-full">
		<input
			type="text"
			bind:value={query}
			onkeydown={handleKeydown}
			placeholder="Search Jackett..."
			class="input-bordered input join-item flex-1"
		/>
		<button
			class={classNames('btn join-item btn-primary', {
				'btn-disabled': !query.trim() || $searchState.searching
			})}
			onclick={handleSubmit}
			disabled={!query.trim() || $searchState.searching}
		>
			{#if $searchState.searching}
				<span class="loading loading-sm loading-spinner"></span>
			{:else}
				Search
			{/if}
		</button>
	</div>

	<!-- Filters -->
	<div class="flex flex-wrap gap-2">
		<select class="select-bordered select select-sm" bind:value={category}>
			{#each categories as [value, label] (value)}
				<option {value}>{label}</option>
			{/each}
		</select>

		<select class="select-bordered select select-sm" bind:value={tracker}>
			<option value="">All indexers</option>
			{#each $searchState.indexers as indexer (indexer.id)}
				<option value={indexer.id}>{indexer.name}</option>
			{/each}
		</select>
	</div>

	<!-- Error -->
	{#if $searchState.error}
		<div class="alert-sm alert alert-error">
			<span>{$searchState.error}</span>
		</div>
	{/if}

	<!-- Loading -->
	{#if $searchState.searching}
		<div class="flex justify-center py-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if hasResults}
		<!-- Results header -->
		<div class="flex items-center justify-between">
			<p class="text-xs text-base-content/40">
				{$searchState.results.length} result{$searchState.results.length !== 1 ? 's' : ''}
			</p>
			<button class="btn btn-ghost btn-sm" onclick={() => jackettSearchService.clearResults()}>
				Clear
			</button>
		</div>

		<!-- Results table -->
		<div class="overflow-x-auto">
			<table class="table table-sm">
				<thead>
					<tr>
						<th class="cursor-pointer hover:bg-base-300" onclick={() => handleSort('name')}>
							Name{getSortIndicator('name')}
						</th>
						<th
							class="w-24 cursor-pointer text-right hover:bg-base-300"
							onclick={() => handleSort('size')}
						>
							Size{getSortIndicator('size')}
						</th>
						<th
							class="w-20 cursor-pointer text-right hover:bg-base-300"
							onclick={() => handleSort('seeders')}
						>
							SE{getSortIndicator('seeders')}
						</th>
						<th
							class="w-20 cursor-pointer text-right hover:bg-base-300"
							onclick={() => handleSort('leechers')}
						>
							LE{getSortIndicator('leechers')}
						</th>
						<th
							class="hidden w-24 cursor-pointer text-right hover:bg-base-300 md:table-cell"
							onclick={() => handleSort('uploadedAt')}
						>
							Uploaded{getSortIndicator('uploadedAt')}
						</th>
						<th class="w-20 text-right">Action</th>
					</tr>
				</thead>
				<tbody>
					{#each sortedResults as result (result.id)}
						{@const isAdding = $searchState.addingTorrents.has(result.infoHash)}
						<tr class="hover">
							<td class="max-w-xs">
								<div class="flex flex-col gap-0.5">
									<span class="truncate" title={result.name}>
										{result.name}
									</span>
									{#if result.tracker}
										<span class="truncate text-xs text-base-content/40">
											{result.tracker}
											{#if result.category}
												&middot; {result.category}
											{/if}
										</span>
									{/if}
								</div>
							</td>
							<td class="text-right text-nowrap">{formatSearchSize(result.size)}</td>
							<td class={classNames('text-right font-medium', getSeedersColor(result.seeders))}>
								{formatSeeders(result.seeders)}
							</td>
							<td class="text-right text-base-content/60">
								{formatSeeders(result.leechers)}
							</td>
							<td class="hidden text-right text-nowrap text-base-content/60 md:table-cell">
								{formatUploadDate(new Date(result.uploadedAt * 1000))}
							</td>
							<td class="text-right">
								<button
									class="btn btn-xs btn-primary"
									onclick={() => handleAdd(result)}
									disabled={isAdding || !canAddTorrents}
								>
									{#if isAdding}
										<span class="loading loading-xs loading-spinner"></span>
									{:else}
										Add
									{/if}
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else if $searchState.query && !$searchState.searching && !$searchState.error}
		<div class="py-8 text-center text-base-content/50">
			<p>No results found</p>
		</div>
	{/if}
</div>

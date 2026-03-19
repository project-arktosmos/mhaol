<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import type {
		TorrentSearchResult,
		TorrentSearchSort,
		TorrentSearchSortField
	} from 'addons/torrent-search-thepiratebay/types';
	import { sortSearchResults } from 'torrent-search-thepiratebay';
	import {
		formatSearchSize,
		formatSeeders,
		getSeedersColor,
		formatUploadDate
	} from 'frontend/utils/torrent-search/format';

	export let results: TorrentSearchResult[] = [];
	export let sort: TorrentSearchSort;
	export let addingTorrents: Set<string> = new Set();
	export let disableAdd: boolean = false;

	const dispatch = createEventDispatcher<{
		add: { magnetLink: string; infoHash: string; name: string };
		sort: { field: TorrentSearchSortField };
	}>();

	$: sortedResults = sortSearchResults(results, sort);

	function handleSort(field: TorrentSearchSortField) {
		dispatch('sort', { field });
	}

	function handleAdd(result: TorrentSearchResult) {
		dispatch('add', {
			magnetLink: result.magnetLink,
			infoHash: result.infoHash,
			name: result.name
		});
	}

	function getSortIndicator(field: TorrentSearchSortField): string {
		if (sort.field !== field) return '';
		return sort.direction === 'asc' ? ' ▲' : ' ▼';
	}
</script>

{#if results.length === 0}
	<div class="py-8 text-center text-base-content/50">
		<p>No results found</p>
	</div>
{:else}
	<div class="overflow-x-auto">
		<table class="table table-sm">
			<thead>
				<tr>
					<th class="cursor-pointer hover:bg-base-300" on:click={() => handleSort('name')}>
						Name{getSortIndicator('name')}
					</th>
					<th
						class="w-24 cursor-pointer text-right hover:bg-base-300"
						on:click={() => handleSort('size')}
					>
						Size{getSortIndicator('size')}
					</th>
					<th
						class="w-20 cursor-pointer text-right hover:bg-base-300"
						on:click={() => handleSort('seeders')}
					>
						SE{getSortIndicator('seeders')}
					</th>
					<th
						class="w-20 cursor-pointer text-right hover:bg-base-300"
						on:click={() => handleSort('leechers')}
					>
						LE{getSortIndicator('leechers')}
					</th>
					<th
						class="hidden w-24 cursor-pointer text-right hover:bg-base-300 md:table-cell"
						on:click={() => handleSort('uploadedAt')}
					>
						Uploaded{getSortIndicator('uploadedAt')}
					</th>
					<th class="w-20 text-right">Action</th>
				</tr>
			</thead>
			<tbody>
				{#each sortedResults as result (result.id)}
					{@const isAdding = addingTorrents.has(result.infoHash)}
					<tr class="hover">
						<td class="max-w-xs">
							<div class="flex items-center gap-2">
								{#if result.isVip}
									<span class="badge badge-xs badge-warning" title="VIP">V</span>
								{:else if result.isTrusted}
									<span class="badge badge-xs badge-success" title="Trusted">T</span>
								{/if}
								<span class="truncate" title={result.name}>
									{result.name}
								</span>
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
							{formatUploadDate(result.uploadedAt)}
						</td>
						<td class="text-right">
							<button
								class="btn btn-xs btn-primary"
								on:click={() => handleAdd(result)}
								disabled={isAdding || disableAdd}
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
	<p class="text-xs text-base-content/40">
		{results.length} result{results.length !== 1 ? 's' : ''}
	</p>
{/if}

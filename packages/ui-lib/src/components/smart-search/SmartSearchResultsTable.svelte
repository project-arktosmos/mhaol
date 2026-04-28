<script lang="ts">
	import classNames from 'classnames';
	import { formatSearchSize, formatSeeders } from 'addons/torrent-search-thepiratebay/format';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';
	import type { ScoredResult } from 'ui-lib/utils/smart-search/score';
	import type { Snippet } from 'svelte';

	let {
		scoredResults,
		bestCandidate,
		columns,
		extraHeadColumns,
		extraRowColumns
	}: {
		scoredResults: ScoredResult[];
		bestCandidate: SmartSearchTorrentResult | null;
		columns: Array<{
			key: string;
			label: string;
			title?: string;
			getBonusValue: (scored: ScoredResult) => number;
			getTitle?: (scored: ScoredResult) => string;
		}>;
		extraHeadColumns?: Snippet;
		extraRowColumns?: Snippet<[ScoredResult]>;
	} = $props();

	const PAGE_SIZE = 10;
	let currentPage = $state(0);
	let totalPages = $derived(Math.max(1, Math.ceil(scoredResults.length / PAGE_SIZE)));
	let pagedResults = $derived(
		scoredResults.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE)
	);

	let bonusColCount = $derived(columns.length);

	$effect(() => {
		if (scoredResults.length) currentPage = 0;
	});
</script>

<div class="overflow-x-auto">
	<table class="table w-full table-xs">
		<thead>
			<tr>
				<th>Name</th>
				{#if extraHeadColumns}
					{@render extraHeadColumns()}
				{/if}
				<th class="text-right">Size</th>
				<th class="text-right" title="Seeders normalized to %">SE%</th>
				<th class="text-right" title="Leechers normalized to %">LE%</th>
				<th class="text-right" title="Relevance %">Rel%</th>
				{#each columns as col (col.key)}
					<th class="text-right" title={col.title}>{col.label}</th>
				{/each}
				<th class="text-right font-semibold">Score</th>
			</tr>
		</thead>
		<tbody>
			{#each pagedResults as scored (scored.result.infoHash)}
				{@const isBest = bestCandidate?.infoHash === scored.result.infoHash}
				<tr
					class={classNames({
						'border-l-2 border-l-primary bg-primary/15': isBest,
						hover: !isBest
					})}
				>
					<td class="max-w-xs">
						<div class="flex items-center gap-1">
							{#if scored.result.isVip}
								<span class="badge badge-xs badge-warning" title="VIP">V</span>
							{:else if scored.result.isTrusted}
								<span class="badge badge-xs badge-success" title="Trusted">T</span>
							{/if}
							<span class="truncate" title={scored.result.name}>{scored.result.name}</span>
						</div>
					</td>
					{#if extraRowColumns}
						{@render extraRowColumns(scored)}
					{/if}
					<td class="text-right text-nowrap">{formatSearchSize(scored.result.size)}</td>
					<td class="text-right" title="{formatSeeders(scored.result.seeders)} seeders">
						{scored.seedersPct}%
					</td>
					<td
						class="text-right text-base-content/60"
						title="{formatSeeders(scored.result.leechers)} leechers"
					>
						{scored.leechersPct}%
					</td>
					{#if scored.result.analyzing}
						<td colspan={bonusColCount + 2} class="text-center">
							<span class="loading loading-xs loading-spinner"></span>
						</td>
					{:else if scored.result.analysis}
						<td
							class={classNames('text-right text-xs font-medium', {
								'text-success': scored.relPct >= 80,
								'text-warning': scored.relPct >= 50 && scored.relPct < 80,
								'text-error': scored.relPct < 50
							})}
							title={scored.result.analysis.reason}
						>
							{scored.relPct}%
						</td>
						{#each columns as col (col.key)}
							{@const bonusValue = col.getBonusValue(scored)}
							<td
								class={classNames('text-right text-xs', {
									'text-success': bonusValue > 0,
									'text-base-content/30': bonusValue === 0
								})}
								title={col.getTitle ? col.getTitle(scored) : undefined}
							>
								{bonusValue > 0 ? '+100' : '0'}
							</td>
						{/each}
						<td
							class={classNames('text-right text-xs font-bold', {
								'text-success': scored.score >= 350,
								'text-warning': scored.score >= 200 && scored.score < 350,
								'text-error': scored.score < 200
							})}
						>
							{scored.score}
						</td>
					{:else}
						<td colspan={bonusColCount + 2}></td>
					{/if}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
{#if totalPages > 1}
	<div class="mt-2 flex items-center justify-between">
		<span class="text-xs text-base-content/50">
			Page {currentPage + 1} of {totalPages}
		</span>
		<div class="join">
			<button
				class="btn join-item btn-xs"
				disabled={currentPage === 0}
				onclick={() => (currentPage = currentPage - 1)}
			>
				&laquo;
			</button>
			{#each Array(totalPages) as _, i}
				<button
					class={classNames('btn join-item btn-xs', {
						'btn-active': i === currentPage
					})}
					onclick={() => (currentPage = i)}
				>
					{i + 1}
				</button>
			{/each}
			<button
				class="btn join-item btn-xs"
				disabled={currentPage === totalPages - 1}
				onclick={() => (currentPage = currentPage + 1)}
			>
				&raquo;
			</button>
		</div>
	</div>
{/if}

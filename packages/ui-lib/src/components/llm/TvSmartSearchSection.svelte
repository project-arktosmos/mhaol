<script lang="ts">
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import SmartSearchResultsTable from 'ui-lib/components/llm/SmartSearchResultsTable.svelte';
	import {
		scoreResults,
		findBestCandidate,
		type ScoredResult
	} from 'ui-lib/utils/smart-search/score';
	import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';

	const searchStore = smartSearchService.store;
	const configStore = smartSearchService.configStore;

	let selection = $derived($searchStore.selection);
	let mediaConfig = $derived(selection ? $configStore['tv'] : null);
	let preferredLanguage = $derived(mediaConfig?.preferredLanguage ?? '');
	let preferredQuality = $derived(mediaConfig?.preferredQuality ?? '');

	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);
	let searchError = $derived($searchStore.searchError);
	let tvResults = $derived($searchStore.tvResults);
	let tvSeasonsMeta = $derived($searchStore.tvSeasonsMeta);
	let activeTvTab = $derived($searchStore.activeTvTab);

	let tabs = $derived.by(() => {
		const t: Array<{ id: 'complete' | number; label: string }> = [
			{ id: 'complete', label: 'Complete' }
		];
		if (tvSeasonsMeta) {
			for (const s of tvSeasonsMeta) {
				if (s.seasonNumber > 0) {
					t.push({ id: s.seasonNumber, label: `S${String(s.seasonNumber).padStart(2, '0')}` });
				}
			}
		} else if (tvResults) {
			const seasonNums = Object.keys(tvResults.seasons)
				.map(Number)
				.sort((a, b) => a - b);
			for (const sn of seasonNums) {
				t.push({ id: sn, label: `S${String(sn).padStart(2, '0')}` });
			}
		}
		return t;
	});

	let activeResults = $derived.by((): SmartSearchTorrentResult[] => {
		if (!tvResults) return [];
		if (activeTvTab === 'complete') return tvResults.complete;
		const seasonData = tvResults.seasons[activeTvTab];
		if (!seasonData) return [];
		const all = [...seasonData.seasonPacks];
		const episodeNums = Object.keys(seasonData.episodes)
			.map(Number)
			.sort((a, b) => a - b);
		for (const en of episodeNums) {
			all.push(...seasonData.episodes[en]);
		}
		return all;
	});

	let scoredResults = $derived(
		scoreResults(activeResults, {
			preferredLanguage,
			preferredQuality,
			preferredConsole: ''
		})
	);

	let bestCandidate = $derived(findBestCandidate(scoredResults, { analyzing, searching }));

	let tabCounts = $derived.by(() => {
		const counts: Record<string, number> = { complete: 0 };
		if (!tvResults) return counts;
		counts.complete = tvResults.complete.length;
		for (const [sn, data] of Object.entries(tvResults.seasons)) {
			let total = data.seasonPacks.length;
			for (const eps of Object.values(data.episodes)) {
				total += eps.length;
			}
			counts[sn] = total;
		}
		return counts;
	});

	let totalResultCount = $derived(
		tvResults
			? tvResults.complete.length +
					Object.values(tvResults.seasons).reduce((sum, s) => {
						return (
							sum +
							s.seasonPacks.length +
							Object.values(s.episodes).reduce((es, e) => es + e.length, 0)
						);
					}, 0)
			: 0
	);

	let columns = $derived([
		{
			key: 'lang',
			label: 'Lang',
			title: `+100 if language matches ${preferredLanguage}`,
			getBonusValue: (s: ScoredResult) => s.langBonus,
			getTitle: (s: ScoredResult) => s.result.analysis?.languages ?? ''
		},
		{
			key: 'quality',
			label: 'Quality',
			title: `+100 if quality matches ${preferredQuality}`,
			getBonusValue: (s: ScoredResult) => s.qualityBonus,
			getTitle: (s: ScoredResult) => s.result.analysis?.quality ?? ''
		}
	]);

	function formatSeEp(result: SmartSearchTorrentResult): string {
		const a = result.analysis;
		if (!a) return '';
		if (a.isCompleteSeries) return 'Complete';
		if (a.seasonNumber != null && a.episodeNumber != null) {
			return `S${String(a.seasonNumber).padStart(2, '0')}E${String(a.episodeNumber).padStart(2, '0')}`;
		}
		if (a.seasonNumber != null) {
			return `S${String(a.seasonNumber).padStart(2, '0')} Pack`;
		}
		return '';
	}
</script>

{#snippet scopeHead()}
	<th class="text-right">Scope</th>
{/snippet}

{#snippet scopeRow(scored: ScoredResult)}
	<td class="text-right text-xs text-nowrap text-base-content/60">
		{formatSeEp(scored.result)}
	</td>
{/snippet}

{#if selection}
	<div class="mb-3 rounded bg-base-100 p-2">
		<div class="flex items-center justify-between">
			<div class="min-w-0 flex-1">
				<div class="truncate text-xs font-semibold">{selection.title}</div>
				<div class="flex items-center gap-1 text-xs text-base-content/50">
					<span>{selection.year}</span>
					<span class="badge badge-xs badge-info">TV</span>
					{#if tvSeasonsMeta}
						<span class="text-base-content/30">
							{tvSeasonsMeta.length} season{tvSeasonsMeta.length !== 1 ? 's' : ''}
						</span>
					{/if}
				</div>
			</div>
			<button class="btn btn-ghost btn-xs" onclick={() => smartSearchService.clear()}>
				&times;
			</button>
		</div>
	</div>

	{#if searching}
		<div class="mt-3 flex items-center justify-center gap-2 py-4">
			<span class="loading loading-sm loading-spinner"></span>
			<span class="text-xs text-base-content/50">Searching torrents for TV show...</span>
		</div>
	{:else if searchError}
		<div class="mt-3 rounded bg-error/10 p-2 text-xs text-error">{searchError}</div>
	{:else if totalResultCount > 0}
		<div class="mt-3">
			<div class="mb-2 flex items-center justify-between">
				<span class="text-xs font-semibold text-base-content/50">
					{totalResultCount} result{totalResultCount !== 1 ? 's' : ''} found
					{#if analyzing}
						<span class="loading ml-1 loading-xs loading-spinner"></span>
					{/if}
				</span>
			</div>

			<!-- Tab bar -->
			<div class="tabs-boxed mb-3 tabs flex-wrap gap-1 bg-base-200 p-1">
				{#each tabs as tab (tab.id)}
					{@const count = tabCounts[String(tab.id)] ?? 0}
					<button
						class={classNames('tab-sm tab', {
							'tab-active': activeTvTab === tab.id
						})}
						onclick={() => smartSearchService.setActiveTvTab(tab.id)}
					>
						{tab.label}
						{#if count > 0}
							<span class="ml-1 badge badge-xs">{count}</span>
						{/if}
					</button>
				{/each}
			</div>

			<!-- Results table -->
			{#if activeResults.length === 0}
				<div class="py-4 text-center text-xs text-base-content/40">
					No torrents found for this {activeTvTab === 'complete' ? 'complete series' : `season`}
				</div>
			{:else}
				<SmartSearchResultsTable
					{scoredResults}
					{bestCandidate}
					{columns}
					extraHeadColumns={scopeHead}
					extraRowColumns={scopeRow}
				/>
			{/if}
		</div>
	{/if}
{/if}

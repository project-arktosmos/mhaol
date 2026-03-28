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
	let musicSelection = $derived(selection?.type === 'music' ? selection : null);
	let mediaConfig = $derived(selection ? $configStore['music'] : null);
	let preferredQuality = $derived(mediaConfig?.preferredQuality ?? '');

	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);
	let searchError = $derived($searchStore.searchError);
	let musicResults = $derived($searchStore.musicResults);
	let activeMusicTab = $derived($searchStore.activeMusicTab);

	let tabs = $derived.by(() => {
		const t: Array<{ id: 'album' | 'discography'; label: string }> = [];
		t.push({ id: 'album', label: 'Album' });
		t.push({ id: 'discography', label: 'Discography' });
		return t;
	});

	let activeResults = $derived.by((): SmartSearchTorrentResult[] => {
		if (!musicResults) return [];
		if (activeMusicTab === 'album') return musicResults.album;
		return musicResults.discography;
	});

	let scoredResults = $derived(
		scoreResults(activeResults, {
			preferredLanguage: '',
			preferredQuality,
			preferredConsole: ''
		})
	);

	let bestCandidate = $derived(findBestCandidate(scoredResults, { analyzing, searching }));

	let tabCounts = $derived.by(() => {
		const counts: Record<string, number> = { album: 0, discography: 0 };
		if (!musicResults) return counts;
		counts.album = musicResults.album.length;
		counts.discography = musicResults.discography.length;
		return counts;
	});

	let totalResultCount = $derived(
		musicResults ? musicResults.album.length + musicResults.discography.length : 0
	);

	let columns = $derived([
		{
			key: 'quality',
			label: 'Quality',
			title: `+100 if quality matches ${preferredQuality}`,
			getBonusValue: (s: ScoredResult) => s.qualityBonus,
			getTitle: (s: ScoredResult) => s.result.analysis?.quality ?? ''
		}
	]);
</script>

{#if selection}
	<div class="mb-3 rounded bg-base-100 p-2">
		<div class="flex items-center justify-between">
			<div class="min-w-0 flex-1">
				<div class="truncate text-xs font-semibold">{selection.title}</div>
				<div class="flex items-center gap-1 text-xs text-base-content/50">
					{#if musicSelection}
						<span>{musicSelection.artist}</span>
					{/if}
					<span class="badge badge-xs badge-secondary">Music</span>
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
			<span class="text-xs text-base-content/50">Searching torrents for music...</span>
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

			{#if tabs.length > 1}
				<div class="tabs-boxed mb-3 tabs flex-wrap gap-1 bg-base-200 p-1">
					{#each tabs as tab (tab.id)}
						{@const count = tabCounts[tab.id] ?? 0}
						<button
							class={classNames('tab-sm tab', {
								'tab-active': activeMusicTab === tab.id
							})}
							onclick={() => smartSearchService.setActiveMusicTab(tab.id)}
						>
							{tab.label}
							{#if count > 0}
								<span class="ml-1 badge badge-xs">{count}</span>
							{/if}
						</button>
					{/each}
				</div>
			{/if}

			{#if activeResults.length === 0}
				<div class="py-4 text-center text-xs text-base-content/40">
					No torrents found for this {activeMusicTab === 'album' ? 'album' : 'discography'}
				</div>
			{:else}
				<SmartSearchResultsTable {scoredResults} {bestCandidate} {columns} />
			{/if}
		</div>
	{/if}
{/if}

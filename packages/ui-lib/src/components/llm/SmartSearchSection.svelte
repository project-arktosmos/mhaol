<script lang="ts">
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import SmartSearchResultsTable from 'ui-lib/components/llm/SmartSearchResultsTable.svelte';
	import { scoreResults, findBestCandidate } from 'ui-lib/utils/smart-search/score';

	const searchStore = smartSearchService.store;
	const configStore = smartSearchService.configStore;

	let selection = $derived($searchStore.selection);
	let isMusic = $derived(selection?.type === 'music');
	let isGame = $derived(selection?.type === 'game');
	let isBook = $derived(selection?.type === 'book');

	let mediaConfig = $derived.by(() => {
		if (!selection) return null;
		const key =
			selection.type === 'movie'
				? 'movies'
				: selection.type === 'tv'
					? 'tv'
					: selection.type === 'music'
						? 'music'
						: selection.type === 'book'
							? 'books'
							: 'games';
		return $configStore[key];
	});
	let preferredLanguage = $derived(mediaConfig?.preferredLanguage ?? '');
	let preferredQuality = $derived(mediaConfig?.preferredQuality ?? '');
	let preferredConsole = $derived(mediaConfig?.preferredConsole ?? '');

	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);

	let scoredResults = $derived(
		scoreResults($searchStore.searchResults, {
			preferredLanguage,
			preferredQuality,
			preferredConsole
		})
	);
	let searchError = $derived($searchStore.searchError);

	let bestCandidate = $derived(findBestCandidate(scoredResults, { analyzing, searching }));

	let columns = $derived.by(() => {
		if (isGame) {
			return [
				{
					key: 'console',
					label: 'Console',
					title: `+100 if console matches ${preferredConsole}`,
					getBonusValue: (s: (typeof scoredResults)[number]) => s.consoleBonus,
					getTitle: (s: (typeof scoredResults)[number]) => s.result.analysis?.quality ?? ''
				}
			];
		}
		const cols: Array<{
			key: string;
			label: string;
			title?: string;
			getBonusValue: (s: (typeof scoredResults)[number]) => number;
			getTitle?: (s: (typeof scoredResults)[number]) => string;
		}> = [];
		if (!isMusic) {
			cols.push({
				key: 'lang',
				label: 'Lang',
				title: `+100 if language matches ${preferredLanguage}`,
				getBonusValue: (s) => s.langBonus,
				getTitle: (s) => s.result.analysis?.languages ?? ''
			});
		}
		cols.push({
			key: 'quality',
			label: 'Quality',
			title: `+100 if quality matches ${preferredQuality}`,
			getBonusValue: (s) => s.qualityBonus,
			getTitle: (s) => s.result.analysis?.quality ?? ''
		});
		return cols;
	});

	let searchTerm = $derived.by(() => {
		if (!selection) return null;
		if (selection.type === 'music') return `${selection.artist} ${selection.title}`;
		if (selection.type === 'game') return `${selection.title} ${selection.consoleName}`;
		if (selection.type === 'book') return `${selection.author} ${selection.title}`;
		return `${selection.title} ${selection.year}`;
	});
</script>

{#if selection}
	<div class="mb-3 rounded bg-base-100 p-2">
		<div class="flex items-center justify-between">
			<div class="min-w-0 flex-1">
				<div class="truncate text-xs font-semibold">{selection.title}</div>
				<div class="flex items-center gap-1 text-xs text-base-content/50">
					<span>{selection.year}</span>
					<span
						class={classNames('badge badge-xs', {
							'badge-primary': selection.type === 'movie',
							'badge-info': selection.type === 'tv',
							'badge-secondary': selection.type === 'music',
							'badge-accent': selection.type === 'game',
							'badge-warning': selection.type === 'book'
						})}
					>
						{selection.type === 'music'
							? 'Music'
							: selection.type === 'movie'
								? 'Movie'
								: selection.type === 'game'
									? 'Game'
									: selection.type === 'book'
										? 'Book'
										: 'TV'}
					</span>
				</div>
				{#if selection.type === 'music'}
					<div class="truncate text-xs text-base-content/40">{selection.artist}</div>
				{:else if selection.type === 'game'}
					<div class="truncate text-xs text-base-content/40">{selection.consoleName}</div>
				{:else if selection.type === 'book'}
					<div class="truncate text-xs text-base-content/40">{selection.author}</div>
				{/if}
			</div>
			<button class="btn btn-ghost btn-xs" onclick={() => smartSearchService.clear()}>
				&times;
			</button>
		</div>
	</div>

	{#if searchTerm}
		<div class="mb-2 text-xs text-base-content/50">
			Search: <span class="font-mono">{searchTerm}</span>
		</div>
	{/if}

	{#if searching}
		<div class="mt-3 flex items-center justify-center gap-2 py-4">
			<span class="loading loading-sm loading-spinner"></span>
			<span class="text-xs text-base-content/50">Searching torrents...</span>
		</div>
	{:else if searchError}
		<div class="mt-3 rounded bg-error/10 p-2 text-xs text-error">{searchError}</div>
	{:else if scoredResults.length > 0}
		<div class="mt-3">
			<div class="mb-1 flex items-center justify-between">
				<span class="text-xs font-semibold text-base-content/50">
					{scoredResults.length} result{scoredResults.length !== 1 ? 's' : ''}
					{#if analyzing}
						<span class="loading ml-1 loading-xs loading-spinner"></span>
					{/if}
				</span>
			</div>
			<SmartSearchResultsTable {scoredResults} {bestCandidate} {columns} />
		</div>
	{/if}
{/if}

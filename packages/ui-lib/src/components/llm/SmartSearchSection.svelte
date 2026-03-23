<script lang="ts">
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { formatSearchSize, formatSeeders } from 'addons/torrent-search-thepiratebay/format';

	const searchStore = smartSearchService.store;
	const configStore = smartSearchService.configStore;

	let selection = $derived($searchStore.selection);
	let isMusic = $derived(selection?.type === 'music');
	let isGame = $derived(selection?.type === 'game');

	let mediaConfig = $derived.by(() => {
		if (!selection) return null;
		const key =
			selection.type === 'movie'
				? 'movies'
				: selection.type === 'tv'
					? 'tv'
					: selection.type === 'music'
						? 'music'
						: 'games';
		return $configStore[key];
	});
	let preferredLanguage = $derived(mediaConfig?.preferredLanguage ?? '');
	let preferredQuality = $derived(mediaConfig?.preferredQuality ?? '');
	let preferredConsole = $derived(mediaConfig?.preferredConsole ?? '');

	let searching = $derived($searchStore.searching);
	let analyzing = $derived($searchStore.analyzing);

	let maxSeeders = $derived(Math.max(1, ...$searchStore.searchResults.map((r) => r.seeders)));
	let maxLeechers = $derived(Math.max(1, ...$searchStore.searchResults.map((r) => r.leechers)));

	interface ScoredResult {
		result: (typeof $searchStore.searchResults)[number];
		seedersPct: number;
		leechersPct: number;
		relPct: number;
		langBonus: number;
		qualityBonus: number;
		consoleBonus: number;
		score: number;
	}

	let scoredResults = $derived<ScoredResult[]>(
		[...$searchStore.searchResults]
			.map((result) => {
				const seedersPct = Math.round((result.seeders / maxSeeders) * 100);
				const leechersPct = Math.round((result.leechers / maxLeechers) * 100);
				const relPct = result.analysis?.relevance ?? 0;
				const langBonus =
					preferredLanguage &&
					result.analysis &&
					result.analysis.languages.toLowerCase().includes(preferredLanguage.toLowerCase())
						? 100
						: 0;
				const qualityBonus =
					preferredQuality &&
					result.analysis &&
					result.analysis.quality.toLowerCase().includes(preferredQuality.toLowerCase())
						? 100
						: 0;
				const consoleBonus =
					preferredConsole &&
					result.analysis &&
					result.analysis.reason.toLowerCase().includes('console matches')
						? 100
						: 0;
				const score = seedersPct + leechersPct + relPct + langBonus + qualityBonus + consoleBonus;
				return {
					result,
					seedersPct,
					leechersPct,
					relPct,
					langBonus,
					qualityBonus,
					consoleBonus,
					score
				};
			})
			.sort((a, b) => b.score - a.score)
	);
	let searchError = $derived($searchStore.searchError);

	let bestCandidate = $derived.by(() => {
		if (analyzing || searching) return null;
		for (const { result } of scoredResults) {
			if (!result.analysis) continue;
			if (result.analysis.relevance < 75) continue;
			return result;
		}
		return null;
	});

	const PAGE_SIZE = 10;
	let currentPage = $state(0);
	let totalPages = $derived(Math.max(1, Math.ceil(scoredResults.length / PAGE_SIZE)));
	let pagedResults = $derived(
		scoredResults.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE)
	);

	$effect(() => {
		// Reset page when results change
		if (scoredResults.length) currentPage = 0;
	});

	let bonusCols = $derived(isGame ? 1 : isMusic ? 1 : 2);

	let searchTerm = $derived.by(() => {
		if (!selection) return null;
		if (selection.type === 'music') return `${selection.artist} ${selection.title}`;
		if (selection.type === 'game') return `${selection.title} ${selection.consoleName}`;
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
							'badge-accent': selection.type === 'game'
						})}
					>
						{selection.type === 'music'
							? 'Music'
							: selection.type === 'movie'
								? 'Movie'
								: selection.type === 'game'
									? 'Game'
									: 'TV'}
					</span>
				</div>
				{#if selection.type === 'music'}
					<div class="truncate text-xs text-base-content/40">{selection.artist}</div>
				{:else if selection.type === 'game'}
					<div class="truncate text-xs text-base-content/40">{selection.consoleName}</div>
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
			<div class="overflow-x-auto">
				<table class="table w-full table-xs">
					<thead>
						<tr>
							<th>Name</th>
							<th class="text-right">Size</th>
							<th class="text-right" title="Seeders normalized to %">SE%</th>
							<th class="text-right" title="Leechers normalized to %">LE%</th>
							<th class="text-right" title="LLM relevance %">Rel%</th>
							{#if isGame}
								<th class="text-right" title="+100 if console matches {preferredConsole}"
									>Console</th
								>
							{:else}
								{#if !isMusic}
									<th class="text-right" title="+100 if language matches {preferredLanguage}"
										>Lang</th
									>
								{/if}
								<th class="text-right" title="+100 if quality matches {preferredQuality}"
									>Quality</th
								>
							{/if}
							<th class="text-right font-semibold">Score</th>
						</tr>
					</thead>
					<tbody>
						{#each pagedResults as { result, seedersPct, leechersPct, relPct, langBonus, qualityBonus, consoleBonus, score } (result.infoHash)}
							{@const isBest = bestCandidate?.infoHash === result.infoHash}
							<tr
								class={classNames({
									'border-l-2 border-l-primary bg-primary/15': isBest,
									hover: !isBest
								})}
							>
								<td class="max-w-xs">
									<div class="flex items-center gap-1">
										{#if result.isVip}
											<span class="badge badge-xs badge-warning" title="VIP">V</span>
										{:else if result.isTrusted}
											<span class="badge badge-xs badge-success" title="Trusted">T</span>
										{/if}
										<span class="truncate" title={result.name}>{result.name}</span>
									</div>
								</td>
								<td class="text-right text-nowrap">{formatSearchSize(result.size)}</td>
								<td class="text-right" title="{formatSeeders(result.seeders)} seeders">
									{seedersPct}%
								</td>
								<td
									class="text-right text-base-content/60"
									title="{formatSeeders(result.leechers)} leechers"
								>
									{leechersPct}%
								</td>
								{#if result.analyzing}
									<td colspan={bonusCols + 2} class="text-center">
										<span class="loading loading-xs loading-spinner"></span>
									</td>
								{:else if result.analysis}
									<td
										class={classNames('text-right text-xs font-medium', {
											'text-success': relPct >= 80,
											'text-warning': relPct >= 50 && relPct < 80,
											'text-error': relPct < 50
										})}
										title={result.analysis.reason}
									>
										{relPct}%
									</td>
									{#if isGame}
										<td
											class={classNames('text-right text-xs', {
												'text-success': consoleBonus > 0,
												'text-base-content/30': consoleBonus === 0
											})}
											title={result.analysis.quality}
										>
											{consoleBonus > 0 ? '+100' : '0'}
										</td>
									{:else}
										{#if !isMusic}
											<td
												class={classNames('text-right text-xs', {
													'text-success': langBonus > 0,
													'text-base-content/30': langBonus === 0
												})}
												title={result.analysis.languages}
											>
												{langBonus > 0 ? '+100' : '0'}
											</td>
										{/if}
										<td
											class={classNames('text-right text-xs', {
												'text-success': qualityBonus > 0,
												'text-base-content/30': qualityBonus === 0
											})}
											title={result.analysis.quality}
										>
											{qualityBonus > 0 ? '+100' : '0'}
										</td>
									{/if}
									<td
										class={classNames('text-right text-xs font-bold', {
											'text-success': score >= 350,
											'text-warning': score >= 200 && score < 350,
											'text-error': score < 200
										})}
									>
										{score}
									</td>
								{:else}
									<td colspan={bonusCols + 2}></td>
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
							«
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
							»
						</button>
					</div>
				</div>
			{/if}
		</div>
	{/if}
{/if}

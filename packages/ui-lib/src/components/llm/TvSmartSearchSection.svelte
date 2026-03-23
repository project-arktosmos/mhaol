<script lang="ts">
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { formatSearchSize, formatSeeders } from 'addons/torrent-search-thepiratebay/format';
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

	let maxSeeders = $derived(Math.max(1, ...activeResults.map((r) => r.seeders)));
	let maxLeechers = $derived(Math.max(1, ...activeResults.map((r) => r.leechers)));

	interface ScoredResult {
		result: SmartSearchTorrentResult;
		seedersPct: number;
		leechersPct: number;
		relPct: number;
		langBonus: number;
		qualityBonus: number;
		score: number;
	}

	let scoredResults = $derived<ScoredResult[]>(
		[...activeResults]
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
				const score = seedersPct + leechersPct + relPct + langBonus + qualityBonus;
				return { result, seedersPct, leechersPct, relPct, langBonus, qualityBonus, score };
			})
			.sort((a, b) => b.score - a.score)
	);

	let bestCandidate = $derived.by(() => {
		if (analyzing || searching) return null;
		for (const { result } of scoredResults) {
			if (!result.analysis) continue;
			if (result.analysis.relevance < 75) continue;
			return result;
		}
		return null;
	});

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

	const PAGE_SIZE = 10;
	let currentPage = $state(0);
	let totalPages = $derived(Math.max(1, Math.ceil(scoredResults.length / PAGE_SIZE)));
	let pagedResults = $derived(
		scoredResults.slice(currentPage * PAGE_SIZE, (currentPage + 1) * PAGE_SIZE)
	);

	$effect(() => {
		if (scoredResults.length) currentPage = 0;
	});

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
				<div class="overflow-x-auto">
					<table class="table w-full table-xs">
						<thead>
							<tr>
								<th>Name</th>
								<th class="text-right">Scope</th>
								<th class="text-right">Size</th>
								<th class="text-right" title="Seeders normalized to %">SE%</th>
								<th class="text-right" title="Leechers normalized to %">LE%</th>
								<th class="text-right" title="Relevance %">Rel%</th>
								<th class="text-right" title="+100 if language matches {preferredLanguage}">Lang</th
								>
								<th class="text-right" title="+100 if quality matches {preferredQuality}"
									>Quality</th
								>
								<th class="text-right font-semibold">Score</th>
							</tr>
						</thead>
						<tbody>
							{#each pagedResults as { result, seedersPct, leechersPct, relPct, langBonus, qualityBonus, score } (result.infoHash)}
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
									<td class="text-right text-xs text-nowrap text-base-content/60">
										{formatSeEp(result)}
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
										<td colspan={4} class="text-center">
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
										<td
											class={classNames('text-right text-xs', {
												'text-success': langBonus > 0,
												'text-base-content/30': langBonus === 0
											})}
											title={result.analysis.languages}
										>
											{langBonus > 0 ? '+100' : '0'}
										</td>
										<td
											class={classNames('text-right text-xs', {
												'text-success': qualityBonus > 0,
												'text-base-content/30': qualityBonus === 0
											})}
											title={result.analysis.quality}
										>
											{qualityBonus > 0 ? '+100' : '0'}
										</td>
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
										<td colspan={4}></td>
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
			{/if}
		</div>
	{/if}
{/if}

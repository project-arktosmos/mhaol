import type { SmartSearchTorrentResult } from 'ui-lib/types/smart-search.type';

export interface ScoringConfig {
	preferredLanguage: string;
	preferredQuality: string;
	preferredConsole: string;
}

export interface ScoredResult {
	result: SmartSearchTorrentResult;
	seedersPct: number;
	leechersPct: number;
	relPct: number;
	langBonus: number;
	qualityBonus: number;
	consoleBonus: number;
	score: number;
}

export function scoreResults(
	results: SmartSearchTorrentResult[],
	config: ScoringConfig
): ScoredResult[] {
	const maxSeeders = Math.max(1, ...results.map((r) => r.seeders));
	const maxLeechers = Math.max(1, ...results.map((r) => r.leechers));

	return [...results]
		.map((result) => {
			const seedersPct = Math.round((result.seeders / maxSeeders) * 100);
			const leechersPct = Math.round((result.leechers / maxLeechers) * 100);
			const relPct = result.analysis?.relevance ?? 0;
			const langBonus =
				config.preferredLanguage &&
				result.analysis &&
				result.analysis.languages.toLowerCase().includes(config.preferredLanguage.toLowerCase())
					? 100
					: 0;
			const qualityBonus =
				config.preferredQuality &&
				result.analysis &&
				result.analysis.quality.toLowerCase().includes(config.preferredQuality.toLowerCase())
					? 100
					: 0;
			const consoleBonus =
				config.preferredConsole &&
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
		.sort((a, b) => b.score - a.score);
}

export function findBestCandidate(
	scoredResults: ScoredResult[],
	options: { analyzing: boolean; searching: boolean }
): SmartSearchTorrentResult | null {
	if (options.analyzing || options.searching) return null;
	for (const { result } of scoredResults) {
		if (!result.analysis) continue;
		if (result.analysis.relevance < 75) continue;
		return result;
	}
	return null;
}

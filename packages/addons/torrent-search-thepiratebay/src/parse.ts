import type { PirateBayApiResult, TorrentSearchResult } from './types.js';
import { buildMagnetLink } from './magnet.js';

export function isNoResults(results: PirateBayApiResult[]): boolean {
	return results.length === 1 && results[0].id === '0' && results[0].name === 'No results returned';
}

export function parseResult(raw: PirateBayApiResult): TorrentSearchResult {
	return {
		id: raw.id,
		name: raw.name,
		infoHash: raw.info_hash,
		magnetLink: buildMagnetLink(raw.info_hash, raw.name),
		seeders: parseInt(raw.seeders, 10) || 0,
		leechers: parseInt(raw.leechers, 10) || 0,
		size: parseInt(raw.size, 10) || 0,
		category: raw.category,
		uploadedBy: raw.username,
		uploadedAt: new Date(parseInt(raw.added, 10) * 1000),
		isVip: raw.status === 'vip',
		isTrusted: raw.status === 'trusted'
	};
}

export function parseResults(rawResults: PirateBayApiResult[]): TorrentSearchResult[] {
	if (!Array.isArray(rawResults) || isNoResults(rawResults)) {
		return [];
	}
	return rawResults.map((r) => parseResult(r));
}

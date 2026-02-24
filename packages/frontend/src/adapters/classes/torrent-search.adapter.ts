import { AdapterClass } from '$adapters/classes/adapter.class';
import type { PirateBayApiResult, TorrentSearchResult } from '$types/torrent-search.type';

const MAGNET_TRACKERS = [
	'udp://tracker.opentrackr.org:1337/announce',
	'udp://tracker.openbittorrent.com:6969/announce',
	'udp://open.stealth.si:80/announce',
	'udp://tracker.torrent.eu.org:451/announce',
	'udp://tracker.dler.org:6969/announce',
	'udp://opentracker.i2p.rocks:6969/announce'
];

export class TorrentSearchAdapter extends AdapterClass {
	constructor() {
		super('torrent-search');
	}

	buildMagnetLink(infoHash: string, name: string): string {
		const encodedName = encodeURIComponent(name);
		const trackerParams = MAGNET_TRACKERS.map((t) => `&tr=${encodeURIComponent(t)}`).join('');
		return `magnet:?xt=urn:btih:${infoHash}&dn=${encodedName}${trackerParams}`;
	}

	isNoResults(results: PirateBayApiResult[]): boolean {
		return (
			results.length === 1 && results[0].id === '0' && results[0].name === 'No results returned'
		);
	}

	fromApi(raw: PirateBayApiResult): TorrentSearchResult {
		return {
			id: raw.id,
			name: raw.name,
			infoHash: raw.info_hash,
			magnetLink: this.buildMagnetLink(raw.info_hash, raw.name),
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

	fromApiResults(rawResults: PirateBayApiResult[]): TorrentSearchResult[] {
		if (!Array.isArray(rawResults) || this.isNoResults(rawResults)) {
			return [];
		}
		return rawResults.map((r) => this.fromApi(r));
	}
}

export const torrentSearchAdapter = new TorrentSearchAdapter();

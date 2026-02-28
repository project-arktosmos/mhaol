import { describe, it, expect } from 'vitest';
import { sortSearchResults } from '../src/sort.js';
import type { TorrentSearchResult } from '../src/types.js';

function makeResult(overrides: Partial<TorrentSearchResult> = {}): TorrentSearchResult {
	return {
		id: '1',
		name: 'Test',
		infoHash: 'abc',
		magnetLink: 'magnet:?xt=urn:btih:abc',
		seeders: 10,
		leechers: 5,
		size: 1000,
		category: '200',
		uploadedBy: 'user',
		uploadedAt: new Date('2024-01-15'),
		isVip: false,
		isTrusted: false,
		...overrides
	};
}

describe('sortSearchResults', () => {
	const results: TorrentSearchResult[] = [
		makeResult({ id: '1', name: 'Bravo', seeders: 20, leechers: 5, size: 300, uploadedAt: new Date('2024-03-01') }),
		makeResult({ id: '2', name: 'Alpha', seeders: 50, leechers: 2, size: 100, uploadedAt: new Date('2024-01-01') }),
		makeResult({ id: '3', name: 'Charlie', seeders: 10, leechers: 15, size: 200, uploadedAt: new Date('2024-02-01') })
	];

	it('sorts by seeders descending', () => {
		const sorted = sortSearchResults(results, { field: 'seeders', direction: 'desc' });
		expect(sorted.map((r) => r.seeders)).toEqual([50, 20, 10]);
	});

	it('sorts by seeders ascending', () => {
		const sorted = sortSearchResults(results, { field: 'seeders', direction: 'asc' });
		expect(sorted.map((r) => r.seeders)).toEqual([10, 20, 50]);
	});

	it('sorts by leechers descending', () => {
		const sorted = sortSearchResults(results, { field: 'leechers', direction: 'desc' });
		expect(sorted.map((r) => r.leechers)).toEqual([15, 5, 2]);
	});

	it('sorts by size ascending', () => {
		const sorted = sortSearchResults(results, { field: 'size', direction: 'asc' });
		expect(sorted.map((r) => r.size)).toEqual([100, 200, 300]);
	});

	it('sorts by name ascending', () => {
		const sorted = sortSearchResults(results, { field: 'name', direction: 'asc' });
		expect(sorted.map((r) => r.name)).toEqual(['Alpha', 'Bravo', 'Charlie']);
	});

	it('sorts by uploadedAt descending', () => {
		const sorted = sortSearchResults(results, { field: 'uploadedAt', direction: 'desc' });
		expect(sorted.map((r) => r.id)).toEqual(['1', '3', '2']);
	});

	it('does not mutate the original array', () => {
		const original = [...results];
		sortSearchResults(results, { field: 'seeders', direction: 'asc' });
		expect(results).toEqual(original);
	});

	it('handles empty arrays', () => {
		expect(sortSearchResults([], { field: 'seeders', direction: 'desc' })).toEqual([]);
	});
});

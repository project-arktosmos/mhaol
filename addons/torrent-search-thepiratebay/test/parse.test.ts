import { describe, it, expect } from 'vitest';
import { parseResult, parseResults, isNoResults } from '../src/parse.js';
import type { PirateBayApiResult } from '../src/types.js';

function makePirateBayResult(overrides: Partial<PirateBayApiResult> = {}): PirateBayApiResult {
	return {
		id: '123',
		name: 'Test Torrent',
		info_hash: 'abc123def456',
		leechers: '10',
		seeders: '50',
		num_files: '3',
		size: '1073741824',
		username: 'uploader1',
		added: '1700000000',
		status: 'trusted',
		category: '200',
		imdb: 'tt1234567',
		...overrides
	};
}

describe('isNoResults', () => {
	it('returns true for PirateBay no-results sentinel', () => {
		const sentinel: PirateBayApiResult[] = [
			makePirateBayResult({ id: '0', name: 'No results returned' })
		];
		expect(isNoResults(sentinel)).toBe(true);
	});

	it('returns false for actual results', () => {
		expect(isNoResults([makePirateBayResult()])).toBe(false);
	});

	it('returns false for multiple results even if first matches sentinel', () => {
		expect(
			isNoResults([
				makePirateBayResult({ id: '0', name: 'No results returned' }),
				makePirateBayResult()
			])
		).toBe(false);
	});
});

describe('parseResult', () => {
	it('maps all fields correctly', () => {
		const raw = makePirateBayResult();
		const result = parseResult(raw);

		expect(result.id).toBe('123');
		expect(result.name).toBe('Test Torrent');
		expect(result.infoHash).toBe('abc123def456');
		expect(result.seeders).toBe(50);
		expect(result.leechers).toBe(10);
		expect(result.size).toBe(1073741824);
		expect(result.category).toBe('200');
		expect(result.uploadedBy).toBe('uploader1');
	});

	it('parses numeric string fields to numbers', () => {
		const result = parseResult(makePirateBayResult({ seeders: '100', leechers: '25', size: '500' }));
		expect(result.seeders).toBe(100);
		expect(result.leechers).toBe(25);
		expect(result.size).toBe(500);
	});

	it('defaults invalid numeric fields to 0', () => {
		const result = parseResult(makePirateBayResult({ seeders: 'bad', leechers: '', size: 'NaN' }));
		expect(result.seeders).toBe(0);
		expect(result.leechers).toBe(0);
		expect(result.size).toBe(0);
	});

	it('converts unix timestamp to Date', () => {
		const result = parseResult(makePirateBayResult({ added: '1700000000' }));
		expect(result.uploadedAt).toEqual(new Date(1700000000 * 1000));
	});

	it('detects VIP status', () => {
		const result = parseResult(makePirateBayResult({ status: 'vip' }));
		expect(result.isVip).toBe(true);
		expect(result.isTrusted).toBe(false);
	});

	it('detects trusted status', () => {
		const result = parseResult(makePirateBayResult({ status: 'trusted' }));
		expect(result.isVip).toBe(false);
		expect(result.isTrusted).toBe(true);
	});

	it('handles unknown status', () => {
		const result = parseResult(makePirateBayResult({ status: '' }));
		expect(result.isVip).toBe(false);
		expect(result.isTrusted).toBe(false);
	});

	it('generates a magnetLink', () => {
		const result = parseResult(makePirateBayResult({ info_hash: 'deadbeef', name: 'My File' }));
		expect(result.magnetLink).toMatch(/^magnet:\?xt=urn:btih:deadbeef&dn=My%20File/);
	});
});

describe('parseResults', () => {
	it('parses an array of raw results', () => {
		const raw = [makePirateBayResult({ id: '1' }), makePirateBayResult({ id: '2' })];
		const results = parseResults(raw);
		expect(results).toHaveLength(2);
		expect(results[0].id).toBe('1');
		expect(results[1].id).toBe('2');
	});

	it('returns empty array for no-results sentinel', () => {
		const sentinel = [makePirateBayResult({ id: '0', name: 'No results returned' })];
		expect(parseResults(sentinel)).toEqual([]);
	});

	it('returns empty array for non-array input', () => {
		expect(parseResults(null as unknown as PirateBayApiResult[])).toEqual([]);
	});
});

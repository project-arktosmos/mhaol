import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { search } from '../src/search.js';
import { TorrentCategory } from '../src/types.js';

describe('search', () => {
	const mockFetch = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('returns empty array for empty query', async () => {
		const results = await search('');
		expect(results).toEqual([]);
		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('returns empty array for whitespace-only query', async () => {
		const results = await search('   ');
		expect(results).toEqual([]);
		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('calls the PirateBay API with encoded query', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () => Promise.resolve([{ id: '0', name: 'No results returned' }])
		});

		await search('test query');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('q=test%20query'),
			expect.any(Object)
		);
	});

	it('passes default category 0', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () => Promise.resolve([{ id: '0', name: 'No results returned' }])
		});

		await search('test');

		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('cat=0'), expect.any(Object));
	});

	it('passes custom category', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () => Promise.resolve([{ id: '0', name: 'No results returned' }])
		});

		await search('test', { category: TorrentCategory.Video });

		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('cat=200'), expect.any(Object));
	});

	it('parses valid results', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve([
					{
						id: '123',
						name: 'Test Torrent',
						info_hash: 'abc123',
						seeders: '50',
						leechers: '10',
						num_files: '3',
						size: '1073741824',
						username: 'uploader',
						added: '1700000000',
						status: 'trusted',
						category: '200',
						imdb: ''
					}
				])
		});

		const results = await search('test');
		expect(results).toHaveLength(1);
		expect(results[0].name).toBe('Test Torrent');
		expect(results[0].seeders).toBe(50);
	});

	it('returns empty array for no-results sentinel', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve([
					{
						id: '0',
						name: 'No results returned',
						info_hash: '',
						seeders: '0',
						leechers: '0',
						num_files: '0',
						size: '0',
						username: '',
						added: '0',
						status: '',
						category: '0',
						imdb: ''
					}
				])
		});

		const results = await search('nonexistent');
		expect(results).toEqual([]);
	});

	it('throws on non-ok response', async () => {
		mockFetch.mockResolvedValue({
			ok: false,
			status: 500
		});

		await expect(search('test')).rejects.toThrow('PirateBay API returned 500');
	});

	it('sets user-agent header', async () => {
		mockFetch.mockResolvedValue({
			ok: true,
			json: () => Promise.resolve([{ id: '0', name: 'No results returned' }])
		});

		await search('test', { userAgent: 'CustomAgent/1.0' });

		expect(mockFetch).toHaveBeenCalledWith(
			expect.any(String),
			expect.objectContaining({
				headers: { 'User-Agent': 'CustomAgent/1.0' }
			})
		);
	});
});

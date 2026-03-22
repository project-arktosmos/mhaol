import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock EventSource before importing torrent service
class MockEventSource {
	url: string;
	onerror: (() => void) | null = null;
	listeners: Record<string, ((event: MessageEvent) => void)[]> = {};
	constructor(url: string) {
		this.url = url;
	}
	addEventListener(type: string, handler: (event: MessageEvent) => void) {
		if (!this.listeners[type]) this.listeners[type] = [];
		this.listeners[type].push(handler);
	}
	close() {}
}

vi.stubGlobal('EventSource', MockEventSource);

describe('TorrentSearchService', () => {
	let torrentSearchService: (typeof import('../../src/services/torrent-search.service'))['torrentSearchService'];
	let TorrentCategory: (typeof import('addons/torrent-search-thepiratebay/types'))['TorrentCategory'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		vi.stubGlobal('EventSource', MockEventSource);

		const categoryMod = await import('addons/torrent-search-thepiratebay/types');
		TorrentCategory = categoryMod.TorrentCategory;

		const mod = await import('../../src/services/torrent-search.service');
		torrentSearchService = mod.torrentSearchService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(torrentSearchService.state);
		expect(state.query).toBe('');
		expect(state.searching).toBe(false);
		expect(state.results).toEqual([]);
		expect(state.error).toBeNull();
		expect(state.sort.field).toBe('seeders');
		expect(state.sort.direction).toBe('desc');
	});

	it('should not search with empty query', async () => {
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		await torrentSearchService.search('   ');

		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('should perform search and update results', async () => {
		const searchResults = [
			{
				id: '1',
				name: 'Test Movie',
				infoHash: 'abc',
				magnetLink: 'magnet:?xt=abc',
				seeders: 100,
				leechers: 10,
				size: 1024,
				category: 'video',
				uploadedBy: 'user1',
				uploadedAt: '2024-01-01T00:00:00Z',
				isVip: false,
				isTrusted: true
			}
		];

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => searchResults
					}) as Response
			)
		);

		await torrentSearchService.search('test movie');

		const state = get(torrentSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.query).toBe('test movie');
		expect(state.results).toHaveLength(1);
		expect(state.results[0].name).toBe('Test Movie');
	});

	it('should set error on search failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('Network error');
			})
		);

		await torrentSearchService.search('test');

		const state = get(torrentSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.error).toContain('Search failed');
	});

	it('should toggle sort direction for same field', () => {
		torrentSearchService.toggleSort('seeders');
		let state = get(torrentSearchService.state);
		expect(state.sort.direction).toBe('asc'); // was desc, toggled to asc

		torrentSearchService.toggleSort('seeders');
		state = get(torrentSearchService.state);
		expect(state.sort.direction).toBe('desc');
	});

	it('should set desc direction for new sort field', () => {
		torrentSearchService.toggleSort('size');
		const state = get(torrentSearchService.state);
		expect(state.sort.field).toBe('size');
		expect(state.sort.direction).toBe('desc');
	});

	it('should mark and unmark adding torrents', () => {
		torrentSearchService.markAdding('hash1');
		let state = get(torrentSearchService.state);
		expect(state.addingTorrents.has('hash1')).toBe(true);

		torrentSearchService.unmarkAdding('hash1');
		state = get(torrentSearchService.state);
		expect(state.addingTorrents.has('hash1')).toBe(false);
	});

	it('should clear results', () => {
		torrentSearchService.state.update((s) => ({
			...s,
			results: [{ id: '1', name: 'test' }] as never,
			error: 'some error'
		}));

		torrentSearchService.clearResults();

		const state = get(torrentSearchService.state);
		expect(state.results).toEqual([]);
		expect(state.error).toBeNull();
	});
});

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { smartSearchService } from '../../src/services/smart-search.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data)
	});
}

const initialState = {
	selection: null,
	visible: false,
	searching: false,
	analyzing: false,
	searchResults: [],
	searchError: null,
	streamingHash: null,
	streamingProgress: 0,
	pendingItemId: null,
	pendingLibraryId: null,
	downloadedHash: null,
	fetchedCandidate: null
};

describe('SmartSearchService', () => {
	beforeEach(() => {
		smartSearchService.store.set({ ...initialState });
	});

	afterEach(() => {
		smartSearchService.clear();
		vi.restoreAllMocks();
	});

	it('exports a singleton smartSearchService', () => {
		expect(smartSearchService).toBeDefined();
		expect(smartSearchService.store).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(smartSearchService.store);
		expect(state.selection).toBeNull();
		expect(state.visible).toBe(false);
		expect(state.searching).toBe(false);
		expect(state.searchResults).toEqual([]);
		expect(state.searchError).toBeNull();
		expect(state.streamingHash).toBeNull();
	});

	it('select sets selection and starts search', () => {
		// Mock fetch so the async searches don't fail
		vi.stubGlobal('fetch', mockFetch({ downloadPath: '/tmp' }));

		const selection = {
			type: 'movie' as const,
			title: 'Inception',
			year: 2010,
			tmdbId: 27205
		};

		smartSearchService.select(selection);

		const state = get(smartSearchService.store);
		expect(state.selection).toEqual(selection);
		expect(state.visible).toBe(false);
		expect(state.searchResults).toEqual([]);
		expect(state.downloadedHash).toBeNull();
	});

	it('clear resets selection and search results', () => {
		smartSearchService.store.set({
			...initialState,
			selection: { type: 'movie' as const, title: 'Test', year: 2020, tmdbId: 1 },
			visible: true,
			searchResults: [{ infoHash: 'abc', name: 'Test' } as never]
		});

		smartSearchService.clear();

		const state = get(smartSearchService.store);
		expect(state.selection).toBeNull();
		expect(state.searchResults).toEqual([]);
		expect(state.searchError).toBeNull();
	});

	it('hide sets visible to false', () => {
		smartSearchService.store.set({
			...initialState,
			visible: true
		});

		smartSearchService.hide();

		expect(get(smartSearchService.store).visible).toBe(false);
	});

	it('updateStreamingProgress updates progress', () => {
		smartSearchService.updateStreamingProgress(42.5);

		const state = get(smartSearchService.store);
		expect(state.streamingProgress).toBe(42.5);
	});

	it('clearStreaming resets streaming state', () => {
		smartSearchService.store.set({
			...initialState,
			streamingHash: 'abc123',
			streamingProgress: 75
		});

		smartSearchService.clearStreaming();

		const state = get(smartSearchService.store);
		expect(state.streamingHash).toBeNull();
		expect(state.streamingProgress).toBe(0);
	});

	it('setFetchedCandidate and getFetchedCandidate work', () => {
		const candidate = {
			infoHash: 'abc',
			name: 'Test.Torrent',
			seeders: 100,
			leechers: 10,
			magnetLink: 'magnet:?xt=urn:btih:abc',
			uploadedAt: new Date(),
			searchQueries: ['test'],
			analysis: null,
			analyzing: false
		};

		smartSearchService.setFetchedCandidate(candidate as never);
		expect(get(smartSearchService.store).fetchedCandidate).toEqual(candidate);

		const retrieved = smartSearchService.getFetchedCandidate();
		expect(retrieved).toEqual(candidate);
	});

	it('checkFetchCache returns cached candidate on success', async () => {
		const mockCandidate = {
			infoHash: 'abc',
			name: 'Cached',
			uploadedAt: '2024-01-01T00:00:00Z'
		};
		vi.stubGlobal('fetch', mockFetch({ candidate: mockCandidate }));

		const result = await smartSearchService.checkFetchCache(12345);

		expect(result).not.toBeNull();
		expect(result!.infoHash).toBe('abc');
		expect(result!.uploadedAt).toBeInstanceOf(Date);
	});

	it('checkFetchCache returns null on failure', async () => {
		vi.stubGlobal('fetch', mockFetch({}, false));

		const result = await smartSearchService.checkFetchCache(12345);
		expect(result).toBeNull();
	});

	it('saveFetchCache sends POST with correct body', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		const candidate = { infoHash: 'abc', name: 'Test' };
		await smartSearchService.saveFetchCache(123, 'movie', candidate as never);

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/catalog/fetch-cache-by-source'),
			expect.objectContaining({
				method: 'POST',
				body: expect.stringContaining('"sourceId":"123"')
			})
		);
	});

	it('startDownload fetches config, creates torrent, and returns infoHash', async () => {
		smartSearchService.store.set({
			...initialState,
			selection: { type: 'movie' as const, title: 'Test', year: 2020, tmdbId: 1 },
			pendingItemId: 'item-1',
			pendingLibraryId: 'lib-1'
		});

		let callIdx = 0;
		const mock = vi.fn().mockImplementation(() => {
			callIdx++;
			if (callIdx === 1)
				return Promise.resolve({
					ok: true,
					json: () => Promise.resolve({ downloadPath: '/downloads' })
				});
			if (callIdx === 2)
				return Promise.resolve({
					ok: true,
					json: () => Promise.resolve({ infoHash: 'hash123', outputPath: '/downloads/movies' })
				});
			// updateItemWithTorrent call
			return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
		});
		vi.stubGlobal('fetch', mock);

		const candidate = {
			infoHash: 'hash123',
			magnetLink: 'magnet:?xt=urn:btih:hash123',
			name: 'Test'
		};

		const result = await smartSearchService.startDownload(candidate as never);

		expect(result).toBe('hash123');
		expect(get(smartSearchService.store).downloadedHash).toBe('hash123');
	});

	it('startDownload returns null when no selection', async () => {
		smartSearchService.store.set({ ...initialState, selection: null });

		const result = await smartSearchService.startDownload({} as never);
		expect(result).toBeNull();
	});

	it('startStream returns infoHash and sets streaming state', async () => {
		smartSearchService.store.set({
			...initialState,
			selection: { type: 'tv' as const, title: 'Show', year: 2023, tmdbId: 2 },
			pendingItemId: 'item-2',
			pendingLibraryId: 'lib-2'
		});

		let callIdx = 0;
		const mock = vi.fn().mockImplementation(() => {
			callIdx++;
			if (callIdx === 1)
				return Promise.resolve({
					ok: true,
					json: () => Promise.resolve({ downloadPath: '/downloads' })
				});
			if (callIdx === 2)
				return Promise.resolve({
					ok: true,
					json: () => Promise.resolve({ infoHash: 'stream-hash', outputPath: '/downloads/tv' })
				});
			return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
		});
		vi.stubGlobal('fetch', mock);

		const candidate = {
			infoHash: 'stream-hash',
			magnetLink: 'magnet:?xt=urn:btih:stream-hash',
			name: 'Show'
		};

		const result = await smartSearchService.startStream(candidate as never);

		expect(result).toBe('stream-hash');
		const state = get(smartSearchService.store);
		expect(state.streamingHash).toBe('stream-hash');
		expect(state.streamingProgress).toBe(0);
	});
});

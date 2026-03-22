import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { youtubeSearchService } from '../../src/services/youtube-search.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data)
	});
}

const initialState = {
	query: '',
	searching: false,
	results: [],
	channels: [],
	continuation: null,
	loadingMore: false,
	error: null
};

describe('YouTubeSearchService', () => {
	beforeEach(() => {
		youtubeSearchService.state.set({ ...initialState });
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton youtubeSearchService', () => {
		expect(youtubeSearchService).toBeDefined();
		expect(youtubeSearchService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(youtubeSearchService.state);
		expect(state.query).toBe('');
		expect(state.searching).toBe(false);
		expect(state.results).toEqual([]);
		expect(state.channels).toEqual([]);
		expect(state.continuation).toBeNull();
		expect(state.error).toBeNull();
	});

	it('search sets searching state and fetches results', async () => {
		const mockData = {
			items: [{ videoId: 'v1', title: 'Video 1' }],
			channels: [{ channelId: 'c1', name: 'Channel 1' }],
			continuation: 'token123'
		};
		vi.stubGlobal('fetch', mockFetch(mockData));

		await youtubeSearchService.search('test query');

		const state = get(youtubeSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.query).toBe('test query');
		expect(state.results).toEqual(mockData.items);
		expect(state.channels).toEqual(mockData.channels);
		expect(state.continuation).toBe('token123');
		expect(state.error).toBeNull();
	});

	it('search skips empty query', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeSearchService.search('   ');

		expect(mock).not.toHaveBeenCalled();
	});

	it('search trims query', async () => {
		const mock = mockFetch({ items: [], channels: [], continuation: null });
		vi.stubGlobal('fetch', mock);

		await youtubeSearchService.search('  hello world  ');

		const state = get(youtubeSearchService.state);
		expect(state.query).toBe('hello world');
		expect(mock).toHaveBeenCalledWith(expect.stringContaining('q=hello+world'));
	});

	it('search handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await youtubeSearchService.search('test');

		const state = get(youtubeSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.error).toContain('Search failed');
		expect(state.error).toContain('Network error');
	});

	it('search handles non-ok response', async () => {
		vi.stubGlobal('fetch', mockFetch({ error: 'Rate limited' }, false));

		await youtubeSearchService.search('test');

		const state = get(youtubeSearchService.state);
		expect(state.error).toContain('Rate limited');
	});

	it('loadMore appends results', async () => {
		youtubeSearchService.state.set({
			...initialState,
			query: 'test',
			results: [{ videoId: 'v1', title: 'Video 1' } as never],
			channels: [],
			continuation: 'page2'
		});

		const mockData = {
			items: [{ videoId: 'v2', title: 'Video 2' }],
			channels: [{ channelId: 'c1', name: 'Channel' }],
			continuation: 'page3'
		};
		vi.stubGlobal('fetch', mockFetch(mockData));

		await youtubeSearchService.loadMore();

		const state = get(youtubeSearchService.state);
		expect(state.results).toHaveLength(2);
		expect(state.continuation).toBe('page3');
		expect(state.loadingMore).toBe(false);
	});

	it('loadMore does nothing without continuation', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeSearchService.loadMore();

		expect(mock).not.toHaveBeenCalled();
	});

	it('loadMore does nothing if already loading', async () => {
		youtubeSearchService.state.set({
			...initialState,
			query: 'test',
			continuation: 'page2',
			loadingMore: true
		});

		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeSearchService.loadMore();

		expect(mock).not.toHaveBeenCalled();
	});

	it('clearResults resets to initial state', () => {
		youtubeSearchService.state.set({
			...initialState,
			query: 'test',
			results: [{ videoId: 'v1' } as never],
			continuation: 'token'
		});

		youtubeSearchService.clearResults();

		const state = get(youtubeSearchService.state);
		expect(state.query).toBe('');
		expect(state.results).toEqual([]);
		expect(state.continuation).toBeNull();
	});
});

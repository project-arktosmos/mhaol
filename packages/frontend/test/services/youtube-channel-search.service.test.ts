import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { youtubeChannelSearchService } from '../../src/services/youtube-channel-search.service';

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
	channels: [],
	continuation: null,
	loadingMore: false,
	error: null
};

describe('YouTubeChannelSearchService', () => {
	beforeEach(() => {
		youtubeChannelSearchService.state.set({ ...initialState });
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton youtubeChannelSearchService', () => {
		expect(youtubeChannelSearchService).toBeDefined();
		expect(youtubeChannelSearchService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(youtubeChannelSearchService.state);
		expect(state.query).toBe('');
		expect(state.searching).toBe(false);
		expect(state.channels).toEqual([]);
		expect(state.continuation).toBeNull();
		expect(state.error).toBeNull();
	});

	it('search fetches channels and updates state', async () => {
		const mockData = {
			items: [],
			channels: [
				{ channelId: 'c1', name: 'Channel 1' },
				{ channelId: 'c2', name: 'Channel 2' }
			],
			continuation: 'next-page'
		};
		vi.stubGlobal('fetch', mockFetch(mockData));

		await youtubeChannelSearchService.search('music channels');

		const state = get(youtubeChannelSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.query).toBe('music channels');
		expect(state.channels).toEqual(mockData.channels);
		expect(state.continuation).toBe('next-page');
	});

	it('search skips empty query', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeChannelSearchService.search('');

		expect(mock).not.toHaveBeenCalled();
	});

	it('search handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Timeout')));

		await youtubeChannelSearchService.search('test');

		const state = get(youtubeChannelSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.error).toContain('Search failed');
		expect(state.error).toContain('Timeout');
	});

	it('loadMore appends channels', async () => {
		youtubeChannelSearchService.state.set({
			...initialState,
			query: 'test',
			channels: [{ channelId: 'c1', name: 'Channel 1' } as never],
			continuation: 'page2'
		});

		const mockData = {
			items: [],
			channels: [{ channelId: 'c2', name: 'Channel 2' }],
			continuation: null
		};
		vi.stubGlobal('fetch', mockFetch(mockData));

		await youtubeChannelSearchService.loadMore();

		const state = get(youtubeChannelSearchService.state);
		expect(state.channels).toHaveLength(2);
		expect(state.continuation).toBeNull();
		expect(state.loadingMore).toBe(false);
	});

	it('loadMore does nothing without continuation', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeChannelSearchService.loadMore();

		expect(mock).not.toHaveBeenCalled();
	});

	it('clearResults resets to initial state', () => {
		youtubeChannelSearchService.state.set({
			...initialState,
			query: 'test',
			channels: [{ channelId: 'c1' } as never],
			continuation: 'token'
		});

		youtubeChannelSearchService.clearResults();

		const state = get(youtubeChannelSearchService.state);
		expect(state.query).toBe('');
		expect(state.channels).toEqual([]);
		expect(state.continuation).toBeNull();
	});
});

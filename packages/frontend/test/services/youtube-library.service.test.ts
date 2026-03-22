import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { youtubeLibraryService } from '../../src/services/youtube-library.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data)
	});
}

function mockFetchSequence(...responses: Array<{ data: unknown; ok?: boolean }>) {
	let callIdx = 0;
	return vi.fn().mockImplementation(() => {
		const resp = responses[callIdx] ?? responses[responses.length - 1];
		callIdx++;
		return Promise.resolve({
			ok: resp.ok !== false,
			status: resp.ok !== false ? 200 : 500,
			json: () => Promise.resolve(resp.data)
		});
	});
}

const initialState = {
	content: [],
	contentLoading: false,
	contentError: null,
	favorites: [],
	favoritesLoading: false
};

describe('YouTubeLibraryService', () => {
	beforeEach(() => {
		youtubeLibraryService.library.set(null);
		youtubeLibraryService.state.set({ ...initialState });
		youtubeLibraryService.reset();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton youtubeLibraryService', () => {
		expect(youtubeLibraryService).toBeDefined();
		expect(youtubeLibraryService.library).toBeDefined();
		expect(youtubeLibraryService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(youtubeLibraryService.state);
		expect(state.content).toEqual([]);
		expect(state.contentLoading).toBe(false);
		expect(state.contentError).toBeNull();
		expect(state.favorites).toEqual([]);
		expect(state.favoritesLoading).toBe(false);
	});

	it('reset clears all state', () => {
		youtubeLibraryService.library.set({ id: 'lib-1', name: 'Test' } as never);
		youtubeLibraryService.state.set({
			content: [{ youtubeId: 'v1' } as never],
			contentLoading: true,
			contentError: 'err',
			favorites: [{ youtubeId: 'v2' } as never],
			favoritesLoading: true
		});

		youtubeLibraryService.reset();

		expect(get(youtubeLibraryService.library)).toBeNull();
		const state = get(youtubeLibraryService.state);
		expect(state.content).toEqual([]);
		expect(state.contentLoading).toBe(false);
		expect(state.contentError).toBeNull();
	});

	it('streamVideoUrl returns correct URL', () => {
		const url = youtubeLibraryService.streamVideoUrl('abc123');
		expect(url).toContain('/api/libraries/content/abc123/stream/video');
	});

	it('streamAudioUrl returns correct URL', () => {
		const url = youtubeLibraryService.streamAudioUrl('abc123');
		expect(url).toContain('/api/libraries/content/abc123/stream/audio');
	});

	it('streamDownloadVideoUrl returns correct URL', () => {
		const url = youtubeLibraryService.streamDownloadVideoUrl('dl-1');
		expect(url).toContain('/api/ytdl/downloads/dl-1/stream/video');
	});

	it('fetchContent updates content state', async () => {
		const mockContent = [
			{ youtubeId: 'v1', title: 'Video 1', durationSeconds: 120 },
			{ youtubeId: 'v2', title: 'Video 2', durationSeconds: 60 }
		];
		vi.stubGlobal('fetch', mockFetch(mockContent));

		await youtubeLibraryService.fetchContent();

		const state = get(youtubeLibraryService.state);
		expect(state.content).toEqual(mockContent);
		expect(state.contentLoading).toBe(false);
	});

	it('fetchContent handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Server down')));

		await youtubeLibraryService.fetchContent();

		const state = get(youtubeLibraryService.state);
		expect(state.contentLoading).toBe(false);
		expect(state.contentError).toContain('Server down');
	});

	it('fetchFavorites updates favorites state', async () => {
		const mockFavorites = [{ youtubeId: 'v1', title: 'Fav 1', isFavorite: true }];
		vi.stubGlobal('fetch', mockFetch(mockFavorites));

		await youtubeLibraryService.fetchFavorites();

		const state = get(youtubeLibraryService.state);
		expect(state.favorites).toEqual(mockFavorites);
		expect(state.favoritesLoading).toBe(false);
	});

	it('toggleFavorite sends PUT and updates content', async () => {
		youtubeLibraryService.state.set({
			...initialState,
			content: [
				{ youtubeId: 'v1', title: 'Video 1', isFavorite: false } as never,
				{ youtubeId: 'v2', title: 'Video 2', isFavorite: false } as never
			]
		});

		const mock = mockFetchSequence(
			{ data: { isFavorite: true } },
			{ data: [] } // fetchFavorites call
		);
		vi.stubGlobal('fetch', mock);

		const result = await youtubeLibraryService.toggleFavorite('v1');

		expect(result).toBe(true);
		const state = get(youtubeLibraryService.state);
		const v1 = state.content.find((c: { youtubeId: string }) => c.youtubeId === 'v1') as {
			isFavorite: boolean;
		};
		expect(v1.isFavorite).toBe(true);
	});

	it('deleteAudio sends DELETE and refreshes content', async () => {
		const mock = mockFetchSequence(
			{ data: null, ok: true },
			{ data: [] } // fetchContent call
		);
		vi.stubGlobal('fetch', mock);

		await youtubeLibraryService.deleteAudio('v1');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/media/v1/audio'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});
});

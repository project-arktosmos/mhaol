import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { lyricsService } from '../../src/services/lyrics.service';

const mockLyrics = {
	id: 'track-1',
	artist: 'Test Artist',
	title: 'Test Song',
	lines: ['Line 1', 'Line 2'],
	plainLyrics: 'Line 1\nLine 2',
	syncedLyrics: [
		{ time: 0, text: 'Line 1' },
		{ time: 5, text: 'Line 2' },
		{ time: 10, text: 'Line 3' }
	]
};

function mockFetchSuccess(data: unknown = mockLyrics) {
	return vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve(data)
	});
}

describe('lyricsService', () => {
	beforeEach(() => {
		lyricsService.clear();
		// Clear the internal cache
		(lyricsService as any).cache = new Map();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should have idle initial state', () => {
		const state = get(lyricsService.store);
		expect(state.status).toBe('idle');
		expect(state.lyrics).toBeNull();
		expect(state.error).toBeNull();
		expect(state.currentTrackId).toBeNull();
	});

	it('should fetch lyrics for an item id', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());

		await lyricsService.fetchForItemId('track-1');

		const state = get(lyricsService.store);
		expect(state.status).toBe('success');
		expect(state.lyrics!.artist).toBe('Test Artist');
		expect(state.currentTrackId).toBe('track-1');
	});

	it('should use cache on second fetch for same id', async () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		await lyricsService.fetchForItemId('track-1');
		await lyricsService.fetchForItemId('track-1');

		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('should handle 404 as not_found', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 404
			})
		);

		await lyricsService.fetchForItemId('unknown');

		const state = get(lyricsService.store);
		expect(state.status).toBe('not_found');
		expect(state.error).toBeNull();
	});

	it('should handle non-404 error', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 500
			})
		);

		await lyricsService.fetchForItemId('track-1');

		const state = get(lyricsService.store);
		expect(state.status).toBe('error');
		expect(state.error).toBe('HTTP 500');
	});

	it('should handle network error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network failure')));

		await lyricsService.fetchForItemId('track-1');

		const state = get(lyricsService.store);
		expect(state.status).toBe('error');
		expect(state.error).toBe('Network failure');
	});

	it('should clear state', () => {
		lyricsService.clear();
		const state = get(lyricsService.store);
		expect(state.status).toBe('idle');
		expect(state.lyrics).toBeNull();
		expect(state.currentTrackId).toBeNull();
	});

	it('should get current line index from synced lyrics', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());
		await lyricsService.fetchForItemId('track-1');

		expect(lyricsService.getCurrentLineIndex(0)).toBe(0);
		expect(lyricsService.getCurrentLineIndex(3)).toBe(0);
		expect(lyricsService.getCurrentLineIndex(5)).toBe(1);
		expect(lyricsService.getCurrentLineIndex(7)).toBe(1);
		expect(lyricsService.getCurrentLineIndex(10)).toBe(2);
	});

	it('should return -1 for getCurrentLineIndex when no synced lyrics', () => {
		expect(lyricsService.getCurrentLineIndex(5)).toBe(-1);
	});

	it('should clear lyrics for non-library file type', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());
		await lyricsService.fetchForItemId('track-1');

		await lyricsService.fetchForFile({ type: 'url', id: 'x', url: 'http://example.com' } as any);

		const state = get(lyricsService.store);
		expect(state.status).toBe('idle');
	});
});

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { searchRecordings, searchArtists, searchReleaseGroups } from '../src/client';

describe('MusicBrainz client', () => {
	const mockFetch = vi.fn();

	beforeEach(() => {
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('searchRecordings', () => {
		it('calls the correct endpoint', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ recordings: [] })
			});

			await searchRecordings('test song');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/recording?query=test%20song'),
				expect.any(Object)
			);
		});

		it('returns recording data', async () => {
			const data = { recordings: [{ id: '1', title: 'Song' }] };
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve(data)
			});

			const result = await searchRecordings('test');
			expect(result.recordings).toHaveLength(1);
		});

		it('throws on API error', async () => {
			mockFetch.mockResolvedValue({ ok: false, status: 503 });
			await expect(searchRecordings('test')).rejects.toThrow('MusicBrainz API error: 503');
		});
	});

	describe('searchArtists', () => {
		it('calls the artist endpoint', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ artists: [] })
			});

			await searchArtists('band name');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/artist?query=band%20name'),
				expect.any(Object)
			);
		});
	});

	describe('searchReleaseGroups', () => {
		it('calls the release-group endpoint', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ 'release-groups': [] })
			});

			await searchReleaseGroups('album');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('/release-group?query=album'),
				expect.any(Object)
			);
		});
	});
});

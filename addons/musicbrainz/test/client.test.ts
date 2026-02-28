import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
	searchArtists,
	searchReleaseGroups,
	searchRecordings,
	fetchArtist,
	fetchReleaseGroup,
	fetchReleasesForReleaseGroup,
	fetchArtistImage
} from '../src/client.js';

function mockFetchResponse(data: unknown, status = 200) {
	return vi.fn().mockResolvedValue({
		ok: status >= 200 && status < 300,
		status,
		json: () => Promise.resolve(data)
	});
}

beforeEach(() => {
	vi.useFakeTimers();
});

afterEach(() => {
	vi.restoreAllMocks();
	vi.useRealTimers();
});

describe('searchArtists', () => {
	it('calls MusicBrainz artist search endpoint with query params', async () => {
		const data = { created: '', count: 0, offset: 0, artists: [] };
		global.fetch = mockFetchResponse(data);

		const promise = searchArtists('Radiohead');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		expect(global.fetch).toHaveBeenCalledOnce();
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/artist');
		expect(url.searchParams.get('query')).toBe('Radiohead');
		expect(url.searchParams.get('fmt')).toBe('json');
		expect(url.searchParams.get('limit')).toBe('25');
		expect(url.searchParams.get('offset')).toBe('0');
	});

	it('passes custom limit and offset', async () => {
		global.fetch = mockFetchResponse({ created: '', count: 0, offset: 0, artists: [] });

		const promise = searchArtists('Test', 10, 50);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.searchParams.get('limit')).toBe('10');
		expect(url.searchParams.get('offset')).toBe('50');
	});

	it('sends User-Agent header', async () => {
		global.fetch = mockFetchResponse({ created: '', count: 0, offset: 0, artists: [] });

		const promise = searchArtists('Test');
		await vi.runAllTimersAsync();
		await promise;

		const options = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][1];
		expect(options.headers['User-Agent']).toBe('MhaolMedia/1.0.0 (https://github.com/mhaol)');
	});
});

describe('searchReleaseGroups', () => {
	it('calls MusicBrainz release-group search endpoint', async () => {
		const data = { created: '', count: 0, offset: 0, 'release-groups': [] };
		global.fetch = mockFetchResponse(data);

		const promise = searchReleaseGroups('OK Computer');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/release-group');
		expect(url.searchParams.get('query')).toBe('OK Computer');
	});
});

describe('searchRecordings', () => {
	it('calls MusicBrainz recording search endpoint', async () => {
		const data = { created: '', count: 0, offset: 0, recordings: [] };
		global.fetch = mockFetchResponse(data);

		const promise = searchRecordings('Creep');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/recording');
		expect(url.searchParams.get('query')).toBe('Creep');
	});
});

describe('fetchArtist', () => {
	it('calls MusicBrainz artist lookup endpoint with inc params', async () => {
		const data = { id: 'abc-123', name: 'Radiohead' };
		global.fetch = mockFetchResponse(data);

		const promise = fetchArtist('abc-123');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/artist/abc-123');
		expect(url.searchParams.get('inc')).toBe('tags+release-groups');
	});

	it('returns null on 404', async () => {
		global.fetch = mockFetchResponse(null, 404);

		const promise = fetchArtist('nonexistent');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toBeNull();
	});
});

describe('fetchReleaseGroup', () => {
	it('calls MusicBrainz release-group lookup endpoint', async () => {
		const data = { id: 'rg-123', title: 'OK Computer' };
		global.fetch = mockFetchResponse(data);

		const promise = fetchReleaseGroup('rg-123');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/release-group/rg-123');
		expect(url.searchParams.get('inc')).toBe('artist-credits');
	});
});

describe('fetchReleasesForReleaseGroup', () => {
	it('calls MusicBrainz release endpoint with release-group filter', async () => {
		const data = { created: '', count: 1, offset: 0, releases: [{ id: 'rel-1' }] };
		global.fetch = mockFetchResponse(data);

		const promise = fetchReleasesForReleaseGroup('rg-123');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/ws/2/release');
		expect(url.searchParams.get('release-group')).toBe('rg-123');
		expect(url.searchParams.get('inc')).toBe('recordings+media+artist-credits+release-groups');
		expect(url.searchParams.get('status')).toBe('official');
		expect(url.searchParams.get('limit')).toBe('1');
	});
});

describe('fetchArtistImage', () => {
	it('returns artist thumbnail from TheAudioDB', async () => {
		global.fetch = mockFetchResponse({
			artists: [{ strArtistThumb: 'https://example.com/thumb.jpg' }]
		});

		const result = await fetchArtistImage('abc-123');

		expect(result).toBe('https://example.com/thumb.jpg');
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.hostname).toBe('www.theaudiodb.com');
		expect(url.searchParams.get('i')).toBe('abc-123');
	});

	it('returns fanart when no thumbnail', async () => {
		global.fetch = mockFetchResponse({
			artists: [{ strArtistFanart: 'https://example.com/fanart.jpg' }]
		});

		const result = await fetchArtistImage('abc-123');

		expect(result).toBe('https://example.com/fanart.jpg');
	});

	it('returns null when no artist found', async () => {
		global.fetch = mockFetchResponse({ artists: null });

		const result = await fetchArtistImage('abc-123');

		expect(result).toBeNull();
	});

	it('returns null on fetch error', async () => {
		global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

		const result = await fetchArtistImage('abc-123');

		expect(result).toBeNull();
	});

	it('returns null on non-ok response', async () => {
		global.fetch = mockFetchResponse(null, 500);

		const result = await fetchArtistImage('abc-123');

		expect(result).toBeNull();
	});
});

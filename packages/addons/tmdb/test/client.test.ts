import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
	searchMovies,
	getNowPlaying,
	getPopular,
	getUpcoming,
	getTopRated,
	fetchMovie,
	searchTvShows,
	getTvAiringToday,
	getTvOnTheAir,
	getTvPopular,
	getTvTopRated,
	fetchTvShow,
	fetchSeasonDetails,
	fetchAllSeasons
} from '../src/client.js';

const API_KEY = 'test-api-key';

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

describe('searchMovies', () => {
	it('calls TMDB search/movie endpoint with query params', async () => {
		const data = { page: 1, results: [], total_pages: 0, total_results: 0 };
		global.fetch = mockFetchResponse(data);

		const promise = searchMovies(API_KEY, 'Inception');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		expect(global.fetch).toHaveBeenCalledOnce();
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/search/movie');
		expect(url.searchParams.get('query')).toBe('Inception');
		expect(url.searchParams.get('api_key')).toBe(API_KEY);
		expect(url.searchParams.get('include_adult')).toBe('false');
	});

	it('includes year param when provided', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 0, total_results: 0 });

		const promise = searchMovies(API_KEY, 'Inception', 1, 2010);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.searchParams.get('year')).toBe('2010');
	});

	it('includes page param', async () => {
		global.fetch = mockFetchResponse({ page: 2, results: [], total_pages: 5, total_results: 100 });

		const promise = searchMovies(API_KEY, 'Test', 2);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.searchParams.get('page')).toBe('2');
	});
});

describe('getNowPlaying', () => {
	it('calls TMDB now_playing endpoint', async () => {
		const data = { page: 1, results: [], total_pages: 1, total_results: 0 };
		global.fetch = mockFetchResponse(data);

		const promise = getNowPlaying(API_KEY);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/movie/now_playing');
	});
});

describe('getPopular', () => {
	it('calls TMDB popular endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getPopular(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/movie/popular');
	});
});

describe('getUpcoming', () => {
	it('calls TMDB upcoming endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getUpcoming(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/movie/upcoming');
	});
});

describe('getTopRated', () => {
	it('calls TMDB top_rated endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getTopRated(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/movie/top_rated');
	});
});

describe('fetchMovie', () => {
	it('calls TMDB movie detail endpoint with credits appended', async () => {
		const movieData = { id: 123, title: 'Test Movie' };
		global.fetch = mockFetchResponse(movieData);

		const promise = fetchMovie(API_KEY, 123);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(movieData);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/movie/123');
		expect(url.searchParams.get('append_to_response')).toBe('credits');
	});

	it('returns null on 404', async () => {
		global.fetch = mockFetchResponse(null, 404);

		const promise = fetchMovie(API_KEY, 999);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toBeNull();
	});
});

describe('searchTvShows', () => {
	it('calls TMDB search/tv endpoint', async () => {
		const data = { page: 1, results: [], total_pages: 0, total_results: 0 };
		global.fetch = mockFetchResponse(data);

		const promise = searchTvShows(API_KEY, 'Breaking Bad');
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(data);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/search/tv');
		expect(url.searchParams.get('query')).toBe('Breaking Bad');
	});

	it('includes first_air_date_year when provided', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 0, total_results: 0 });

		const promise = searchTvShows(API_KEY, 'Test', 1, 2020);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.searchParams.get('first_air_date_year')).toBe('2020');
	});
});

describe('getTvAiringToday', () => {
	it('calls TMDB tv/airing_today endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getTvAiringToday(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/airing_today');
	});
});

describe('getTvOnTheAir', () => {
	it('calls TMDB tv/on_the_air endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getTvOnTheAir(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/on_the_air');
	});
});

describe('getTvPopular', () => {
	it('calls TMDB tv/popular endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getTvPopular(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/popular');
	});
});

describe('getTvTopRated', () => {
	it('calls TMDB tv/top_rated endpoint', async () => {
		global.fetch = mockFetchResponse({ page: 1, results: [], total_pages: 1, total_results: 0 });

		const promise = getTvTopRated(API_KEY);
		await vi.runAllTimersAsync();
		await promise;

		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/top_rated');
	});
});

describe('fetchTvShow', () => {
	it('calls TMDB tv detail endpoint with credits appended', async () => {
		const tvData = { id: 456, name: 'Test Show' };
		global.fetch = mockFetchResponse(tvData);

		const promise = fetchTvShow(API_KEY, 456);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(tvData);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/456');
		expect(url.searchParams.get('append_to_response')).toBe('credits');
	});

	it('returns null on 404', async () => {
		global.fetch = mockFetchResponse(null, 404);

		const promise = fetchTvShow(API_KEY, 999);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toBeNull();
	});
});

describe('fetchSeasonDetails', () => {
	it('calls TMDB season detail endpoint', async () => {
		const seasonData = { id: 1, name: 'Season 1', episodes: [] };
		global.fetch = mockFetchResponse(seasonData);

		const promise = fetchSeasonDetails(API_KEY, 456, 1);
		await vi.runAllTimersAsync();
		const result = await promise;

		expect(result).toEqual(seasonData);
		const url = new URL((global.fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]);
		expect(url.pathname).toBe('/3/tv/456/season/1');
	});
});

describe('fetchAllSeasons', () => {
	it('fetches all specified seasons', async () => {
		const season1 = { id: 1, name: 'Season 1', episodes: [] };
		const season2 = { id: 2, name: 'Season 2', episodes: [] };
		let callCount = 0;
		global.fetch = vi.fn().mockImplementation(() => {
			callCount++;
			const data = callCount === 1 ? season1 : season2;
			return Promise.resolve({
				ok: true,
				status: 200,
				json: () => Promise.resolve(data)
			});
		});

		const promise = fetchAllSeasons(API_KEY, 456, [1, 2]);
		await vi.runAllTimersAsync();
		const results = await promise;

		expect(results).toHaveLength(2);
		expect(results[0]).toEqual(season1);
		expect(results[1]).toEqual(season2);
	});

	it('skips seasons that return null (404)', async () => {
		let callCount = 0;
		global.fetch = vi.fn().mockImplementation(() => {
			callCount++;
			if (callCount === 1) {
				return Promise.resolve({
					ok: false,
					status: 404,
					json: () => Promise.resolve(null)
				});
			}
			return Promise.resolve({
				ok: true,
				status: 200,
				json: () => Promise.resolve({ id: 2, name: 'Season 2', episodes: [] })
			});
		});

		const promise = fetchAllSeasons(API_KEY, 456, [1, 2]);
		await vi.runAllTimersAsync();
		const results = await promise;

		expect(results).toHaveLength(1);
		expect(results[0].name).toBe('Season 2');
	});
});

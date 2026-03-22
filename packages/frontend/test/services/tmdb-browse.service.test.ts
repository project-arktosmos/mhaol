import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

describe('TmdbBrowseService', () => {
	let tmdbBrowseService: (typeof import('../../src/services/tmdb-browse.service'))['tmdbBrowseService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/tmdb-browse.service');
		tmdbBrowseService = mod.tmdbBrowseService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(tmdbBrowseService.state);
		expect(state.popularMovies).toEqual([]);
		expect(state.popularTv).toEqual([]);
		expect(state.popularMoviesPage).toBe(1);
		expect(state.popularTvPage).toBe(1);
		expect(state.discoverMovies).toEqual([]);
		expect(state.discoverTv).toEqual([]);
		expect(state.movieGenres).toEqual([]);
		expect(state.tvGenres).toEqual([]);
		expect(state.searchQuery).toBe('');
		expect(state.loading).toEqual({});
		expect(state.error).toBeNull();
		expect(state.selectedGenreId).toBeNull();
	});

	it('should load genres', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('genres/movie')) {
					return {
						ok: true,
						json: async () => ({
							genres: [
								{ id: 28, name: 'Action' },
								{ id: 35, name: 'Comedy' }
							]
						})
					} as Response;
				}
				if (url.includes('genres/tv')) {
					return {
						ok: true,
						json: async () => ({
							genres: [
								{ id: 18, name: 'Drama' },
								{ id: 10759, name: 'Action & Adventure' }
							]
						})
					} as Response;
				}
				return { ok: false, status: 404 } as Response;
			})
		);

		await tmdbBrowseService.loadGenres();

		const state = get(tmdbBrowseService.state);
		expect(state.movieGenres).toHaveLength(2);
		expect(state.movieGenres[0].name).toBe('Action');
		expect(state.tvGenres).toHaveLength(2);
		expect(state.tvGenres[0].name).toBe('Drama');
	});

	it('should only load genres once', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({ genres: [] })
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await tmdbBrowseService.loadGenres();
		await tmdbBrowseService.loadGenres();

		// Should be called twice (movie + tv) from the first call only
		expect(mockFetch).toHaveBeenCalledTimes(2);
	});

	it('should load popular movies', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 1,
									title: 'Test Movie',
									poster_path: '/test.jpg',
									overview: 'desc',
									release_date: '2024-01-01',
									vote_average: 7.5
								}
							],
							page: 1,
							total_pages: 10
						})
					}) as Response
			)
		);

		await tmdbBrowseService.loadPopularMovies();

		const state = get(tmdbBrowseService.state);
		expect(state.popularMovies).toHaveLength(1);
		expect(state.popularMoviesPage).toBe(1);
		expect(state.popularMoviesTotalPages).toBe(10);
	});

	it('should load popular TV shows', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 1,
									name: 'Test Show',
									poster_path: '/test.jpg',
									overview: 'desc',
									first_air_date: '2024-01-01',
									vote_average: 8.0
								}
							],
							page: 2,
							total_pages: 5
						})
					}) as Response
			)
		);

		await tmdbBrowseService.loadPopularTv(2);

		const state = get(tmdbBrowseService.state);
		expect(state.popularTv).toHaveLength(1);
		expect(state.popularTvPage).toBe(2);
		expect(state.popularTvTotalPages).toBe(5);
	});

	it('should set error on popular movies failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: false,
						status: 500
					}) as Response
			)
		);

		await tmdbBrowseService.loadPopularMovies();

		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should load discover movies with genre filter', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({
						results: [],
						page: 1,
						total_pages: 1
					})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await tmdbBrowseService.loadDiscoverMovies(1, 28);

		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('with_genres=28'));

		const state = get(tmdbBrowseService.state);
		expect(state.selectedGenreId).toBe(28);
	});

	it('should search movies', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 1,
									title: 'Inception',
									poster_path: '/inception.jpg',
									overview: 'dreams',
									release_date: '2010-07-16',
									vote_average: 8.8
								}
							],
							page: 1,
							total_pages: 1
						})
					}) as Response
			)
		);

		await tmdbBrowseService.searchMovies('inception');

		const state = get(tmdbBrowseService.state);
		expect(state.searchMovies).toHaveLength(1);
		expect(state.searchQuery).toBe('inception');
	});

	it('should not search with empty query', async () => {
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		await tmdbBrowseService.searchMovies('   ');

		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('should load discover TV', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({
						results: [
							{
								id: 1,
								name: 'Show',
								poster_path: '/s.jpg',
								overview: '',
								first_air_date: '2024-01-01',
								vote_average: 7
							}
						],
						page: 1,
						total_pages: 3
					})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await tmdbBrowseService.loadDiscoverTv(1, 18);

		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('with_genres=18'));
		const state = get(tmdbBrowseService.state);
		expect(state.discoverTv).toHaveLength(1);
		expect(state.selectedGenreId).toBe(18);
	});

	it('should load discover TV without genre', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({ results: [], page: 1, total_pages: 1 })
					}) as Response
			)
		);

		await tmdbBrowseService.loadDiscoverTv(1, null);

		const state = get(tmdbBrowseService.state);
		expect(state.selectedGenreId).toBeNull();
	});

	it('should search TV shows', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 1,
									name: 'Breaking Bad',
									poster_path: '/bb.jpg',
									overview: 'meth',
									first_air_date: '2008-01-01',
									vote_average: 9
								}
							],
							page: 1,
							total_pages: 1
						})
					}) as Response
			)
		);

		await tmdbBrowseService.searchTv('breaking bad');

		const state = get(tmdbBrowseService.state);
		expect(state.searchTv).toHaveLength(1);
		expect(state.searchQuery).toBe('breaking bad');
	});

	it('should not search TV with empty query', async () => {
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);
		await tmdbBrowseService.searchTv('');
		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('should set error on search TV failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);
		await tmdbBrowseService.searchTv('test');
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should load movie recommendations', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 2,
									title: 'Similar Movie',
									poster_path: '/sm.jpg',
									overview: '',
									release_date: '2024-06-01',
									vote_average: 7
								}
							],
							page: 1,
							total_pages: 2
						})
					}) as Response
			)
		);

		await tmdbBrowseService.loadRecommendations(1, 'movie');

		const state = get(tmdbBrowseService.state);
		expect(state.recommendations).toHaveLength(1);
		expect(state.recommendationSourceId).toBe(1);
		expect(state.recommendationSourceType).toBe('movie');
	});

	it('should load TV recommendations', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							results: [
								{
									id: 3,
									name: 'Similar Show',
									poster_path: '/ss.jpg',
									overview: '',
									first_air_date: '2024-01-01',
									vote_average: 8
								}
							],
							page: 1,
							total_pages: 1
						})
					}) as Response
			)
		);

		await tmdbBrowseService.loadRecommendations(1, 'tv');

		const state = get(tmdbBrowseService.state);
		expect(state.recommendations).toHaveLength(1);
		expect(state.recommendationSourceType).toBe('tv');
	});

	it('should set error on recommendations failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 404 }) as Response)
		);
		await tmdbBrowseService.loadRecommendations(999, 'movie');
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should set error on popular TV failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);
		await tmdbBrowseService.loadPopularTv();
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should set error on discover movies failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);
		await tmdbBrowseService.loadDiscoverMovies();
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should set error on discover TV failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);
		await tmdbBrowseService.loadDiscoverTv();
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should set error on search movies failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);
		await tmdbBrowseService.searchMovies('test');
		const state = get(tmdbBrowseService.state);
		expect(state.error).toBeTruthy();
	});

	it('should load discover movies without genre', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({ results: [], page: 1, total_pages: 1 })
					}) as Response
			)
		);
		await tmdbBrowseService.loadDiscoverMovies(1, null);
		const state = get(tmdbBrowseService.state);
		expect(state.selectedGenreId).toBeNull();
	});

	it('should reset state', async () => {
		// First load some data
		tmdbBrowseService.state.update((s) => ({
			...s,
			popularMovies: [{ id: 1 }] as never,
			searchQuery: 'test',
			error: 'some error'
		}));

		tmdbBrowseService.reset();

		const state = get(tmdbBrowseService.state);
		expect(state.popularMovies).toEqual([]);
		expect(state.searchQuery).toBe('');
		expect(state.error).toBeNull();
	});
});

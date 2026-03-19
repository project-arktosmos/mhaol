import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';
import { moviesToDisplay, tvShowsToDisplay } from 'addons/tmdb/transform';
import type { TmdbBrowseState, TmdbGenre } from 'frontend/types/tmdb-browse.type';
import type { DisplayTMDBMovie, DisplayTMDBTvShow, TMDBMovie, TMDBTvShow } from 'addons/tmdb/types';

interface TmdbPagedResponse<T> {
	results: T[];
	total_pages: number;
	page: number;
}

const initialState: TmdbBrowseState = {
	popularMovies: [],
	popularTv: [],
	popularMoviesPage: 1,
	popularTvPage: 1,
	popularMoviesTotalPages: 1,
	popularTvTotalPages: 1,

	discoverMovies: [],
	discoverTv: [],
	discoverMoviesPage: 1,
	discoverTvPage: 1,
	discoverMoviesTotalPages: 1,
	discoverTvTotalPages: 1,
	selectedGenreId: null,

	movieGenres: [],
	tvGenres: [],

	recommendations: [],
	recommendationsPage: 1,
	recommendationsTotalPages: 1,
	recommendationSourceId: null,
	recommendationSourceType: null,

	loading: {},
	error: null
};

class TmdbBrowseService {
	public state: Writable<TmdbBrowseState> = writable(initialState);
	private genresLoaded = false;

	private setLoading(key: string, value: boolean) {
		this.state.update((s) => ({
			...s,
			loading: { ...s.loading, [key]: value },
			error: value ? null : s.error
		}));
	}

	private async fetchJson<T>(path: string): Promise<T> {
		const response = await fetch(apiUrl(path));
		if (!response.ok) throw new Error(`HTTP ${response.status}`);
		return response.json();
	}

	async loadGenres(): Promise<void> {
		if (!browser || this.genresLoaded) return;
		this.genresLoaded = true;
		try {
			const [movieRes, tvRes] = await Promise.all([
				this.fetchJson<{ genres: TmdbGenre[] }>('/api/tmdb/genres/movie'),
				this.fetchJson<{ genres: TmdbGenre[] }>('/api/tmdb/genres/tv')
			]);
			this.state.update((s) => ({
				...s,
				movieGenres: movieRes.genres,
				tvGenres: tvRes.genres
			}));
		} catch (e) {
			console.error('[tmdb-browse] Failed to load genres:', e);
		}
	}

	async loadPopularMovies(page: number = 1): Promise<void> {
		if (!browser) return;
		this.setLoading('popularMovies', true);
		try {
			const data = await this.fetchJson<TmdbPagedResponse<TMDBMovie>>(
				`/api/tmdb/popular/movies?page=${page}`
			);
			this.state.update((s) => ({
				...s,
				popularMovies: moviesToDisplay(data.results),
				popularMoviesPage: data.page,
				popularMoviesTotalPages: data.total_pages
			}));
		} catch (e) {
			this.state.update((s) => ({
				...s,
				error: e instanceof Error ? e.message : 'Failed to load popular movies'
			}));
		} finally {
			this.setLoading('popularMovies', false);
		}
	}

	async loadPopularTv(page: number = 1): Promise<void> {
		if (!browser) return;
		this.setLoading('popularTv', true);
		try {
			const data = await this.fetchJson<TmdbPagedResponse<TMDBTvShow>>(
				`/api/tmdb/popular/tv?page=${page}`
			);
			this.state.update((s) => ({
				...s,
				popularTv: tvShowsToDisplay(data.results),
				popularTvPage: data.page,
				popularTvTotalPages: data.total_pages
			}));
		} catch (e) {
			this.state.update((s) => ({
				...s,
				error: e instanceof Error ? e.message : 'Failed to load popular TV'
			}));
		} finally {
			this.setLoading('popularTv', false);
		}
	}

	async loadDiscoverMovies(page: number = 1, genreId: number | null = null): Promise<void> {
		if (!browser) return;
		this.setLoading('discoverMovies', true);
		try {
			let url = `/api/tmdb/discover/movies?page=${page}`;
			if (genreId) url += `&with_genres=${genreId}`;
			const data = await this.fetchJson<TmdbPagedResponse<TMDBMovie>>(url);
			this.state.update((s) => ({
				...s,
				discoverMovies: moviesToDisplay(data.results),
				discoverMoviesPage: data.page,
				discoverMoviesTotalPages: data.total_pages,
				selectedGenreId: genreId
			}));
		} catch (e) {
			this.state.update((s) => ({
				...s,
				error: e instanceof Error ? e.message : 'Failed to discover movies'
			}));
		} finally {
			this.setLoading('discoverMovies', false);
		}
	}

	async loadDiscoverTv(page: number = 1, genreId: number | null = null): Promise<void> {
		if (!browser) return;
		this.setLoading('discoverTv', true);
		try {
			let url = `/api/tmdb/discover/tv?page=${page}`;
			if (genreId) url += `&with_genres=${genreId}`;
			const data = await this.fetchJson<TmdbPagedResponse<TMDBTvShow>>(url);
			this.state.update((s) => ({
				...s,
				discoverTv: tvShowsToDisplay(data.results),
				discoverTvPage: data.page,
				discoverTvTotalPages: data.total_pages,
				selectedGenreId: genreId
			}));
		} catch (e) {
			this.state.update((s) => ({
				...s,
				error: e instanceof Error ? e.message : 'Failed to discover TV'
			}));
		} finally {
			this.setLoading('discoverTv', false);
		}
	}

	async loadRecommendations(tmdbId: number, type: 'movie' | 'tv', page: number = 1): Promise<void> {
		if (!browser) return;
		this.setLoading('recommendations', true);
		try {
			const endpoint =
				type === 'movie'
					? `/api/tmdb/movies/${tmdbId}/recommendations?page=${page}`
					: `/api/tmdb/tv/${tmdbId}/recommendations?page=${page}`;
			let items: (DisplayTMDBMovie | DisplayTMDBTvShow)[];
			let pageNum: number;
			let totalPagesNum: number;
			if (type === 'movie') {
				const data = await this.fetchJson<TmdbPagedResponse<TMDBMovie>>(endpoint);
				items = moviesToDisplay(data.results);
				pageNum = data.page;
				totalPagesNum = data.total_pages;
			} else {
				const data = await this.fetchJson<TmdbPagedResponse<TMDBTvShow>>(endpoint);
				items = tvShowsToDisplay(data.results);
				pageNum = data.page;
				totalPagesNum = data.total_pages;
			}
			this.state.update((s) => ({
				...s,
				recommendations: items,
				recommendationsPage: pageNum,
				recommendationsTotalPages: totalPagesNum,
				recommendationSourceId: tmdbId,
				recommendationSourceType: type
			}));
		} catch (e) {
			this.state.update((s) => ({
				...s,
				error: e instanceof Error ? e.message : 'Failed to load recommendations'
			}));
		} finally {
			this.setLoading('recommendations', false);
		}
	}

	reset(): void {
		this.state.set(initialState);
		this.genresLoaded = false;
	}
}

export const tmdbBrowseService = new TmdbBrowseService();

import type {
	TMDBSearchResponse,
	TMDBMovieDetails,
	TMDBTvSearchResponse,
	TMDBTvShowDetails,
	TMDBSeasonDetails
} from '$types/tmdb.type';
import { tmdbRateLimiter } from '$utils/rate-limiter.util';

class TMDBService {
	private baseUrl = 'https://api.themoviedb.org/3';
	private apiKey: string | null = null;

	constructor() {
		const envKey = import.meta.env.VITE_TMDB_API_KEY;
		if (envKey) {
			this.apiKey = envKey;
		}
	}

	setApiKey(key: string) {
		this.apiKey = key;
	}

	getApiKey(): string | null {
		return this.apiKey;
	}

	isConfigured(): boolean {
		return this.apiKey !== null && this.apiKey.length > 0;
	}

	private async fetch<T>(
		endpoint: string,
		params: Record<string, string> = {}
	): Promise<T | null> {
		if (!this.apiKey) {
			throw new Error('TMDB API key not configured');
		}

		try {
			return await tmdbRateLimiter.enqueue(async () => {
				const searchParams = new URLSearchParams({
					...params,
					api_key: this.apiKey!
				});

				const url = `${this.baseUrl}${endpoint}?${searchParams.toString()}`;

				const response = await fetch(url, {
					headers: {
						Accept: 'application/json'
					}
				});

				if (!response.ok) {
					if (response.status === 404) {
						return null;
					}
					if (response.status === 429) {
						throw new Error('429 Rate Limited');
					}
					return null;
				}

				return await response.json();
			});
		} catch (error) {
			throw error;
		}
	}

	// =========================================================================
	// Movie Methods
	// =========================================================================

	async searchMovies(
		query: string,
		page: number = 1,
		year?: number
	): Promise<TMDBSearchResponse | null> {
		const params: Record<string, string> = {
			query,
			page: page.toString(),
			include_adult: 'false'
		};
		if (year) {
			params.year = year.toString();
		}
		return this.fetch<TMDBSearchResponse>('/search/movie', params);
	}

	async getNowPlaying(page: number = 1): Promise<TMDBSearchResponse | null> {
		return this.fetch<TMDBSearchResponse>('/movie/now_playing', {
			page: page.toString()
		});
	}

	async getPopular(page: number = 1): Promise<TMDBSearchResponse | null> {
		return this.fetch<TMDBSearchResponse>('/movie/popular', {
			page: page.toString()
		});
	}

	async getUpcoming(page: number = 1): Promise<TMDBSearchResponse | null> {
		return this.fetch<TMDBSearchResponse>('/movie/upcoming', {
			page: page.toString()
		});
	}

	async getTopRated(page: number = 1): Promise<TMDBSearchResponse | null> {
		return this.fetch<TMDBSearchResponse>('/movie/top_rated', {
			page: page.toString()
		});
	}

	async fetchMovie(id: number): Promise<TMDBMovieDetails | null> {
		return this.fetch<TMDBMovieDetails>(`/movie/${id}`, {
			append_to_response: 'credits'
		});
	}

	// =========================================================================
	// TV Show Methods
	// =========================================================================

	async searchTvShows(
		query: string,
		page: number = 1,
		firstAirYear?: number
	): Promise<TMDBTvSearchResponse | null> {
		const params: Record<string, string> = {
			query,
			page: page.toString(),
			include_adult: 'false'
		};
		if (firstAirYear) {
			params.first_air_date_year = firstAirYear.toString();
		}
		return this.fetch<TMDBTvSearchResponse>('/search/tv', params);
	}

	async getTvAiringToday(page: number = 1): Promise<TMDBTvSearchResponse | null> {
		return this.fetch<TMDBTvSearchResponse>('/tv/airing_today', {
			page: page.toString()
		});
	}

	async getTvOnTheAir(page: number = 1): Promise<TMDBTvSearchResponse | null> {
		return this.fetch<TMDBTvSearchResponse>('/tv/on_the_air', {
			page: page.toString()
		});
	}

	async getTvPopular(page: number = 1): Promise<TMDBTvSearchResponse | null> {
		return this.fetch<TMDBTvSearchResponse>('/tv/popular', {
			page: page.toString()
		});
	}

	async getTvTopRated(page: number = 1): Promise<TMDBTvSearchResponse | null> {
		return this.fetch<TMDBTvSearchResponse>('/tv/top_rated', {
			page: page.toString()
		});
	}

	async fetchTvShow(id: number): Promise<TMDBTvShowDetails | null> {
		return this.fetch<TMDBTvShowDetails>(`/tv/${id}`, {
			append_to_response: 'credits'
		});
	}

	async fetchSeasonDetails(
		tvShowId: number,
		seasonNumber: number
	): Promise<TMDBSeasonDetails | null> {
		return this.fetch<TMDBSeasonDetails>(`/tv/${tvShowId}/season/${seasonNumber}`);
	}

	async fetchAllSeasons(
		tvShowId: number,
		seasonNumbers: number[]
	): Promise<TMDBSeasonDetails[]> {
		const results: TMDBSeasonDetails[] = [];
		for (const seasonNumber of seasonNumbers) {
			const season = await this.fetchSeasonDetails(tvShowId, seasonNumber);
			if (season) {
				results.push(season);
			}
		}
		return results;
	}

	get pendingRequests(): number {
		return tmdbRateLimiter.queueLength;
	}
}

export const tmdbService = new TMDBService();

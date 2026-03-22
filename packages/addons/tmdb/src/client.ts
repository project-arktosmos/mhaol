import type {
	TMDBSearchResponse,
	TMDBMovieDetails,
	TMDBTvSearchResponse,
	TMDBTvShowDetails,
	TMDBSeasonDetails
} from './types.js';
import { tmdbRateLimiter } from './rate-limiter.js';

const TMDB_BASE_URL = 'https://api.themoviedb.org/3';

async function tmdbFetch<T>(
	apiKey: string,
	endpoint: string,
	params: Record<string, string> = {}
): Promise<T | null> {
	return tmdbRateLimiter.enqueue(async () => {
		const searchParams = new URLSearchParams({
			...params,
			api_key: apiKey
		});

		const url = `${TMDB_BASE_URL}${endpoint}?${searchParams.toString()}`;

		const response = await fetch(url, {
			headers: { Accept: 'application/json' }
		});

		if (!response.ok) {
			if (response.status === 404) return null;
			if (response.status === 429) throw new Error('429 Rate Limited');
			return null;
		}

		return await response.json();
	});
}

// Movie functions

export async function searchMovies(
	apiKey: string,
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
	return tmdbFetch<TMDBSearchResponse>(apiKey, '/search/movie', params);
}

export async function getNowPlaying(
	apiKey: string,
	page: number = 1
): Promise<TMDBSearchResponse | null> {
	return tmdbFetch<TMDBSearchResponse>(apiKey, '/movie/now_playing', {
		page: page.toString()
	});
}

export async function getPopular(
	apiKey: string,
	page: number = 1
): Promise<TMDBSearchResponse | null> {
	return tmdbFetch<TMDBSearchResponse>(apiKey, '/movie/popular', {
		page: page.toString()
	});
}

export async function getUpcoming(
	apiKey: string,
	page: number = 1
): Promise<TMDBSearchResponse | null> {
	return tmdbFetch<TMDBSearchResponse>(apiKey, '/movie/upcoming', {
		page: page.toString()
	});
}

export async function getTopRated(
	apiKey: string,
	page: number = 1
): Promise<TMDBSearchResponse | null> {
	return tmdbFetch<TMDBSearchResponse>(apiKey, '/movie/top_rated', {
		page: page.toString()
	});
}

export async function fetchMovie(apiKey: string, id: number): Promise<TMDBMovieDetails | null> {
	return tmdbFetch<TMDBMovieDetails>(apiKey, `/movie/${id}`, {
		append_to_response: 'credits,images',
		include_image_language: 'en,null'
	});
}

// TV functions

export async function searchTvShows(
	apiKey: string,
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
	return tmdbFetch<TMDBTvSearchResponse>(apiKey, '/search/tv', params);
}

export async function getTvAiringToday(
	apiKey: string,
	page: number = 1
): Promise<TMDBTvSearchResponse | null> {
	return tmdbFetch<TMDBTvSearchResponse>(apiKey, '/tv/airing_today', {
		page: page.toString()
	});
}

export async function getTvOnTheAir(
	apiKey: string,
	page: number = 1
): Promise<TMDBTvSearchResponse | null> {
	return tmdbFetch<TMDBTvSearchResponse>(apiKey, '/tv/on_the_air', {
		page: page.toString()
	});
}

export async function getTvPopular(
	apiKey: string,
	page: number = 1
): Promise<TMDBTvSearchResponse | null> {
	return tmdbFetch<TMDBTvSearchResponse>(apiKey, '/tv/popular', {
		page: page.toString()
	});
}

export async function getTvTopRated(
	apiKey: string,
	page: number = 1
): Promise<TMDBTvSearchResponse | null> {
	return tmdbFetch<TMDBTvSearchResponse>(apiKey, '/tv/top_rated', {
		page: page.toString()
	});
}

export async function fetchTvShow(apiKey: string, id: number): Promise<TMDBTvShowDetails | null> {
	return tmdbFetch<TMDBTvShowDetails>(apiKey, `/tv/${id}`, {
		append_to_response: 'credits,images',
		include_image_language: 'en,null'
	});
}

export async function fetchSeasonDetails(
	apiKey: string,
	tvShowId: number,
	seasonNumber: number
): Promise<TMDBSeasonDetails | null> {
	return tmdbFetch<TMDBSeasonDetails>(apiKey, `/tv/${tvShowId}/season/${seasonNumber}`);
}

export async function fetchAllSeasons(
	apiKey: string,
	tvShowId: number,
	seasonNumbers: number[]
): Promise<TMDBSeasonDetails[]> {
	const results: TMDBSeasonDetails[] = [];
	for (const seasonNumber of seasonNumbers) {
		const season = await fetchSeasonDetails(apiKey, tvShowId, seasonNumber);
		if (season) {
			results.push(season);
		}
	}
	return results;
}

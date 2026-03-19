import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'tmdb/types';

export interface TmdbGenre {
	id: number;
	name: string;
}

export type BrowseMediaType = 'movies' | 'tv';

export interface TmdbBrowseState {
	popularMovies: DisplayTMDBMovie[];
	popularTv: DisplayTMDBTvShow[];
	popularMoviesPage: number;
	popularTvPage: number;
	popularMoviesTotalPages: number;
	popularTvTotalPages: number;

	discoverMovies: DisplayTMDBMovie[];
	discoverTv: DisplayTMDBTvShow[];
	discoverMoviesPage: number;
	discoverTvPage: number;
	discoverMoviesTotalPages: number;
	discoverTvTotalPages: number;
	selectedGenreId: number | null;

	movieGenres: TmdbGenre[];
	tvGenres: TmdbGenre[];

	recommendations: (DisplayTMDBMovie | DisplayTMDBTvShow)[];
	recommendationsPage: number;
	recommendationsTotalPages: number;
	recommendationSourceId: number | null;
	recommendationSourceType: 'movie' | 'tv' | null;

	loading: Record<string, boolean>;
	error: string | null;
}

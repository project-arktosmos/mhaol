// TMDB API response types

export interface TMDBMovie {
	id: number;
	title: string;
	original_title: string;
	tagline?: string;
	overview: string;
	release_date: string;
	runtime?: number;
	genres?: TMDBGenre[];
	poster_path: string | null;
	backdrop_path: string | null;
	vote_average: number;
	vote_count: number;
	budget?: number;
	revenue?: number;
	imdb_id?: string;
	adult: boolean;
	original_language: string;
	popularity: number;
	status?: string;
}

export interface TMDBGenre {
	id: number;
	name: string;
}

export interface TMDBCastMember {
	id: number;
	name: string;
	character: string;
	profile_path: string | null;
	order: number;
}

export interface TMDBCrewMember {
	id: number;
	name: string;
	job: string;
	department: string;
	profile_path: string | null;
}

export interface TMDBCredits {
	cast: TMDBCastMember[];
	crew: TMDBCrewMember[];
}

export interface TMDBMovieDetails extends TMDBMovie {
	credits?: TMDBCredits;
}

export interface TMDBSearchResponse {
	page: number;
	results: TMDBMovie[];
	total_pages: number;
	total_results: number;
}

export interface DisplayTMDBMovie {
	id: number;
	title: string;
	originalTitle: string;
	releaseYear: string;
	overview: string;
	posterUrl: string | null;
	backdropUrl: string | null;
	voteAverage: number;
	voteCount: number;
	genres: string[];
}

export interface DisplayTMDBMovieDetails extends DisplayTMDBMovie {
	tagline: string | null;
	runtime: string | null;
	budget: string | null;
	revenue: string | null;
	imdbId: string | null;
	cast: DisplayTMDBCastMember[];
	director: string | null;
}

export interface DisplayTMDBCastMember {
	id: number;
	name: string;
	character: string;
	profileUrl: string | null;
}

export interface TMDBTvShow {
	id: number;
	name: string;
	original_name: string;
	overview: string;
	first_air_date: string;
	last_air_date?: string;
	poster_path: string | null;
	backdrop_path: string | null;
	vote_average: number;
	vote_count: number;
	adult: boolean;
	original_language: string;
	popularity: number;
	status?: string;
	tagline?: string;
	genres?: TMDBGenre[];
	number_of_seasons?: number;
	number_of_episodes?: number;
	episode_run_time?: number[];
	networks?: TMDBNetwork[];
	created_by?: TMDBCreator[];
}

export interface TMDBNetwork {
	id: number;
	name: string;
	logo_path: string | null;
	origin_country: string;
}

export interface TMDBCreator {
	id: number;
	name: string;
	profile_path: string | null;
}

export interface TMDBSeason {
	id: number;
	name: string;
	overview: string;
	air_date: string | null;
	episode_count: number;
	poster_path: string | null;
	season_number: number;
}

export interface TMDBEpisode {
	id: number;
	name: string;
	overview: string;
	air_date: string | null;
	episode_number: number;
	season_number: number;
	still_path: string | null;
	vote_average: number;
	vote_count: number;
	runtime?: number;
}

export interface TMDBSeasonDetails {
	id: number;
	name: string;
	overview: string;
	air_date: string | null;
	poster_path: string | null;
	season_number: number;
	episodes: TMDBEpisode[];
}

export interface TMDBTvShowDetails extends TMDBTvShow {
	seasons?: TMDBSeason[];
	credits?: TMDBCredits;
}

export interface TMDBTvSearchResponse {
	page: number;
	results: TMDBTvShow[];
	total_pages: number;
	total_results: number;
}

export interface DisplayTMDBTvShow {
	id: number;
	name: string;
	originalName: string;
	firstAirYear: string;
	lastAirYear: string | null;
	overview: string;
	posterUrl: string | null;
	backdropUrl: string | null;
	voteAverage: number;
	voteCount: number;
	genres: string[];
	numberOfSeasons: number | null;
	numberOfEpisodes: number | null;
}

export interface DisplayTMDBTvShowDetails extends DisplayTMDBTvShow {
	tagline: string | null;
	status: string | null;
	networks: string[];
	createdBy: string[];
	cast: DisplayTMDBCastMember[];
	seasons: DisplayTMDBSeason[];
}

export interface DisplayTMDBSeason {
	id: number;
	name: string;
	overview: string;
	airDate: string | null;
	episodeCount: number;
	posterUrl: string | null;
	seasonNumber: number;
}

export interface DisplayTMDBEpisode {
	id: number;
	name: string;
	overview: string;
	airDate: string | null;
	episodeNumber: number;
	seasonNumber: number;
	stillUrl: string | null;
	voteAverage: number;
	runtime: number | null;
}

export interface DisplayTMDBSeasonDetails {
	id: number;
	name: string;
	overview: string;
	airDate: string | null;
	posterUrl: string | null;
	seasonNumber: number;
	episodes: DisplayTMDBEpisode[];
}

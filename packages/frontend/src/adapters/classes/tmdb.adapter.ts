import { AdapterClass } from '$adapters/classes/adapter.class';
import type {
	TMDBMovie,
	TMDBMovieDetails,
	TMDBCastMember,
	DisplayTMDBMovie,
	DisplayTMDBMovieDetails,
	DisplayTMDBCastMember,
	TMDBTvShow,
	TMDBTvShowDetails,
	TMDBSeason,
	TMDBSeasonDetails,
	TMDBEpisode,
	DisplayTMDBTvShow,
	DisplayTMDBTvShowDetails,
	DisplayTMDBSeason,
	DisplayTMDBSeasonDetails,
	DisplayTMDBEpisode
} from '$types/tmdb.type';

export class TMDBAdapter extends AdapterClass {
	constructor() {
		super('tmdb');
	}

	private baseImageUrl = 'https://image.tmdb.org/t/p';

	getPosterUrl(path: string | null, size: 'w185' | 'w342' | 'w500' = 'w342'): string | null {
		if (!path) return null;
		return `${this.baseImageUrl}/${size}${path}`;
	}

	getBackdropUrl(path: string | null, size: 'w780' | 'w1280' | 'original' = 'w780'): string | null {
		if (!path) return null;
		return `${this.baseImageUrl}/${size}${path}`;
	}

	getProfileUrl(path: string | null, size: 'w45' | 'w185' | 'h632' = 'w185'): string | null {
		if (!path) return null;
		return `${this.baseImageUrl}/${size}${path}`;
	}

	getStillUrl(path: string | null, size: 'w185' | 'w300' | 'w500' = 'w300'): string | null {
		if (!path) return null;
		return `${this.baseImageUrl}/${size}${path}`;
	}

	formatRuntime(minutes: number | undefined): string | null {
		if (!minutes) return null;
		const hours = Math.floor(minutes / 60);
		const mins = minutes % 60;
		if (hours > 0) {
			return `${hours}h ${mins}m`;
		}
		return `${mins}m`;
	}

	formatCurrency(amount: number | undefined): string | null {
		if (!amount || amount === 0) return null;
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			maximumFractionDigits: 0
		}).format(amount);
	}

	extractYear(dateString: string | undefined): string {
		if (!dateString) return 'Unknown';
		return dateString.split('-')[0] || 'Unknown';
	}

	movieToDisplay(movie: TMDBMovie): DisplayTMDBMovie {
		return {
			id: movie.id,
			title: movie.title,
			originalTitle: movie.original_title,
			releaseYear: this.extractYear(movie.release_date),
			overview: movie.overview || '',
			posterUrl: this.getPosterUrl(movie.poster_path),
			backdropUrl: this.getBackdropUrl(movie.backdrop_path),
			voteAverage: movie.vote_average,
			voteCount: movie.vote_count,
			genres: movie.genres?.map((g) => g.name) || []
		};
	}

	moviesToDisplay(movies: TMDBMovie[]): DisplayTMDBMovie[] {
		return movies.map((m) => this.movieToDisplay(m));
	}

	castMemberToDisplay(cast: TMDBCastMember): DisplayTMDBCastMember {
		return {
			id: cast.id,
			name: cast.name,
			character: cast.character,
			profileUrl: this.getProfileUrl(cast.profile_path)
		};
	}

	movieDetailsToDisplay(movie: TMDBMovieDetails): DisplayTMDBMovieDetails {
		const director = movie.credits?.crew.find((c) => c.job === 'Director');
		const topCast = (movie.credits?.cast || [])
			.slice(0, 10)
			.map((c) => this.castMemberToDisplay(c));

		return {
			...this.movieToDisplay(movie),
			tagline: movie.tagline || null,
			runtime: this.formatRuntime(movie.runtime),
			budget: this.formatCurrency(movie.budget),
			revenue: this.formatCurrency(movie.revenue),
			imdbId: movie.imdb_id || null,
			cast: topCast,
			director: director?.name || null
		};
	}

	// =========================================================================
	// TV Show transformations
	// =========================================================================

	tvShowToDisplay(tvShow: TMDBTvShow): DisplayTMDBTvShow {
		return {
			id: tvShow.id,
			name: tvShow.name,
			originalName: tvShow.original_name,
			firstAirYear: this.extractYear(tvShow.first_air_date),
			lastAirYear: tvShow.last_air_date ? this.extractYear(tvShow.last_air_date) : null,
			overview: tvShow.overview || '',
			posterUrl: this.getPosterUrl(tvShow.poster_path),
			backdropUrl: this.getBackdropUrl(tvShow.backdrop_path),
			voteAverage: tvShow.vote_average,
			voteCount: tvShow.vote_count,
			genres: tvShow.genres?.map((g) => g.name) || [],
			numberOfSeasons: tvShow.number_of_seasons || null,
			numberOfEpisodes: tvShow.number_of_episodes || null
		};
	}

	tvShowsToDisplay(tvShows: TMDBTvShow[]): DisplayTMDBTvShow[] {
		return tvShows.map((t) => this.tvShowToDisplay(t));
	}

	seasonToDisplay(season: TMDBSeason): DisplayTMDBSeason {
		return {
			id: season.id,
			name: season.name,
			overview: season.overview || '',
			airDate: season.air_date,
			episodeCount: season.episode_count,
			posterUrl: this.getPosterUrl(season.poster_path),
			seasonNumber: season.season_number
		};
	}

	episodeToDisplay(episode: TMDBEpisode): DisplayTMDBEpisode {
		return {
			id: episode.id,
			name: episode.name,
			overview: episode.overview || '',
			airDate: episode.air_date,
			episodeNumber: episode.episode_number,
			seasonNumber: episode.season_number,
			stillUrl: this.getStillUrl(episode.still_path),
			voteAverage: episode.vote_average,
			runtime: episode.runtime || null
		};
	}

	seasonDetailsToDisplay(season: TMDBSeasonDetails): DisplayTMDBSeasonDetails {
		return {
			id: season.id,
			name: season.name,
			overview: season.overview || '',
			airDate: season.air_date,
			posterUrl: this.getPosterUrl(season.poster_path),
			seasonNumber: season.season_number,
			episodes: season.episodes.map((e) => this.episodeToDisplay(e))
		};
	}

	tvShowDetailsToDisplay(tvShow: TMDBTvShowDetails): DisplayTMDBTvShowDetails {
		const topCast = (tvShow.credits?.cast || [])
			.slice(0, 10)
			.map((c) => this.castMemberToDisplay(c));
		const seasons = (tvShow.seasons || []).map((s) => this.seasonToDisplay(s));

		return {
			...this.tvShowToDisplay(tvShow),
			tagline: tvShow.tagline || null,
			status: tvShow.status || null,
			networks: tvShow.networks?.map((n) => n.name) || [],
			createdBy: tvShow.created_by?.map((c) => c.name) || [],
			cast: topCast,
			seasons
		};
	}
}

export const tmdbAdapter = new TMDBAdapter();

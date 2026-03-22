import type {
	TMDBMovie,
	TMDBMovieDetails,
	TMDBCastMember,
	TMDBImage,
	DisplayTMDBMovie,
	DisplayTMDBMovieDetails,
	DisplayTMDBCastMember,
	DisplayTMDBImage,
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
} from './types.js';
import { extractYear } from '../../common/src/utils';

export { extractYear };

let imageBaseUrl = 'https://image.tmdb.org/t/p';

/** Set the base URL for TMDB images (e.g. to route through local backend cache). */
export function setImageBaseUrl(url: string) {
	imageBaseUrl = url;
}

// Image URL helpers

export function getPosterUrl(
	path: string | null,
	size: 'w185' | 'w342' | 'w500' = 'w342'
): string | null {
	if (!path) return null;
	return `${imageBaseUrl}/${size}${path}`;
}

export function getBackdropUrl(
	path: string | null,
	size: 'w780' | 'w1280' | 'original' = 'w780'
): string | null {
	if (!path) return null;
	return `${imageBaseUrl}/${size}${path}`;
}

export function getProfileUrl(
	path: string | null,
	size: 'w45' | 'w185' | 'h632' = 'w185'
): string | null {
	if (!path) return null;
	return `${imageBaseUrl}/${size}${path}`;
}

export function getStillUrl(
	path: string | null,
	size: 'w185' | 'w300' | 'w500' = 'w300'
): string | null {
	if (!path) return null;
	return `${imageBaseUrl}/${size}${path}`;
}

// Formatting helpers

export function formatRuntime(minutes: number | undefined): string | null {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	if (hours > 0) {
		return `${hours}h ${mins}m`;
	}
	return `${mins}m`;
}

export function formatCurrency(amount: number | undefined): string | null {
	if (!amount || amount === 0) return null;
	return new Intl.NumberFormat('en-US', {
		style: 'currency',
		currency: 'USD',
		maximumFractionDigits: 0
	}).format(amount);
}

// Movie transforms

export function movieToDisplay(movie: TMDBMovie): DisplayTMDBMovie {
	return {
		id: movie.id,
		title: movie.title,
		originalTitle: movie.original_title,
		releaseYear: extractYear(movie.release_date),
		overview: movie.overview || '',
		posterUrl: getPosterUrl(movie.poster_path),
		backdropUrl: getBackdropUrl(movie.backdrop_path),
		voteAverage: movie.vote_average,
		voteCount: movie.vote_count,
		genres: movie.genres?.map((g) => g.name) || []
	};
}

export function moviesToDisplay(movies: TMDBMovie[]): DisplayTMDBMovie[] {
	return movies.map((m) => movieToDisplay(m));
}

export function castMemberToDisplay(cast: TMDBCastMember): DisplayTMDBCastMember {
	return {
		id: cast.id,
		name: cast.name,
		character: cast.character,
		profileUrl: getProfileUrl(cast.profile_path)
	};
}

export function imageToDisplay(image: TMDBImage, type: 'backdrop' | 'poster'): DisplayTMDBImage {
	const thumbnailUrl =
		type === 'backdrop'
			? getBackdropUrl(image.file_path, 'w780')!
			: getPosterUrl(image.file_path, 'w342')!;
	const fullUrl = `${imageBaseUrl}/original${image.file_path}`;
	return {
		thumbnailUrl,
		fullUrl,
		width: image.width,
		height: image.height
	};
}

export function imagesToDisplay(
	images: { backdrops: TMDBImage[]; posters: TMDBImage[] } | undefined
): DisplayTMDBImage[] {
	if (!images) return [];
	const backdrops = images.backdrops.map((img) => imageToDisplay(img, 'backdrop'));
	const posters = images.posters.map((img) => imageToDisplay(img, 'poster'));
	return [...backdrops, ...posters];
}

export function movieDetailsToDisplay(movie: TMDBMovieDetails): DisplayTMDBMovieDetails {
	const director = movie.credits?.crew.find((c) => c.job === 'Director');
	const topCast = (movie.credits?.cast || []).slice(0, 10).map((c) => castMemberToDisplay(c));

	return {
		...movieToDisplay(movie),
		tagline: movie.tagline || null,
		runtime: formatRuntime(movie.runtime),
		budget: formatCurrency(movie.budget),
		revenue: formatCurrency(movie.revenue),
		imdbId: movie.imdb_id || null,
		cast: topCast,
		director: director?.name || null,
		images: imagesToDisplay(movie.images)
	};
}

// TV transforms

export function tvShowToDisplay(tvShow: TMDBTvShow): DisplayTMDBTvShow {
	return {
		id: tvShow.id,
		name: tvShow.name,
		originalName: tvShow.original_name,
		firstAirYear: extractYear(tvShow.first_air_date),
		lastAirYear: tvShow.last_air_date ? extractYear(tvShow.last_air_date) : null,
		overview: tvShow.overview || '',
		posterUrl: getPosterUrl(tvShow.poster_path),
		backdropUrl: getBackdropUrl(tvShow.backdrop_path),
		voteAverage: tvShow.vote_average,
		voteCount: tvShow.vote_count,
		genres: tvShow.genres?.map((g) => g.name) || [],
		numberOfSeasons: tvShow.number_of_seasons || null,
		numberOfEpisodes: tvShow.number_of_episodes || null
	};
}

export function tvShowsToDisplay(tvShows: TMDBTvShow[]): DisplayTMDBTvShow[] {
	return tvShows.map((t) => tvShowToDisplay(t));
}

export function seasonToDisplay(season: TMDBSeason): DisplayTMDBSeason {
	return {
		id: season.id,
		name: season.name,
		overview: season.overview || '',
		airDate: season.air_date,
		episodeCount: season.episode_count,
		posterUrl: getPosterUrl(season.poster_path),
		seasonNumber: season.season_number
	};
}

export function episodeToDisplay(episode: TMDBEpisode): DisplayTMDBEpisode {
	return {
		id: episode.id,
		name: episode.name,
		overview: episode.overview || '',
		airDate: episode.air_date,
		episodeNumber: episode.episode_number,
		seasonNumber: episode.season_number,
		stillUrl: getStillUrl(episode.still_path),
		voteAverage: episode.vote_average,
		runtime: episode.runtime || null
	};
}

export function seasonDetailsToDisplay(season: TMDBSeasonDetails): DisplayTMDBSeasonDetails {
	return {
		id: season.id,
		name: season.name,
		overview: season.overview || '',
		airDate: season.air_date,
		posterUrl: getPosterUrl(season.poster_path),
		seasonNumber: season.season_number,
		episodes: season.episodes.map((e) => episodeToDisplay(e))
	};
}

export function tvShowDetailsToDisplay(tvShow: TMDBTvShowDetails): DisplayTMDBTvShowDetails {
	const topCast = (tvShow.credits?.cast || []).slice(0, 10).map((c) => castMemberToDisplay(c));
	const seasons = (tvShow.seasons || []).map((s) => seasonToDisplay(s));

	return {
		...tvShowToDisplay(tvShow),
		tagline: tvShow.tagline || null,
		status: tvShow.status || null,
		networks: tvShow.networks?.map((n) => n.name) || [],
		createdBy: tvShow.created_by?.map((c) => c.name) || [],
		cast: topCast,
		seasons,
		images: imagesToDisplay(tvShow.images)
	};
}

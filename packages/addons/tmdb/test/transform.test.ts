import { describe, it, expect } from 'vitest';
import {
	getPosterUrl,
	getBackdropUrl,
	getProfileUrl,
	getStillUrl,
	formatRuntime,
	formatCurrency,
	extractYear,
	movieToDisplay,
	moviesToDisplay,
	castMemberToDisplay,
	movieDetailsToDisplay,
	tvShowToDisplay,
	tvShowsToDisplay,
	seasonToDisplay,
	episodeToDisplay,
	seasonDetailsToDisplay,
	tvShowDetailsToDisplay
} from '../src/transform.js';
import type {
	TMDBMovie,
	TMDBMovieDetails,
	TMDBCastMember,
	TMDBTvShow,
	TMDBTvShowDetails,
	TMDBSeason,
	TMDBSeasonDetails,
	TMDBEpisode
} from '../src/types.js';

function makeMovie(overrides: Partial<TMDBMovie> = {}): TMDBMovie {
	return {
		id: 1,
		title: 'Test Movie',
		original_title: 'Test Movie Original',
		overview: 'A test movie.',
		release_date: '2024-06-15',
		poster_path: '/poster.jpg',
		backdrop_path: '/backdrop.jpg',
		vote_average: 7.5,
		vote_count: 100,
		adult: false,
		original_language: 'en',
		popularity: 50,
		...overrides
	};
}

function makeCastMember(overrides: Partial<TMDBCastMember> = {}): TMDBCastMember {
	return {
		id: 10,
		name: 'Actor Name',
		character: 'Character Name',
		profile_path: '/actor.jpg',
		order: 0,
		...overrides
	};
}

function makeMovieDetails(overrides: Partial<TMDBMovieDetails> = {}): TMDBMovieDetails {
	return {
		...makeMovie(),
		credits: {
			cast: [makeCastMember()],
			crew: [{ id: 20, name: 'Director Name', job: 'Director', department: 'Directing', profile_path: null }]
		},
		tagline: 'A great tagline',
		runtime: 120,
		budget: 50000000,
		revenue: 200000000,
		imdb_id: 'tt1234567',
		...overrides
	};
}

function makeTvShow(overrides: Partial<TMDBTvShow> = {}): TMDBTvShow {
	return {
		id: 100,
		name: 'Test Show',
		original_name: 'Test Show Original',
		overview: 'A test TV show.',
		first_air_date: '2020-01-10',
		poster_path: '/tv-poster.jpg',
		backdrop_path: '/tv-backdrop.jpg',
		vote_average: 8.0,
		vote_count: 200,
		adult: false,
		original_language: 'en',
		popularity: 75,
		...overrides
	};
}

function makeSeason(overrides: Partial<TMDBSeason> = {}): TMDBSeason {
	return {
		id: 500,
		name: 'Season 1',
		overview: 'First season.',
		air_date: '2020-01-10',
		episode_count: 10,
		poster_path: '/season1.jpg',
		season_number: 1,
		...overrides
	};
}

function makeEpisode(overrides: Partial<TMDBEpisode> = {}): TMDBEpisode {
	return {
		id: 600,
		name: 'Pilot',
		overview: 'The first episode.',
		air_date: '2020-01-10',
		episode_number: 1,
		season_number: 1,
		still_path: '/still.jpg',
		vote_average: 8.5,
		vote_count: 50,
		runtime: 45,
		...overrides
	};
}

// Image URL helpers

describe('getPosterUrl', () => {
	it('returns full URL with default size', () => {
		expect(getPosterUrl('/poster.jpg')).toBe('https://image.tmdb.org/t/p/w342/poster.jpg');
	});

	it('returns full URL with custom size', () => {
		expect(getPosterUrl('/poster.jpg', 'w500')).toBe('https://image.tmdb.org/t/p/w500/poster.jpg');
	});

	it('returns null for null path', () => {
		expect(getPosterUrl(null)).toBeNull();
	});
});

describe('getBackdropUrl', () => {
	it('returns full URL with default size', () => {
		expect(getBackdropUrl('/backdrop.jpg')).toBe('https://image.tmdb.org/t/p/w780/backdrop.jpg');
	});

	it('returns full URL with custom size', () => {
		expect(getBackdropUrl('/backdrop.jpg', 'original')).toBe(
			'https://image.tmdb.org/t/p/original/backdrop.jpg'
		);
	});

	it('returns null for null path', () => {
		expect(getBackdropUrl(null)).toBeNull();
	});
});

describe('getProfileUrl', () => {
	it('returns full URL with default size', () => {
		expect(getProfileUrl('/profile.jpg')).toBe('https://image.tmdb.org/t/p/w185/profile.jpg');
	});

	it('returns null for null path', () => {
		expect(getProfileUrl(null)).toBeNull();
	});
});

describe('getStillUrl', () => {
	it('returns full URL with default size', () => {
		expect(getStillUrl('/still.jpg')).toBe('https://image.tmdb.org/t/p/w300/still.jpg');
	});

	it('returns null for null path', () => {
		expect(getStillUrl(null)).toBeNull();
	});
});

// Formatting helpers

describe('formatRuntime', () => {
	it('formats hours and minutes', () => {
		expect(formatRuntime(120)).toBe('2h 0m');
	});

	it('formats minutes only when under 60', () => {
		expect(formatRuntime(45)).toBe('45m');
	});

	it('returns null for undefined', () => {
		expect(formatRuntime(undefined)).toBeNull();
	});

	it('returns null for 0', () => {
		expect(formatRuntime(0)).toBeNull();
	});

	it('formats mixed hours and minutes', () => {
		expect(formatRuntime(150)).toBe('2h 30m');
	});
});

describe('formatCurrency', () => {
	it('formats USD amount', () => {
		expect(formatCurrency(50000000)).toBe('$50,000,000');
	});

	it('returns null for undefined', () => {
		expect(formatCurrency(undefined)).toBeNull();
	});

	it('returns null for 0', () => {
		expect(formatCurrency(0)).toBeNull();
	});
});

describe('extractYear', () => {
	it('extracts year from date string', () => {
		expect(extractYear('2024-06-15')).toBe('2024');
	});

	it('returns Unknown for undefined', () => {
		expect(extractYear(undefined)).toBe('Unknown');
	});

	it('returns Unknown for empty string', () => {
		expect(extractYear('')).toBe('Unknown');
	});
});

// Movie transforms

describe('movieToDisplay', () => {
	it('maps all fields correctly', () => {
		const movie = makeMovie({ genres: [{ id: 1, name: 'Action' }, { id: 2, name: 'Drama' }] });
		const display = movieToDisplay(movie);

		expect(display.id).toBe(1);
		expect(display.title).toBe('Test Movie');
		expect(display.originalTitle).toBe('Test Movie Original');
		expect(display.releaseYear).toBe('2024');
		expect(display.overview).toBe('A test movie.');
		expect(display.posterUrl).toBe('https://image.tmdb.org/t/p/w342/poster.jpg');
		expect(display.backdropUrl).toBe('https://image.tmdb.org/t/p/w780/backdrop.jpg');
		expect(display.voteAverage).toBe(7.5);
		expect(display.voteCount).toBe(100);
		expect(display.genres).toEqual(['Action', 'Drama']);
	});

	it('handles null poster and backdrop paths', () => {
		const movie = makeMovie({ poster_path: null, backdrop_path: null });
		const display = movieToDisplay(movie);
		expect(display.posterUrl).toBeNull();
		expect(display.backdropUrl).toBeNull();
	});

	it('handles missing genres', () => {
		const movie = makeMovie();
		const display = movieToDisplay(movie);
		expect(display.genres).toEqual([]);
	});

	it('handles empty overview', () => {
		const movie = makeMovie({ overview: '' });
		const display = movieToDisplay(movie);
		expect(display.overview).toBe('');
	});
});

describe('moviesToDisplay', () => {
	it('maps an array of movies', () => {
		const movies = [makeMovie({ id: 1 }), makeMovie({ id: 2 })];
		const display = moviesToDisplay(movies);
		expect(display).toHaveLength(2);
		expect(display[0].id).toBe(1);
		expect(display[1].id).toBe(2);
	});
});

describe('castMemberToDisplay', () => {
	it('maps all fields correctly', () => {
		const cast = makeCastMember();
		const display = castMemberToDisplay(cast);
		expect(display.id).toBe(10);
		expect(display.name).toBe('Actor Name');
		expect(display.character).toBe('Character Name');
		expect(display.profileUrl).toBe('https://image.tmdb.org/t/p/w185/actor.jpg');
	});

	it('handles null profile path', () => {
		const cast = makeCastMember({ profile_path: null });
		const display = castMemberToDisplay(cast);
		expect(display.profileUrl).toBeNull();
	});
});

describe('movieDetailsToDisplay', () => {
	it('maps all detail fields', () => {
		const details = makeMovieDetails();
		const display = movieDetailsToDisplay(details);

		expect(display.tagline).toBe('A great tagline');
		expect(display.runtime).toBe('2h 0m');
		expect(display.budget).toBe('$50,000,000');
		expect(display.revenue).toBe('$200,000,000');
		expect(display.imdbId).toBe('tt1234567');
		expect(display.director).toBe('Director Name');
		expect(display.cast).toHaveLength(1);
		expect(display.cast[0].name).toBe('Actor Name');
	});

	it('handles missing credits', () => {
		const details = makeMovieDetails({ credits: undefined });
		const display = movieDetailsToDisplay(details);
		expect(display.director).toBeNull();
		expect(display.cast).toEqual([]);
	});

	it('handles missing tagline and imdb_id', () => {
		const details = makeMovieDetails({ tagline: undefined, imdb_id: undefined });
		const display = movieDetailsToDisplay(details);
		expect(display.tagline).toBeNull();
		expect(display.imdbId).toBeNull();
	});

	it('limits cast to 10 members', () => {
		const cast = Array.from({ length: 15 }, (_, i) => makeCastMember({ id: i, order: i }));
		const details = makeMovieDetails({
			credits: { cast, crew: [] }
		});
		const display = movieDetailsToDisplay(details);
		expect(display.cast).toHaveLength(10);
	});
});

// TV transforms

describe('tvShowToDisplay', () => {
	it('maps all fields correctly', () => {
		const tvShow = makeTvShow({
			genres: [{ id: 1, name: 'Sci-Fi' }],
			number_of_seasons: 3,
			number_of_episodes: 30,
			last_air_date: '2023-12-20'
		});
		const display = tvShowToDisplay(tvShow);

		expect(display.id).toBe(100);
		expect(display.name).toBe('Test Show');
		expect(display.originalName).toBe('Test Show Original');
		expect(display.firstAirYear).toBe('2020');
		expect(display.lastAirYear).toBe('2023');
		expect(display.overview).toBe('A test TV show.');
		expect(display.posterUrl).toBe('https://image.tmdb.org/t/p/w342/tv-poster.jpg');
		expect(display.backdropUrl).toBe('https://image.tmdb.org/t/p/w780/tv-backdrop.jpg');
		expect(display.voteAverage).toBe(8.0);
		expect(display.voteCount).toBe(200);
		expect(display.genres).toEqual(['Sci-Fi']);
		expect(display.numberOfSeasons).toBe(3);
		expect(display.numberOfEpisodes).toBe(30);
	});

	it('handles missing last_air_date', () => {
		const tvShow = makeTvShow();
		const display = tvShowToDisplay(tvShow);
		expect(display.lastAirYear).toBeNull();
	});

	it('handles missing season/episode counts', () => {
		const tvShow = makeTvShow();
		const display = tvShowToDisplay(tvShow);
		expect(display.numberOfSeasons).toBeNull();
		expect(display.numberOfEpisodes).toBeNull();
	});
});

describe('tvShowsToDisplay', () => {
	it('maps an array of TV shows', () => {
		const shows = [makeTvShow({ id: 1 }), makeTvShow({ id: 2 })];
		const display = tvShowsToDisplay(shows);
		expect(display).toHaveLength(2);
		expect(display[0].id).toBe(1);
		expect(display[1].id).toBe(2);
	});
});

describe('seasonToDisplay', () => {
	it('maps all fields correctly', () => {
		const season = makeSeason();
		const display = seasonToDisplay(season);

		expect(display.id).toBe(500);
		expect(display.name).toBe('Season 1');
		expect(display.overview).toBe('First season.');
		expect(display.airDate).toBe('2020-01-10');
		expect(display.episodeCount).toBe(10);
		expect(display.posterUrl).toBe('https://image.tmdb.org/t/p/w342/season1.jpg');
		expect(display.seasonNumber).toBe(1);
	});

	it('handles null air_date and poster_path', () => {
		const season = makeSeason({ air_date: null, poster_path: null });
		const display = seasonToDisplay(season);
		expect(display.airDate).toBeNull();
		expect(display.posterUrl).toBeNull();
	});
});

describe('episodeToDisplay', () => {
	it('maps all fields correctly', () => {
		const episode = makeEpisode();
		const display = episodeToDisplay(episode);

		expect(display.id).toBe(600);
		expect(display.name).toBe('Pilot');
		expect(display.overview).toBe('The first episode.');
		expect(display.airDate).toBe('2020-01-10');
		expect(display.episodeNumber).toBe(1);
		expect(display.seasonNumber).toBe(1);
		expect(display.stillUrl).toBe('https://image.tmdb.org/t/p/w300/still.jpg');
		expect(display.voteAverage).toBe(8.5);
		expect(display.runtime).toBe(45);
	});

	it('handles null still_path and missing runtime', () => {
		const episode = makeEpisode({ still_path: null, runtime: undefined });
		const display = episodeToDisplay(episode);
		expect(display.stillUrl).toBeNull();
		expect(display.runtime).toBeNull();
	});
});

describe('seasonDetailsToDisplay', () => {
	it('maps season details with episodes', () => {
		const seasonDetails: TMDBSeasonDetails = {
			id: 500,
			name: 'Season 1',
			overview: 'First season.',
			air_date: '2020-01-10',
			poster_path: '/season1.jpg',
			season_number: 1,
			episodes: [makeEpisode({ id: 601 }), makeEpisode({ id: 602, episode_number: 2 })]
		};
		const display = seasonDetailsToDisplay(seasonDetails);

		expect(display.id).toBe(500);
		expect(display.name).toBe('Season 1');
		expect(display.episodes).toHaveLength(2);
		expect(display.episodes[0].id).toBe(601);
		expect(display.episodes[1].id).toBe(602);
	});
});

describe('tvShowDetailsToDisplay', () => {
	it('maps all detail fields', () => {
		const tvShowDetails: TMDBTvShowDetails = {
			...makeTvShow({
				tagline: 'Best show ever',
				status: 'Ended',
				networks: [{ id: 1, name: 'HBO', logo_path: null, origin_country: 'US' }],
				created_by: [{ id: 1, name: 'Creator Name', profile_path: null }]
			}),
			seasons: [makeSeason()],
			credits: {
				cast: [makeCastMember()],
				crew: []
			}
		};
		const display = tvShowDetailsToDisplay(tvShowDetails);

		expect(display.tagline).toBe('Best show ever');
		expect(display.status).toBe('Ended');
		expect(display.networks).toEqual(['HBO']);
		expect(display.createdBy).toEqual(['Creator Name']);
		expect(display.cast).toHaveLength(1);
		expect(display.cast[0].name).toBe('Actor Name');
		expect(display.seasons).toHaveLength(1);
		expect(display.seasons[0].name).toBe('Season 1');
	});

	it('handles missing optional fields', () => {
		const tvShowDetails: TMDBTvShowDetails = {
			...makeTvShow()
		};
		const display = tvShowDetailsToDisplay(tvShowDetails);
		expect(display.tagline).toBeNull();
		expect(display.status).toBeNull();
		expect(display.networks).toEqual([]);
		expect(display.createdBy).toEqual([]);
		expect(display.cast).toEqual([]);
		expect(display.seasons).toEqual([]);
	});

	it('limits cast to 10 members', () => {
		const cast = Array.from({ length: 15 }, (_, i) => makeCastMember({ id: i, order: i }));
		const tvShowDetails: TMDBTvShowDetails = {
			...makeTvShow(),
			credits: { cast, crew: [] }
		};
		const display = tvShowDetailsToDisplay(tvShowDetails);
		expect(display.cast).toHaveLength(10);
	});
});

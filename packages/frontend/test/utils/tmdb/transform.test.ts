import { describe, it, expect } from 'vitest';
import {
	getPosterUrl,
	getBackdropUrl,
	getProfileUrl,
	getStillUrl,
	formatRuntime,
	formatCurrency,
	movieToDisplay,
	movieDetailsToDisplay,
	tvShowToDisplay,
	episodeToDisplay,
	tvShowDetailsToDisplay
} from '../../../src/utils/tmdb/transform';

describe('image URL helpers', () => {
	describe('getPosterUrl', () => {
		it('returns null for null path', () => {
			expect(getPosterUrl(null)).toBeNull();
		});

		it('returns URL with default size', () => {
			expect(getPosterUrl('/abc.jpg')).toBe('https://image.tmdb.org/t/p/w342/abc.jpg');
		});

		it('returns URL with custom size', () => {
			expect(getPosterUrl('/abc.jpg', 'w500')).toBe('https://image.tmdb.org/t/p/w500/abc.jpg');
		});
	});

	describe('getBackdropUrl', () => {
		it('returns null for null path', () => {
			expect(getBackdropUrl(null)).toBeNull();
		});

		it('returns URL with default size', () => {
			expect(getBackdropUrl('/bg.jpg')).toBe('https://image.tmdb.org/t/p/w780/bg.jpg');
		});
	});

	describe('getProfileUrl', () => {
		it('returns null for null path', () => {
			expect(getProfileUrl(null)).toBeNull();
		});

		it('returns URL with default size', () => {
			expect(getProfileUrl('/face.jpg')).toBe('https://image.tmdb.org/t/p/w185/face.jpg');
		});
	});

	describe('getStillUrl', () => {
		it('returns null for null path', () => {
			expect(getStillUrl(null)).toBeNull();
		});

		it('returns URL with default size', () => {
			expect(getStillUrl('/still.jpg')).toBe('https://image.tmdb.org/t/p/w300/still.jpg');
		});
	});
});

describe('formatRuntime', () => {
	it('returns null for undefined', () => {
		expect(formatRuntime(undefined)).toBeNull();
	});

	it('returns null for zero', () => {
		expect(formatRuntime(0)).toBeNull();
	});

	it('formats minutes only for < 60', () => {
		expect(formatRuntime(45)).toBe('45m');
	});

	it('formats hours and minutes', () => {
		expect(formatRuntime(125)).toBe('2h 5m');
	});

	it('formats exact hours', () => {
		expect(formatRuntime(120)).toBe('2h 0m');
	});
});

describe('formatCurrency', () => {
	it('returns null for undefined', () => {
		expect(formatCurrency(undefined)).toBeNull();
	});

	it('returns null for zero', () => {
		expect(formatCurrency(0)).toBeNull();
	});

	it('formats USD currency', () => {
		const result = formatCurrency(1000000);
		expect(result).toContain('1,000,000');
	});
});

describe('movieToDisplay', () => {
	it('transforms a TMDB movie to display format', () => {
		const movie = {
			id: 1,
			title: 'Test Movie',
			original_title: 'Test Movie',
			release_date: '2023-05-15',
			overview: 'A test movie',
			poster_path: '/poster.jpg',
			backdrop_path: '/backdrop.jpg',
			vote_average: 7.5,
			vote_count: 100,
			genres: [{ id: 1, name: 'Action' }]
		};

		const result = movieToDisplay(movie as any);
		expect(result.title).toBe('Test Movie');
		expect(result.releaseYear).toBe('2023');
		expect(result.posterUrl).toContain('/poster.jpg');
		expect(result.genres).toEqual(['Action']);
	});

	it('handles missing overview', () => {
		const movie = {
			id: 1,
			title: 'Test',
			original_title: 'Test',
			release_date: '2023-01-01',
			poster_path: null,
			backdrop_path: null,
			vote_average: 0,
			vote_count: 0
		};

		const result = movieToDisplay(movie as any);
		expect(result.overview).toBe('');
		expect(result.posterUrl).toBeNull();
		expect(result.genres).toEqual([]);
	});
});

describe('movieDetailsToDisplay', () => {
	it('includes director and cast', () => {
		const movie = {
			id: 1,
			title: 'Test',
			original_title: 'Test',
			release_date: '2023-01-01',
			overview: '',
			poster_path: null,
			backdrop_path: null,
			vote_average: 7.0,
			vote_count: 50,
			tagline: 'A tagline',
			runtime: 120,
			budget: 1000000,
			revenue: 5000000,
			imdb_id: 'tt1234567',
			credits: {
				crew: [{ job: 'Director', name: 'John Doe' }],
				cast: [
					{ id: 1, name: 'Actor One', character: 'Hero', profile_path: '/a1.jpg' },
					{ id: 2, name: 'Actor Two', character: 'Villain', profile_path: null }
				]
			}
		};

		const result = movieDetailsToDisplay(movie as any);
		expect(result.director).toBe('John Doe');
		expect(result.cast).toHaveLength(2);
		expect(result.runtime).toBe('2h 0m');
		expect(result.tagline).toBe('A tagline');
	});
});

describe('tvShowToDisplay', () => {
	it('transforms a TMDB TV show', () => {
		const tvShow = {
			id: 1,
			name: 'Test Show',
			original_name: 'Test Show',
			first_air_date: '2020-01-01',
			last_air_date: '2023-12-31',
			overview: 'A test show',
			poster_path: '/poster.jpg',
			backdrop_path: '/backdrop.jpg',
			vote_average: 8.0,
			vote_count: 200,
			genres: [{ id: 1, name: 'Drama' }],
			number_of_seasons: 3,
			number_of_episodes: 30
		};

		const result = tvShowToDisplay(tvShow as any);
		expect(result.name).toBe('Test Show');
		expect(result.firstAirYear).toBe('2020');
		expect(result.lastAirYear).toBe('2023');
		expect(result.numberOfSeasons).toBe(3);
	});
});

describe('episodeToDisplay', () => {
	it('transforms a TMDB episode', () => {
		const episode = {
			id: 1,
			name: 'Pilot',
			overview: 'First episode',
			air_date: '2020-01-01',
			episode_number: 1,
			season_number: 1,
			still_path: '/still.jpg',
			vote_average: 8.5,
			runtime: 45
		};

		const result = episodeToDisplay(episode as any);
		expect(result.name).toBe('Pilot');
		expect(result.episodeNumber).toBe(1);
		expect(result.stillUrl).toContain('/still.jpg');
		expect(result.runtime).toBe(45);
	});
});

describe('tvShowDetailsToDisplay', () => {
	it('includes cast and seasons', () => {
		const tvShow = {
			id: 1,
			name: 'Show',
			original_name: 'Show',
			first_air_date: '2020-01-01',
			overview: '',
			poster_path: null,
			backdrop_path: null,
			vote_average: 0,
			vote_count: 0,
			tagline: 'A show',
			status: 'Ended',
			networks: [{ name: 'HBO' }],
			created_by: [{ name: 'Creator' }],
			credits: {
				cast: [{ id: 1, name: 'Star', character: 'Lead', profile_path: null }]
			},
			seasons: [
				{
					id: 1,
					name: 'Season 1',
					overview: '',
					air_date: '2020-01-01',
					episode_count: 10,
					poster_path: null,
					season_number: 1
				}
			]
		};

		const result = tvShowDetailsToDisplay(tvShow as any);
		expect(result.networks).toEqual(['HBO']);
		expect(result.createdBy).toEqual(['Creator']);
		expect(result.seasons).toHaveLength(1);
		expect(result.cast).toHaveLength(1);
	});
});

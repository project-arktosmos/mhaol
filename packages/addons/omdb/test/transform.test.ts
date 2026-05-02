import { describe, it, expect } from 'vitest';
import { parseRatingValue, reviewsFromOmdb } from '../src/transform.js';
import type { OMDBDetails } from '../src/types.js';

describe('parseRatingValue', () => {
	it('parses percent values', () => {
		expect(parseRatingValue('92%')).toEqual({ score: 92, maxScore: 100 });
	});

	it('parses x/10 fractions', () => {
		expect(parseRatingValue('7.8/10')).toEqual({ score: 7.8, maxScore: 10 });
	});

	it('parses x/100 fractions', () => {
		expect(parseRatingValue('78/100')).toEqual({ score: 78, maxScore: 100 });
	});

	it('returns null for unparseable values', () => {
		expect(parseRatingValue('N/A')).toBeNull();
		expect(parseRatingValue('')).toBeNull();
	});
});

describe('reviewsFromOmdb', () => {
	const payload: OMDBDetails = {
		Title: 'The Shawshank Redemption',
		Year: '1994',
		imdbID: 'tt0111161',
		Type: 'movie',
		imdbRating: '9.3',
		imdbVotes: '2,945,123',
		Metascore: '82',
		Ratings: [
			{ Source: 'Internet Movie Database', Value: '9.3/10' },
			{ Source: 'Rotten Tomatoes', Value: '91%' },
			{ Source: 'Metacritic', Value: '82/100' }
		],
		Response: 'True'
	};

	it('canonicalises source labels and parses every entry', () => {
		const reviews = reviewsFromOmdb(payload);
		expect(reviews).toEqual([
			{ label: 'IMDb', score: 9.3, maxScore: 10, voteCount: 2945123 },
			{ label: 'Rotten Tomatoes', score: 91, maxScore: 100 },
			{ label: 'Metacritic', score: 82, maxScore: 100 }
		]);
	});

	it('drops Ratings entries with unparseable values', () => {
		const reviews = reviewsFromOmdb({
			...payload,
			Ratings: [
				{ Source: 'Rotten Tomatoes', Value: 'N/A' },
				{ Source: 'Metacritic', Value: '78/100' }
			]
		});
		expect(reviews.map((r) => r.label)).toEqual(['Metacritic']);
	});

	it('returns an empty array when Ratings is missing', () => {
		const reviews = reviewsFromOmdb({ ...payload, Ratings: undefined });
		expect(reviews).toEqual([]);
	});
});

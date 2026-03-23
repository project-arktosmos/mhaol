/* eslint-disable @typescript-eslint/no-explicit-any */
import { describe, it, expect } from 'vitest';
import {
	getCoverUrl,
	getAuthorPhotoUrl,
	searchDocToDisplay,
	searchDocsToDisplay,
	subjectWorkToDisplay,
	subjectWorksToDisplay,
	workToDisplayDetails,
	authorToDisplay
} from '../src/transform';

describe('getCoverUrl', () => {
	it('returns null for null coverId', () => {
		expect(getCoverUrl(null)).toBeNull();
	});

	it('returns null for undefined coverId', () => {
		expect(getCoverUrl(undefined)).toBeNull();
	});

	it('returns null for zero coverId', () => {
		expect(getCoverUrl(0)).toBeNull();
	});

	it('returns URL with default M size', () => {
		expect(getCoverUrl(12345)).toBe('https://covers.openlibrary.org/b/id/12345-M.jpg');
	});

	it('returns URL with S size', () => {
		expect(getCoverUrl(12345, 'S')).toBe('https://covers.openlibrary.org/b/id/12345-S.jpg');
	});

	it('returns URL with L size', () => {
		expect(getCoverUrl(12345, 'L')).toBe('https://covers.openlibrary.org/b/id/12345-L.jpg');
	});
});

describe('getAuthorPhotoUrl', () => {
	it('returns URL with default M size', () => {
		expect(getAuthorPhotoUrl(67890)).toBe('https://covers.openlibrary.org/a/id/67890-M.jpg');
	});

	it('returns URL with S size', () => {
		expect(getAuthorPhotoUrl(67890, 'S')).toBe('https://covers.openlibrary.org/a/id/67890-S.jpg');
	});

	it('returns URL with L size', () => {
		expect(getAuthorPhotoUrl(67890, 'L')).toBe('https://covers.openlibrary.org/a/id/67890-L.jpg');
	});
});

describe('searchDocToDisplay', () => {
	it('transforms a complete search doc', () => {
		const doc = {
			key: '/works/OL12345W',
			title: 'The Great Gatsby',
			author_name: ['F. Scott Fitzgerald'],
			author_key: ['/authors/OL100A'],
			first_publish_year: 1925,
			cover_i: 11111,
			isbn: ['9780743273565'],
			subject: ['Fiction', 'Classic', 'American Literature'],
			publisher: ['Scribner'],
			number_of_pages_median: 180,
			edition_count: 500,
			ratings_average: 3.9,
			ratings_count: 12000
		};

		const result = searchDocToDisplay(doc as any);

		expect(result.key).toBe('OL12345W');
		expect(result.title).toBe('The Great Gatsby');
		expect(result.authors).toEqual(['F. Scott Fitzgerald']);
		expect(result.authorKeys).toEqual(['OL100A']);
		expect(result.firstPublishYear).toBe('1925');
		expect(result.coverId).toBe(11111);
		expect(result.coverUrl).toBe('https://covers.openlibrary.org/b/id/11111-M.jpg');
		expect(result.subjects).toEqual(['Fiction', 'Classic', 'American Literature']);
		expect(result.publishers).toEqual(['Scribner']);
		expect(result.pageCount).toBe(180);
		expect(result.editionCount).toBe(500);
		expect(result.isbn).toBe('9780743273565');
		expect(result.ratingsAverage).toBe(3.9);
		expect(result.ratingsCount).toBe(12000);
	});

	it('handles doc with missing optional fields', () => {
		const doc = {
			key: '/works/OL999W',
			title: 'Minimal Book'
		};

		const result = searchDocToDisplay(doc as any);

		expect(result.key).toBe('OL999W');
		expect(result.title).toBe('Minimal Book');
		expect(result.authors).toEqual([]);
		expect(result.authorKeys).toEqual([]);
		expect(result.firstPublishYear).toBe('');
		expect(result.coverId).toBeNull();
		expect(result.coverUrl).toBeNull();
		expect(result.subjects).toEqual([]);
		expect(result.publishers).toEqual([]);
		expect(result.pageCount).toBeNull();
		expect(result.editionCount).toBe(0);
		expect(result.isbn).toBeNull();
		expect(result.ratingsAverage).toBeNull();
		expect(result.ratingsCount).toBe(0);
	});

	it('strips /works/ prefix from key', () => {
		const doc = { key: '/works/OL555W', title: 'Test' };
		expect(searchDocToDisplay(doc as any).key).toBe('OL555W');
	});

	it('strips /authors/ prefix from author keys', () => {
		const doc = {
			key: '/works/OL1W',
			title: 'Test',
			author_key: ['/authors/OL1A', '/authors/OL2A']
		};
		const result = searchDocToDisplay(doc as any);
		expect(result.authorKeys).toEqual(['OL1A', 'OL2A']);
	});

	it('limits subjects to 10', () => {
		const doc = {
			key: '/works/OL1W',
			title: 'Test',
			subject: Array.from({ length: 15 }, (_, i) => `Subject ${i}`)
		};
		const result = searchDocToDisplay(doc as any);
		expect(result.subjects).toHaveLength(10);
	});

	it('limits publishers to 3', () => {
		const doc = {
			key: '/works/OL1W',
			title: 'Test',
			publisher: ['P1', 'P2', 'P3', 'P4', 'P5']
		};
		const result = searchDocToDisplay(doc as any);
		expect(result.publishers).toHaveLength(3);
	});

	it('takes first isbn from array', () => {
		const doc = {
			key: '/works/OL1W',
			title: 'Test',
			isbn: ['isbn-first', 'isbn-second']
		};
		const result = searchDocToDisplay(doc as any);
		expect(result.isbn).toBe('isbn-first');
	});
});

describe('searchDocsToDisplay', () => {
	it('transforms an array of docs', () => {
		const docs = [
			{ key: '/works/OL1W', title: 'Book One' },
			{ key: '/works/OL2W', title: 'Book Two' }
		];

		const results = searchDocsToDisplay(docs as any);
		expect(results).toHaveLength(2);
		expect(results[0].title).toBe('Book One');
		expect(results[1].title).toBe('Book Two');
	});

	it('returns empty array for empty input', () => {
		expect(searchDocsToDisplay([])).toEqual([]);
	});
});

describe('subjectWorkToDisplay', () => {
	it('transforms a subject work', () => {
		const work = {
			key: '/works/OL789W',
			title: 'Fantasy Novel',
			authors: [
				{ name: 'Author One', key: '/authors/OL10A' },
				{ name: 'Author Two', key: '/authors/OL20A' }
			],
			cover_id: 55555,
			first_publish_year: 2015,
			edition_count: 12,
			subject: ['Fantasy', 'Adventure', 'Magic']
		};

		const result = subjectWorkToDisplay(work as any);

		expect(result.key).toBe('OL789W');
		expect(result.title).toBe('Fantasy Novel');
		expect(result.authors).toEqual(['Author One', 'Author Two']);
		expect(result.authorKeys).toEqual(['OL10A', 'OL20A']);
		expect(result.firstPublishYear).toBe('2015');
		expect(result.coverId).toBe(55555);
		expect(result.coverUrl).toBe('https://covers.openlibrary.org/b/id/55555-M.jpg');
		expect(result.editionCount).toBe(12);
		expect(result.subjects).toEqual(['Fantasy', 'Adventure', 'Magic']);
	});

	it('handles subject work with missing optional fields', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Bare Book',
			authors: []
		};

		const result = subjectWorkToDisplay(work as any);

		expect(result.authors).toEqual([]);
		expect(result.authorKeys).toEqual([]);
		expect(result.firstPublishYear).toBe('');
		expect(result.coverId).toBeNull();
		expect(result.coverUrl).toBeNull();
		expect(result.publishers).toEqual([]);
		expect(result.pageCount).toBeNull();
		expect(result.isbn).toBeNull();
		expect(result.ratingsAverage).toBeNull();
		expect(result.ratingsCount).toBe(0);
	});

	it('handles subject work with no authors', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Anonymous'
		};

		const result = subjectWorkToDisplay(work as any);
		expect(result.authors).toEqual([]);
		expect(result.authorKeys).toEqual([]);
	});
});

describe('subjectWorksToDisplay', () => {
	it('transforms an array of subject works', () => {
		const works = [
			{ key: '/works/OL1W', title: 'A', authors: [] },
			{ key: '/works/OL2W', title: 'B', authors: [] }
		];

		const results = subjectWorksToDisplay(works as any);
		expect(results).toHaveLength(2);
		expect(results[0].key).toBe('OL1W');
		expect(results[1].key).toBe('OL2W');
	});

	it('returns empty array for empty input', () => {
		expect(subjectWorksToDisplay([])).toEqual([]);
	});
});

describe('workToDisplayDetails', () => {
	it('transforms a work with authors', () => {
		const work = {
			key: '/works/OL100W',
			title: 'Detailed Book',
			description: 'A fascinating story.',
			covers: [111, 222],
			subjects: ['Fiction', 'Drama'],
			first_publish_date: '2010-01-15'
		};
		const authors = [
			{ key: 'OL1A', name: 'Author', birthDate: null, deathDate: null, bio: null, photoUrl: null }
		];

		const result = workToDisplayDetails(work as any, authors);

		expect(result.key).toBe('OL100W');
		expect(result.title).toBe('Detailed Book');
		expect(result.description).toBe('A fascinating story.');
		expect(result.covers).toEqual([111, 222]);
		expect(result.subjects).toEqual(['Fiction', 'Drama']);
		expect(result.firstPublishYear).toBe('2010');
		expect(result.authors).toHaveLength(1);
		expect(result.coverUrl).toBe('https://covers.openlibrary.org/b/id/111-L.jpg');
	});

	it('extracts description from object format', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Test',
			description: { value: 'Object description' }
		};

		const result = workToDisplayDetails(work as any, []);
		expect(result.description).toBe('Object description');
	});

	it('handles missing description', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Test'
		};

		const result = workToDisplayDetails(work as any, []);
		expect(result.description).toBeNull();
	});

	it('falls back to searchDoc data for missing fields', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Test'
		};
		const searchDoc = {
			key: 'OL1W',
			title: 'Test',
			authors: [],
			authorKeys: [],
			firstPublishYear: '1999',
			coverId: 123,
			coverUrl: 'https://fallback.jpg',
			subjects: [],
			publishers: [],
			pageCount: 300,
			editionCount: 10,
			isbn: '978-1234567890',
			ratingsAverage: null,
			ratingsCount: 0
		};

		const result = workToDisplayDetails(work as any, [], searchDoc);

		expect(result.firstPublishYear).toBe('1999');
		expect(result.pageCount).toBe(300);
		expect(result.isbn).toBe('978-1234567890');
		expect(result.coverUrl).toBe('https://fallback.jpg');
	});

	it('limits subjects to 20', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Test',
			subjects: Array.from({ length: 30 }, (_, i) => `Subject ${i}`)
		};

		const result = workToDisplayDetails(work as any, []);
		expect(result.subjects).toHaveLength(20);
	});

	it('handles work with no covers', () => {
		const work = {
			key: '/works/OL1W',
			title: 'No Cover'
		};

		const result = workToDisplayDetails(work as any, []);
		expect(result.covers).toEqual([]);
		expect(result.coverUrl).toBeNull();
	});

	it('extracts year from first_publish_date', () => {
		const work = {
			key: '/works/OL1W',
			title: 'Test',
			first_publish_date: '1984-06-08'
		};

		const result = workToDisplayDetails(work as any, []);
		expect(result.firstPublishYear).toBe('1984');
	});
});

describe('authorToDisplay', () => {
	it('transforms a complete author', () => {
		const author = {
			key: '/authors/OL50A',
			name: 'Jane Austen',
			birth_date: '16 December 1775',
			death_date: '18 July 1817',
			bio: 'English novelist.',
			photos: [12345, 67890]
		};

		const result = authorToDisplay(author as any);

		expect(result.key).toBe('OL50A');
		expect(result.name).toBe('Jane Austen');
		expect(result.birthDate).toBe('16 December 1775');
		expect(result.deathDate).toBe('18 July 1817');
		expect(result.bio).toBe('English novelist.');
		expect(result.photoUrl).toBe('https://covers.openlibrary.org/a/id/12345-M.jpg');
	});

	it('handles author with missing optional fields', () => {
		const author = {
			key: '/authors/OL99A',
			name: 'Unknown Author'
		};

		const result = authorToDisplay(author as any);

		expect(result.key).toBe('OL99A');
		expect(result.name).toBe('Unknown Author');
		expect(result.birthDate).toBeNull();
		expect(result.deathDate).toBeNull();
		expect(result.bio).toBeNull();
		expect(result.photoUrl).toBeNull();
	});

	it('handles bio in object format', () => {
		const author = {
			key: '/authors/OL1A',
			name: 'Writer',
			bio: { value: 'A prolific writer.' }
		};

		const result = authorToDisplay(author as any);
		expect(result.bio).toBe('A prolific writer.');
	});

	it('handles author with empty photos array', () => {
		const author = {
			key: '/authors/OL1A',
			name: 'Writer',
			photos: []
		};

		const result = authorToDisplay(author as any);
		expect(result.photoUrl).toBeNull();
	});

	it('strips /authors/ prefix from key', () => {
		const author = { key: '/authors/OL42A', name: 'Test' };
		expect(authorToDisplay(author as any).key).toBe('OL42A');
	});
});

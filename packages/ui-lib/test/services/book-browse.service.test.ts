import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

function mockFetchOk(data: unknown) {
	return vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve(data),
		text: () => Promise.resolve(JSON.stringify(data)),
		body: null
	});
}

describe('BookBrowseService', () => {
	let bookBrowseService: (typeof import('../../src/services/book-browse.service'))['bookBrowseService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/book-browse.service');
		bookBrowseService = mod.bookBrowseService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('has correct initial state', () => {
		const state = get(bookBrowseService.state);
		expect(state.searchQuery).toBe('');
		expect(state.searchResults).toEqual([]);
		expect(state.searchPage).toBe(1);
		expect(state.searchTotalPages).toBe(1);
		expect(state.trendingResults).toEqual([]);
		expect(state.trendingPage).toBe(1);
		expect(state.trendingTotalPages).toBe(1);
		expect(state.selectedSubject).toBe('fiction');
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	// ===== searchBooks =====

	it('searchBooks sets loading true then false on success', async () => {
		let capturedLoading = false;
		const unsub = bookBrowseService.state.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal('fetch', mockFetchOk({ numFound: 0, start: 0, docs: [] }));

		await bookBrowseService.searchBooks('test query');
		unsub();

		expect(capturedLoading).toBe(true);
		const state = get(bookBrowseService.state);
		expect(state.loading).toBe(false);
	});

	it('searchBooks stores search query in state', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ numFound: 0, start: 0, docs: [] }));

		await bookBrowseService.searchBooks('tolkien');

		const state = get(bookBrowseService.state);
		expect(state.searchQuery).toBe('tolkien');
	});

	it('searchBooks transforms docs to display format', async () => {
		vi.stubGlobal(
			'fetch',
			mockFetchOk({
				numFound: 1,
				start: 0,
				docs: [
					{
						key: '/works/OL123W',
						title: 'The Hobbit',
						author_name: ['J.R.R. Tolkien'],
						author_key: ['OL26320A'],
						first_publish_year: 1937,
						cover_i: 12345,
						edition_count: 100,
						ratings_average: 4.2,
						ratings_count: 500
					}
				]
			})
		);

		await bookBrowseService.searchBooks('hobbit');

		const state = get(bookBrowseService.state);
		expect(state.searchResults).toHaveLength(1);
		expect(state.searchResults[0].title).toBe('The Hobbit');
		expect(state.searchResults[0].key).toBe('OL123W');
		expect(state.searchResults[0].authors).toEqual(['J.R.R. Tolkien']);
	});

	it('searchBooks calculates total pages', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ numFound: 45, start: 0, docs: [] }));

		await bookBrowseService.searchBooks('fiction');

		const state = get(bookBrowseService.state);
		// 45 / 20 = 2.25 => ceil = 3
		expect(state.searchTotalPages).toBe(3);
	});

	it('searchBooks updates page number', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ numFound: 100, start: 0, docs: [] }));

		await bookBrowseService.searchBooks('science', 3);

		const state = get(bookBrowseService.state);
		expect(state.searchPage).toBe(3);
	});

	it('searchBooks handles fetch error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await bookBrowseService.searchBooks('broken');

		const state = get(bookBrowseService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('Network error');
	});

	it('searchBooks handles HTTP error response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				json: () => Promise.resolve({}),
				text: () => Promise.resolve('')
			})
		);

		await bookBrowseService.searchBooks('test');

		const state = get(bookBrowseService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('HTTP 500');
	});

	it('searchBooks handles null response fields gracefully', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ numFound: null, docs: null }));

		await bookBrowseService.searchBooks('test');

		const state = get(bookBrowseService.state);
		expect(state.searchResults).toEqual([]);
		expect(state.searchTotalPages).toBe(0);
	});

	it('searchBooks clears error on new search', async () => {
		// First cause an error
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('fail')));
		await bookBrowseService.searchBooks('bad');
		expect(get(bookBrowseService.state).error).toBe('fail');

		// Then succeed
		vi.stubGlobal('fetch', mockFetchOk({ numFound: 0, start: 0, docs: [] }));
		await bookBrowseService.searchBooks('good');

		const state = get(bookBrowseService.state);
		expect(state.error).toBeNull();
	});

	// ===== loadTrendingBooks =====

	it('loadTrendingBooks sets loading true then false on success', async () => {
		let capturedLoading = false;
		const unsub = bookBrowseService.state.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal('fetch', mockFetchOk({ name: 'fiction', work_count: 0, works: [] }));

		await bookBrowseService.loadTrendingBooks('fiction');
		unsub();

		expect(capturedLoading).toBe(true);
		const state = get(bookBrowseService.state);
		expect(state.loading).toBe(false);
	});

	it('loadTrendingBooks updates selected subject', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ name: 'science_fiction', work_count: 0, works: [] }));

		await bookBrowseService.loadTrendingBooks('science_fiction');

		const state = get(bookBrowseService.state);
		expect(state.selectedSubject).toBe('science_fiction');
	});

	it('loadTrendingBooks transforms subject works to display format', async () => {
		vi.stubGlobal(
			'fetch',
			mockFetchOk({
				name: 'fiction',
				work_count: 1,
				works: [
					{
						key: '/works/OL456W',
						title: 'Great Novel',
						authors: [{ name: 'Author Name', key: '/authors/OL100A' }],
						cover_id: 99999,
						first_publish_year: 2020,
						edition_count: 5
					}
				]
			})
		);

		await bookBrowseService.loadTrendingBooks('fiction');

		const state = get(bookBrowseService.state);
		expect(state.trendingResults).toHaveLength(1);
		expect(state.trendingResults[0].title).toBe('Great Novel');
		expect(state.trendingResults[0].authors).toEqual(['Author Name']);
	});

	it('loadTrendingBooks calculates total pages', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ name: 'history', work_count: 60, works: [] }));

		await bookBrowseService.loadTrendingBooks('history');

		const state = get(bookBrowseService.state);
		// 60 / 20 = 3
		expect(state.trendingTotalPages).toBe(3);
	});

	it('loadTrendingBooks updates page number', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ name: 'mystery', work_count: 100, works: [] }));

		await bookBrowseService.loadTrendingBooks('mystery', 2);

		const state = get(bookBrowseService.state);
		expect(state.trendingPage).toBe(2);
	});

	it('loadTrendingBooks handles fetch error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Timeout')));

		await bookBrowseService.loadTrendingBooks('fantasy');

		const state = get(bookBrowseService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('Timeout');
	});

	it('loadTrendingBooks handles non-Error throws', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue('string error'));

		await bookBrowseService.loadTrendingBooks('horror');

		const state = get(bookBrowseService.state);
		expect(state.error).toBe('string error');
	});
});

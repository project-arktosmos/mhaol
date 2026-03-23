/* eslint-disable @typescript-eslint/no-explicit-any */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('OpenLibrary client', () => {
	let mockFetch: ReturnType<typeof vi.fn>;
	let searchBooks: typeof import('../src/client').searchBooks;
	let getWork: typeof import('../src/client').getWork;
	let getAuthor: typeof import('../src/client').getAuthor;
	let getSubjectBooks: typeof import('../src/client').getSubjectBooks;

	beforeEach(async () => {
		vi.resetModules();
		mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		const mod = await import('../src/client');
		searchBooks = mod.searchBooks;
		getWork = mod.getWork;
		getAuthor = mod.getAuthor;
		getSubjectBooks = mod.getSubjectBooks;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	describe('searchBooks', () => {
		it('calls the correct endpoint with encoded query', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ numFound: 0, start: 0, docs: [] })
			});

			await searchBooks('lord of the rings');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('openlibrary.org/search.json?'),
				expect.any(Object)
			);
			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('q=lord+of+the+rings');
		});

		it('passes page and limit parameters', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ numFound: 0, start: 0, docs: [] })
			});

			await searchBooks('test', 3, 10);

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('page=3');
			expect(url).toContain('limit=10');
		});

		it('returns search response data', async () => {
			const data = { numFound: 1, start: 0, docs: [{ key: '/works/OL1W', title: 'Book' }] };
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve(data)
			});

			const result = await searchBooks('test');
			expect(result).not.toBeNull();
			expect(result!.numFound).toBe(1);
			expect(result!.docs).toHaveLength(1);
		});

		it('includes required fields parameter', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ numFound: 0, docs: [] })
			});

			await searchBooks('test');

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('fields=');
			expect(url).toContain('title');
			expect(url).toContain('author_name');
			expect(url).toContain('cover_i');
		});

		it('sends correct headers', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ numFound: 0, docs: [] })
			});

			await searchBooks('test');

			const headers = mockFetch.mock.calls[0][1].headers;
			expect(headers.Accept).toBe('application/json');
			expect(headers['User-Agent']).toContain('Mhaol');
		});

		it('returns null on 404', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve(null)
			});

			const result = await searchBooks('nonexistent');
			expect(result).toBeNull();
		});

		it('throws on 429 rate limited', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 429,
				json: () => Promise.resolve(null)
			});

			await expect(searchBooks('test')).rejects.toThrow('429 Rate Limited');
		});

		it('returns null on other non-ok responses', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 500,
				json: () => Promise.resolve(null)
			});

			const result = await searchBooks('test');
			expect(result).toBeNull();
		});

		it('uses default page and limit', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ numFound: 0, docs: [] })
			});

			await searchBooks('test');

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('page=1');
			expect(url).toContain('limit=20');
		});
	});

	describe('getWork', () => {
		it('calls the correct endpoint with work key', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ key: '/works/OL123W', title: 'Test' })
			});

			await getWork('OL123W');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('openlibrary.org/works/OL123W.json'),
				expect.any(Object)
			);
		});

		it('handles key already prefixed with /works/', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ key: '/works/OL123W', title: 'Test' })
			});

			await getWork('/works/OL123W');

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('/works/OL123W.json');
			expect(url).not.toContain('/works//works/');
		});

		it('returns work data', async () => {
			const work = { key: '/works/OL1W', title: 'Great Book', description: 'A book.' };
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve(work)
			});

			const result = await getWork('OL1W');
			expect(result).not.toBeNull();
			expect(result!.title).toBe('Great Book');
		});

		it('returns null on 404', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve(null)
			});

			const result = await getWork('OL999W');
			expect(result).toBeNull();
		});
	});

	describe('getAuthor', () => {
		it('calls the correct endpoint with author key', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ key: '/authors/OL100A', name: 'Author' })
			});

			await getAuthor('OL100A');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('openlibrary.org/authors/OL100A.json'),
				expect.any(Object)
			);
		});

		it('handles key already prefixed with /authors/', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ key: '/authors/OL100A', name: 'Author' })
			});

			await getAuthor('/authors/OL100A');

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('/authors/OL100A.json');
			expect(url).not.toContain('/authors//authors/');
		});

		it('returns author data', async () => {
			const author = {
				key: '/authors/OL1A',
				name: 'Famous Writer',
				birth_date: '1900',
				bio: 'A writer.'
			};
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve(author)
			});

			const result = await getAuthor('OL1A');
			expect(result).not.toBeNull();
			expect(result!.name).toBe('Famous Writer');
		});

		it('returns null on 404', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve(null)
			});

			const result = await getAuthor('OL999A');
			expect(result).toBeNull();
		});
	});

	describe('getSubjectBooks', () => {
		it('calls the correct endpoint with subject', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ name: 'fiction', work_count: 0, works: [] })
			});

			await getSubjectBooks('fiction');

			expect(mockFetch).toHaveBeenCalledWith(
				expect.stringContaining('openlibrary.org/subjects/fiction.json?'),
				expect.any(Object)
			);
		});

		it('passes limit and offset parameters', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ name: 'science', work_count: 0, works: [] })
			});

			await getSubjectBooks('science', 10, 20);

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('limit=10');
			expect(url).toContain('offset=20');
		});

		it('returns subject response data', async () => {
			const data = {
				name: 'fantasy',
				work_count: 100,
				works: [{ key: '/works/OL1W', title: 'Fantasy Book' }]
			};
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve(data)
			});

			const result = await getSubjectBooks('fantasy');
			expect(result).not.toBeNull();
			expect(result!.work_count).toBe(100);
			expect(result!.works).toHaveLength(1);
		});

		it('uses default limit and offset', async () => {
			mockFetch.mockResolvedValue({
				ok: true,
				json: () => Promise.resolve({ name: 'fiction', work_count: 0, works: [] })
			});

			await getSubjectBooks('fiction');

			const url = mockFetch.mock.calls[0][0];
			expect(url).toContain('limit=20');
			expect(url).toContain('offset=0');
		});

		it('returns null on 404', async () => {
			mockFetch.mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve(null)
			});

			const result = await getSubjectBooks('nonexistent');
			expect(result).toBeNull();
		});
	});
});

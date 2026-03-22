import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

describe('JackettSearchService', () => {
	let jackettSearchService: (typeof import('../../src/services/jackett-search.service'))['jackettSearchService'];
	let JackettCategory: (typeof import('../../src/services/jackett-search.service'))['JackettCategory'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/jackett-search.service');
		jackettSearchService = mod.jackettSearchService;
		JackettCategory = mod.JackettCategory;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(jackettSearchService.state);
		expect(state.query).toBe('');
		expect(state.searching).toBe(false);
		expect(state.results).toEqual([]);
		expect(state.error).toBeNull();
		expect(state.indexers).toEqual([]);
		expect(state.sort.field).toBe('seeders');
		expect(state.sort.direction).toBe('desc');
		expect(state.filters.category).toBe(JackettCategory.All);
		expect(state.filters.tracker).toBe('');
	});

	it('should not search with empty query', async () => {
		const mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);

		await jackettSearchService.search('  ', { category: JackettCategory.All, tracker: '' });

		expect(mockFetch).not.toHaveBeenCalled();
	});

	it('should search and update results and indexers', async () => {
		const response = {
			results: [
				{
					id: '1',
					name: 'Test Result',
					infoHash: 'abc',
					magnetLink: 'magnet:?xt=abc',
					seeders: 50,
					leechers: 5,
					size: 2048,
					category: 'Movies',
					uploadedAt: 1700000000,
					tracker: 'tracker1'
				}
			],
			indexers: [{ id: 'idx1', name: 'Tracker One' }]
		};

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => response
					}) as Response
			)
		);

		await jackettSearchService.search('test', { category: JackettCategory.Movies, tracker: '' });

		const state = get(jackettSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.results).toHaveLength(1);
		expect(state.results[0].name).toBe('Test Result');
		expect(state.indexers).toHaveLength(1);
		expect(state.query).toBe('test');
	});

	it('should keep previous indexers when response has empty array', async () => {
		// Set up initial indexers
		jackettSearchService.state.update((s) => ({
			...s,
			indexers: [{ id: 'idx1', name: 'Previous' }]
		}));

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({ results: [], indexers: [] })
					}) as Response
			)
		);

		await jackettSearchService.search('test', { category: JackettCategory.All, tracker: '' });

		const state = get(jackettSearchService.state);
		expect(state.indexers).toEqual([{ id: 'idx1', name: 'Previous' }]);
	});

	it('should set error on search failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: false,
						status: 500,
						json: async () => ({ error: 'Jackett unavailable' })
					}) as Response
			)
		);

		await jackettSearchService.search('test', { category: JackettCategory.All, tracker: '' });

		const state = get(jackettSearchService.state);
		expect(state.searching).toBe(false);
		expect(state.error).toContain('Jackett unavailable');
	});

	it('should toggle sort direction', () => {
		// Initial: seeders desc
		jackettSearchService.toggleSort('seeders');
		let state = get(jackettSearchService.state);
		expect(state.sort.direction).toBe('asc');

		jackettSearchService.toggleSort('seeders');
		state = get(jackettSearchService.state);
		expect(state.sort.direction).toBe('desc');
	});

	it('should switch sort field with desc direction', () => {
		jackettSearchService.toggleSort('size');
		const state = get(jackettSearchService.state);
		expect(state.sort.field).toBe('size');
		expect(state.sort.direction).toBe('desc');
	});

	it('should mark and unmark adding torrents', () => {
		jackettSearchService.markAdding('hash1');
		let state = get(jackettSearchService.state);
		expect(state.addingTorrents.has('hash1')).toBe(true);

		jackettSearchService.markAdding('hash2');
		state = get(jackettSearchService.state);
		expect(state.addingTorrents.size).toBe(2);

		jackettSearchService.unmarkAdding('hash1');
		state = get(jackettSearchService.state);
		expect(state.addingTorrents.has('hash1')).toBe(false);
		expect(state.addingTorrents.has('hash2')).toBe(true);
	});

	it('should clear results and error', () => {
		jackettSearchService.state.update((s) => ({
			...s,
			results: [{ id: '1' }] as never,
			error: 'previous error'
		}));

		jackettSearchService.clearResults();

		const state = get(jackettSearchService.state);
		expect(state.results).toEqual([]);
		expect(state.error).toBeNull();
	});

	it('should pass tracker filter in search params', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({ results: [], indexers: [] })
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await jackettSearchService.search('test', {
			category: JackettCategory.TV,
			tracker: 'mytracker'
		});

		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('tracker=mytracker'));
		expect(mockFetch).toHaveBeenCalledWith(expect.stringContaining('cat=5000'));
	});
});

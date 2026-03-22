import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

describe('ImageTaggerService', () => {
	let imageTaggerService: (typeof import('../../src/services/image-tagger.service'))['imageTaggerService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/image-tagger.service');
		imageTaggerService = mod.imageTaggerService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(imageTaggerService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.taggerReady).toBe(false);
		expect(state.taggerInitializing).toBe(false);
		expect(state.taggerStatus).toBe('idle');
		expect(state.taggerProgress).toBe(0);
		expect(state.taggerError).toBeNull();
		expect(state.taggingItemIds).toEqual([]);
		expect(state.filter).toBe('');

		const store = get(imageTaggerService.store);
		expect(store).toEqual([]);
	});

	it('should set filter value', () => {
		imageTaggerService.setFilter('landscape');
		const state = get(imageTaggerService.state);
		expect(state.filter).toBe('landscape');
	});

	it('should check tagger status', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({
							ready: true,
							status: 'ready',
							overallProgress: 100,
							error: null
						})
					}) as Response
			)
		);

		await imageTaggerService.checkTaggerStatus();

		const state = get(imageTaggerService.state);
		expect(state.taggerReady).toBe(true);
		expect(state.taggerStatus).toBe('ready');
		expect(state.taggerProgress).toBe(100);
		expect(state.taggerError).toBeNull();
	});

	it('should tag a single image', async () => {
		imageTaggerService.store.set([
			{ id: 'img1', name: 'test.jpg', tags: [], path: '/test.jpg' } as never
		]);

		const mockFetch = vi.fn(async (url: string) => {
			if (url.includes('/api/images/tag')) {
				return {
					ok: true,
					json: async () => ({
						tags: [
							{ tag: 'nature', score: 0.95 },
							{ tag: 'landscape', score: 0.8 }
						]
					})
				} as Response;
			}
			// tagger-status for polling
			return {
				ok: true,
				json: async () => ({ ready: true, status: 'ready', overallProgress: 100, error: null })
			} as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await imageTaggerService.tagImage('img1');

		const store = get(imageTaggerService.store);
		expect(store[0].tags).toHaveLength(2);
		expect(store[0].tags[0].tag).toBe('nature');

		const state = get(imageTaggerService.state);
		expect(state.taggerReady).toBe(true);
		expect(state.taggingItemIds).not.toContain('img1');
	});

	it('should set error on tag failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/api/images/tag')) {
					return {
						ok: false,
						status: 500,
						json: async () => ({ error: 'Model not loaded' })
					} as Response;
				}
				return {
					ok: true,
					json: async () => ({ ready: false, status: 'idle', overallProgress: 0, error: null })
				} as Response;
			})
		);

		await imageTaggerService.tagImage('img1');

		const state = get(imageTaggerService.state);
		expect(state.error).toContain('Failed to tag image');
		expect(state.taggerInitializing).toBe(false);
	});

	it('should add a manual tag', async () => {
		imageTaggerService.store.set([
			{ id: 'img1', name: 'test.jpg', tags: [], path: '/test.jpg' } as never
		]);

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({})
					}) as Response
			)
		);

		await imageTaggerService.addTag('img1', ' Nature ');

		const store = get(imageTaggerService.store);
		expect(store[0].tags).toHaveLength(1);
		expect(store[0].tags[0].tag).toBe('nature');
		expect(store[0].tags[0].score).toBe(1.0);
	});

	it('should remove a tag', async () => {
		imageTaggerService.store.set([
			{
				id: 'img1',
				name: 'test.jpg',
				tags: [
					{ tag: 'nature', score: 0.9 },
					{ tag: 'landscape', score: 0.8 }
				],
				path: '/test.jpg'
			} as never
		]);

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({})
					}) as Response
			)
		);

		await imageTaggerService.removeTag('img1', 'nature');

		const store = get(imageTaggerService.store);
		expect(store[0].tags).toHaveLength(1);
		expect(store[0].tags[0].tag).toBe('landscape');
	});

	it('should batch tag images', async () => {
		imageTaggerService.store.set([
			{ id: 'img1', name: 'a.jpg', tags: [], path: '/a.jpg' } as never,
			{ id: 'img2', name: 'b.jpg', tags: [], path: '/b.jpg' } as never
		]);

		const mockFetch = vi.fn(async (url: string) => {
			if (url.includes('/api/images/tag-batch')) {
				return {
					ok: true,
					json: async () => ({
						results: {
							img1: [{ tag: 'cat', score: 0.9 }],
							img2: [{ tag: 'dog', score: 0.85 }]
						}
					})
				} as Response;
			}
			return {
				ok: true,
				json: async () => ({ ready: true, status: 'ready', overallProgress: 100, error: null })
			} as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await imageTaggerService.tagBatch(['img1', 'img2']);

		const store = get(imageTaggerService.store);
		expect(store[0].tags[0].tag).toBe('cat');
		expect(store[1].tags[0].tag).toBe('dog');
	});
});

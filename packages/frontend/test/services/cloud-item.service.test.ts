import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { cloudItemService } from '../../src/services/cloud-item.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data)
	});
}

const initialState = {
	currentItem: null,
	loading: false,
	searchResults: [],
	searchLoading: false,
	distinctKeys: [],
	distinctValues: {}
};

describe('CloudItemService', () => {
	beforeEach(() => {
		cloudItemService.state.set({ ...initialState });
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton cloudItemService', () => {
		expect(cloudItemService).toBeDefined();
		expect(cloudItemService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(cloudItemService.state);
		expect(state.currentItem).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.searchResults).toEqual([]);
		expect(state.searchLoading).toBe(false);
		expect(state.distinctKeys).toEqual([]);
		expect(state.distinctValues).toEqual({});
	});

	it('getItem fetches and sets current item', async () => {
		const mockItem = {
			id: 'item-1',
			filename: 'test.jpg',
			extension: 'jpg',
			mimeType: 'image/jpeg',
			sizeBytes: 1024,
			attributes: []
		};
		vi.stubGlobal('fetch', mockFetch(mockItem));

		const result = await cloudItemService.getItem('item-1');

		expect(result).toEqual(mockItem);
		const state = get(cloudItemService.state);
		expect(state.currentItem).toEqual(mockItem);
		expect(state.loading).toBe(false);
	});

	it('getItem returns null and logs error on failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Not found')));

		const result = await cloudItemService.getItem('bad-id');

		expect(result).toBeNull();
		expect(get(cloudItemService.state).loading).toBe(false);
	});

	it('setAttribute sends PUT and updates current item attributes', async () => {
		cloudItemService.state.set({
			...initialState,
			currentItem: {
				id: 'item-1',
				filename: 'test.jpg',
				extension: 'jpg',
				mimeType: 'image/jpeg',
				sizeBytes: 1024,
				attributes: [
					{ key: 'tag', value: 'old', typeId: 'string', source: 'user', confidence: null }
				]
			} as never
		});

		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await cloudItemService.setAttribute('item-1', 'tag', 'new-value');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/cloud/items/item-1/attributes'),
			expect.objectContaining({ method: 'PUT' })
		);

		const state = get(cloudItemService.state);
		const attrs = (state.currentItem as { attributes: Array<{ key: string; value: string }> })
			.attributes;
		expect(attrs.find((a) => a.key === 'tag')?.value).toBe('new-value');
	});

	it('removeAttribute sends DELETE and removes from current item', async () => {
		cloudItemService.state.set({
			...initialState,
			currentItem: {
				id: 'item-1',
				filename: 'test.jpg',
				extension: 'jpg',
				mimeType: 'image/jpeg',
				sizeBytes: 1024,
				attributes: [
					{ key: 'tag', value: 'val', typeId: 'string', source: 'user', confidence: null },
					{ key: 'color', value: 'red', typeId: 'string', source: 'user', confidence: null }
				]
			} as never
		});

		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await cloudItemService.removeAttribute('item-1', 'tag');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/cloud/items/item-1/attributes/tag'),
			expect.objectContaining({ method: 'DELETE' })
		);

		const state = get(cloudItemService.state);
		const attrs = (state.currentItem as { attributes: Array<{ key: string }> }).attributes;
		expect(attrs).toHaveLength(1);
		expect(attrs[0].key).toBe('color');
	});

	it('search fetches with query params', async () => {
		const mockResults = [
			{ id: 'item-1', filename: 'a.jpg' },
			{ id: 'item-2', filename: 'b.jpg' }
		];
		const mock = mockFetch(mockResults);
		vi.stubGlobal('fetch', mock);

		await cloudItemService.search('photos', 'tag', 'nature');

		expect(mock).toHaveBeenCalledWith(
			expect.stringMatching(/q=photos.*key=tag.*value=nature/),
			expect.anything()
		);

		const state = get(cloudItemService.state);
		expect(state.searchResults).toEqual(mockResults);
		expect(state.searchLoading).toBe(false);
	});

	it('search handles errors gracefully', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Search error')));

		await cloudItemService.search('test');

		expect(get(cloudItemService.state).searchLoading).toBe(false);
	});

	it('fetchDistinctKeys updates distinct keys', async () => {
		vi.stubGlobal('fetch', mockFetch(['tag', 'color', 'size']));

		await cloudItemService.fetchDistinctKeys();

		expect(get(cloudItemService.state).distinctKeys).toEqual(['tag', 'color', 'size']);
	});

	it('fetchDistinctValues updates values for a key', async () => {
		vi.stubGlobal('fetch', mockFetch(['red', 'blue', 'green']));

		await cloudItemService.fetchDistinctValues('color');

		const state = get(cloudItemService.state);
		expect(state.distinctValues.color).toEqual(['red', 'blue', 'green']);
	});

	it('fetchDistinctValues preserves other keys', async () => {
		cloudItemService.state.set({
			...initialState,
			distinctValues: { tag: ['nature', 'city'] }
		});

		vi.stubGlobal('fetch', mockFetch(['red', 'blue']));

		await cloudItemService.fetchDistinctValues('color');

		const state = get(cloudItemService.state);
		expect(state.distinctValues.tag).toEqual(['nature', 'city']);
		expect(state.distinctValues.color).toEqual(['red', 'blue']);
	});
});

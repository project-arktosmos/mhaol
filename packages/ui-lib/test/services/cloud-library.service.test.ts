import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { cloudLibraryService } from '../../src/services/cloud-library.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data)
	});
}

function mockFetchSequence(...responses: Array<{ data: unknown; ok?: boolean }>) {
	let callIdx = 0;
	return vi.fn().mockImplementation(() => {
		const resp = responses[callIdx] ?? responses[responses.length - 1];
		callIdx++;
		return Promise.resolve({
			ok: resp.ok !== false,
			status: resp.ok !== false ? 200 : 500,
			json: () => Promise.resolve(resp.data)
		});
	});
}

const initialState = {
	items: {},
	itemsLoading: {},
	browsing: false,
	browseError: null,
	currentBrowsePath: '',
	browseDirectories: [],
	browseParent: null,
	showAddForm: false,
	selectedPath: '',
	selectedName: ''
};

describe('CloudLibraryService', () => {
	beforeEach(() => {
		cloudLibraryService.store.set([]);
		cloudLibraryService.state.set({ ...initialState });
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton cloudLibraryService', () => {
		expect(cloudLibraryService).toBeDefined();
		expect(cloudLibraryService.store).toBeDefined();
		expect(cloudLibraryService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(cloudLibraryService.state);
		expect(state.items).toEqual({});
		expect(state.browsing).toBe(false);
		expect(state.showAddForm).toBe(false);
		expect(state.selectedPath).toBe('');
		expect(get(cloudLibraryService.store)).toEqual([]);
	});

	it('addLibrary sends POST and updates store', async () => {
		const newLib = { id: 'lib-1', name: 'Photos', path: '/photos', kind: 'filesystem' };
		vi.stubGlobal('fetch', mockFetch(newLib));

		await cloudLibraryService.addLibrary('Photos', '/photos');

		const libs = get(cloudLibraryService.store);
		expect(libs).toHaveLength(1);
		expect(libs[0].name).toBe('Photos');
	});

	it('removeLibrary sends DELETE and removes from store', async () => {
		cloudLibraryService.store.set([
			{ id: 'lib-1', name: 'Photos' } as never,
			{ id: 'lib-2', name: 'Music' } as never
		]);
		cloudLibraryService.state.set({
			...initialState,
			items: { 'lib-1': [], 'lib-2': [] }
		});

		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await cloudLibraryService.removeLibrary('lib-1');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/cloud/libraries/lib-1'),
			expect.objectContaining({ method: 'DELETE' })
		);

		const libs = get(cloudLibraryService.store);
		expect(libs).toHaveLength(1);
		expect(libs[0].id).toBe('lib-2');

		const state = get(cloudLibraryService.state);
		expect(state.items['lib-1']).toBeUndefined();
	});

	it('scanLibrary sends POST and updates items', async () => {
		const scanResponse = {
			items: [{ id: 'item-1', filename: 'photo.jpg' }],
			itemCount: 1
		};
		vi.stubGlobal('fetch', mockFetch(scanResponse));

		cloudLibraryService.store.set([
			{ id: 'lib-1', name: 'Photos', itemCount: 0, scanStatus: 'idle' } as never
		]);

		await cloudLibraryService.scanLibrary('lib-1');

		const state = get(cloudLibraryService.state);
		expect(state.items['lib-1']).toHaveLength(1);
		expect(state.itemsLoading['lib-1']).toBe(false);

		const libs = get(cloudLibraryService.store);
		expect(libs[0].itemCount).toBe(1);
	});

	it('fetchItems fetches and stores items for a library', async () => {
		const mockItems = [
			{ id: 'item-1', filename: 'a.jpg' },
			{ id: 'item-2', filename: 'b.jpg' }
		];
		vi.stubGlobal('fetch', mockFetch(mockItems));

		await cloudLibraryService.fetchItems('lib-1');

		const state = get(cloudLibraryService.state);
		expect(state.items['lib-1']).toEqual(mockItems);
		expect(state.itemsLoading['lib-1']).toBe(false);
	});

	it('fetchItems handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		await cloudLibraryService.fetchItems('lib-1');

		const state = get(cloudLibraryService.state);
		expect(state.itemsLoading['lib-1']).toBe(false);
	});

	it('browseDirectory fetches directory listing', async () => {
		const browseResponse = {
			path: '/home/user',
			directories: [
				{ name: 'Photos', path: '/home/user/Photos' },
				{ name: 'Music', path: '/home/user/Music' }
			],
			parent: '/home'
		};
		vi.stubGlobal('fetch', mockFetch(browseResponse));

		await cloudLibraryService.browseDirectory('/home/user');

		const state = get(cloudLibraryService.state);
		expect(state.browsing).toBe(false);
		expect(state.currentBrowsePath).toBe('/home/user');
		expect(state.browseDirectories).toHaveLength(2);
		expect(state.browseParent).toBe('/home');
	});

	it('browseDirectory handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Permission denied')));

		await cloudLibraryService.browseDirectory('/root');

		const state = get(cloudLibraryService.state);
		expect(state.browsing).toBe(false);
		expect(state.browseError).toContain('Failed to browse directory');
	});

	it('selectDirectory updates selected path and name', () => {
		cloudLibraryService.selectDirectory('/home/user/Photos', 'Photos');

		const state = get(cloudLibraryService.state);
		expect(state.selectedPath).toBe('/home/user/Photos');
		expect(state.selectedName).toBe('Photos');
	});

	it('selectDirectory preserves existing selectedName', () => {
		cloudLibraryService.state.update((s) => ({ ...s, selectedName: 'My Custom Name' }));

		cloudLibraryService.selectDirectory('/home/user/Photos', 'Photos');

		const state = get(cloudLibraryService.state);
		expect(state.selectedPath).toBe('/home/user/Photos');
		expect(state.selectedName).toBe('My Custom Name');
	});

	it('setSelectedName updates name', () => {
		cloudLibraryService.setSelectedName('New Name');
		expect(get(cloudLibraryService.state).selectedName).toBe('New Name');
	});

	it('closeAddForm resets form state', () => {
		cloudLibraryService.state.set({
			...initialState,
			showAddForm: true,
			selectedPath: '/some/path',
			selectedName: 'Some Name',
			currentBrowsePath: '/some',
			browseDirectories: [{ name: 'dir', path: '/some/dir' } as never],
			browseParent: '/'
		});

		cloudLibraryService.closeAddForm();

		const state = get(cloudLibraryService.state);
		expect(state.showAddForm).toBe(false);
		expect(state.selectedPath).toBe('');
		expect(state.selectedName).toBe('');
		expect(state.currentBrowsePath).toBe('');
		expect(state.browseDirectories).toEqual([]);
		expect(state.browseParent).toBeNull();
	});
});

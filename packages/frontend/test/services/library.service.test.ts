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

function mockFetchError(errorMsg: string, status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status,
		json: () => Promise.resolve({ error: errorMsg }),
		text: () => Promise.resolve(''),
		body: null
	});
}

function mockFetchByUrl(handlers: Record<string, unknown>) {
	return vi.fn(async (url: string) => {
		for (const [pattern, data] of Object.entries(handlers)) {
			if (url.includes(pattern)) {
				return {
					ok: true,
					json: async () => data,
					text: async () => JSON.stringify(data)
				} as Response;
			}
		}
		return { ok: true, json: async () => ({}), text: async () => '{}' } as Response;
	});
}

describe('LibraryService', () => {
	let libraryService: (typeof import('../../src/services/library.service'))['libraryService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/library.service');
		libraryService = mod.libraryService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('should have correct initial state', () => {
		const state = get(libraryService.state);
		expect(state.showAddForm).toBe(false);
		expect(state.browsing).toBe(false);
		expect(state.browseError).toBeNull();
		expect(state.currentBrowsePath).toBe('');
		expect(state.browseDirectories).toEqual([]);
		expect(state.browseParent).toBeNull();
		expect(state.selectedPath).toBe('');
		expect(state.selectedName).toBe('');
		expect(state.selectedMediaTypes).toEqual([]);
		expect(state.selectedLibraryType).toBeNull();
		expect(state.libraryFiles).toEqual({});
		expect(state.libraryFilesLoading).toEqual({});
		expect(state.libraryFilesError).toEqual({});
	});

	it('should have empty initial store', () => {
		const libraries = get(libraryService.store);
		expect(libraries).toEqual([]);
	});

	// ===== initialize =====

	it('should initialize and fetch libraries', async () => {
		const libraries = [
			{ id: 'lib-1', name: 'Movies', path: '/movies', mediaTypes: ['video'] },
			{ id: 'lib-2', name: 'Music', path: '/music', mediaTypes: ['audio'] }
		];

		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (
					url.includes('/api/libraries/lib-1/files') ||
					url.includes('/api/libraries/lib-2/files')
				) {
					return { ok: true, json: async () => ({ files: [] }) } as Response;
				}
				return { ok: true, json: async () => libraries } as Response;
			})
		);

		await libraryService.initialize();

		const store = get(libraryService.store);
		expect(store).toHaveLength(2);
		expect(store[0].name).toBe('Movies');
	});

	it('should not initialize twice', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => []
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		await libraryService.initialize();
		const callCount = mockFn.mock.calls.length;

		await libraryService.initialize();
		expect(mockFn.mock.calls.length).toBe(callCount);
	});

	it('should handle initialization error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await libraryService.initialize();

		const store = get(libraryService.store);
		expect(store).toEqual([]);
	});

	// ===== browseDirectory =====

	it('should browse directory and update state', async () => {
		const browseResponse = {
			path: '/home/user',
			directories: [
				{ name: 'Documents', path: '/home/user/Documents' },
				{ name: 'Music', path: '/home/user/Music' }
			],
			parent: '/home'
		};

		vi.stubGlobal('fetch', mockFetchOk(browseResponse));

		await libraryService.browseDirectory('/home/user');

		const state = get(libraryService.state);
		expect(state.browsing).toBe(false);
		expect(state.currentBrowsePath).toBe('/home/user');
		expect(state.browseDirectories).toHaveLength(2);
		expect(state.browseParent).toBe('/home');
	});

	it('should browse root directory without path argument', async () => {
		const browseResponse = {
			path: '/',
			directories: [{ name: 'home', path: '/home' }],
			parent: null
		};

		const mockFn = mockFetchOk(browseResponse);
		vi.stubGlobal('fetch', mockFn);

		await libraryService.browseDirectory();

		expect(mockFn).toHaveBeenCalledWith(expect.not.stringContaining('?path='), expect.anything());

		const state = get(libraryService.state);
		expect(state.currentBrowsePath).toBe('/');
	});

	it('should set browse error on failure', async () => {
		vi.stubGlobal('fetch', mockFetchError('Permission denied'));

		await libraryService.browseDirectory('/root');

		const state = get(libraryService.state);
		expect(state.browsing).toBe(false);
		expect(state.browseError).toContain('Failed to browse directory');
		expect(state.browseError).toContain('Permission denied');
	});

	it('should set browsing to true during browse', async () => {
		let capturedBrowsing = false;
		const unsub = libraryService.state.subscribe((s) => {
			if (s.browsing) capturedBrowsing = true;
		});

		vi.stubGlobal('fetch', mockFetchOk({ path: '/', directories: [], parent: null }));

		await libraryService.browseDirectory();
		unsub();
		expect(capturedBrowsing).toBe(true);
	});

	it('should handle browseDirectory with non-Error throw', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue('string error'));

		await libraryService.browseDirectory('/test');

		const state = get(libraryService.state);
		expect(state.browseError).toContain('string error');
	});

	// ===== addLibrary =====

	it('should add library and reset form', async () => {
		const newLibrary = {
			id: 'lib-1',
			name: 'Videos',
			path: '/home/user/Videos',
			mediaTypes: ['video']
		};

		vi.stubGlobal('fetch', mockFetchOk(newLibrary));

		await libraryService.addLibrary('Videos', '/home/user/Videos', ['video'] as never);

		const libraries = get(libraryService.store);
		expect(libraries).toHaveLength(1);
		expect(libraries[0].name).toBe('Videos');

		const state = get(libraryService.state);
		expect(state.showAddForm).toBe(false);
		expect(state.selectedPath).toBe('');
	});

	it('should handle addLibrary failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Create failed')));

		await libraryService.addLibrary('Test', '/path', []);

		const libraries = get(libraryService.store);
		expect(libraries).toHaveLength(0);
	});

	// ===== removeLibrary =====

	it('should remove library from store', async () => {
		const lib = { id: 'lib-1', name: 'Videos', path: '/home/user/Videos', mediaTypes: ['video'] };
		libraryService.store.set([lib as never]);

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.removeLibrary(lib as never);

		const libraries = get(libraryService.store);
		expect(libraries).toHaveLength(0);
	});

	it('should handle removeLibrary failure', async () => {
		const lib = { id: 'lib-1', name: 'Videos', path: '/path', mediaTypes: [] };
		libraryService.store.set([lib as never]);

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Delete failed')));

		await libraryService.removeLibrary(lib as never);

		// Store should remain unchanged on error
		const libraries = get(libraryService.store);
		expect(libraries).toHaveLength(1);
	});

	// ===== openAddForm / closeAddForm =====

	it('should open add form and start browsing', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ path: '/', directories: [], parent: null }));

		libraryService.openAddForm();

		const state = get(libraryService.state);
		expect(state.showAddForm).toBe(true);
		expect(state.selectedPath).toBe('');
		expect(state.selectedName).toBe('');
		expect(state.selectedMediaTypes).toEqual([]);
		expect(state.selectedLibraryType).toBeNull();
		expect(state.browseError).toBeNull();
	});

	it('should close add form and reset all fields', () => {
		libraryService.state.update((s) => ({
			...s,
			showAddForm: true,
			selectedPath: '/some/path',
			selectedName: 'Test',
			selectedMediaTypes: ['video'] as never,
			selectedLibraryType: 'local' as never,
			currentBrowsePath: '/some',
			browseDirectories: [{ name: 'path', path: '/some/path' }] as never,
			browseParent: '/',
			browseError: 'some error'
		}));

		libraryService.closeAddForm();

		const state = get(libraryService.state);
		expect(state.showAddForm).toBe(false);
		expect(state.selectedPath).toBe('');
		expect(state.selectedName).toBe('');
		expect(state.selectedMediaTypes).toEqual([]);
		expect(state.selectedLibraryType).toBeNull();
		expect(state.currentBrowsePath).toBe('');
		expect(state.browseDirectories).toEqual([]);
		expect(state.browseParent).toBeNull();
		expect(state.browseError).toBeNull();
	});

	// ===== selectDirectory =====

	it('should select a directory', () => {
		libraryService.selectDirectory('/home/user/Videos', 'Videos');

		const state = get(libraryService.state);
		expect(state.selectedPath).toBe('/home/user/Videos');
		expect(state.selectedName).toBe('Videos');
	});

	it('should not override user-set name when selecting directory', () => {
		libraryService.setSelectedName('My Library');
		libraryService.selectDirectory('/home/user/Videos', 'Videos');

		const state = get(libraryService.state);
		expect(state.selectedName).toBe('My Library');
	});

	// ===== setSelectedName =====

	it('should set selected name', () => {
		libraryService.setSelectedName('Custom Name');
		const state = get(libraryService.state);
		expect(state.selectedName).toBe('Custom Name');
	});

	// ===== setLibraryType =====

	it('should set library type', () => {
		libraryService.setLibraryType('local' as never);
		const state = get(libraryService.state);
		expect(state.selectedLibraryType).toBe('local');
	});

	// ===== toggleMediaType =====

	it('should toggle media types', () => {
		libraryService.toggleMediaType('video' as never);
		let state = get(libraryService.state);
		expect(state.selectedMediaTypes).toContain('video');

		libraryService.toggleMediaType('audio' as never);
		state = get(libraryService.state);
		expect(state.selectedMediaTypes).toContain('video');
		expect(state.selectedMediaTypes).toContain('audio');

		libraryService.toggleMediaType('video' as never);
		state = get(libraryService.state);
		expect(state.selectedMediaTypes).not.toContain('video');
		expect(state.selectedMediaTypes).toContain('audio');
	});

	// ===== fetchLibraryFiles =====

	it('should fetch library files', async () => {
		const filesResponse = {
			files: [{ id: 'f1', name: 'movie.mp4', extension: 'mp4', mediaType: 'video', links: {} }]
		};

		vi.stubGlobal('fetch', mockFetchOk(filesResponse));

		await libraryService.fetchLibraryFiles('lib-1');

		const state = get(libraryService.state);
		expect(state.libraryFiles['lib-1']).toHaveLength(1);
		expect(state.libraryFilesLoading['lib-1']).toBe(false);
		expect(state.libraryFilesError['lib-1']).toBeNull();
	});

	it('should handle fetchLibraryFiles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Fetch failed')));

		await libraryService.fetchLibraryFiles('lib-1');

		const state = get(libraryService.state);
		expect(state.libraryFilesLoading['lib-1']).toBe(false);
		expect(state.libraryFilesError['lib-1']).toContain('Fetch failed');
	});

	it('should set loading state during fetchLibraryFiles', async () => {
		let capturedLoading = false;
		const unsub = libraryService.state.subscribe((s) => {
			if (s.libraryFilesLoading['lib-1']) capturedLoading = true;
		});

		vi.stubGlobal('fetch', mockFetchOk({ files: [] }));

		await libraryService.fetchLibraryFiles('lib-1');
		unsub();
		expect(capturedLoading).toBe(true);
	});

	// ===== scanLibraryFiles =====

	it('should scan library files', async () => {
		const filesResponse = {
			files: [
				{ id: 'f1', name: 'movie.mp4', mediaType: 'video', links: {} },
				{ id: 'f2', name: 'song.mp3', mediaType: 'audio', links: {} }
			]
		};

		vi.stubGlobal('fetch', mockFetchOk(filesResponse));

		await libraryService.scanLibraryFiles('lib-1');

		const state = get(libraryService.state);
		expect(state.libraryFiles['lib-1']).toHaveLength(2);
		expect(state.libraryFilesLoading['lib-1']).toBe(false);
	});

	it('should send POST when scanning library files', async () => {
		const mockFn = mockFetchOk({ files: [] });
		vi.stubGlobal('fetch', mockFn);

		await libraryService.scanLibraryFiles('lib-1');

		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/libraries/lib-1/scan'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should handle scanLibraryFiles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Scan failed')));

		await libraryService.scanLibraryFiles('lib-1');

		const state = get(libraryService.state);
		expect(state.libraryFilesLoading['lib-1']).toBe(false);
		expect(state.libraryFilesError['lib-1']).toContain('Scan failed');
	});

	// ===== scanAllLibraries =====

	it('should scan all libraries', async () => {
		libraryService.store.set([
			{ id: 'lib-1', name: 'Movies' } as never,
			{ id: 'lib-2', name: 'Music' } as never
		]);

		const mockFn = mockFetchOk({ files: [] });
		vi.stubGlobal('fetch', mockFn);

		await libraryService.scanAllLibraries();

		// Should have called scan for both libraries
		const scanCalls = mockFn.mock.calls.filter((c: string[]) => c[0].includes('/scan'));
		expect(scanCalls).toHaveLength(2);
	});

	// ===== linkTmdb =====

	it('should link TMDB to a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [
					{ id: 'item-1', name: 'movie.mp4', links: {} } as never,
					{ id: 'item-2', name: 'other.mp4', links: {} } as never
				]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.linkTmdb('lib-1', 'item-1', 12345);

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.tmdb).toEqual({
			serviceId: '12345',
			seasonNumber: null,
			episodeNumber: null
		});
	});

	it('should link TMDB with season and episode', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [{ id: 'item-1', name: 'ep.mp4', links: {} } as never]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.linkTmdb('lib-1', 'item-1', 99, 2, 5);

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.tmdb).toEqual({
			serviceId: '99',
			seasonNumber: 2,
			episodeNumber: 5
		});
	});

	it('should handle linkTmdb when library files not loaded', async () => {
		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.linkTmdb('lib-1', 'item-1', 123);

		const state = get(libraryService.state);
		expect(state.libraryFiles['lib-1']).toBeUndefined();
	});

	// ===== unlinkTmdb =====

	it('should unlink TMDB from a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [
					{
						id: 'item-1',
						name: 'movie.mp4',
						links: { tmdb: { serviceId: '123', seasonNumber: null, episodeNumber: null } }
					} as never
				]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.unlinkTmdb('lib-1', 'item-1');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.tmdb).toBeUndefined();
	});

	// ===== linkYoutube =====

	it('should link YouTube to a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [{ id: 'item-1', name: 'video.mp4', links: {} } as never]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.linkYoutube('lib-1', 'item-1', 'dQw4w9WgXcQ');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.youtube).toEqual({
			serviceId: 'dQw4w9WgXcQ',
			seasonNumber: null,
			episodeNumber: null
		});
	});

	// ===== unlinkYoutube =====

	it('should unlink YouTube from a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [
					{
						id: 'item-1',
						name: 'video.mp4',
						links: { youtube: { serviceId: 'abc', seasonNumber: null, episodeNumber: null } }
					} as never
				]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.unlinkYoutube('lib-1', 'item-1');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.youtube).toBeUndefined();
	});

	// ===== linkMusicBrainz =====

	it('should link MusicBrainz to a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [{ id: 'item-1', name: 'song.mp3', links: {} } as never]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.linkMusicBrainz('lib-1', 'item-1', 'mb-id-123');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.musicbrainz).toEqual({
			serviceId: 'mb-id-123',
			seasonNumber: null,
			episodeNumber: null
		});
	});

	// ===== unlinkMusicBrainz =====

	it('should unlink MusicBrainz from a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [
					{
						id: 'item-1',
						name: 'song.mp3',
						links: { musicbrainz: { serviceId: 'mb-1', seasonNumber: null, episodeNumber: null } }
					} as never
				]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.unlinkMusicBrainz('lib-1', 'item-1');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.links.musicbrainz).toBeUndefined();
	});

	// ===== updateCategory =====

	it('should update category for a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [{ id: 'item-1', name: 'movie.mp4', categoryId: null, links: {} } as never]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.updateCategory('lib-1', 'item-1', 'cat-action');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.categoryId).toBe('cat-action');
	});

	// ===== clearCategory =====

	it('should clear category for a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [{ id: 'item-1', name: 'movie.mp4', categoryId: 'cat-action', links: {} } as never]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.clearCategory('lib-1', 'item-1');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.categoryId).toBeNull();
	});

	// ===== updateMediaType =====

	it('should update media type and clear category for a library item', async () => {
		libraryService.state.update((s) => ({
			...s,
			libraryFiles: {
				'lib-1': [
					{
						id: 'item-1',
						name: 'file.mp4',
						mediaType: 'video',
						categoryId: 'cat-1',
						links: {}
					} as never
				]
			}
		}));

		vi.stubGlobal('fetch', mockFetchOk({}));

		await libraryService.updateMediaType('lib-1', 'item-1', 'audio');

		const state = get(libraryService.state);
		const file = state.libraryFiles['lib-1'].find((f) => f.id === 'item-1');
		expect(file?.mediaType).toBe('audio');
		expect(file?.categoryId).toBeNull();
	});

	// ===== fetchMediaTypes =====

	it('should fetch media types', async () => {
		const mediaTypes = [
			{ id: 'video', label: 'Video' },
			{ id: 'audio', label: 'Audio' }
		];

		vi.stubGlobal('fetch', mockFetchOk(mediaTypes));

		const result = await libraryService.fetchMediaTypes();

		expect(result).toHaveLength(2);
		expect(result[0].id).toBe('video');
	});

	// ===== fetchCategories =====

	it('should fetch categories without media type filter', async () => {
		const categories = [{ id: 'cat-1', label: 'Action' }];

		const mockFn = mockFetchOk(categories);
		vi.stubGlobal('fetch', mockFn);

		const result = await libraryService.fetchCategories();

		expect(result).toHaveLength(1);
		expect(mockFn).toHaveBeenCalledWith(
			expect.not.stringContaining('?mediaType='),
			expect.anything()
		);
	});

	it('should fetch categories with media type filter', async () => {
		const categories = [{ id: 'cat-1', label: 'Action' }];

		const mockFn = mockFetchOk(categories);
		vi.stubGlobal('fetch', mockFn);

		const result = await libraryService.fetchCategories('video');

		expect(result).toHaveLength(1);
		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('?mediaType=video'),
			expect.anything()
		);
	});

	// ===== fetchJson error handling =====

	it('should throw error with server error message on non-ok response', async () => {
		vi.stubGlobal('fetch', mockFetchError('Custom error message'));

		await libraryService.browseDirectory('/test');

		const state = get(libraryService.state);
		expect(state.browseError).toContain('Custom error message');
	});

	it('should throw error with HTTP status when server error has no message', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 404,
				json: () => Promise.resolve({})
			})
		);

		await libraryService.browseDirectory('/test');

		const state = get(libraryService.state);
		expect(state.browseError).toContain('HTTP 404');
	});
});

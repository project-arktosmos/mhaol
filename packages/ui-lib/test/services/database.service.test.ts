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

function mockFetchError(status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status,
		json: () => Promise.resolve({}),
		text: () => Promise.resolve(''),
		body: null
	});
}

describe('DatabaseService', () => {
	let databaseService: (typeof import('../../src/services/database.service'))['databaseService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/database.service');
		databaseService = mod.databaseService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('should have correct initial state', () => {
		const state = get(databaseService.state);
		expect(state.tables).toEqual([]);
		expect(state.selectedTable).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	// ===== fetchTables =====

	it('should fetch tables and update state', async () => {
		const tables = [
			{ name: 'users', columns: [{ name: 'id', type: 'INTEGER' }], rowCount: 10 },
			{ name: 'posts', columns: [{ name: 'id', type: 'INTEGER' }], rowCount: 25 }
		];
		vi.stubGlobal('fetch', mockFetchOk(tables));

		await databaseService.fetchTables();

		const state = get(databaseService.state);
		expect(state.tables).toHaveLength(2);
		expect(state.tables[0].name).toBe('users');
		expect(state.tables[1].name).toBe('posts');
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should set loading to true during fetchTables', async () => {
		let capturedLoading = false;
		const unsub = databaseService.state.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal('fetch', mockFetchOk([]));
		await databaseService.fetchTables();

		unsub();
		expect(capturedLoading).toBe(true);
	});

	it('should handle fetchTables HTTP error', async () => {
		vi.stubGlobal('fetch', mockFetchError(500));

		await databaseService.fetchTables();

		const state = get(databaseService.state);
		expect(state.tables).toEqual([]);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('HTTP 500');
	});

	it('should handle fetchTables network error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network failure')));

		await databaseService.fetchTables();

		const state = get(databaseService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('Network failure');
	});

	it('should clear error on new fetchTables call', async () => {
		// First call fails
		vi.stubGlobal('fetch', mockFetchError(500));
		await databaseService.fetchTables();
		expect(get(databaseService.state).error).toBe('HTTP 500');

		// Second call succeeds
		vi.stubGlobal('fetch', mockFetchOk([]));
		await databaseService.fetchTables();
		expect(get(databaseService.state).error).toBeNull();
	});

	// ===== fetchTableRows =====

	it('should fetch table rows and set selectedTable', async () => {
		const detail = {
			table: 'users',
			columns: [{ name: 'id', type: 'INTEGER' }],
			rows: [{ id: 1 }, { id: 2 }],
			pagination: { page: 1, limit: 20, total: 2, totalPages: 1 }
		};
		vi.stubGlobal('fetch', mockFetchOk(detail));

		await databaseService.fetchTableRows('users');

		const state = get(databaseService.state);
		expect(state.selectedTable).not.toBeNull();
		expect(state.selectedTable!.table).toBe('users');
		expect(state.selectedTable!.rows).toHaveLength(2);
		expect(state.loading).toBe(false);
	});

	it('should pass page and limit parameters', async () => {
		const mockFn = mockFetchOk({
			table: 'users',
			columns: [],
			rows: [],
			pagination: { page: 2, limit: 10, total: 30, totalPages: 3 }
		});
		vi.stubGlobal('fetch', mockFn);

		await databaseService.fetchTableRows('users', 2, 10);

		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/database/tables/users?page=2&limit=10')
		);
	});

	it('should handle fetchTableRows HTTP error', async () => {
		vi.stubGlobal('fetch', mockFetchError(404));

		await databaseService.fetchTableRows('nonexistent');

		const state = get(databaseService.state);
		expect(state.selectedTable).toBeNull();
		expect(state.loading).toBe(false);
		expect(state.error).toBe('HTTP 404');
	});

	it('should handle fetchTableRows network error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Connection refused')));

		await databaseService.fetchTableRows('users');

		const state = get(databaseService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('Connection refused');
	});

	// ===== clearSelection =====

	it('should clear the selected table', async () => {
		const detail = {
			table: 'users',
			columns: [],
			rows: [],
			pagination: { page: 1, limit: 20, total: 0, totalPages: 0 }
		};
		vi.stubGlobal('fetch', mockFetchOk(detail));
		await databaseService.fetchTableRows('users');
		expect(get(databaseService.state).selectedTable).not.toBeNull();

		databaseService.clearSelection();

		expect(get(databaseService.state).selectedTable).toBeNull();
	});

	it('should not affect tables list when clearing selection', async () => {
		const tables = [{ name: 'users', columns: [], rowCount: 5 }];
		vi.stubGlobal('fetch', mockFetchOk(tables));
		await databaseService.fetchTables();

		databaseService.clearSelection();

		const state = get(databaseService.state);
		expect(state.tables).toHaveLength(1);
		expect(state.selectedTable).toBeNull();
	});
});

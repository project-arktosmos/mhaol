import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { identityService } from '../../src/services/identity.service';

const mockIdentities = [
	{ name: 'Alice', address: '0xabc', passport: 'passport-1' },
	{ name: 'Bob', address: '0xdef', passport: 'passport-2' }
];

function mockFetchSuccess(data: unknown = mockIdentities) {
	return vi.fn().mockResolvedValue({
		ok: true,
		json: () => Promise.resolve(data)
	});
}

function mockFetchError(status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status
	});
}

describe('identityService', () => {
	beforeEach(() => {
		identityService.state.set({
			loading: false,
			identities: [],
			error: null
		});
		// Reset _initialized via casting to allow re-initialization in tests
		(identityService as any)._initialized = false;
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should have empty initial state', () => {
		const state = get(identityService.state);
		expect(state.identities).toEqual([]);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should fetch identities on initialize', async () => {
		vi.stubGlobal('fetch', mockFetchSuccess());

		await identityService.initialize();

		const state = get(identityService.state);
		expect(state.identities).toHaveLength(2);
		expect(state.identities[0].name).toBe('Alice');
		expect(state.loading).toBe(false);
	});

	it('should not re-initialize if already initialized', async () => {
		const fetchMock = mockFetchSuccess();
		vi.stubGlobal('fetch', fetchMock);

		await identityService.initialize();
		await identityService.initialize();

		expect(fetchMock).toHaveBeenCalledTimes(1);
	});

	it('should handle fetch error', async () => {
		vi.stubGlobal('fetch', mockFetchError(500));

		await identityService.refresh();

		const state = get(identityService.state);
		expect(state.error).toBe('HTTP 500');
		expect(state.loading).toBe(false);
	});

	it('should refresh identities', async () => {
		vi.stubGlobal(
			'fetch',
			mockFetchSuccess([{ name: 'Charlie', address: '0x123', passport: 'p3' }])
		);

		await identityService.refresh();

		const state = get(identityService.state);
		expect(state.identities).toHaveLength(1);
		expect(state.identities[0].name).toBe('Charlie');
	});

	it('should set loading to true during fetch', async () => {
		let loadingDuringFetch = false;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation(() => {
				loadingDuringFetch = get(identityService.state).loading;
				return Promise.resolve({
					ok: true,
					json: () => Promise.resolve([])
				});
			})
		);

		await identityService.refresh();
		expect(loadingDuringFetch).toBe(true);
	});
});

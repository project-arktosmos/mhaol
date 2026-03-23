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

describe('RosterService', () => {
	let rosterService: (typeof import('../../src/services/roster.service'))['rosterService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/roster.service');
		rosterService = mod.rosterService;
	});

	afterEach(() => {
		rosterService.destroy();
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('should have correct initial state', () => {
		const state = get(rosterService.state);
		expect(state.loading).toBe(false);
		expect(state.entries).toEqual([]);
		expect(state.error).toBeNull();
		expect(state.signalingRoomId).toBe('handshakes');
	});

	// ===== initialize (api mode) =====

	it('should initialize and load contacts from API', async () => {
		const contacts = [
			{ name: 'Alice', address: '0xabc' },
			{ name: 'Bob', address: '0xdef' }
		];
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': contacts,
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		const state = get(rosterService.state);
		expect(state.entries).toHaveLength(2);
		expect(state.entries[0].name).toBe('Alice');
		expect(state.entries[1].name).toBe('Bob');
	});

	it('should not initialize twice', async () => {
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': [],
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');
		const callCount = mockFn.mock.calls.length;

		await rosterService.initialize('api');
		expect(mockFn.mock.calls.length).toBe(callCount);
	});

	// ===== initialize (local mode) =====

	it('should initialize and load contacts from localStorage', async () => {
		localStorage.setItem(
			'roster-entries',
			JSON.stringify([{ name: 'Local User', address: '0x123' }])
		);

		vi.stubGlobal('fetch', mockFetchOk({ peers: [] }));

		await rosterService.initialize('local');

		const state = get(rosterService.state);
		expect(state.entries).toHaveLength(1);
		expect(state.entries[0].name).toBe('Local User');
	});

	// ===== addEntry (api mode) =====

	it('should add an entry via API and refresh', async () => {
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': [{ name: 'New Contact', address: '0xnew' }],
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');
		await rosterService.addEntry({ name: 'New Contact', address: '0xnew' });

		// Verify POST was called
		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/roster'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	// ===== addEntry (local mode) =====

	it('should add an entry to localStorage', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ peers: [] }));

		await rosterService.initialize('local');
		await rosterService.addEntry({ name: 'Local Add', address: '0xlocal' });

		const stored = JSON.parse(localStorage.getItem('roster-entries') || '[]');
		expect(stored).toHaveLength(1);
		expect(stored[0].name).toBe('Local Add');
	});

	it('should update existing entry in localStorage if address matches', async () => {
		localStorage.setItem(
			'roster-entries',
			JSON.stringify([{ name: 'Old Name', address: '0xaddr' }])
		);
		vi.stubGlobal('fetch', mockFetchOk({ peers: [] }));

		await rosterService.initialize('local');
		await rosterService.addEntry({ name: 'New Name', address: '0xaddr' });

		const stored = JSON.parse(localStorage.getItem('roster-entries') || '[]');
		expect(stored).toHaveLength(1);
		expect(stored[0].name).toBe('New Name');
	});

	// ===== removeEntry (api mode) =====

	it('should remove an entry via API', async () => {
		const contacts = [{ name: 'Alice', address: '0xabc' }];
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': contacts,
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');
		await rosterService.removeEntry('0xabc');

		expect(mockFn).toHaveBeenCalledWith(
			expect.stringContaining('/api/roster/0xabc'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	// ===== removeEntry (local mode) =====

	it('should remove an entry from localStorage', async () => {
		localStorage.setItem(
			'roster-entries',
			JSON.stringify([
				{ name: 'Keep', address: '0xkeep' },
				{ name: 'Remove', address: '0xremove' }
			])
		);
		vi.stubGlobal('fetch', mockFetchOk({ peers: [] }));

		await rosterService.initialize('local');
		await rosterService.removeEntry('0xremove');

		const stored = JSON.parse(localStorage.getItem('roster-entries') || '[]');
		expect(stored).toHaveLength(1);
		expect(stored[0].address).toBe('0xkeep');
	});

	// ===== checkOnlineStatus =====

	it('should mark entries as online when signaling returns matching peers', async () => {
		const contacts = [{ name: 'Alice', address: '0xabc' }];
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': contacts,
			'/party/': { peers: [{ peer_id: '0xabc' }] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		const state = get(rosterService.state);
		const alice = state.entries.find((e) => e.address === '0xabc');
		expect(alice?.status).toBe('online');
	});

	it('should mark entries as offline when signaling returns no matching peers', async () => {
		const contacts = [{ name: 'Alice', address: '0xabc' }];
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': contacts,
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		const state = get(rosterService.state);
		const alice = state.entries.find((e) => e.address === '0xabc');
		expect(alice?.status).toBe('offline');
	});

	it('should mark all entries offline when signaling check fails', async () => {
		const contacts = [{ name: 'Alice', address: '0xabc' }];
		let callCount = 0;
		const mockFn = vi.fn(async (url: string) => {
			if (url.includes('/api/signaling/status')) {
				return { ok: true, json: async () => ({ devAvailable: false }) } as Response;
			}
			if (url.includes('/api/roster')) {
				return { ok: true, json: async () => contacts } as Response;
			}
			if (url.includes('/party/')) {
				callCount++;
				if (callCount > 1) {
					throw new Error('Signaling down');
				}
				return { ok: true, json: async () => ({ peers: [] }) } as Response;
			}
			return { ok: true, json: async () => ({}) } as Response;
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		// Trigger another check that will fail
		await rosterService.checkOnlineStatus();

		const state = get(rosterService.state);
		expect(state.entries[0].status).toBe('offline');
	});

	// ===== refresh error handling =====

	it('should set error when API load fails', async () => {
		const mockFn = vi.fn(async (url: string) => {
			if (url.includes('/api/signaling/status')) {
				return { ok: true, json: async () => ({ devAvailable: false }) } as Response;
			}
			if (url.includes('/api/roster')) {
				return { ok: false, status: 500, json: async () => ({}) } as Response;
			}
			return { ok: true, json: async () => ({}) } as Response;
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		const state = get(rosterService.state);
		expect(state.error).toBe('HTTP 500');
		expect(state.loading).toBe(false);
	});

	// ===== destroy =====

	it('should reset initialized flag on destroy', async () => {
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: false },
			'/api/roster': [],
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');
		rosterService.destroy();

		// Should be able to initialize again
		await rosterService.initialize('api');
		// If it re-initialized, fetch was called again
		expect(mockFn.mock.calls.length).toBeGreaterThan(3);
	});

	// ===== fetchSignalingUrl =====

	it('should update signaling URL when dev is available', async () => {
		const mockFn = mockFetchByUrl({
			'/api/signaling/status': { devAvailable: true, devUrl: 'http://localhost:1999' },
			'/api/roster': [],
			'/party/': { peers: [] }
		});
		vi.stubGlobal('fetch', mockFn);

		await rosterService.initialize('api');

		const state = get(rosterService.state);
		expect(state.signalingServerUrl).toBe('http://localhost:1999');
	});
});

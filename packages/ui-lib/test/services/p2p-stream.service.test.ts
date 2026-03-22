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

describe('P2pStreamService', () => {
	let p2pStreamService: (typeof import('../../src/services/p2p-stream.service'))['p2pStreamService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/p2p-stream.service');
		p2pStreamService = mod.p2pStreamService;
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	// ===== Initial state =====

	it('should have correct initial settings', () => {
		const settings = p2pStreamService.get();
		expect(settings.id).toBe('p2p-stream-settings');
		expect(settings.stunServer).toBe('stun:stun.l.google.com:19302');
		expect(settings.turnServers).toEqual([]);
		expect(settings.videoCodec).toBe('vp8');
		expect(settings.audioCodec).toBe('opus');
		expect(settings.defaultStreamMode).toBe('video');
		expect(settings.videoQuality).toBe('native');
	});

	it('should have correct initial state', () => {
		const state = get(p2pStreamService.state);
		expect(state.initialized).toBe(false);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.serverAvailable).toBe(false);
	});

	// ===== initialize =====

	it('should initialize with settings and status', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/api/p2p-stream/settings')) {
					return {
						ok: true,
						json: async () => ({
							stunServer: 'stun:custom.server:3478',
							turnServers: ['turn:example.com'],
							videoCodec: 'vp9',
							audioCodec: 'opus',
							defaultStreamMode: 'audio',
							videoQuality: '720p'
						})
					} as Response;
				}
				if (url.includes('/api/player/stream-status')) {
					return {
						ok: true,
						json: async () => ({ available: true })
					} as Response;
				}
				return { ok: false, status: 404 } as Response;
			})
		);

		await p2pStreamService.initialize();

		const settings = p2pStreamService.get();
		expect(settings.stunServer).toBe('stun:custom.server:3478');
		expect(settings.turnServers).toEqual(['turn:example.com']);
		expect(settings.videoCodec).toBe('vp9');
		expect(settings.videoQuality).toBe('720p');
		expect(settings.defaultStreamMode).toBe('audio');

		const state = get(p2pStreamService.state);
		expect(state.initialized).toBe(true);
		expect(state.serverAvailable).toBe(true);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
	});

	it('should set loading true during initialization', async () => {
		let capturedLoading = false;
		const unsub = p2pStreamService.state.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/api/p2p-stream/settings')) {
					return {
						ok: true,
						json: async () => ({
							stunServer: '',
							turnServers: [],
							videoCodec: 'vp8',
							audioCodec: 'opus',
							defaultStreamMode: 'video',
							videoQuality: 'native'
						})
					} as Response;
				}
				return { ok: true, json: async () => ({ available: false }) } as Response;
			})
		);

		await p2pStreamService.initialize();
		unsub();
		expect(capturedLoading).toBe(true);
	});

	it('should not initialize twice', async () => {
		const mockFn = vi.fn(async (url: string) => {
			if (url.includes('/api/p2p-stream/settings')) {
				return {
					ok: true,
					json: async () => ({
						stunServer: '',
						turnServers: [],
						videoCodec: 'vp8',
						audioCodec: 'opus',
						defaultStreamMode: 'video',
						videoQuality: 'native'
					})
				} as Response;
			}
			return { ok: true, json: async () => ({ available: false }) } as Response;
		});
		vi.stubGlobal('fetch', mockFn);

		await p2pStreamService.initialize();
		const callCount = mockFn.mock.calls.length;

		await p2pStreamService.initialize();
		expect(mockFn.mock.calls.length).toBe(callCount);
	});

	it('should set error on initialization failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('Connection refused');
			})
		);

		await p2pStreamService.initialize();

		const state = get(p2pStreamService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toContain('Failed to load P2P stream settings');
		expect(state.error).toContain('Connection refused');
		expect(state.initialized).toBe(false);
	});

	// ===== updateSettings =====

	it('should update settings and call API', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await p2pStreamService.updateSettings({ videoCodec: 'h264' as never });

		const settings = p2pStreamService.get();
		expect(settings.videoCodec).toBe('h264');
		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/p2p-stream/settings'),
			expect.objectContaining({ method: 'PUT' })
		);
	});

	it('should revert settings on API failure', async () => {
		const originalCodec = p2pStreamService.get().videoCodec;

		vi.stubGlobal('fetch', mockFetchError('Server error'));

		await p2pStreamService.updateSettings({ videoCodec: 'h264' as never });

		const settings = p2pStreamService.get();
		expect(settings.videoCodec).toBe(originalCodec);

		const state = get(p2pStreamService.state);
		expect(state.error).toContain('Failed to save settings');
	});

	it('should strip id from payload when updating settings', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		await p2pStreamService.updateSettings({ videoCodec: 'vp9' as never });

		const call = mockFn.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.id).toBeUndefined();
		expect(body.videoCodec).toBe('vp9');
	});

	// ===== setStunServer =====

	it('should set STUN server via updateSettings', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.setStunServer('stun:custom.com:3478');

		// Wait for async updateSettings
		await vi.waitFor(() => {
			expect(p2pStreamService.get().stunServer).toBe('stun:custom.com:3478');
		});
	});

	// ===== addTurnServer =====

	it('should add a TURN server', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.addTurnServer('turn:new-server.com');

		await vi.waitFor(() => {
			expect(p2pStreamService.get().turnServers).toContain('turn:new-server.com');
		});
	});

	it('should not add duplicate TURN server', async () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: 'stun:stun.l.google.com:19302',
			turnServers: ['turn:existing.com'],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.addTurnServer('turn:existing.com');

		// fetch should not be called since it's a duplicate
		expect(mockFn).not.toHaveBeenCalled();
		expect(p2pStreamService.get().turnServers).toEqual(['turn:existing.com']);
	});

	// ===== removeTurnServer =====

	it('should remove a TURN server', async () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: 'stun:stun.l.google.com:19302',
			turnServers: ['turn:a.com', 'turn:b.com'],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.removeTurnServer('turn:a.com');

		await vi.waitFor(() => {
			expect(p2pStreamService.get().turnServers).toEqual(['turn:b.com']);
		});
	});

	// ===== setVideoCodec =====

	it('should set video codec', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.setVideoCodec('h264' as never);

		await vi.waitFor(() => {
			expect(p2pStreamService.get().videoCodec).toBe('h264');
		});
	});

	// ===== setDefaultStreamMode =====

	it('should set default stream mode', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.setDefaultStreamMode('audio');

		await vi.waitFor(() => {
			expect(p2pStreamService.get().defaultStreamMode).toBe('audio');
		});
	});

	// ===== setVideoQuality =====

	it('should set video quality', async () => {
		const mockFn = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFn);

		p2pStreamService.setVideoQuality('720p' as never);

		await vi.waitFor(() => {
			expect(p2pStreamService.get().videoQuality).toBe('720p');
		});
	});

	// ===== getSessionConfig =====

	it('should return session config', () => {
		const config = p2pStreamService.getSessionConfig();
		expect(config).toEqual({
			video_codec: 'vp8',
			video_quality: 'native'
		});
	});

	it('should return updated session config after settings change', () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: 'stun:stun.l.google.com:19302',
			turnServers: [],
			videoCodec: 'h264' as never,
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: '1080p' as never
		});

		const config = p2pStreamService.getSessionConfig();
		expect(config).toEqual({
			video_codec: 'h264',
			video_quality: '1080p'
		});
	});

	// ===== checkHealth =====

	it('should check health and return true when available', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ available: true }));

		const result = await p2pStreamService.checkHealth();
		expect(result).toBe(true);

		const state = get(p2pStreamService.state);
		expect(state.serverAvailable).toBe(true);
	});

	it('should check health and return false when unavailable', async () => {
		vi.stubGlobal('fetch', mockFetchOk({ available: false }));

		const result = await p2pStreamService.checkHealth();
		expect(result).toBe(false);

		const state = get(p2pStreamService.state);
		expect(state.serverAvailable).toBe(false);
	});

	it('should return false on health check failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('unreachable');
			})
		);

		const result = await p2pStreamService.checkHealth();
		expect(result).toBe(false);

		const state = get(p2pStreamService.state);
		expect(state.serverAvailable).toBe(false);
	});

	// ===== getIceServers =====

	it('should return ICE servers from settings', () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: 'stun:example.com:3478',
			turnServers: ['turn:turn1.com', 'turn:turn2.com'],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const iceServers = p2pStreamService.getIceServers();
		expect(iceServers).toHaveLength(3);
		expect(iceServers[0].urls).toBe('stun:example.com:3478');
		expect(iceServers[1].urls).toBe('turn:turn1.com');
		expect(iceServers[2].urls).toBe('turn:turn2.com');
	});

	it('should return default STUN when no servers configured', () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: '',
			turnServers: [],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const iceServers = p2pStreamService.getIceServers();
		expect(iceServers).toHaveLength(1);
		expect(iceServers[0].urls).toBe('stun:stun.l.google.com:19302');
	});

	it('should return only STUN server when no TURN servers', () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: 'stun:my-stun.com:3478',
			turnServers: [],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const iceServers = p2pStreamService.getIceServers();
		expect(iceServers).toHaveLength(1);
		expect(iceServers[0].urls).toBe('stun:my-stun.com:3478');
	});

	it('should return only TURN servers when STUN is empty', () => {
		p2pStreamService.store.set({
			id: 'p2p-stream-settings',
			stunServer: '',
			turnServers: ['turn:a.com'],
			videoCodec: 'vp8',
			audioCodec: 'opus',
			defaultStreamMode: 'video',
			videoQuality: 'native'
		});

		const iceServers = p2pStreamService.getIceServers();
		expect(iceServers).toHaveLength(1);
		expect(iceServers[0].urls).toBe('turn:a.com');
	});

	// ===== fetchJson error handling =====

	it('should handle non-Error initialization failure message', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw 'string error';
			})
		);

		await p2pStreamService.initialize();

		const state = get(p2pStreamService.state);
		expect(state.error).toContain('string error');
	});
});

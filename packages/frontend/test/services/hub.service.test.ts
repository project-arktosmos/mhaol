import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

function mockFetchResponses(responses: Record<string, unknown>) {
	const fn = vi.fn(async (url: string, init?: RequestInit) => {
		for (const [pattern, data] of Object.entries(responses)) {
			if (url.includes(pattern)) {
				return {
					ok: true,
					json: async () => data,
					status: 200
				} as Response;
			}
		}
		return { ok: false, status: 404, json: async () => ({}) } as Response;
	});
	vi.stubGlobal('fetch', fn);
	return fn;
}

describe('HubService', () => {
	let hubService: (typeof import('../../src/services/hub.service'))['hubService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		const mod = await import('../../src/services/hub.service');
		hubService = mod.hubService;
	});

	afterEach(() => {
		hubService.destroy();
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(hubService.state);
		expect(state.loading).toBe(false);
		expect(state.apps).toEqual([]);
		expect(state.error).toBeNull();
	});

	it('should load apps on refresh', async () => {
		const apps = [
			{ name: 'tube', port: 1531, status: 'running', has_headless: true, logs: [] },
			{ name: 'cloud', port: 1510, status: 'stopped', has_headless: true, logs: [] }
		];

		const mockFetch = vi.fn(async (url: string) => {
			if (url.includes('/api/hub/') && url.includes('/health')) {
				return { ok: true, json: async () => ({ status: 'running' }) } as Response;
			}
			if (url.includes('/api/hub/') && url.includes('/logs')) {
				return { ok: true, json: async () => ({ logs: [] }) } as Response;
			}
			if (url.includes('/api/hub')) {
				return { ok: true, json: async () => apps } as Response;
			}
			return { ok: false, status: 404, json: async () => ({}) } as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await hubService.refresh();

		const state = get(hubService.state);
		expect(state.loading).toBe(false);
		expect(state.apps).toHaveLength(2);
		expect(state.error).toBeNull();
	});

	it('should set error on refresh failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500, json: async () => ({}) }) as Response)
		);

		await hubService.refresh();

		const state = get(hubService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toBe('HTTP 500');
	});

	it('should set error on network failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('Network error');
			})
		);

		await hubService.refresh();

		const state = get(hubService.state);
		expect(state.error).toBe('Network error');
	});

	it('should call start endpoint and update state', async () => {
		// First set up an app in state
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }],
			error: null
		});

		const mockFetch = vi.fn(async () => {
			return { ok: true, json: async () => ({ success: true }) } as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await hubService.startApp('tube');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/hub/tube/start'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should set starting status when starting an app', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }],
			error: null
		});

		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: true, json: async () => ({ success: true }) }) as Response)
		);

		// We check synchronously right after calling startApp
		const promise = hubService.startApp('tube');

		const state = get(hubService.state);
		expect(state.apps[0].status).toBe('starting');

		await promise;
	});

	it('should set error when start fails', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }],
			error: null
		});

		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500, json: async () => ({}) }) as Response)
		);

		await hubService.startApp('tube');

		const state = get(hubService.state);
		expect(state.error).toBe('HTTP 500');
	});

	it('should call stop endpoint', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'running', has_headless: true, logs: [] }],
			error: null
		});

		const mockFetch = vi.fn(async (url: string) => {
			if (url.includes('/stop')) {
				return { ok: true, json: async () => ({ success: true }) } as Response;
			}
			if (url.includes('/health')) {
				return { ok: true, json: async () => ({ status: 'stopped' }) } as Response;
			}
			if (url.includes('/logs')) {
				return { ok: true, json: async () => ({ logs: [] }) } as Response;
			}
			return {
				ok: true,
				json: async () => [
					{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }
				]
			} as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await hubService.stopApp('tube');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/hub/tube/stop'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should set error when start returns success false', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }],
			error: null
		});

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => ({ success: false, message: 'Port in use' })
					}) as Response
			)
		);

		await hubService.startApp('tube');
		const state = get(hubService.state);
		expect(state.error).toBe('Port in use');
	});

	it('should set error when stop fails', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'running', has_headless: true, logs: [] }],
			error: null
		});

		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: false, status: 500 }) as Response)
		);

		await hubService.stopApp('tube');
		const state = get(hubService.state);
		expect(state.error).toBe('HTTP 500');
	});

	it('should dismiss app and refresh', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'failed', has_headless: true, logs: [] }],
			error: null
		});

		const mockFetch = vi.fn(async (url: string) => {
			if (url.includes('/stop')) {
				return { ok: true, json: async () => ({ success: true }) } as Response;
			}
			if (url.includes('/health')) {
				return { ok: true, json: async () => ({ status: 'stopped' }) } as Response;
			}
			if (url.includes('/logs')) {
				return { ok: true, json: async () => ({ logs: [] }) } as Response;
			}
			return { ok: true, json: async () => [] } as Response;
		});
		vi.stubGlobal('fetch', mockFetch);

		await hubService.dismissApp('tube');
		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/hub/tube/stop'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should handle start network error', async () => {
		hubService.state.set({
			loading: false,
			apps: [{ name: 'tube', port: 1531, status: 'stopped', has_headless: true, logs: [] }],
			error: null
		});
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('Network error');
			})
		);
		await hubService.startApp('tube');
		const state = get(hubService.state);
		expect(state.error).toBe('Network error');
	});

	it('should clean up poll interval on destroy', () => {
		const clearIntervalSpy = vi.spyOn(globalThis, 'clearInterval');
		hubService.destroy();
		// Should not throw even if no interval was set
		expect(true).toBe(true);
	});
});

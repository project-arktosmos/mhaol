import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock EventSource globally
class MockEventSource {
	url: string;
	onmessage: ((event: MessageEvent) => void) | null = null;
	onerror: (() => void) | null = null;
	listeners: Record<string, ((event: MessageEvent) => void)[]> = {};

	constructor(url: string) {
		this.url = url;
	}

	addEventListener(type: string, handler: (event: MessageEvent) => void) {
		if (!this.listeners[type]) this.listeners[type] = [];
		this.listeners[type].push(handler);
	}

	close() {}
}

vi.stubGlobal('EventSource', MockEventSource);

describe('TorrentService', () => {
	let torrentService: (typeof import('../../src/services/torrent.service'))['torrentService'];

	beforeEach(async () => {
		vi.resetModules();
		vi.stubGlobal('fetch', vi.fn());
		vi.stubGlobal('EventSource', MockEventSource);
		const mod = await import('../../src/services/torrent.service');
		torrentService = mod.torrentService;
	});

	afterEach(() => {
		torrentService.destroy();
		vi.unstubAllGlobals();
	});

	it('should have correct initial state', () => {
		const state = get(torrentService.state);
		expect(state.initialized).toBe(false);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.torrents).toEqual([]);
		expect(state.stats).toBeNull();
		expect(state.downloadPath).toBe('');
	});

	it('should initialize with status and config', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/status')) {
					return {
						ok: true,
						json: async () => ({ initialized: true, downloadPath: '/downloads', stats: null })
					} as Response;
				}
				if (url.includes('/config')) {
					return {
						ok: true,
						json: async () => ({ downloadPath: '/downloads' })
					} as Response;
				}
				return { ok: false, status: 404 } as Response;
			})
		);

		await torrentService.initialize();

		const state = get(torrentService.state);
		expect(state.initialized).toBe(true);
		expect(state.loading).toBe(false);
		expect(state.downloadPath).toBe('/downloads');
	});

	it('should set error on initialization failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new Error('Connection refused');
			})
		);

		await torrentService.initialize();

		const state = get(torrentService.state);
		expect(state.loading).toBe(false);
		expect(state.error).toContain('Failed to connect to torrent server');
	});

	it('should add a torrent with placeholder then replace', async () => {
		const torrentInfo = {
			infoHash: 'abc123',
			name: 'Test Torrent',
			size: 1024,
			progress: 0,
			downloadSpeed: 0,
			uploadSpeed: 0,
			peers: 0,
			seeds: 0,
			state: 'initializing',
			addedAt: 1000,
			eta: null,
			outputPath: null
		};

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => torrentInfo
					}) as Response
			)
		);

		const result = await torrentService.addTorrent('magnet:?xt=test');

		expect(result).not.toBeNull();
		expect(result!.infoHash).toBe('abc123');

		const state = get(torrentService.state);
		expect(state.torrents.some((t) => t.infoHash === 'abc123')).toBe(true);
		expect(state.torrents.some((t) => t.infoHash.startsWith('pending-'))).toBe(false);
	});

	it('should remove placeholder on add failure', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: false,
						status: 500
					}) as Response
			)
		);

		const result = await torrentService.addTorrent('magnet:?xt=test');

		expect(result).toBeNull();

		const state = get(torrentService.state);
		expect(state.torrents).toHaveLength(0);
		expect(state.error).toContain('Failed to add torrent');
	});

	it('should pause a torrent', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await torrentService.pauseTorrent('abc123');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/torrent/torrents/abc123/pause'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should resume a torrent', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await torrentService.resumeTorrent('abc123');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/torrent/torrents/abc123/resume'),
			expect.objectContaining({ method: 'POST' })
		);
	});

	it('should remove a torrent', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await torrentService.removeTorrent('abc123');

		expect(mockFetch).toHaveBeenCalledWith(
			expect.stringContaining('/api/torrent/torrents/abc123'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	it('should search torrents', async () => {
		const searchResults = [{ name: 'Test', seeders: 100 }];

		vi.stubGlobal(
			'fetch',
			vi.fn(
				async () =>
					({
						ok: true,
						json: async () => searchResults
					}) as Response
			)
		);

		const result = await torrentService.search('test query', 'video');

		expect(result.results).toEqual(searchResults);
	});

	it('should set download path', async () => {
		const mockFetch = vi.fn(
			async () =>
				({
					ok: true,
					json: async () => ({})
				}) as Response
		);
		vi.stubGlobal('fetch', mockFetch);

		await torrentService.setDownloadPath('/new/path');

		const state = get(torrentService.state);
		expect(state.downloadPath).toBe('/new/path');
	});

	it('should clean up on destroy', () => {
		torrentService.destroy();

		const state = get(torrentService.state);
		expect(state.initialized).toBe(false);
	});
});

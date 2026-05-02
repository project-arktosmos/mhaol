import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw, subscribeSSE } from '$transport/fetch-helpers';
import type { TransportEventSource } from '$transport/transport.type';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type { TorrentSettings, TorrentServiceState, TorrentInfo } from '$types/torrent.type';

const initialSettings: TorrentSettings = {
	id: 'torrent-settings',
	downloadPath: ''
};

const initialState: TorrentServiceState = {
	initialized: false,
	loading: false,
	error: null,
	torrents: [],
	allTorrents: [],
	stats: null,
	downloadPath: '',
	appName: '',
	appDownloadPath: '',
	libraryId: ''
};

class TorrentService extends ObjectServiceClass<TorrentSettings> {
	public state: Writable<TorrentServiceState> = writable(initialState);

	private _initialized = false;
	private appName = '';
	private appDownloadPath = '';
	private eventSource: TransportEventSource | null = null;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private reconnectDelay = 1000;
	private allTorrents: TorrentInfo[] = [];

	constructor() {
		super('torrent-settings', initialSettings);
	}

	// ===== Initialization =====

	async initialize(appName?: string): Promise<void> {
		if (!browser || this._initialized) return;

		this.appName = appName ?? '';
		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const [statusRes, configRes] = await Promise.all([
				fetchRaw('/api/torrent/status'),
				fetchRaw('/api/torrent/config')
			]);

			if (!statusRes.ok || !configRes.ok) {
				throw new Error('Failed to fetch torrent status');
			}

			const status = await statusRes.json();
			const config = await configRes.json();

			const downloadPath = config.downloadPath ?? status.downloadPath ?? '';
			this.appDownloadPath = this.appName ? `${downloadPath}/${this.appName}` : downloadPath;

			this.state.update((s) => ({
				...s,
				initialized: status.initialized,
				loading: false,
				downloadPath,
				appName: this.appName,
				appDownloadPath: this.appDownloadPath,
				stats: status.stats ?? null,
				error: null
			}));

			this._initialized = true;
			this.connectEvents();
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to connect to torrent server: ${errorMsg}`
			}));
		}
	}

	// ===== SSE Events =====

	private connectEvents(): void {
		if (this.eventSource) {
			this.eventSource.close();
		}

		this.eventSource = subscribeSSE('/api/torrent/torrents/events');

		this.eventSource.addEventListener('torrents', (data: string) => {
			try {
				const allTorrents: TorrentInfo[] = JSON.parse(data);
				this.allTorrents = allTorrents;
				const torrents = this.appName
					? allTorrents.filter((t) => t.outputPath && t.outputPath.startsWith(this.appDownloadPath))
					: allTorrents;
				this.state.update((s) => ({ ...s, torrents, allTorrents }));
			} catch {
				// ignore parse errors
			}
		});

		this.reconnectDelay = 1000;
	}

	private scheduleReconnect(): void {
		if (this.reconnectTimer) return;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connectEvents();
			this.reconnectDelay = Math.min(this.reconnectDelay * 2, 10000);
		}, this.reconnectDelay);
	}

	/** Find a torrent by hash across ALL torrents (ignores app-name filtering). Case-insensitive. */
	findByHash(infoHash: string): TorrentInfo | undefined {
		const target = infoHash.toLowerCase();
		return this.allTorrents.find((t) => t.infoHash.toLowerCase() === target);
	}

	// ===== Torrent Operations =====

	async addTorrent(source: string, downloadPath?: string): Promise<TorrentInfo | null> {
		if (!browser) return null;

		const placeholderId = `pending-${Date.now()}`;
		const placeholder: TorrentInfo = {
			infoHash: placeholderId,
			name: source.length > 60 ? source.slice(0, 60) + '...' : source,
			size: 0,
			progress: 0,
			downloadSpeed: 0,
			uploadSpeed: 0,
			peers: 0,
			seeds: 0,
			state: 'initializing',
			addedAt: Math.floor(Date.now() / 1000),
			eta: null,
			outputPath: null
		};

		this.state.update((s) => ({
			...s,
			torrents: [placeholder, ...s.torrents]
		}));

		try {
			const body: Record<string, unknown> = { source };
			const effectivePath = downloadPath || this.appDownloadPath;
			if (effectivePath) body.downloadPath = effectivePath;

			const res = await fetchRaw('/api/torrent/torrents', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			});

			if (!res.ok) {
				throw new Error(`HTTP ${res.status}`);
			}

			const torrent: TorrentInfo = await res.json();

			this.state.update((s) => ({
				...s,
				torrents: s.torrents.map((t) => (t.infoHash === placeholderId ? torrent : t))
			}));

			return torrent;
		} catch (error) {
			this.state.update((s) => ({
				...s,
				torrents: s.torrents.filter((t) => t.infoHash !== placeholderId),
				error: `Failed to add torrent: ${error instanceof Error ? error.message : String(error)}`
			}));
			return null;
		}
	}

	async pauseTorrent(infoHash: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw(`/api/torrent/torrents/${infoHash}/pause`, {
				method: 'POST'
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to pause torrent: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	async resumeTorrent(infoHash: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw(`/api/torrent/torrents/${infoHash}/resume`, {
				method: 'POST'
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to resume torrent: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	async removeTorrent(infoHash: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw(`/api/torrent/torrents/${infoHash}`, {
				method: 'DELETE'
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to remove torrent: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	async removeAll(): Promise<void> {
		if (!browser) return;
		try {
			if (this.appName) {
				// Remove only this app's torrents
				let currentTorrents: TorrentInfo[] = [];
				this.state.subscribe((s) => (currentTorrents = s.torrents))();
				await Promise.all(currentTorrents.map((t) => this.removeTorrent(t.infoHash)));
			} else {
				const res = await fetchRaw('/api/torrent/torrents/remove-all', {
					method: 'POST'
				});
				if (!res.ok) throw new Error(`HTTP ${res.status}`);
			}
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to remove all torrents: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	// ===== Config =====

	async setLibrary(_libraryId: string): Promise<void> {
		// No-op in standalone torrent app
	}

	async setDownloadPath(downloadPath: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw('/api/torrent/config', {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ downloadPath })
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			this.state.update((s) => ({ ...s, downloadPath }));
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to set download path: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	// ===== Search =====

	async search(
		query: string,
		category?: string
	): Promise<{ results: Array<Record<string, unknown>> }> {
		const params = new URLSearchParams({ q: query });
		if (category) params.set('cat', category);
		const res = await fetchRaw(`/api/torrent/search?${params}`);
		if (!res.ok) throw new Error(`HTTP ${res.status}`);
		const results = await res.json();
		return { results };
	}

	// ===== Debug & Storage =====

	async getDebugInfo(): Promise<string[]> {
		if (!browser) return [];
		try {
			const res = await fetchRaw('/api/torrent/debug');
			if (!res.ok) return [];
			const data = await res.json();
			return data.debug ?? [];
		} catch {
			return [];
		}
	}

	async clearStorage(): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw('/api/torrent/storage/clear', {
				method: 'POST'
			});
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to clear storage: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	// ===== Settings =====

	updateSettings(updates: Partial<TorrentSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== Lifecycle =====

	destroy(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
		this._initialized = false;
	}
}

export const torrentService = new TorrentService();

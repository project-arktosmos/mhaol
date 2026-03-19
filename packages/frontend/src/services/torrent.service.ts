import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from 'frontend/services/classes/object-service.class';
import type {
	TorrentSettings,
	TorrentServiceState,
	TorrentInfo,
	TorrentStats
} from 'frontend/types/torrent.type';

const initialSettings: TorrentSettings = {
	id: 'torrent-settings',
	downloadPath: ''
};

const initialState: TorrentServiceState = {
	initialized: false,
	loading: false,
	error: null,
	torrents: [],
	stats: null,
	downloadPath: '',
	libraryId: ''
};

type PendingRequest = {
	resolve: (value: unknown) => void;
	reject: (error: Error) => void;
	type: string;
};

class TorrentService extends ObjectServiceClass<TorrentSettings> {
	public state: Writable<TorrentServiceState> = writable(initialState);

	private ws: WebSocket | null = null;
	private _initialized = false;
	private pendingRequests: PendingRequest[] = [];
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private reconnectDelay = 1000;

	constructor() {
		super('torrent-settings', initialSettings);
	}

	// ===== WebSocket Connection =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			await this.connect();

			const [status, config] = await Promise.all([
				this.send<{ initialized: boolean; downloadPath: string; stats: TorrentStats | null }>(
					{ type: 'getStatus' },
					'status'
				),
				this.send<{ downloadPath: string }>({ type: 'getConfig' }, 'config')
			]);

			this.state.update((s) => ({
				...s,
				initialized: status.initialized,
				loading: false,
				downloadPath: status.downloadPath,
				stats: status.stats,
				error: null
			}));

			this._initialized = true;
			this.reconnectDelay = 1000;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to connect to torrent server: ${errorMsg}`
			}));
		}
	}

	private connect(): Promise<void> {
		return new Promise((resolve, reject) => {
			const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
			const url = `${protocol}//${window.location.host}/api/torrent/ws`;

			this.ws = new WebSocket(url);

			this.ws.onopen = () => resolve();

			this.ws.onerror = () => {
				if (!this._initialized) {
					reject(new Error('WebSocket connection failed'));
				}
			};

			this.ws.onmessage = (event) => {
				try {
					const msg = JSON.parse(event.data);
					this.handleMessage(msg);
				} catch {
					// ignore parse errors
				}
			};

			this.ws.onclose = () => {
				if (this._initialized) {
					this.scheduleReconnect();
				}
			};
		});
	}

	private handleMessage(msg: Record<string, unknown>): void {
		const type = msg.type as string;

		// Check if this resolves a pending request
		const pendingIdx = this.pendingRequests.findIndex((p) => p.type === type);
		if (pendingIdx !== -1) {
			const pending = this.pendingRequests[pendingIdx];
			this.pendingRequests.splice(pendingIdx, 1);
			pending.resolve(msg);
			return;
		}

		// Check for error responses
		if (type === 'error') {
			const errPending = this.pendingRequests.shift();
			if (errPending) {
				errPending.reject(new Error(msg.error as string));
				return;
			}
		}

		// Handle pushed updates
		switch (type) {
			case 'torrents':
				this.state.update((s) => ({
					...s,
					torrents: msg.torrents as TorrentInfo[]
				}));
				break;
			case 'stats':
				this.state.update((s) => ({
					...s,
					stats: msg.stats as TorrentStats
				}));
				break;
		}
	}

	private scheduleReconnect(): void {
		if (this.reconnectTimer) return;
		console.warn(`[Torrent] WS disconnected, reconnecting in ${this.reconnectDelay}ms...`);
		this.reconnectTimer = setTimeout(async () => {
			this.reconnectTimer = null;
			try {
				await this.connect();
				this.reconnectDelay = 1000;
				console.info('[Torrent] WS reconnected');
			} catch {
				this.reconnectDelay = Math.min(this.reconnectDelay * 2, 10000);
				this.scheduleReconnect();
			}
		}, this.reconnectDelay);
	}

	/** Send a message and await a specific response type */
	send<T>(msg: Record<string, unknown>, expectType: string): Promise<T> {
		return new Promise((resolve, reject) => {
			if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
				reject(new Error('WebSocket not connected'));
				return;
			}

			this.pendingRequests.push({
				resolve: resolve as (v: unknown) => void,
				reject,
				type: expectType
			});

			this.ws.send(JSON.stringify(msg));

			// Timeout after 30s
			setTimeout(() => {
				const idx = this.pendingRequests.findIndex(
					(p) => p.resolve === (resolve as (v: unknown) => void)
				);
				if (idx !== -1) {
					this.pendingRequests.splice(idx, 1);
					reject(new Error('Request timed out'));
				}
			}, 30000);
		});
	}

	// ===== Torrent Operations =====

	async addTorrent(source: string, downloadPath?: string): Promise<TorrentInfo | null> {
		if (!browser) return null;

		// Optimistic placeholder
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
			const result = await this.send<{ torrent: TorrentInfo }>(
				{ type: 'addTorrent', source, downloadPath },
				'torrentAdded'
			);

			this.state.update((s) => ({
				...s,
				torrents: s.torrents.map((t) => (t.infoHash === placeholderId ? result.torrent : t))
			}));

			return result.torrent;
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
			await this.send({ type: 'pauseTorrent', id: this.findTorrentId(infoHash) }, 'ok');
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
			await this.send({ type: 'resumeTorrent', id: this.findTorrentId(infoHash) }, 'ok');
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
			await this.send({ type: 'removeTorrent', id: this.findTorrentId(infoHash) }, 'ok');
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
			await this.send({ type: 'removeAll' }, 'removed');
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to remove all torrents: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	// ===== Config =====

	async setLibrary(libraryId: string): Promise<void> {
		// No-op in standalone torrent app (no library API)
	}

	async setDownloadPath(downloadPath: string): Promise<void> {
		if (!browser) return;
		try {
			await this.send({ type: 'setConfig', downloadPath }, 'config');
			this.state.update((s) => ({ ...s, downloadPath }));
		} catch (error) {
			this.state.update((s) => ({
				...s,
				error: `Failed to set download path: ${error instanceof Error ? error.message : String(error)}`
			}));
		}
	}

	// ===== Search (via WebSocket) =====

	async search(
		query: string,
		category?: string
	): Promise<{ results: Array<Record<string, unknown>> }> {
		return this.send({ type: 'search', query, category }, 'searchResults');
	}

	// ===== Debug & Storage =====

	async getDebugInfo(): Promise<string[]> {
		if (!browser) return [];
		try {
			const result = await this.send<{ logs: string[] }>({ type: 'getDebug' }, 'debug');
			return result.logs;
		} catch (error) {
			console.error('[Torrent] Failed to get debug info:', error);
			return [];
		}
	}

	async clearStorage(): Promise<void> {
		if (!browser) return;
		try {
			await this.send({ type: 'clearStorage' }, 'ok');
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

	// ===== Helpers =====

	private findTorrentId(infoHash: string): number {
		let id = -1;
		this.state.subscribe((s) => {
			const t = s.torrents.find((t) => t.infoHash === infoHash);
			if (t) id = (t as TorrentInfo & { id?: number }).id ?? -1;
		})();
		return id;
	}

	// ===== Lifecycle =====

	destroy(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		this._initialized = false;
		this.pendingRequests = [];
	}
}

export const torrentService = new TorrentService();

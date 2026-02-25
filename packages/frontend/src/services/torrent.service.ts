import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type {
	TorrentSettings,
	TorrentServiceState,
	TorrentInfo,
	TorrentStats,
	TorrentStatusResponse
} from '$types/torrent.type';

// Default settings stored in localStorage
const initialSettings: TorrentSettings = {
	id: 'torrent-settings',
	downloadPath: ''
};

// Initial service state
const initialState: TorrentServiceState = {
	initialized: false,
	loading: false,
	error: null,
	torrents: [],
	stats: null,
	downloadPath: '',
	libraryId: ''
};

class TorrentService extends ObjectServiceClass<TorrentSettings> {
	public state: Writable<TorrentServiceState> = writable(initialState);

	private eventSource: EventSource | null = null;
	private _initialized = false;

	constructor() {
		super('torrent-settings', initialSettings);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const [status, config] = await Promise.all([
				this.fetchJson<TorrentStatusResponse>('/api/torrent/status'),
				this.fetchJson<{ download_path: string; library_id: string }>('/api/torrent/config')
			]);

			this.state.update((s) => ({
				...s,
				initialized: status.initialized,
				loading: false,
				downloadPath: status.download_path,
				libraryId: config.library_id || '',
				stats: status.stats,
				error: null
			}));

			this._initialized = true;

			// Connect SSE for real-time updates
			this.connectSSE();
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to connect to torrent server: ${errorMsg}`
			}));
		}
	}

	// ===== Torrent Operations =====

	async addTorrent(source: string, downloadPath?: string): Promise<TorrentInfo | null> {
		if (!browser) return null;

		try {
			const info = await this.fetchJson<TorrentInfo>('/api/torrent/torrents', {
				method: 'POST',
				body: JSON.stringify({ source, downloadPath })
			});
			return info;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to add torrent: ${errorMsg}`
			}));
			return null;
		}
	}

	async listTorrents(): Promise<TorrentInfo[]> {
		if (!browser) return [];

		try {
			return await this.fetchJson<TorrentInfo[]>('/api/torrent/torrents');
		} catch (error) {
			console.error('[Torrent] Failed to list torrents:', error);
			return [];
		}
	}

	async pauseTorrent(infoHash: string): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson(`/api/torrent/torrents/${infoHash}/pause`, { method: 'POST' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to pause torrent: ${errorMsg}`
			}));
		}
	}

	async resumeTorrent(infoHash: string): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson(`/api/torrent/torrents/${infoHash}/resume`, { method: 'POST' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to resume torrent: ${errorMsg}`
			}));
		}
	}

	async removeTorrent(infoHash: string): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson(`/api/torrent/torrents/${infoHash}`, { method: 'DELETE' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to remove torrent: ${errorMsg}`
			}));
		}
	}

	async removeAll(): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson('/api/torrent/torrents/remove-all', { method: 'POST' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to remove all torrents: ${errorMsg}`
			}));
		}
	}

	// ===== Config =====

	async setLibrary(libraryId: string): Promise<void> {
		if (!browser) return;

		try {
			const result = await this.fetchJson<{ download_path: string }>('/api/torrent/config', {
				method: 'PUT',
				body: JSON.stringify({ library_id: libraryId })
			});

			this.state.update((s) => ({ ...s, downloadPath: result.download_path, libraryId }));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to set library: ${errorMsg}`
			}));
		}
	}

	// ===== Debug & Storage =====

	async getDebugInfo(): Promise<string[]> {
		if (!browser) return [];

		try {
			const result = await this.fetchJson<{ logs: string[] }>('/api/torrent/debug');
			return result.logs;
		} catch (error) {
			console.error('[Torrent] Failed to get debug info:', error);
			return [];
		}
	}

	async clearStorage(): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson('/api/torrent/storage/clear', { method: 'POST' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to clear storage: ${errorMsg}`
			}));
		}
	}

	// ===== Settings =====

	updateSettings(updates: Partial<TorrentSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== SSE Connection =====

	private connectSSE(): void {
		if (!browser) return;

		this.eventSource = new EventSource('/api/torrent/torrents/events');

		this.eventSource.addEventListener('torrents', (e: MessageEvent) => {
			try {
				const torrents = JSON.parse(e.data) as TorrentInfo[];
				this.state.update((s) => ({ ...s, torrents }));
			} catch {
				// ignore parse errors
			}
		});

		this.eventSource.addEventListener('stats', (e: MessageEvent) => {
			try {
				const stats = JSON.parse(e.data) as TorrentStats;
				this.state.update((s) => ({ ...s, stats }));
			} catch {
				// ignore parse errors
			}
		});

		this.eventSource.onerror = () => {
			console.warn('[Torrent] SSE connection error, reconnecting...');
		};
	}

	// ===== HTTP Helper =====

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(path, {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}

	// ===== Lifecycle =====

	destroy(): void {
		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
	}
}

export const torrentService = new TorrentService();

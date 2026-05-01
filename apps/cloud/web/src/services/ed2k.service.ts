import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw, subscribeSSE } from '$transport/fetch-helpers';
import type { TransportEventSource } from '$transport/transport.type';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type {
	Ed2kFileInfo,
	Ed2kSearchResult,
	Ed2kServer,
	Ed2kServiceState,
	Ed2kSettings,
	Ed2kStats
} from '$types/ed2k.type';

const initialSettings: Ed2kSettings = {
	id: 'ed2k-settings',
	downloadPath: ''
};

const initialState: Ed2kServiceState = {
	initialized: false,
	loading: false,
	error: null,
	files: [],
	stats: null,
	server: null,
	downloadPath: '',
	searchQuery: '',
	searching: false,
	searchResults: []
};

class Ed2kService extends ObjectServiceClass<Ed2kSettings> {
	public state: Writable<Ed2kServiceState> = writable(initialState);

	private _initialized = false;
	private eventSource: TransportEventSource | null = null;

	constructor() {
		super('ed2k-settings', initialSettings);
	}

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const res = await fetchRaw('/api/ed2k/status');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const status = (await res.json()) as {
				initialized: boolean;
				downloadPath: string;
				stats: Ed2kStats | null;
				server: Ed2kServer | null;
			};

			this.state.update((s) => ({
				...s,
				initialized: status.initialized,
				loading: false,
				downloadPath: status.downloadPath ?? '',
				stats: status.stats,
				server: status.server,
				error: null
			}));

			this._initialized = true;
			this.connectEvents();

			if (status.initialized && !status.server) {
				void this.connectServer();
			}
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to connect to ed2k server: ${errorMsg}`
			}));
		}
	}

	private connectEvents(): void {
		if (this.eventSource) this.eventSource.close();
		this.eventSource = subscribeSSE('/api/ed2k/files/events');
		this.eventSource.addEventListener('files', (data: string) => {
			try {
				const files: Ed2kFileInfo[] = JSON.parse(data);
				this.state.update((s) => ({ ...s, files }));
			} catch {
				// ignore parse errors
			}
		});
	}

	async refreshStatus(): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw('/api/ed2k/status');
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const status = (await res.json()) as {
				initialized: boolean;
				downloadPath: string;
				stats: Ed2kStats | null;
				server: Ed2kServer | null;
			};
			this.state.update((s) => ({
				...s,
				initialized: status.initialized,
				downloadPath: status.downloadPath ?? '',
				stats: status.stats,
				server: status.server
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to refresh status: ${errorMsg}` }));
		}
	}

	async connectServer(): Promise<Ed2kServer | null> {
		if (!browser) return null;
		this.state.update((s) => ({ ...s, error: null }));
		try {
			const res = await fetchRaw('/api/ed2k/server/connect', { method: 'POST' });
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				throw new Error(data.error ?? `HTTP ${res.status}`);
			}
			const data = (await res.json()) as { server: Ed2kServer };
			this.state.update((s) => ({ ...s, server: data.server }));
			return data.server;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to connect: ${errorMsg}` }));
			return null;
		}
	}

	async search(query: string): Promise<Ed2kSearchResult[]> {
		if (!browser) return [];
		const trimmed = query.trim();
		if (!trimmed) return [];
		this.state.update((s) => ({
			...s,
			searching: true,
			searchQuery: trimmed,
			error: null
		}));
		try {
			const res = await fetchRaw(`/api/ed2k/search?q=${encodeURIComponent(trimmed)}`);
			const data = await res.json();
			if (!res.ok) {
				const errorMsg = (data && data.error) || `HTTP ${res.status}`;
				const results: Ed2kSearchResult[] = Array.isArray(data?.results) ? data.results : [];
				this.state.update((s) => ({
					...s,
					searching: false,
					searchResults: results,
					error: `Search warning: ${errorMsg}`
				}));
				return results;
			}
			const results: Ed2kSearchResult[] = Array.isArray(data) ? data : [];
			this.state.update((s) => ({ ...s, searching: false, searchResults: results }));
			return results;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				searching: false,
				error: `Search failed: ${errorMsg}`
			}));
			return [];
		}
	}

	clearSearch(): void {
		this.state.update((s) => ({ ...s, searchResults: [], searchQuery: '' }));
	}

	async addFile(source: string, downloadPath?: string): Promise<Ed2kFileInfo | null> {
		if (!browser) return null;
		try {
			const body: Record<string, unknown> = { source };
			if (downloadPath) body.downloadPath = downloadPath;
			const res = await fetchRaw('/api/ed2k/files', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			});
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				throw new Error(data.error ?? `HTTP ${res.status}`);
			}
			const file: Ed2kFileInfo = await res.json();
			this.state.update((s) => {
				const without = s.files.filter((f) => f.fileHash !== file.fileHash);
				return { ...s, files: [file, ...without] };
			});
			return file;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to add file: ${errorMsg}` }));
			return null;
		}
	}

	async pauseFile(fileHash: string): Promise<void> {
		await this.simpleAction(`/api/ed2k/files/${fileHash}/pause`, 'pause');
	}

	async resumeFile(fileHash: string): Promise<void> {
		await this.simpleAction(`/api/ed2k/files/${fileHash}/resume`, 'resume');
	}

	async removeFile(fileHash: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw(`/api/ed2k/files/${fileHash}`, { method: 'DELETE' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			this.state.update((s) => ({
				...s,
				files: s.files.filter((f) => f.fileHash !== fileHash)
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to remove: ${errorMsg}` }));
		}
	}

	async removeAll(): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw('/api/ed2k/files/remove-all', { method: 'POST' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			this.state.update((s) => ({ ...s, files: [] }));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to remove all: ${errorMsg}` }));
		}
	}

	async getDebugInfo(): Promise<string[]> {
		if (!browser) return [];
		try {
			const res = await fetchRaw('/api/ed2k/debug');
			if (!res.ok) return [];
			const data = await res.json();
			return data.debug ?? [];
		} catch {
			return [];
		}
	}

	private async simpleAction(path: string, label: string): Promise<void> {
		if (!browser) return;
		try {
			const res = await fetchRaw(path, { method: 'POST' });
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to ${label}: ${errorMsg}` }));
		}
	}

	dismissError(): void {
		this.state.update((s) => ({ ...s, error: null }));
	}

	destroy(): void {
		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
		this._initialized = false;
	}
}

export const ed2kService = new Ed2kService();

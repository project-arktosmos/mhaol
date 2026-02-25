import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import type {
	P2pStreamSettings,
	P2pStreamServiceState,
	P2pVideoCodec,
	P2pStreamMode
} from '$types/p2p-stream.type';

const initialSettings: P2pStreamSettings = {
	id: 'p2p-stream-settings',
	stunServer: 'stun:stun.l.google.com:19302',
	turnServers: [],
	videoCodec: 'vp8',
	audioCodec: 'opus',
	defaultStreamMode: 'video'
};

const initialState: P2pStreamServiceState = {
	initialized: false,
	loading: false,
	error: null,
	serverAvailable: false
};

class P2pStreamService {
	public store: Writable<P2pStreamSettings> = writable(initialSettings);
	public state: Writable<P2pStreamServiceState> = writable(initialState);

	private _initialized = false;

	get(): P2pStreamSettings {
		return get(this.store);
	}

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const [settings, status] = await Promise.all([
				this.fetchJson<Omit<P2pStreamSettings, 'id'>>('/api/p2p-stream/settings'),
				this.fetchJson<{ available: boolean }>('/api/player/stream-status')
			]);

			this.store.set({ ...settings, id: 'p2p-stream-settings' });
			this.state.update((s) => ({
				...s,
				initialized: true,
				loading: false,
				serverAvailable: status.available,
				error: null
			}));
			this._initialized = true;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to load P2P stream settings: ${errorMsg}`
			}));
		}
	}

	async updateSettings(updates: Partial<P2pStreamSettings>): Promise<void> {
		if (!browser) return;
		const current = this.get();
		const merged = { ...current, ...updates };
		this.store.set(merged);

		const { id: _id, ...payload } = updates as Partial<P2pStreamSettings> & { id?: unknown };

		try {
			await this.fetchJson('/api/p2p-stream/settings', {
				method: 'PUT',
				body: JSON.stringify(payload)
			});
		} catch (error) {
			this.store.set(current);
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({ ...s, error: `Failed to save settings: ${errorMsg}` }));
		}
	}

	setStunServer(stunServer: string): void {
		this.updateSettings({ stunServer });
	}

	addTurnServer(url: string): void {
		const current = this.get();
		if (!current.turnServers.includes(url)) {
			this.updateSettings({ turnServers: [...current.turnServers, url] });
		}
	}

	removeTurnServer(url: string): void {
		const current = this.get();
		this.updateSettings({ turnServers: current.turnServers.filter((t) => t !== url) });
	}

	setVideoCodec(codec: P2pVideoCodec): void {
		this.updateSettings({ videoCodec: codec });
	}

	setDefaultStreamMode(mode: P2pStreamMode): void {
		this.updateSettings({ defaultStreamMode: mode });
	}

	async checkHealth(): Promise<boolean> {
		try {
			const status = await this.fetchJson<{ available: boolean }>('/api/player/stream-status');
			this.state.update((s) => ({ ...s, serverAvailable: status.available }));
			return status.available;
		} catch {
			this.state.update((s) => ({ ...s, serverAvailable: false }));
			return false;
		}
	}

	getIceServers(): RTCIceServer[] {
		const settings = this.get();
		const servers: RTCIceServer[] = [];
		if (settings.stunServer) {
			servers.push({ urls: settings.stunServer });
		}
		for (const turn of settings.turnServers) {
			servers.push({ urls: turn });
		}
		return servers.length > 0 ? servers : [{ urls: 'stun:stun.l.google.com:19302' }];
	}

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(path, {
			...init,
			headers: { 'Content-Type': 'application/json', ...init?.headers }
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}
}

export const p2pStreamService = new P2pStreamService();

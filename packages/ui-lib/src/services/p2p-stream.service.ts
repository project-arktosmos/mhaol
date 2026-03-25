import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type {
	P2pStreamSettings,
	P2pStreamServiceState,
	P2pVideoCodec,
	P2pVideoQuality,
	P2pStreamMode,
	TurnServerConfig
} from 'ui-lib/types/p2p-stream.type';

const initialSettings: P2pStreamSettings = {
	id: 'p2p-stream-settings',
	stunServer: 'stun:stun.l.google.com:19302',
	turnServers: [],
	videoCodec: 'vp8',
	audioCodec: 'opus',
	defaultStreamMode: 'video',
	videoQuality: 'native'
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
	private _mode: 'api' | 'local' = 'api';

	private static LOCAL_STORAGE_KEY = 'p2p-stream-settings';

	get(): P2pStreamSettings {
		return get(this.store);
	}

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._mode = 'api';
		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const [settings, status] = await Promise.all([
				fetchJson<Omit<P2pStreamSettings, 'id'>>('/api/p2p-stream/settings'),
				fetchJson<{ available: boolean }>('/api/player/stream-status')
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

	initializeLocal(): void {
		if (!browser || this._initialized) return;
		this._mode = 'local';
		this._initialized = true;

		try {
			const raw = localStorage.getItem(P2pStreamService.LOCAL_STORAGE_KEY);
			if (raw) {
				const stored = JSON.parse(raw) as Partial<P2pStreamSettings>;
				this.store.set({ ...initialSettings, ...stored, id: 'p2p-stream-settings' });
			}
		} catch {
			// ignore
		}

		this.state.update((s) => ({ ...s, initialized: true }));
	}

	private saveLocal(): void {
		if (this._mode !== 'local' || !browser) return;
		const { id: _, ...settings } = this.get();
		localStorage.setItem(P2pStreamService.LOCAL_STORAGE_KEY, JSON.stringify(settings));
	}

	async updateSettings(updates: Partial<P2pStreamSettings>): Promise<void> {
		if (!browser) return;
		const current = this.get();
		const merged = { ...current, ...updates };
		this.store.set(merged);

		if (this._mode === 'local') {
			this.saveLocal();
			return;
		}

		const { id: _id, ...payload } = updates as Partial<P2pStreamSettings> & { id?: unknown };

		try {
			await fetchJson('/api/p2p-stream/settings', {
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

	addTurnServer(config: TurnServerConfig): void {
		const current = this.get();
		if (!current.turnServers.some((t) => t.url === config.url)) {
			this.updateSettings({ turnServers: [...current.turnServers, config] });
		}
	}

	removeTurnServer(url: string): void {
		const current = this.get();
		this.updateSettings({ turnServers: current.turnServers.filter((t) => t.url !== url) });
	}

	setVideoCodec(codec: P2pVideoCodec): void {
		this.updateSettings({ videoCodec: codec });
	}

	setDefaultStreamMode(mode: P2pStreamMode): void {
		this.updateSettings({ defaultStreamMode: mode });
	}

	setVideoQuality(quality: P2pVideoQuality): void {
		this.updateSettings({ videoQuality: quality });
	}

	getSessionConfig(): { video_codec: string; video_quality: string } {
		const settings = this.get();
		return {
			video_codec: settings.videoCodec,
			video_quality: settings.videoQuality
		};
	}

	async checkHealth(): Promise<boolean> {
		try {
			const status = await fetchJson<{ available: boolean }>('/api/player/stream-status');
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
			const entry: RTCIceServer = { urls: turn.url };
			if (turn.username) entry.username = turn.username;
			if (turn.credential) entry.credential = turn.credential;
			servers.push(entry);
		}
		if (servers.length === 0) {
			servers.push({
				urls: [
					'stun:stun.l.google.com:19302',
					'stun:stun1.l.google.com:19302',
					'stun:stun2.l.google.com:19302'
				]
			});
		}
		return servers;
	}
}

export const p2pStreamService = new P2pStreamService();

import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl, DEFAULT_SIGNALING_URL } from 'frontend/lib/api-base';
import type { Identity } from 'frontend/types/identity.type';
import type { RosterEntry, RosterState } from 'frontend/types/roster.type';

const initialState: RosterState = {
	loading: false,
	entries: [],
	signalingServerUrl: DEFAULT_SIGNALING_URL,
	signalingRoomId: 'default',
	error: null
};

class RosterService {
	public state: Writable<RosterState> = writable(initialState);

	private _initialized = false;
	private _pollTimer: ReturnType<typeof setInterval> | null = null;

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;

		await this.fetchSignalingUrl();
		await this.refresh();
		this._pollTimer = setInterval(() => this.checkOnlineStatus(), 15_000);
	}

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch(apiUrl('/api/identities'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const identities: Identity[] = await res.json();

			const entries: RosterEntry[] = identities.map((id) => ({
				name: id.name,
				address: id.address,
				status: 'offline'
			}));

			this.state.update((s) => ({ ...s, loading: false, entries }));
			await this.checkOnlineStatus();
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load roster';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async checkOnlineStatus(): Promise<void> {
		let serverUrl = '';
		let roomId = '';
		let entries: RosterEntry[] = [];

		this.state.subscribe((s) => {
			serverUrl = s.signalingServerUrl;
			roomId = s.signalingRoomId;
			entries = s.entries;
		})();

		if (!serverUrl || entries.length === 0) return;

		this.state.update((s) => ({
			...s,
			entries: s.entries.map((e) => ({ ...e, status: 'checking' }))
		}));

		try {
			const url = new URL(serverUrl);
			const roomUrl = `${url.protocol}//${url.host}/party/${roomId}`;
			const res = await fetch(roomUrl);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);

			const data: { peers: string[] } = await res.json();
			const onlinePeers = new Set(data.peers.map((p) => p.toLowerCase()));

			this.state.update((s) => ({
				...s,
				entries: s.entries.map((e) => ({
					...e,
					status: onlinePeers.has(e.address.toLowerCase()) ? 'online' : 'offline'
				}))
			}));
		} catch {
			this.state.update((s) => ({
				...s,
				entries: s.entries.map((e) => ({ ...e, status: 'offline' }))
			}));
		}
	}

	private async fetchSignalingUrl(): Promise<void> {
		try {
			const res = await fetch(apiUrl('/api/signaling/status'));
			if (!res.ok) return;
			const status = await res.json();
			if (status.devAvailable && status.devUrl) {
				this.state.update((s) => ({ ...s, signalingServerUrl: status.devUrl }));
			}
		} catch {
			// Ignore
		}
	}

	destroy(): void {
		if (this._pollTimer) {
			clearInterval(this._pollTimer);
			this._pollTimer = null;
		}
		this._initialized = false;
	}
}

export const rosterService = new RosterService();

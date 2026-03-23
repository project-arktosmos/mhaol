import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl, getSignalingUrl } from 'ui-lib/lib/api-base';
import type { RosterEntry, RosterState, RosterStorageMode } from 'ui-lib/types/roster.type';

const LOCAL_STORAGE_KEY = 'roster-entries';

const initialState: RosterState = {
	loading: false,
	entries: [],
	signalingServerUrl: getSignalingUrl(),
	signalingRoomId: 'handshakes',
	error: null
};

interface StoredEntry {
	name: string;
	address: string;
	passport?: string;
	instanceType?: string;
	endorsement?: string;
}

class RosterService {
	public state: Writable<RosterState> = writable(initialState);

	private _initialized = false;
	private _mode: RosterStorageMode = 'api';
	private _pollTimer: ReturnType<typeof setInterval> | null = null;

	async initialize(mode: RosterStorageMode = 'api'): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		this._mode = mode;

		if (this._mode === 'api') {
			await this.fetchSignalingUrl();
		}
		await this.refresh();
		this._pollTimer = setInterval(() => this.checkOnlineStatus(), 15_000);
	}

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const entries = this._mode === 'api' ? await this.loadFromApi() : this.loadFromLocal();
			this.state.update((s) => ({ ...s, loading: false, entries }));
			await this.checkOnlineStatus();
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to load roster';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		}
	}

	async addEntry(entry: {
		name: string;
		address: string;
		passport?: string;
		instanceType?: string;
		endorsement?: string;
	}): Promise<void> {
		if (this._mode === 'api') {
			await fetch(apiUrl('/api/roster'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(entry)
			});
		} else {
			const stored = this.readLocal();
			const idx = stored.findIndex((e) => e.address.toLowerCase() === entry.address.toLowerCase());
			if (idx >= 0) {
				stored[idx] = { ...stored[idx], ...entry };
			} else {
				stored.push(entry);
			}
			this.writeLocal(stored);
		}
		await this.refresh();
	}

	async removeEntry(address: string): Promise<void> {
		if (this._mode === 'api') {
			await fetch(apiUrl(`/api/roster/${encodeURIComponent(address)}`), { method: 'DELETE' });
		} else {
			const stored = this.readLocal().filter(
				(e) => e.address.toLowerCase() !== address.toLowerCase()
			);
			this.writeLocal(stored);
		}
		await this.refresh();
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

			const data: { peers: { peer_id: string }[] } = await res.json();
			const onlinePeers = new Set(data.peers.map((p) => p.peer_id.toLowerCase()));

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

	private async loadFromApi(): Promise<RosterEntry[]> {
		const res = await fetch(apiUrl('/api/roster'));
		if (!res.ok) throw new Error(`HTTP ${res.status}`);
		const contacts: { name: string; address: string; passport?: string; instance_type?: string; endorsement?: string }[] =
			await res.json();
		return contacts.map((c) => ({
			name: c.name,
			address: c.address,
			status: 'offline',
			passport: c.passport ?? undefined,
			instanceType: c.instance_type ?? undefined,
			endorsement: c.endorsement ?? undefined
		}));
	}

	private loadFromLocal(): RosterEntry[] {
		const stored = this.readLocal();
		return stored.map((e) => ({
			name: e.name,
			address: e.address,
			status: 'offline',
			passport: e.passport,
			instanceType: e.instanceType,
			endorsement: e.endorsement
		}));
	}

	private readLocal(): StoredEntry[] {
		try {
			const raw = localStorage.getItem(LOCAL_STORAGE_KEY);
			if (raw) return JSON.parse(raw);
		} catch {
			// ignore
		}
		return [];
	}

	private writeLocal(entries: StoredEntry[]): void {
		localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(entries));
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

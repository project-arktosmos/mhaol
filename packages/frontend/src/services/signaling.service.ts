import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ArrayServiceClass } from '$services/classes/array-service.class';
import type {
	SignalingServer,
	SignalingState,
	ServerStatus,
	SignalingStatusResponse
} from '$types/signaling.type';

const DEFAULT_SERVER: SignalingServer = {
	id: 'local-default',
	name: 'Local',
	url: 'http://localhost:3002',
	addedAt: new Date().toISOString()
};

const initialState: SignalingState = {
	initialized: false,
	showAddForm: false,
	serverStatuses: {}
};

const defaultStatus = (): ServerStatus => ({
	online: false,
	totalPeers: 0,
	rooms: [],
	checking: false,
	lastChecked: null,
	error: null
});

class SignalingService extends ArrayServiceClass<SignalingServer> {
	public state: Writable<SignalingState> = writable(initialState);

	private pollInterval: ReturnType<typeof setInterval> | null = null;
	private _initialized = false;

	constructor() {
		super('signaling-servers', []);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		// Seed the default local server if the list is empty
		if (this.all().length === 0) {
			this.add(DEFAULT_SERVER);
		}

		this._initialized = true;
		this.state.update((s) => ({ ...s, initialized: true }));

		await this.refreshAllStatuses();

		// Poll every 30 seconds
		this.pollInterval = setInterval(() => {
			this.refreshAllStatuses();
		}, 30_000);
	}

	// ===== Server management =====

	async addServer(name: string, url: string): Promise<void> {
		const trimmedUrl = url.replace(/\/$/, '');
		const server: SignalingServer = {
			id: crypto.randomUUID(),
			name: name.trim(),
			url: trimmedUrl,
			addedAt: new Date().toISOString()
		};

		this.add(server);
		this.closeAddForm();
		await this.checkServerStatus(server);
	}

	removeServer(server: SignalingServer): void {
		this.remove(server);
		this.state.update((s) => {
			const { [String(server.id)]: _removed, ...rest } = s.serverStatuses;
			return { ...s, serverStatuses: rest };
		});
	}

	// ===== Status checking =====

	async refreshAllStatuses(): Promise<void> {
		const servers = this.all();
		await Promise.all(servers.map((s) => this.checkServerStatus(s)));
	}

	async checkServerStatus(server: SignalingServer): Promise<void> {
		const id = String(server.id);

		this.state.update((s) => ({
			...s,
			serverStatuses: {
				...s.serverStatuses,
				[id]: { ...(s.serverStatuses[id] ?? defaultStatus()), checking: true }
			}
		}));

		try {
			const encodedUrl = encodeURIComponent(server.url);
			const res = await fetch(`/api/signaling/status?url=${encodedUrl}`, {
				signal: AbortSignal.timeout(5000)
			});

			if (!res.ok) {
				const body = (await res.json().catch(() => ({}))) as { error?: string };
				this.setServerStatus(id, {
					online: false,
					totalPeers: 0,
					rooms: [],
					checking: false,
					lastChecked: new Date().toISOString(),
					error: body.error ?? `HTTP ${res.status}`
				});
				return;
			}

			const data = (await res.json()) as SignalingStatusResponse;
			this.setServerStatus(id, {
				online: true,
				totalPeers: data.totalPeers,
				rooms: data.rooms,
				checking: false,
				lastChecked: new Date().toISOString(),
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Connection failed';
			this.setServerStatus(id, {
				online: false,
				totalPeers: 0,
				rooms: [],
				checking: false,
				lastChecked: new Date().toISOString(),
				error: message
			});
		}
	}

	private setServerStatus(id: string, status: ServerStatus): void {
		this.state.update((s) => ({
			...s,
			serverStatuses: { ...s.serverStatuses, [id]: status }
		}));
	}

	// ===== UI helpers =====

	openAddForm(): void {
		this.state.update((s) => ({ ...s, showAddForm: true }));
	}

	closeAddForm(): void {
		this.state.update((s) => ({ ...s, showAddForm: false }));
	}

	getStatus(server: SignalingServer): ServerStatus {
		return get(this.state).serverStatuses[String(server.id)] ?? defaultStatus();
	}

	// ===== Lifecycle =====

	destroy(): void {
		if (this.pollInterval !== null) {
			clearInterval(this.pollInterval);
			this.pollInterval = null;
		}
	}
}

export const signalingService = new SignalingService();

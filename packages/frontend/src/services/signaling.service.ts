import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ArrayServiceClass } from '$services/classes/array-service.class';
import type {
	SignalingServer,
	SignalingState,
	ServerStatus,
	SignalingStatusResponse
} from '$types/signaling.type';

const LOBBY_ROOM = 'signaling-page-lobby';

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
	error: null,
	wsConnected: false,
	ownPeerId: null,
	lobbyPeers: []
});

class SignalingService extends ArrayServiceClass<SignalingServer> {
	public state: Writable<SignalingState> = writable(initialState);

	private pollInterval: ReturnType<typeof setInterval> | null = null;
	private connections: Map<string, WebSocket> = new Map();
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

		// Check HTTP status and open WebSocket for each server
		const servers = this.all();
		await Promise.all(servers.map((s) => this.checkServerStatus(s)));
		servers.forEach((s) => this.connectToServer(s));

		// Re-check HTTP status every 30 seconds
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
		this.connectToServer(server);
	}

	removeServer(server: SignalingServer): void {
		this.disconnectFromServer(String(server.id));
		this.remove(server);
		this.state.update((s) => {
			const { [String(server.id)]: _removed, ...rest } = s.serverStatuses;
			return { ...s, serverStatuses: rest };
		});
	}

	// ===== HTTP status checking =====

	async refreshAllStatuses(): Promise<void> {
		const servers = this.all();
		await Promise.all(servers.map((s) => this.checkServerStatus(s)));
	}

	async checkServerStatus(server: SignalingServer): Promise<void> {
		const id = String(server.id);

		this.patchServerStatus(id, { checking: true });

		try {
			const encodedUrl = encodeURIComponent(server.url);
			const res = await fetch(`/api/signaling/status?url=${encodedUrl}`, {
				signal: AbortSignal.timeout(5000)
			});

			if (!res.ok) {
				const body = (await res.json().catch(() => ({}))) as { error?: string };
				this.patchServerStatus(id, {
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
			this.patchServerStatus(id, {
				online: true,
				totalPeers: data.totalPeers,
				rooms: data.rooms,
				checking: false,
				lastChecked: new Date().toISOString(),
				error: null
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Connection failed';
			this.patchServerStatus(id, {
				online: false,
				totalPeers: 0,
				rooms: [],
				checking: false,
				lastChecked: new Date().toISOString(),
				error: message
			});
		}
	}

	// ===== WebSocket lobby connection =====

	private connectToServer(server: SignalingServer): void {
		const id = String(server.id);

		// Don't open a second connection if one already exists
		const existing = this.connections.get(id);
		if (existing && existing.readyState <= WebSocket.OPEN) return;

		const wsUrl = server.url.replace(/^http/, 'ws');
		let ws: WebSocket;
		try {
			ws = new WebSocket(wsUrl);
		} catch {
			return;
		}

		this.connections.set(id, ws);

		ws.onopen = () => {
			ws.send(JSON.stringify({ type: 'join-room', room_id: LOBBY_ROOM }));
		};

		ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string) as {
					type: string;
					peer_id?: string;
					peers?: string[];
					room_id?: string;
				};

				switch (msg.type) {
					case 'connected':
						this.patchServerStatus(id, {
							wsConnected: true,
							ownPeerId: msg.peer_id ?? null
						});
						break;
					case 'room-peers':
						if (msg.room_id === LOBBY_ROOM) {
							this.patchServerStatus(id, { lobbyPeers: msg.peers ?? [] });
						}
						break;
					case 'peer-joined':
						if (msg.room_id === LOBBY_ROOM && msg.peer_id) {
							this.state.update((s) => {
								const current = s.serverStatuses[id] ?? defaultStatus();
								return {
									...s,
									serverStatuses: {
										...s.serverStatuses,
										[id]: {
											...current,
											lobbyPeers: [...current.lobbyPeers, msg.peer_id!]
										}
									}
								};
							});
						}
						break;
					case 'peer-left':
						if (msg.room_id === LOBBY_ROOM && msg.peer_id) {
							this.state.update((s) => {
								const current = s.serverStatuses[id] ?? defaultStatus();
								return {
									...s,
									serverStatuses: {
										...s.serverStatuses,
										[id]: {
											...current,
											lobbyPeers: current.lobbyPeers.filter((p) => p !== msg.peer_id)
										}
									}
								};
							});
						}
						break;
				}
			} catch {
				// Ignore unparseable messages
			}
		};

		ws.onclose = () => {
			this.connections.delete(id);
			this.patchServerStatus(id, { wsConnected: false, ownPeerId: null, lobbyPeers: [] });
		};

		ws.onerror = () => {
			this.patchServerStatus(id, { wsConnected: false });
		};
	}

	private disconnectFromServer(id: string): void {
		const ws = this.connections.get(id);
		if (ws) {
			ws.close();
			this.connections.delete(id);
		}
	}

	// ===== State helpers =====

	private patchServerStatus(id: string, patch: Partial<ServerStatus>): void {
		this.state.update((s) => ({
			...s,
			serverStatuses: {
				...s.serverStatuses,
				[id]: { ...(s.serverStatuses[id] ?? defaultStatus()), ...patch }
			}
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
		for (const id of this.connections.keys()) {
			this.disconnectFromServer(id);
		}
		this._initialized = false;
	}
}

export const signalingService = new SignalingService();

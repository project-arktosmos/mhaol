import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { signalingAdapter } from '$adapters/classes/signaling.adapter';
import type {
	SignalingChatState,
	SignalingChatMessage,
	RoomState,
	PeerConnectionStatus
} from '$types/signaling.type';
import type { DataChannelEnvelope, PeerLibraryMessage } from '$types/peer-library.type';
import type { CloudPeerMessage } from '$types/cloud-peer.type';
import type { ServerCatalogMessage } from '$types/server-catalog.type';
import type { PassportData, Endorsement } from 'webrtc/types';
import { RoomConnection, type RoomConnectionCallbacks } from '$services/room-connection';

const initialState: SignalingChatState = {
	rooms: {},
	localPeerId: null,
	peerIds: [],
	activePeerId: null,
	activeRoomId: null,
	messages: [],
	error: null
};

class SignalingChatService {
	public state: Writable<SignalingChatState> = writable(initialState);

	// ===== Peer Lifecycle Callbacks =====

	private peerChannelOpenListeners: ((peerId: string) => void)[] = [];
	private peerDisconnectedListeners: ((peerId: string) => void)[] = [];

	public onPeerLibraryMessage: ((peerId: string, msg: PeerLibraryMessage) => void) | null = null;
	public onCloudMessage: ((peerId: string, msg: CloudPeerMessage) => void) | null = null;
	public onContactMessage: ((peerId: string, msg: unknown) => void) | null = null;
	public onServerCatalogMessage: ((peerId: string, msg: ServerCatalogMessage) => void) | null =
		null;
	public onRpcMessage: ((peerId: string, msg: unknown) => void) | null = null;

	addPeerChannelOpenListener(fn: (peerId: string) => void): () => void {
		this.peerChannelOpenListeners.push(fn);
		return () => {
			this.peerChannelOpenListeners = this.peerChannelOpenListeners.filter((l) => l !== fn);
		};
	}

	addPeerDisconnectedListener(fn: (peerId: string) => void): () => void {
		this.peerDisconnectedListeners.push(fn);
		return () => {
			this.peerDisconnectedListeners = this.peerDisconnectedListeners.filter((l) => l !== fn);
		};
	}

	private rooms: Map<string, RoomConnection> = new Map();
	private localAddress: string | null = null;
	private sharedIceServers: RTCIceServer[] = [];

	// ===== Connection =====

	async connectToRoom(
		serverUrl: string,
		roomId: string,
		passport: PassportData,
		signMessage: (message: string) => Promise<string>,
		endorsement?: Endorsement
	): Promise<void> {
		if (!browser) return;
		if (this.rooms.has(roomId)) return;

		const payload = JSON.parse(passport.raw);
		const address = payload.address.toLowerCase();
		this.localAddress = address;

		const callbacks: RoomConnectionCallbacks = {
			onStateChange: (rid, update) => {
				this.state.update((s) => {
					const existing = s.rooms[rid] ?? {
						roomId: rid,
						phase: 'disconnected',
						roomPeers: [],
						peerConnectionStates: {}
					};
					return {
						...s,
						rooms: { ...s.rooms, [rid]: { ...existing, ...update } }
					};
				});
			},
			onPeerIdChange: () => {
				this.updatePeerIds();
			},
			onDataMessage: (_rid, peerId, data) => {
				this.handleDataMessage(peerId, data);
			},
			onPeerChannelOpen: (_rid, peerId) => {
				this.peerChannelOpenListeners.forEach((fn) => fn(peerId));
			},
			onPeerDisconnected: (_rid, peerId) => {
				this.peerDisconnectedListeners.forEach((fn) => fn(peerId));
			},
			onLocalPeerId: (peerId) => {
				this.state.update((s) => ({ ...s, localPeerId: peerId }));
			},
			onIceServers: (servers) => {
				this.sharedIceServers = servers;
				// Share with other rooms that don't have ICE servers yet
				for (const room of this.rooms.values()) {
					room.setIceServers(servers);
				}
			},
			onSystemMessage: (content) => {
				this.addSystemMessage(content);
			},
			onError: (message) => {
				this.state.update((s) => ({ ...s, error: message }));
			}
		};

		// Initialize room state
		this.state.update((s) => ({
			...s,
			rooms: {
				...s.rooms,
				[roomId]: {
					roomId,
					phase: 'connecting',
					roomPeers: [],
					peerConnectionStates: {}
				}
			},
			activeRoomId: s.activeRoomId ?? roomId
		}));

		const room = new RoomConnection(roomId, callbacks, this.sharedIceServers);
		this.rooms.set(roomId, room);

		await room.connect(serverUrl, address, passport, signMessage, endorsement);
	}

	disconnectFromRoom(roomId: string): void {
		const room = this.rooms.get(roomId);
		if (room) {
			room.disconnect();
			this.rooms.delete(roomId);
			this.state.update((s) => {
				const { [roomId]: _, ...rest } = s.rooms;
				return {
					...s,
					rooms: rest,
					activeRoomId: s.activeRoomId === roomId ? null : s.activeRoomId
				};
			});
			this.updatePeerIds();
		}
	}

	/** Backward-compatible wrapper: connects to a single room. */
	async connect(
		serverUrl: string,
		roomId: string,
		passport: PassportData,
		signMessage: (message: string) => Promise<string>
	): Promise<void> {
		return this.connectToRoom(serverUrl, roomId, passport, signMessage);
	}

	disconnect(): void {
		for (const room of this.rooms.values()) {
			room.disconnect();
		}
		this.rooms.clear();
		this.sharedIceServers = [];
		this.state.set(initialState);
	}

	// ===== Chat =====

	sendMessage(content: string): void {
		if (!this.localAddress) return;

		let activePeerId: string | null = null;
		this.state.subscribe((s) => {
			activePeerId = s.activePeerId;
		})();

		if (!activePeerId) return;

		const room = this.findRoomForPeer(activePeerId);
		if (!room) return;

		const message = signalingAdapter.createMessage(this.localAddress, content);
		const envelope: DataChannelEnvelope = { channel: 'chat', payload: message };
		room.sendToPeer(activePeerId, envelope);

		this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
	}

	// ===== Data Channel Send =====

	sendToPeer(peerId: string, envelope: DataChannelEnvelope): void {
		const room = this.findRoomForPeer(peerId);
		if (room) {
			room.sendToPeer(peerId, envelope);
		}
	}

	broadcast(envelope: DataChannelEnvelope): void {
		const sent = new Set<string>();
		for (const room of this.rooms.values()) {
			room.broadcast(envelope, sent);
		}
	}

	// ===== Peer Management =====

	connectToPeer(peerId: string): void {
		const room = this.findRoomWithRoomPeer(peerId);
		if (room) {
			this.state.update((s) => ({ ...s, activePeerId: peerId }));
			room.connectToPeer(peerId);
		}
	}

	setActivePeer(peerId: string | null): void {
		this.state.update((s) => ({ ...s, activePeerId: peerId }));
	}

	disconnectPeer(peerId: string): void {
		for (const room of this.rooms.values()) {
			if (room.hasRoomPeer(peerId)) {
				room.disconnectPeer(peerId);
			}
		}
		this.state.update((s) => ({
			...s,
			activePeerId: s.activePeerId === peerId ? null : s.activePeerId
		}));
	}

	// ===== Data Message Routing =====

	private handleDataMessage(peerId: string, parsed: unknown): void {
		const data = parsed as Record<string, unknown>;
		if (data.channel === 'chat') {
			const msg = data.payload as SignalingChatMessage;
			this.state.update((s) => ({ ...s, messages: [...s.messages, msg] }));
		} else if (data.channel === 'peer-library') {
			this.onPeerLibraryMessage?.(peerId, data.payload as PeerLibraryMessage);
		} else if (data.channel === 'cloud') {
			this.onCloudMessage?.(peerId, data.payload as CloudPeerMessage);
		} else if (data.channel === 'contact') {
			this.onContactMessage?.(peerId, data.payload);
		} else if (data.channel === 'server-catalog') {
			this.onServerCatalogMessage?.(peerId, data.payload as ServerCatalogMessage);
		} else if (data.channel === 'rpc') {
			this.onRpcMessage?.(peerId, data.payload);
		} else if (
			(data as Record<string, unknown>).id &&
			(data as Record<string, unknown>).address &&
			(data as Record<string, unknown>).content
		) {
			// Legacy format: raw SignalingChatMessage (backward compat)
			const msg = data as unknown as SignalingChatMessage;
			this.state.update((s) => ({ ...s, messages: [...s.messages, msg] }));
		}
	}

	// ===== Room Helpers =====

	private findRoomForPeer(peerId: string): RoomConnection | undefined {
		for (const room of this.rooms.values()) {
			if (room.hasPeer(peerId)) return room;
		}
		return undefined;
	}

	private findRoomWithRoomPeer(peerId: string): RoomConnection | undefined {
		for (const room of this.rooms.values()) {
			if (room.hasRoomPeer(peerId)) return room;
		}
		return undefined;
	}

	private updatePeerIds(): void {
		const peerIds = new Set<string>();
		for (const room of this.rooms.values()) {
			for (const id of room.getConnectedPeerIds()) {
				peerIds.add(id);
			}
		}
		this.state.update((s) => ({ ...s, peerIds: Array.from(peerIds) }));
	}

	// ===== Identity =====

	getAddress(): string | null {
		return this.localAddress;
	}

	getPeerConnectionStatus(peerId: string): PeerConnectionStatus | undefined {
		for (const room of this.rooms.values()) {
			const status = room.getPeerConnectionStatus(peerId);
			if (status) return status;
		}
		return undefined;
	}

	// ===== System Messages =====

	private addSystemMessage(content: string): void {
		const message: SignalingChatMessage = {
			id:
				typeof crypto.randomUUID === 'function'
					? crypto.randomUUID()
					: Math.random().toString(36).slice(2) + Date.now().toString(36),
			address: 'system',
			content,
			timestamp: new Date().toISOString(),
			system: true
		};
		this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
	}

	// ===== Lifecycle =====

	destroy(): void {
		this.disconnect();
	}
}

export const signalingChatService = new SignalingChatService();

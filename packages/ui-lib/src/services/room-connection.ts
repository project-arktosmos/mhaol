import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
import type {
	PeerConnectionStatus,
	SignalingServerMessage,
	SignalingClientMessage,
	SignalingPeerInfo,
	RoomState
} from 'ui-lib/types/signaling.type';
import type { DataChannelEnvelope } from 'ui-lib/types/peer-library.type';
import type { Endorsement } from 'webrtc/types';

const DATA_CHANNEL_LABEL = 'signaling-chat';

export interface RoomConnectionCallbacks {
	onStateChange(roomId: string, update: Partial<RoomState>): void;
	onPeerIdChange(roomId: string): void;
	onDataMessage(roomId: string, peerId: string, data: unknown): void;
	onPeerChannelOpen(roomId: string, peerId: string): void;
	onPeerDisconnected(roomId: string, peerId: string): void;
	onLocalPeerId(peerId: string): void;
	onIceServers(servers: RTCIceServer[]): void;
	onSystemMessage(content: string): void;
	onError(message: string): void;
}

export class RoomConnection {
	readonly roomId: string;
	private ws: WebSocket | null = null;
	private peerConnections: Map<string, RTCPeerConnection> = new Map();
	private dataChannels: Map<string, RTCDataChannel> = new Map();
	private remoteDescriptionSet: Map<string, boolean> = new Map();
	private pendingCandidates: Map<string, RTCIceCandidateInit[]> = new Map();
	private callbacks: RoomConnectionCallbacks;
	private iceServers: RTCIceServer[];

	constructor(roomId: string, callbacks: RoomConnectionCallbacks, iceServers: RTCIceServer[]) {
		this.roomId = roomId;
		this.callbacks = callbacks;
		this.iceServers = iceServers;
	}

	async connect(
		serverUrl: string,
		address: string,
		passport: { raw: string; signature: string },
		signMessage: (message: string) => Promise<string>,
		endorsement?: Endorsement
	): Promise<void> {
		this.disconnect();

		this.callbacks.onStateChange(this.roomId, { phase: 'connecting' });

		try {
			const timestamp = String(Date.now());
			const message = `partykit-auth:${this.roomId}:${timestamp}`;
			const signature = await signMessage(message);

			const wsUrl = signalingAdapter.buildWsUrl(serverUrl, this.roomId);
			const params = new URLSearchParams({
				address,
				signature,
				timestamp,
				passport_raw: passport.raw,
				passport_signature: passport.signature
			});

			if (endorsement) {
				params.set('endorser_signature', endorsement.endorserSignature);
				params.set('endorser_address', endorsement.endorserAddress);
			}

			const fullUrl = `${wsUrl}?${params.toString()}`;

			console.log(`[RoomConnection:${this.roomId}] Connecting to ${wsUrl}`);
			this.ws = new WebSocket(fullUrl);

			this.ws.onmessage = (event) => {
				try {
					const msg = JSON.parse(event.data as string) as SignalingServerMessage;
					this.handleServerMessage(msg);
				} catch {
					// Ignore unparseable messages
				}
			};

			this.ws.onerror = () => {
				this.callbacks.onStateChange(this.roomId, { phase: 'error' });
				this.callbacks.onError(`WebSocket error in room ${this.roomId}`);
			};

			this.ws.onclose = () => {
				this.cleanupAllPeers();
				this.callbacks.onStateChange(this.roomId, { phase: 'disconnected', roomPeers: [] });
			};
		} catch (err) {
			const msg = err instanceof Error ? err.message : 'Connection failed';
			this.callbacks.onStateChange(this.roomId, { phase: 'error' });
			this.callbacks.onError(msg);
		}
	}

	disconnect(): void {
		this.cleanupAllPeers();
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
	}

	connectToPeer(peerId: string): void {
		const status = this.getPeerConnectionStatus(peerId);
		if (status === 'offering' || status === 'answering' || status === 'connected') return;

		this.updatePeerConnectionStatus(peerId, 'offering');
		this.callbacks.onSystemMessage(
			`Initiating connection to ${signalingAdapter.shortAddress(peerId)}...`
		);
		this.createPeerConnection(peerId, true);
	}

	disconnectPeer(peerId: string): void {
		this.callbacks.onSystemMessage(`Disconnected from ${signalingAdapter.shortAddress(peerId)}`);
		this.removePeer(peerId);
		this.updatePeerConnectionStatus(peerId, undefined);
	}

	sendToPeer(peerId: string, envelope: DataChannelEnvelope): void {
		const channel = this.dataChannels.get(peerId);
		if (channel?.readyState === 'open') {
			channel.send(JSON.stringify(envelope));
		}
	}

	broadcast(envelope: DataChannelEnvelope, excludePeers?: Set<string>): void {
		const serialized = JSON.stringify(envelope);
		for (const [peerId, channel] of this.dataChannels) {
			if (channel.readyState === 'open' && !excludePeers?.has(peerId)) {
				channel.send(serialized);
				excludePeers?.add(peerId);
			}
		}
	}

	hasPeer(peerId: string): boolean {
		return this.dataChannels.has(peerId) && this.dataChannels.get(peerId)!.readyState === 'open';
	}

	hasRoomPeer(peerId: string): boolean {
		return (
			this.peerConnections.has(peerId) || this.getRoomPeers().some((p) => p.peer_id === peerId)
		);
	}

	getConnectedPeerIds(): string[] {
		return Array.from(this.dataChannels.entries())
			.filter(([, ch]) => ch.readyState === 'open')
			.map(([id]) => id);
	}

	getPeerConnectionStatus(peerId: string): PeerConnectionStatus | undefined {
		return this._peerConnectionStates[peerId];
	}

	setIceServers(servers: RTCIceServer[]): void {
		this.iceServers = servers;
	}

	private _roomPeers: SignalingPeerInfo[] = [];
	private _peerConnectionStates: Record<string, PeerConnectionStatus> = {};

	getRoomPeers(): SignalingPeerInfo[] {
		return this._roomPeers;
	}

	// ===== Protocol Handling =====

	private handleServerMessage(msg: SignalingServerMessage): void {
		switch (msg.type) {
			case 'connected':
				if (msg.ice_servers && msg.ice_servers.length > 0) {
					const servers = msg.ice_servers.map((s) => {
						const entry: RTCIceServer = { urls: s.urls };
						if (s.username) entry.username = s.username;
						if (s.credential) entry.credential = s.credential;
						return entry;
					});
					this.iceServers = servers;
					this.callbacks.onIceServers(servers);
				}
				this.callbacks.onLocalPeerId(msg.peer_id);
				this.callbacks.onStateChange(this.roomId, { phase: 'connected' });
				break;
			case 'room-peers':
				this._roomPeers = msg.peers;
				this.callbacks.onStateChange(this.roomId, { roomPeers: msg.peers });
				break;
			case 'peer-joined': {
				const peerInfo: SignalingPeerInfo = {
					peer_id: msg.peer_id,
					name: msg.name,
					instance_type: msg.instance_type
				};
				if (!this._roomPeers.some((p) => p.peer_id === msg.peer_id)) {
					this._roomPeers = [...this._roomPeers, peerInfo];
				}
				this.callbacks.onStateChange(this.roomId, { roomPeers: this._roomPeers });
				this.callbacks.onSystemMessage(
					`Peer ${signalingAdapter.shortAddress(msg.peer_id)} joined room ${this.roomId}`
				);
				break;
			}
			case 'peer-left':
				this._roomPeers = this._roomPeers.filter((p) => p.peer_id !== msg.peer_id);
				delete this._peerConnectionStates[msg.peer_id];
				this.callbacks.onStateChange(this.roomId, {
					roomPeers: this._roomPeers,
					peerConnectionStates: { ...this._peerConnectionStates }
				});
				this.removePeer(msg.peer_id);
				this.callbacks.onSystemMessage(
					`Peer ${signalingAdapter.shortAddress(msg.peer_id)} left room ${this.roomId}`
				);
				break;
			case 'offer': {
				const offerStatus = this.getPeerConnectionStatus(msg.from_peer_id);
				if (
					offerStatus === 'offering' ||
					offerStatus === 'answering' ||
					offerStatus === 'connected'
				) {
					break;
				}
				this.callbacks.onSystemMessage(
					`Incoming connection from ${signalingAdapter.shortAddress(msg.from_peer_id)}`
				);
				this.updatePeerConnectionStatus(msg.from_peer_id, 'answering');
				this.handleOffer(msg.from_peer_id, msg.sdp);
				break;
			}
			case 'answer':
				this.handleAnswer(msg.from_peer_id, msg.sdp);
				break;
			case 'ice-candidate':
				this.handleIceCandidate(msg.from_peer_id, msg.candidate, msg.sdp_m_line_index, msg.sdp_mid);
				break;
			case 'error':
				this.callbacks.onError(msg.message);
				break;
		}
	}

	// ===== WebRTC Peer Management =====

	private async createPeerConnection(peerId: string, createOffer: boolean): Promise<void> {
		this.removePeerConnection(peerId);

		this.remoteDescriptionSet.set(peerId, false);
		this.pendingCandidates.set(peerId, []);

		const pc = new RTCPeerConnection({ iceServers: this.iceServers });
		this.peerConnections.set(peerId, pc);

		pc.onicecandidate = (event) => {
			if (event.candidate) {
				this.sendSignaling({
					type: 'ice-candidate',
					target_peer_id: peerId,
					candidate: event.candidate.candidate,
					sdp_m_line_index: event.candidate.sdpMLineIndex ?? 0,
					sdp_mid: event.candidate.sdpMid ?? undefined
				});
			}
		};

		pc.oniceconnectionstatechange = () => {
			const currentStatus = this.getPeerConnectionStatus(peerId);
			if (
				(pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') &&
				currentStatus !== 'connected'
			) {
				this.updatePeerConnectionStatus(peerId, 'connected');
				this.callbacks.onSystemMessage(
					`WebRTC connected to ${signalingAdapter.shortAddress(peerId)}`
				);
			} else if (pc.iceConnectionState === 'failed') {
				this.updatePeerConnectionStatus(peerId, 'failed');
				this.callbacks.onSystemMessage(
					`Connection to ${signalingAdapter.shortAddress(peerId)} failed`
				);
				this.removePeer(peerId);
			} else if (pc.iceConnectionState === 'disconnected') {
				this.removePeer(peerId);
			}
			this.callbacks.onPeerIdChange(this.roomId);
		};

		pc.onconnectionstatechange = () => {
			const currentStatus = this.getPeerConnectionStatus(peerId);
			if (pc.connectionState === 'connected' && currentStatus !== 'connected') {
				this.updatePeerConnectionStatus(peerId, 'connected');
				this.callbacks.onSystemMessage(
					`WebRTC connected to ${signalingAdapter.shortAddress(peerId)}`
				);
			} else if (pc.connectionState === 'failed') {
				this.updatePeerConnectionStatus(peerId, 'failed');
				this.removePeer(peerId);
			}
		};

		pc.ondatachannel = (event) => {
			this.setupDataChannel(peerId, event.channel);
		};

		if (createOffer) {
			const channel = pc.createDataChannel(DATA_CHANNEL_LABEL);
			this.setupDataChannel(peerId, channel);

			const offer = await pc.createOffer();
			await pc.setLocalDescription(offer);

			this.sendSignaling({
				type: 'offer',
				target_peer_id: peerId,
				sdp: offer.sdp!
			});
		}

		this.callbacks.onPeerIdChange(this.roomId);
	}

	private async handleOffer(fromPeerId: string, sdp: string): Promise<void> {
		if (!this.peerConnections.has(fromPeerId)) {
			await this.createPeerConnection(fromPeerId, false);
		}

		const pc = this.peerConnections.get(fromPeerId)!;

		try {
			await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }));
			this.remoteDescriptionSet.set(fromPeerId, true);
			await this.flushPendingCandidates(fromPeerId);

			const answer = await pc.createAnswer();
			await pc.setLocalDescription(answer);

			this.sendSignaling({
				type: 'answer',
				target_peer_id: fromPeerId,
				sdp: answer.sdp!
			});
		} catch (err) {
			console.error(`[RoomConnection:${this.roomId}] SDP negotiation error:`, err);
		}
	}

	private async handleAnswer(fromPeerId: string, sdp: string): Promise<void> {
		const pc = this.peerConnections.get(fromPeerId);
		if (!pc) return;

		try {
			await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp }));
			this.remoteDescriptionSet.set(fromPeerId, true);
			await this.flushPendingCandidates(fromPeerId);
		} catch (err) {
			console.error(`[RoomConnection:${this.roomId}] Failed to set answer:`, err);
		}
	}

	private async handleIceCandidate(
		fromPeerId: string,
		candidate: string,
		sdpMLineIndex: number,
		sdpMid?: string
	): Promise<void> {
		const candidateInit: RTCIceCandidateInit = {
			candidate,
			sdpMLineIndex,
			sdpMid: sdpMid ?? null
		};

		if (this.remoteDescriptionSet.get(fromPeerId) && this.peerConnections.has(fromPeerId)) {
			const pc = this.peerConnections.get(fromPeerId)!;
			pc.addIceCandidate(candidateInit).catch((err) => {
				console.error(`[RoomConnection:${this.roomId}] Failed to add ICE candidate:`, err);
			});
		} else {
			const pending = this.pendingCandidates.get(fromPeerId) ?? [];
			pending.push(candidateInit);
			this.pendingCandidates.set(fromPeerId, pending);
		}
	}

	private async flushPendingCandidates(peerId: string): Promise<void> {
		const pc = this.peerConnections.get(peerId);
		const pending = this.pendingCandidates.get(peerId) ?? [];
		if (!pc || pending.length === 0) return;

		for (const candidate of pending) {
			try {
				await pc.addIceCandidate(candidate);
			} catch (err) {
				console.error(`[RoomConnection:${this.roomId}] Failed to flush ICE candidate:`, err);
			}
		}
		this.pendingCandidates.set(peerId, []);
	}

	// ===== DataChannel =====

	private setupDataChannel(peerId: string, channel: RTCDataChannel): void {
		this.dataChannels.set(peerId, channel);

		channel.onopen = () => {
			this.callbacks.onPeerIdChange(this.roomId);
			this.callbacks.onPeerChannelOpen(this.roomId, peerId);
		};
		channel.onclose = () => this.callbacks.onPeerIdChange(this.roomId);
		channel.onerror = () => this.callbacks.onPeerIdChange(this.roomId);

		channel.onmessage = (event) => {
			try {
				const parsed = JSON.parse(event.data as string);
				this.callbacks.onDataMessage(this.roomId, peerId, parsed);
			} catch {
				// Ignore unparseable messages
			}
		};
	}

	// ===== Cleanup =====

	private removePeer(peerId: string): void {
		this.removePeerConnection(peerId);
		this.callbacks.onPeerDisconnected(this.roomId, peerId);
		this.callbacks.onPeerIdChange(this.roomId);
	}

	private removePeerConnection(peerId: string): void {
		const channel = this.dataChannels.get(peerId);
		if (channel) {
			channel.close();
			this.dataChannels.delete(peerId);
		}
		const pc = this.peerConnections.get(peerId);
		if (pc) {
			pc.close();
			this.peerConnections.delete(peerId);
		}
		this.remoteDescriptionSet.delete(peerId);
		this.pendingCandidates.delete(peerId);
	}

	private cleanupAllPeers(): void {
		for (const peerId of this.peerConnections.keys()) {
			this.removePeerConnection(peerId);
		}
		this.callbacks.onPeerIdChange(this.roomId);
	}

	// ===== Signaling Send =====

	private sendSignaling(msg: SignalingClientMessage): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
		}
	}

	private updatePeerConnectionStatus(
		peerId: string,
		status: PeerConnectionStatus | undefined
	): void {
		if (status === undefined) {
			delete this._peerConnectionStates[peerId];
		} else {
			this._peerConnectionStates[peerId] = status;
		}
		this.callbacks.onStateChange(this.roomId, {
			peerConnectionStates: { ...this._peerConnectionStates }
		});
	}
}

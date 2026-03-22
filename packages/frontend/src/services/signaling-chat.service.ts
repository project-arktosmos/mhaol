import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { p2pStreamService } from 'frontend/services/p2p-stream.service';
import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';
import type {
	PeerConnectionStatus,
	SignalingChatState,
	SignalingServerMessage,
	SignalingClientMessage,
	SignalingChatMessage
} from 'frontend/types/signaling.type';
import type { DataChannelEnvelope, PeerLibraryMessage } from 'frontend/types/peer-library.type';
import type { CloudPeerMessage } from 'frontend/types/cloud-peer.type';

const DATA_CHANNEL_LABEL = 'signaling-chat';

const initialState: SignalingChatState = {
	phase: 'disconnected',
	roomId: '',
	localPeerId: null,
	peerIds: [],
	roomPeerIds: [],
	activePeerId: null,
	peerConnectionStates: {},
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

	private ws: WebSocket | null = null;
	private peerConnections: Map<string, RTCPeerConnection> = new Map();
	private dataChannels: Map<string, RTCDataChannel> = new Map();
	private remoteDescriptionSet: Map<string, boolean> = new Map();
	private pendingCandidates: Map<string, RTCIceCandidateInit[]> = new Map();
	private ephemeralAccount = browser ? privateKeyToAccount(generatePrivateKey()) : null;

	// ===== Connection =====

	async connect(serverUrl: string, roomId: string): Promise<void> {
		if (!browser) return;
		this.disconnect();

		this.state.update((s) => ({
			...s,
			phase: 'connecting',
			roomId,
			error: null
		}));

		try {
			if (!this.ephemeralAccount) throw new Error('No ephemeral account available');
			const address = this.ephemeralAccount.address.toLowerCase();

			const timestamp = String(Date.now());
			const message = `partykit-auth:${roomId}:${timestamp}`;
			const signature = await this.ephemeralAccount.signMessage({ message });

			const wsUrl = signalingAdapter.buildWsUrl(serverUrl, roomId);
			const params = new URLSearchParams({ address, signature, timestamp });
			const fullUrl = `${wsUrl}?${params.toString()}`;

			console.log(`[SignalingChat] Connecting to ${wsUrl}`);
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
				this.state.update((s) => ({
					...s,
					phase: 'error',
					error: 'WebSocket connection error'
				}));
			};

			this.ws.onclose = () => {
				this.cleanupAllPeers();
				this.state.update((s) => ({
					...s,
					phase: 'disconnected',
					localPeerId: null
				}));
			};
		} catch (err) {
			const msg = err instanceof Error ? err.message : 'Connection failed';
			this.state.update((s) => ({ ...s, phase: 'error', error: msg }));
		}
	}

	disconnect(): void {
		this.cleanupAllPeers();
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		this.state.update((s) => ({
			...initialState,
			roomId: s.roomId
		}));
	}

	// ===== Chat =====

	sendMessage(content: string): void {
		const address = this.ephemeralAccount?.address.toLowerCase();
		if (!address) return;

		let activePeerId: string | null = null;
		this.state.subscribe((s) => {
			activePeerId = s.activePeerId;
		})();

		if (!activePeerId) return;

		const channel = this.dataChannels.get(activePeerId);
		if (!channel || channel.readyState !== 'open') return;

		const message = signalingAdapter.createMessage(address, content);
		const envelope: DataChannelEnvelope = { channel: 'chat', payload: message };
		channel.send(JSON.stringify(envelope));

		this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
	}

	// ===== Data Channel Send =====

	sendToPeer(peerId: string, envelope: DataChannelEnvelope): void {
		const channel = this.dataChannels.get(peerId);
		if (channel?.readyState === 'open') {
			channel.send(JSON.stringify(envelope));
		}
	}

	broadcast(envelope: DataChannelEnvelope): void {
		const serialized = JSON.stringify(envelope);
		for (const [, channel] of this.dataChannels) {
			if (channel.readyState === 'open') {
				channel.send(serialized);
			}
		}
	}

	// ===== Peer Management =====

	connectToPeer(peerId: string): void {
		const status = this.getPeerConnectionStatus(peerId);
		if (status === 'offering' || status === 'answering' || status === 'connected') return;

		this.updatePeerConnectionStatus(peerId, 'offering');
		this.state.update((s) => ({ ...s, activePeerId: peerId }));
		this.addSystemMessage(`Initiating connection to ${signalingAdapter.shortAddress(peerId)}...`);
		this.createPeerConnection(peerId, true);
	}

	setActivePeer(peerId: string | null): void {
		this.state.update((s) => ({ ...s, activePeerId: peerId }));
	}

	disconnectPeer(peerId: string): void {
		this.addSystemMessage(`Disconnected from ${signalingAdapter.shortAddress(peerId)}`);
		this.removePeer(peerId);
		this.state.update((s) => {
			const { [peerId]: _, ...rest } = s.peerConnectionStates;
			return {
				...s,
				activePeerId: s.activePeerId === peerId ? null : s.activePeerId,
				peerConnectionStates: rest
			};
		});
	}

	// ===== Protocol Handling =====

	private handleServerMessage(msg: SignalingServerMessage): void {
		switch (msg.type) {
			case 'connected':
				this.state.update((s) => ({
					...s,
					phase: 'connected',
					localPeerId: msg.peer_id
				}));
				break;
			case 'room-peers':
				this.state.update((s) => ({
					...s,
					roomPeerIds: msg.peers.filter((p) => p !== s.localPeerId)
				}));
				break;
			case 'peer-joined':
				this.state.update((s) => ({
					...s,
					roomPeerIds: s.roomPeerIds.includes(msg.peer_id)
						? s.roomPeerIds
						: [...s.roomPeerIds, msg.peer_id]
				}));
				this.addSystemMessage(`Peer ${signalingAdapter.shortAddress(msg.peer_id)} joined the room`);
				break;
			case 'peer-left':
				this.state.update((s) => {
					const { [msg.peer_id]: _, ...rest } = s.peerConnectionStates;
					return {
						...s,
						roomPeerIds: s.roomPeerIds.filter((id) => id !== msg.peer_id),
						activePeerId: s.activePeerId === msg.peer_id ? null : s.activePeerId,
						peerConnectionStates: rest
					};
				});
				this.removePeer(msg.peer_id);
				this.addSystemMessage(`Peer ${signalingAdapter.shortAddress(msg.peer_id)} left the room`);
				break;
			case 'offer': {
				const offerStatus = this.getPeerConnectionStatus(msg.from_peer_id);
				console.log(
					`[SignalingChat] Offer from ${msg.from_peer_id}, current status: ${offerStatus ?? 'none'}`
				);
				if (
					offerStatus === 'offering' ||
					offerStatus === 'answering' ||
					offerStatus === 'connected'
				) {
					console.log(`[SignalingChat] Ignoring offer — already ${offerStatus}`);
					break;
				}
				this.addSystemMessage(
					`Incoming connection from ${signalingAdapter.shortAddress(msg.from_peer_id)}`
				);
				this.updatePeerConnectionStatus(msg.from_peer_id, 'answering');
				this.state.update((s) => ({
					...s,
					activePeerId: s.activePeerId ?? msg.from_peer_id
				}));
				this.handleOffer(msg.from_peer_id, msg.sdp);
				break;
			}
			case 'answer':
				this.addSystemMessage(
					`Answer received from ${signalingAdapter.shortAddress(msg.from_peer_id)}`
				);
				this.handleAnswer(msg.from_peer_id, msg.sdp);
				break;
			case 'ice-candidate':
				this.handleIceCandidate(msg.from_peer_id, msg.candidate, msg.sdp_m_line_index, msg.sdp_mid);
				break;
			case 'error':
				this.state.update((s) => ({ ...s, error: msg.message }));
				break;
		}
	}

	// ===== WebRTC Peer Management =====

	private async createPeerConnection(peerId: string, createOffer: boolean): Promise<void> {
		this.removePeerConnection(peerId);

		this.remoteDescriptionSet.set(peerId, false);
		this.pendingCandidates.set(peerId, []);

		const iceServers = p2pStreamService.getIceServers();
		console.log('[SignalingChat] ICE servers:', JSON.stringify(iceServers));
		const pc = new RTCPeerConnection({ iceServers });
		this.peerConnections.set(peerId, pc);

		pc.onicecandidate = (event) => {
			if (event.candidate) {
				console.log(
					`[SignalingChat] Sending ICE candidate to ${peerId}:`,
					event.candidate.candidate.substring(0, 60)
				);
				this.sendSignaling({
					type: 'ice-candidate',
					target_peer_id: peerId,
					candidate: event.candidate.candidate,
					sdp_m_line_index: event.candidate.sdpMLineIndex ?? 0,
					sdp_mid: event.candidate.sdpMid ?? undefined
				});
			} else {
				console.log(`[SignalingChat] ICE gathering complete for ${peerId}`);
			}
		};

		pc.oniceconnectionstatechange = () => {
			console.log(`[SignalingChat] ICE state for ${peerId}: ${pc.iceConnectionState}`);
			const currentStatus = this.getPeerConnectionStatus(peerId);
			if (pc.iceConnectionState === 'checking') {
				this.addSystemMessage(`ICE checking for ${signalingAdapter.shortAddress(peerId)}...`);
			} else if (
				(pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') &&
				currentStatus !== 'connected'
			) {
				this.updatePeerConnectionStatus(peerId, 'connected');
				this.addSystemMessage(`WebRTC connected to ${signalingAdapter.shortAddress(peerId)}`);
			} else if (pc.iceConnectionState === 'failed') {
				this.updatePeerConnectionStatus(peerId, 'failed');
				this.addSystemMessage(`Connection to ${signalingAdapter.shortAddress(peerId)} failed`);
				this.removePeer(peerId);
			} else if (pc.iceConnectionState === 'disconnected') {
				this.removePeer(peerId);
			}
			this.updatePeerIds();
		};

		pc.onconnectionstatechange = () => {
			console.log(`[SignalingChat] Connection state for ${peerId}: ${pc.connectionState}`);
			const currentStatus = this.getPeerConnectionStatus(peerId);
			if (pc.connectionState === 'connected' && currentStatus !== 'connected') {
				this.updatePeerConnectionStatus(peerId, 'connected');
				this.addSystemMessage(`WebRTC connected to ${signalingAdapter.shortAddress(peerId)}`);
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

		this.updatePeerIds();
	}

	private async handleOffer(fromPeerId: string, sdp: string): Promise<void> {
		console.log(`[SignalingChat] Received offer from ${fromPeerId}`);
		if (!this.peerConnections.has(fromPeerId)) {
			await this.createPeerConnection(fromPeerId, false);
		}

		const pc = this.peerConnections.get(fromPeerId)!;

		try {
			await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }));
			this.remoteDescriptionSet.set(fromPeerId, true);

			const pendingCount = this.pendingCandidates.get(fromPeerId)?.length ?? 0;
			console.log(`[SignalingChat] Flushing ${pendingCount} pending candidates for ${fromPeerId}`);
			await this.flushPendingCandidates(fromPeerId);

			const answer = await pc.createAnswer();
			await pc.setLocalDescription(answer);

			console.log(`[SignalingChat] Sending answer to ${fromPeerId}`);
			this.sendSignaling({
				type: 'answer',
				target_peer_id: fromPeerId,
				sdp: answer.sdp!
			});
		} catch (err) {
			console.error('[SignalingChat] SDP negotiation error:', err);
		}
	}

	private async handleAnswer(fromPeerId: string, sdp: string): Promise<void> {
		console.log(`[SignalingChat] Received answer from ${fromPeerId}`);
		const pc = this.peerConnections.get(fromPeerId);
		if (!pc) return;

		try {
			await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp }));
			this.remoteDescriptionSet.set(fromPeerId, true);

			const pendingCount = this.pendingCandidates.get(fromPeerId)?.length ?? 0;
			console.log(`[SignalingChat] Flushing ${pendingCount} pending candidates for ${fromPeerId}`);
			await this.flushPendingCandidates(fromPeerId);
		} catch (err) {
			console.error('[SignalingChat] Failed to set answer:', err);
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
			console.log(
				`[SignalingChat] Adding ICE candidate from ${fromPeerId}:`,
				candidate.substring(0, 60)
			);
			const pc = this.peerConnections.get(fromPeerId)!;
			pc.addIceCandidate(candidateInit).catch((err) => {
				console.error('[SignalingChat] Failed to add ICE candidate:', err);
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
				console.error('[SignalingChat] Failed to flush ICE candidate:', err);
			}
		}
		this.pendingCandidates.set(peerId, []);
	}

	// ===== DataChannel =====

	private setupDataChannel(peerId: string, channel: RTCDataChannel): void {
		this.dataChannels.set(peerId, channel);

		channel.onopen = () => {
			this.updatePeerIds();
			this.peerChannelOpenListeners.forEach((fn) => fn(peerId));
		};
		channel.onclose = () => this.updatePeerIds();
		channel.onerror = () => this.updatePeerIds();

		channel.onmessage = (event) => {
			try {
				const parsed = JSON.parse(event.data as string);

				if (parsed.channel === 'chat') {
					const msg = parsed.payload as SignalingChatMessage;
					this.state.update((s) => ({ ...s, messages: [...s.messages, msg] }));
				} else if (parsed.channel === 'peer-library') {
					this.onPeerLibraryMessage?.(peerId, parsed.payload as PeerLibraryMessage);
				} else if (parsed.channel === 'cloud') {
					this.onCloudMessage?.(peerId, parsed.payload as CloudPeerMessage);
				} else if (parsed.channel === 'contact') {
					this.onContactMessage?.(peerId, parsed.payload);
				} else if (parsed.id && parsed.address && parsed.content) {
					// Legacy format: raw SignalingChatMessage (backward compat)
					const msg = parsed as SignalingChatMessage;
					this.state.update((s) => ({ ...s, messages: [...s.messages, msg] }));
				}
			} catch {
				// Ignore unparseable messages
			}
		};
	}

	// ===== Cleanup =====

	private removePeer(peerId: string): void {
		this.removePeerConnection(peerId);
		this.peerDisconnectedListeners.forEach((fn) => fn(peerId));
		this.updatePeerIds();
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
		this.updatePeerIds();
	}

	private updatePeerIds(): void {
		const peerIds = Array.from(this.peerConnections.keys());
		this.state.update((s) => ({ ...s, peerIds }));
	}

	// ===== Signaling Send =====

	private sendSignaling(msg: SignalingClientMessage): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
		}
	}

	// ===== Ephemeral Identity =====

	getAddress(): string | null {
		return this.ephemeralAccount?.address.toLowerCase() ?? null;
	}

	regenerateIdentity(): void {
		if (!browser) return;
		this.ephemeralAccount = privateKeyToAccount(generatePrivateKey());
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

	getPeerConnectionStatus(peerId: string): PeerConnectionStatus | undefined {
		let status: PeerConnectionStatus | undefined;
		this.state.subscribe((s) => {
			status = s.peerConnectionStates[peerId];
		})();
		return status;
	}

	private updatePeerConnectionStatus(peerId: string, status: PeerConnectionStatus): void {
		this.state.update((s) => ({
			...s,
			peerConnectionStates: { ...s.peerConnectionStates, [peerId]: status }
		}));
	}

	// ===== Lifecycle =====

	destroy(): void {
		this.disconnect();
	}
}

export const signalingChatService = new SignalingChatService();

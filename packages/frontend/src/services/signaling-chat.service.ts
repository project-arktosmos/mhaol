import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import { p2pStreamService } from '$services/p2p-stream.service';
import { signalingAdapter } from '$adapters/classes/signaling.adapter';
import type {
	SignalingChatState,
	SignalingServerTarget,
	SignalingServerMessage,
	SignalingClientMessage,
	SignalingChatMessage
} from '$types/signaling.type';

const DATA_CHANNEL_LABEL = 'signaling-chat';

const initialState: SignalingChatState = {
	phase: 'disconnected',
	serverTarget: 'dev',
	roomId: '',
	localPeerId: null,
	peerIds: [],
	messages: [],
	error: null
};

class SignalingChatService {
	public state: Writable<SignalingChatState> = writable(initialState);

	private ws: WebSocket | null = null;
	private peerConnections: Map<string, RTCPeerConnection> = new Map();
	private dataChannels: Map<string, RTCDataChannel> = new Map();
	private ephemeralAccount = browser
		? privateKeyToAccount(generatePrivateKey())
		: null;

	// ===== Connection =====

	async connect(
		serverUrl: string,
		roomId: string,
		target: SignalingServerTarget
	): Promise<void> {
		if (!browser) return;
		this.disconnect();

		this.state.update((s) => ({
			...s,
			phase: 'connecting',
			serverTarget: target,
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
			serverTarget: s.serverTarget,
			roomId: s.roomId
		}));
	}

	// ===== Chat =====

	sendMessage(content: string): void {
		const address = this.ephemeralAccount?.address.toLowerCase();
		if (!address) return;

		const message = signalingAdapter.createMessage(address, content);
		const serialized = JSON.stringify(message);

		for (const [, channel] of this.dataChannels) {
			if (channel.readyState === 'open') {
				channel.send(serialized);
			}
		}

		this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
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
				// We just joined — existing peers will receive peer-joined and initiate offers to us.
				// We do NOT create offers here to avoid glare.
				break;
			case 'peer-joined':
				// A new peer joined — as the existing peer, we initiate the offer.
				this.createPeerConnection(msg.peer_id, true);
				break;
			case 'peer-left':
				this.removePeer(msg.peer_id);
				break;
			case 'offer':
				this.handleOffer(msg.from_peer_id, msg.sdp);
				break;
			case 'answer':
				this.handleAnswer(msg.from_peer_id, msg.sdp);
				break;
			case 'ice-candidate':
				this.handleIceCandidate(msg.from_peer_id, msg.candidate, msg.sdp_m_line_index);
				break;
			case 'error':
				this.state.update((s) => ({ ...s, error: msg.message }));
				break;
		}
	}

	// ===== WebRTC Peer Management =====

	private async createPeerConnection(peerId: string, createOffer: boolean): Promise<void> {
		this.removePeerConnection(peerId);

		const pc = new RTCPeerConnection({ iceServers: p2pStreamService.getIceServers() });
		this.peerConnections.set(peerId, pc);

		pc.onicecandidate = (event) => {
			if (event.candidate) {
				this.sendSignaling({
					type: 'ice-candidate',
					target_peer_id: peerId,
					candidate: event.candidate.candidate,
					sdp_m_line_index: event.candidate.sdpMLineIndex ?? 0
				});
			}
		};

		pc.oniceconnectionstatechange = () => {
			this.updatePeerIds();
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
		if (!this.peerConnections.has(fromPeerId)) {
			await this.createPeerConnection(fromPeerId, false);
		}

		const pc = this.peerConnections.get(fromPeerId)!;
		await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }));

		const answer = await pc.createAnswer();
		await pc.setLocalDescription(answer);

		this.sendSignaling({
			type: 'answer',
			target_peer_id: fromPeerId,
			sdp: answer.sdp!
		});
	}

	private async handleAnswer(fromPeerId: string, sdp: string): Promise<void> {
		const pc = this.peerConnections.get(fromPeerId);
		if (!pc) return;
		await pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp }));
	}

	private async handleIceCandidate(
		fromPeerId: string,
		candidate: string,
		sdpMLineIndex: number
	): Promise<void> {
		const pc = this.peerConnections.get(fromPeerId);
		if (!pc) return;
		await pc.addIceCandidate(new RTCIceCandidate({ candidate, sdpMLineIndex }));
	}

	// ===== DataChannel =====

	private setupDataChannel(peerId: string, channel: RTCDataChannel): void {
		this.dataChannels.set(peerId, channel);

		channel.onopen = () => this.updatePeerIds();
		channel.onclose = () => this.updatePeerIds();
		channel.onerror = () => this.updatePeerIds();

		channel.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string) as SignalingChatMessage;
				this.state.update((s) => ({ ...s, messages: [...s.messages, msg] }));
			} catch {
				// Ignore unparseable messages
			}
		};
	}

	// ===== Cleanup =====

	private removePeer(peerId: string): void {
		this.removePeerConnection(peerId);
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

	// ===== Lifecycle =====

	destroy(): void {
		this.disconnect();
	}
}

export const signalingChatService = new SignalingChatService();

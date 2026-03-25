import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { getAddress } from 'viem';
import { setApiBase } from 'ui-lib/lib/api-base';
import { setTransport } from 'ui-lib/transport/transport-context';
import { HttpTransport } from 'ui-lib/transport/http-transport';
import { WebRtcTransport } from 'ui-lib/transport/webrtc-transport';
import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
import { clientIdentityService } from 'ui-lib/services/client-identity.service';
import { contactHandshakeService } from 'webrtc/service';
import type { ConnectionConfig } from 'ui-lib/types/connection-config.type';
import type { ContactHandshakeMessage, AcceptedContact } from 'webrtc/types';
import type { RpcMessage } from 'ui-lib/transport/rpc.type';

export type NodeConnectionPhase =
	| 'idle'
	| 'connecting'
	| 'signaling'
	| 'peer-discovery'
	| 'webrtc'
	| 'handshake'
	| 'ready'
	| 'error';

export interface NodeConnectionState {
	phase: NodeConnectionPhase;
	error: string | null;
}

const initialState: NodeConnectionState = {
	phase: 'idle',
	error: null
};

class NodeConnectionService {
	state: Writable<NodeConnectionState> = writable(initialState);

	private webRtcTransport: WebRtcTransport | null = null;
	private unsubscribers: (() => void)[] = [];

	async connectHttp(config: ConnectionConfig): Promise<void> {
		this.state.set({ phase: 'connecting', error: null });

		try {
			setApiBase(config.serverUrl);
			setTransport(new HttpTransport());

			// Verify connectivity with a simple health check
			const response = await globalThis.fetch(`${config.serverUrl}/api/health`);
			if (!response.ok) {
				throw new Error(`Server returned HTTP ${response.status}`);
			}

			this.state.set({ phase: 'ready', error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Connection failed';
			this.state.set({ phase: 'error', error: message });
			throw err;
		}
	}

	async connectWebRtc(config: ConnectionConfig): Promise<void> {
		if (!browser) return;

		this.state.set({ phase: 'connecting', error: null });

		try {
			// Step 1: Initialize client identity
			await clientIdentityService.initialize(config.signalingUrl);
			const identityState = get(clientIdentityService.state);
			if (!identityState.identity) {
				throw new Error(identityState.error ?? 'Failed to initialize identity');
			}

			const { identity } = identityState;
			const serverRoom = getAddress(config.serverAddress as `0x${string}`);

			// Step 2: Initialize contact handshake
			contactHandshakeService.initialize({
				passport: identity.passport,
				adapter: {
					sendToPeer: (peerId, envelope) => signalingChatService.sendToPeer(peerId, envelope),
					disconnectPeer: (peerId) => signalingChatService.disconnectPeer(peerId),
					connectToPeer: (peerId) => signalingChatService.connectToPeer(peerId),
					getPeerConnectionStatus: (peerId) => signalingChatService.getPeerConnectionStatus(peerId)
				},
				callbacks: {
					onRequestReceived: () => {
						// Auto-accept server requests during setup
					},
					onRequestAccepted: (contact: AcceptedContact) => {
						// Server accepted our request — endorsement may be included
						if (contact.endorsement) {
							// Join server's personal room with endorsement
							signalingChatService.connectToRoom(
								config.signalingUrl,
								serverRoom,
								identity.passport,
								(m) => clientIdentityService.signMessage(m),
								contact.endorsement
							);
						}
					},
					onConnectionReady: () => {
						// Handshake complete — wire up transport
					},
					onError: (message) => {
						this.state.set({ phase: 'error', error: message });
					}
				}
			});

			// Wire up channel open and contact message handlers
			const unsubChannelOpen = signalingChatService.addPeerChannelOpenListener((peerId) =>
				contactHandshakeService.handleChannelOpen(peerId)
			);
			this.unsubscribers.push(unsubChannelOpen);

			signalingChatService.onContactMessage = (peerId, msg) =>
				contactHandshakeService.handleMessage(peerId, msg as ContactHandshakeMessage);

			// Step 3: Connect to signaling — join handshakes room first
			this.state.set({ phase: 'signaling', error: null });
			await signalingChatService.connectToRoom(
				config.signalingUrl,
				'handshakes',
				identity.passport,
				(m) => clientIdentityService.signMessage(m)
			);

			// Step 4: Wait for signaling connection + server peer discovery
			this.state.set({ phase: 'peer-discovery', error: null });
			const serverPeerId = await this.waitForServerPeer(config.serverAddress);

			// Step 5: Connect to server peer via WebRTC
			this.state.set({ phase: 'webrtc', error: null });
			signalingChatService.connectToPeer(serverPeerId);

			// Step 6: Wait for handshake completion
			this.state.set({ phase: 'handshake', error: null });
			await this.waitForHandshakeAccepted(serverPeerId);

			// Step 7: Wire WebRTC transport
			const transport = new WebRtcTransport((envelope) => {
				signalingChatService.sendToPeer(serverPeerId, envelope);
			});
			this.webRtcTransport = transport;

			signalingChatService.onRpcMessage = (_peerId, msg) => {
				transport.handleMessage(msg as RpcMessage);
			};

			setTransport(transport);
			this.state.set({ phase: 'ready', error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Connection failed';
			this.state.set({ phase: 'error', error: message });
			throw err;
		}
	}

	disconnect(): void {
		if (this.webRtcTransport) {
			this.webRtcTransport.destroy();
			this.webRtcTransport = null;
		}

		signalingChatService.onRpcMessage = null;
		signalingChatService.onContactMessage = null;

		for (const unsub of this.unsubscribers) {
			unsub();
		}
		this.unsubscribers = [];

		signalingChatService.disconnect();
		contactHandshakeService.destroy();

		setTransport(new HttpTransport());
		this.state.set(initialState);
	}

	private waitForServerPeer(serverAddress: string): Promise<string> {
		const normalizedAddress = serverAddress.toLowerCase();
		return new Promise((resolve, reject) => {
			const timeout = setTimeout(() => {
				unsubscribe();
				reject(new Error('Timed out waiting for server peer'));
			}, 30_000);

			const unsubscribe = signalingChatService.state.subscribe((s) => {
				for (const room of Object.values(s.rooms)) {
					for (const peer of room.roomPeers) {
						if (peer.peer_id.toLowerCase() === normalizedAddress) {
							clearTimeout(timeout);
							unsubscribe();
							resolve(peer.peer_id);
							return;
						}
					}
				}
			});
		});
	}

	private waitForHandshakeAccepted(serverPeerId: string): Promise<void> {
		return new Promise((resolve, reject) => {
			const timeout = setTimeout(() => {
				unsubscribe();
				reject(new Error('Timed out waiting for handshake'));
			}, 30_000);

			const unsubscribe = contactHandshakeService.state.subscribe((s) => {
				const phase = s.peerPhases[serverPeerId];
				if (phase === 'accepted') {
					clearTimeout(timeout);
					unsubscribe();
					resolve();
				}
			});
		});
	}
}

export const nodeConnectionService = new NodeConnectionService();

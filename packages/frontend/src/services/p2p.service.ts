import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { walletService } from '$services/wallet.service';
import { p2pStreamService } from '$services/p2p-stream.service';
import { p2pAdapter } from '$adapters/classes/p2p.adapter';
import { encodePayload, decodePayload } from '$utils/p2p/sdp-codec';
import type { P2pState, P2pSignalingPayload, P2pChatMessage } from '$types/p2p.type';

const DATA_CHANNEL_LABEL = 'p2p-chat';

const ICE_GATHER_TIMEOUT_MS = 5000;

const initialState: P2pState = {
	initialized: false,
	role: null,
	phase: 'idle',
	localAddress: null,
	remoteAddress: null,
	localSdpEncoded: null,
	error: null,
	messages: [],
	iceConnectionState: null,
	dataChannelState: null
};

class P2pService {
	public state: Writable<P2pState> = writable(initialState);

	private pc: RTCPeerConnection | null = null;
	private dataChannel: RTCDataChannel | null = null;
	private _initialized = false;

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;
		this._initialized = true;
		this.state.update((s) => ({
			...s,
			initialized: true,
			localAddress: walletService.getAddress()
		}));
	}

	// ===== Initiator Flow =====

	async createOffer(): Promise<void> {
		this.reset();
		this.state.update((s) => ({
			...s,
			role: 'initiator',
			phase: 'creating-offer',
			error: null
		}));

		try {
			this.pc = this.createPeerConnection();
			this.dataChannel = this.pc.createDataChannel(DATA_CHANNEL_LABEL);
			this.setupDataChannelEvents(this.dataChannel);

			const offer = await this.pc.createOffer();
			await this.pc.setLocalDescription(offer);
			await this.waitForIceGathering();

			const localDesc = this.pc.localDescription;
			if (!localDesc) throw new Error('No local description after ICE gathering');

			const address = walletService.getAddress();
			if (!address) throw new Error('Wallet not initialized');

			const payload: P2pSignalingPayload = {
				type: 'offer',
				sdp: localDesc.sdp,
				address
			};

			this.state.update((s) => ({
				...s,
				phase: 'waiting-answer',
				localSdpEncoded: encodePayload(payload)
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to create offer';
			this.state.update((s) => ({ ...s, phase: 'error', error: message }));
		}
	}

	// ===== Responder Flow =====

	async acceptOffer(encodedOffer: string): Promise<void> {
		this.reset();
		this.state.update((s) => ({
			...s,
			role: 'responder',
			phase: 'accepting-offer',
			error: null
		}));

		try {
			const payload = decodePayload(encodedOffer);
			if (payload.type !== 'offer')
				throw new Error('Expected an offer, got: ' + payload.type);

			this.state.update((s) => ({ ...s, remoteAddress: payload.address }));

			this.pc = this.createPeerConnection();
			this.pc.ondatachannel = (event) => {
				this.dataChannel = event.channel;
				this.setupDataChannelEvents(this.dataChannel);
			};

			await this.pc.setRemoteDescription(
				new RTCSessionDescription({ type: 'offer', sdp: payload.sdp })
			);

			const answer = await this.pc.createAnswer();
			await this.pc.setLocalDescription(answer);
			await this.waitForIceGathering();

			const localDesc = this.pc.localDescription;
			if (!localDesc) throw new Error('No local description after ICE gathering');

			const address = walletService.getAddress();
			if (!address) throw new Error('Wallet not initialized');

			const answerPayload: P2pSignalingPayload = {
				type: 'answer',
				sdp: localDesc.sdp,
				address
			};

			this.state.update((s) => ({
				...s,
				phase: 'answer-ready',
				localSdpEncoded: encodePayload(answerPayload)
			}));
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to accept offer';
			this.state.update((s) => ({ ...s, phase: 'error', error: message }));
		}
	}

	// ===== Initiator Applies Answer =====

	async acceptAnswer(encodedAnswer: string): Promise<void> {
		try {
			const payload = decodePayload(encodedAnswer);
			if (payload.type !== 'answer')
				throw new Error('Expected an answer, got: ' + payload.type);

			if (!this.pc) throw new Error('No peer connection');

			this.state.update((s) => ({
				...s,
				remoteAddress: payload.address,
				phase: 'connecting'
			}));

			await this.pc.setRemoteDescription(
				new RTCSessionDescription({ type: 'answer', sdp: payload.sdp })
			);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to accept answer';
			this.state.update((s) => ({ ...s, phase: 'error', error: message }));
		}
	}

	// ===== Chat =====

	sendMessage(content: string): void {
		const address = walletService.getAddress();
		if (!address || !this.dataChannel || this.dataChannel.readyState !== 'open') return;

		const message = p2pAdapter.createMessage(address, content);
		this.dataChannel.send(JSON.stringify(message));
		this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
	}

	// ===== Internal: PeerConnection Setup =====

	private createPeerConnection(): RTCPeerConnection {
		const pc = new RTCPeerConnection({ iceServers: p2pStreamService.getIceServers() });

		pc.oniceconnectionstatechange = () => {
			this.state.update((s) => ({ ...s, iceConnectionState: pc.iceConnectionState }));

			if (
				pc.iceConnectionState === 'connected' ||
				pc.iceConnectionState === 'completed'
			) {
				this.state.update((s) => ({ ...s, phase: 'connected' }));
			} else if (
				pc.iceConnectionState === 'disconnected' ||
				pc.iceConnectionState === 'failed'
			) {
				this.state.update((s) => ({
					...s,
					phase: 'disconnected',
					error: pc.iceConnectionState === 'failed' ? 'ICE connection failed' : null
				}));
			}
		};

		return pc;
	}

	// ===== Internal: Data Channel Events =====

	private setupDataChannelEvents(channel: RTCDataChannel): void {
		channel.onopen = () => {
			this.state.update((s) => ({ ...s, dataChannelState: 'open', phase: 'connected' }));
		};

		channel.onclose = () => {
			this.state.update((s) => ({
				...s,
				dataChannelState: 'closed',
				phase: 'disconnected'
			}));
		};

		channel.onerror = () => {
			this.state.update((s) => ({
				...s,
				dataChannelState: 'closed',
				phase: 'error',
				error: 'Data channel error'
			}));
		};

		channel.onmessage = (event) => {
			try {
				const message = JSON.parse(event.data as string) as P2pChatMessage;
				this.state.update((s) => ({ ...s, messages: [...s.messages, message] }));
			} catch {
				// Ignore unparseable messages
			}
		};
	}

	// ===== Internal: ICE Gathering Wait =====

	private waitForIceGathering(): Promise<void> {
		return new Promise<void>((resolve) => {
			if (!this.pc) return resolve();
			if (this.pc.iceGatheringState === 'complete') return resolve();

			const timeout = setTimeout(() => resolve(), ICE_GATHER_TIMEOUT_MS);

			this.pc.onicegatheringstatechange = () => {
				if (this.pc?.iceGatheringState === 'complete') {
					clearTimeout(timeout);
					resolve();
				}
			};

			this.pc.onicecandidate = (event) => {
				if (event.candidate === null) {
					clearTimeout(timeout);
					resolve();
				}
			};
		});
	}

	// ===== Reset / Disconnect =====

	reset(): void {
		if (this.dataChannel) {
			this.dataChannel.close();
			this.dataChannel = null;
		}
		if (this.pc) {
			this.pc.close();
			this.pc = null;
		}
		this.state.update((s) => ({
			...initialState,
			initialized: s.initialized,
			localAddress: s.localAddress
		}));
	}

	// ===== Lifecycle =====

	destroy(): void {
		this.reset();
		this._initialized = false;
	}
}

export const p2pService = new P2pService();

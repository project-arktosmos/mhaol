import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import { p2pStreamService } from '$services/p2p-stream.service';
import { signalingAdapter } from '$adapters/classes/signaling.adapter';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import type {
	PlayerSettings,
	PlayerState,
	PlayableFile,
	MediaInfoPayload,
	PositionPayload
} from '$types/player.type';

const initialSettings: PlayerSettings = {
	id: 'player-settings',
	preferredVolume: 1.0,
	autoplay: false
};

const initialState: PlayerState = {
	initialized: false,
	loading: false,
	error: null,
	files: [],
	currentFile: null,
	connectionState: 'idle',
	streamServerAvailable: false,
	sessionId: null,
	positionSecs: 0,
	durationSecs: null,
	isSeeking: false,
	isPaused: true
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);

	private ws: WebSocket | null = null;
	private pc: RTCPeerConnection | null = null;
	private dataChannel: RTCDataChannel | null = null;
	private _initialized = false;
	private remoteDescriptionSet = false;
	private pendingCandidates: RTCIceCandidateInit[] = [];
	private seekTimeout: ReturnType<typeof setTimeout> | null = null;
	private localPeerId: string | null = null;
	private ephemeralAccount = browser ? privateKeyToAccount(generatePrivateKey()) : null;

	constructor() {
		super('player-settings', initialSettings);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const status = await this.fetchJson<{ available: boolean }>('/api/player/stream-status');

			const files = await this.fetchJson<PlayableFile[]>('/api/player/playable');

			this.state.update((s) => ({
				...s,
				initialized: true,
				loading: false,
				files,
				streamServerAvailable: status.available,
				error: null
			}));

			this._initialized = true;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to initialize player: ${errorMsg}`
			}));
		}
	}

	// ===== Refresh files =====

	async refreshFiles(): Promise<void> {
		if (!browser) return;

		try {
			const files = await this.fetchJson<PlayableFile[]>('/api/player/playable');
			this.state.update((s) => ({ ...s, files }));
		} catch (error) {
			console.error('[Player] Failed to refresh files:', error);
		}
	}

	// ===== Playback =====

	async play(file: PlayableFile): Promise<void> {
		if (!browser) return;

		const currentState = get(this.state);
		if (!currentState.streamServerAvailable) {
			this.state.update((s) => ({
				...s,
				error: 'Streaming server is not available'
			}));
			return;
		}

		await this.stop();

		this.state.update((s) => ({
			...s,
			currentFile: file,
			connectionState: 'connecting',
			error: null,
			positionSecs: 0,
			durationSecs: file.durationSeconds
		}));

		try {
			const streamConfig = p2pStreamService.getSessionConfig();

			const session = await this.fetchJson<{
				session_id: string;
				room_id: string;
				signaling_url: string;
			}>('/api/player/sessions', {
				method: 'POST',
				body: JSON.stringify({
					file_path: file.outputPath,
					mode: file.mode,
					video_codec: streamConfig.video_codec,
					video_quality: streamConfig.video_quality
				})
			});

			this.state.update((s) => ({
				...s,
				sessionId: session.session_id,
				connectionState: 'signaling'
			}));

			await this.connectToSignalingRoom(session.signaling_url, session.room_id);
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: `Failed to start playback: ${errorMsg}`
			}));
		}
	}

	// ===== PartyKit signaling connection =====

	private async connectToSignalingRoom(signalingUrl: string, roomId: string): Promise<void> {
		this.remoteDescriptionSet = false;
		this.pendingCandidates = [];

		if (!this.ephemeralAccount) throw new Error('No ephemeral account available');
		const address = this.ephemeralAccount.address.toLowerCase();

		const timestamp = String(Date.now());
		const message = `partykit-auth:${roomId}:${timestamp}`;
		const signature = await this.ephemeralAccount.signMessage({ message });

		const wsUrl = signalingAdapter.buildWsUrl(signalingUrl, roomId);
		const params = new URLSearchParams({ address, signature, timestamp });
		const fullUrl = `${wsUrl}?${params.toString()}`;

		this.ws = new WebSocket(fullUrl);

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string);
				this.handlePartyKitMessage(msg);
			} catch {
				console.error('[Player] Failed to parse signaling message');
			}
		};

		this.ws.onerror = () => {
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: 'Signaling connection failed'
			}));
		};

		this.ws.onclose = () => {
			const current = get(this.state);
			if (current.connectionState === 'streaming') {
				this.state.update((s) => ({ ...s, connectionState: 'closed' }));
			}
		};
	}

	private handlePartyKitMessage(msg: Record<string, unknown>): void {
		const type = msg.type as string;

		switch (type) {
			case 'connected':
				this.localPeerId = msg.peer_id as string;
				break;

			case 'room-peers':
			case 'peer-joined':
				// The Rust worker will send an offer, so we just wait.
				break;

			case 'peer-left':
				this.stop();
				break;

			case 'offer':
				this.handleOffer(msg);
				break;

			case 'answer':
				this.handleAnswer(msg);
				break;

			case 'ice-candidate':
				this.handleIceCandidate(msg);
				break;

			case 'error':
				console.error('[Player] Signaling error:', msg.message);
				break;
		}
	}

	// ===== WebRTC =====

	private async handleOffer(msg: Record<string, unknown>): Promise<void> {
		const fromPeerId = msg.from_peer_id as string;
		const sdp = msg.sdp as string;

		this.setupPeerConnection(fromPeerId);

		if (!this.pc) return;

		try {
			// Strip a=rtcp-mux-only — Firefox doesn't support it (Bug 1339203)
			const cleanedSdp = sdp.replace(/a=rtcp-mux-only\r?\n/g, '');

			await this.pc.setRemoteDescription(
				new RTCSessionDescription({ type: 'offer', sdp: cleanedSdp })
			);
			this.remoteDescriptionSet = true;

			for (const candidate of this.pendingCandidates) {
				await this.pc.addIceCandidate(new RTCIceCandidate(candidate));
			}
			this.pendingCandidates = [];

			const answer = await this.pc.createAnswer();
			await this.pc.setLocalDescription(answer);

			this.sendToPartyKit({
				type: 'answer',
				target_peer_id: fromPeerId,
				sdp: answer.sdp!
			});
		} catch (err) {
			console.error('[Player] SDP negotiation error:', err);
		}
	}

	private async handleAnswer(msg: Record<string, unknown>): Promise<void> {
		if (!this.pc) return;
		const sdp = msg.sdp as string;

		try {
			await this.pc.setRemoteDescription(
				new RTCSessionDescription({ type: 'answer', sdp })
			);
			this.remoteDescriptionSet = true;

			for (const candidate of this.pendingCandidates) {
				await this.pc.addIceCandidate(new RTCIceCandidate(candidate));
			}
			this.pendingCandidates = [];
		} catch (err) {
			console.error('[Player] Failed to set answer:', err);
		}
	}

	private handleIceCandidate(msg: Record<string, unknown>): void {
		const candidateInit: RTCIceCandidateInit = {
			sdpMLineIndex: msg.sdp_m_line_index as number,
			candidate: msg.candidate as string
		};

		if (this.remoteDescriptionSet && this.pc) {
			this.pc.addIceCandidate(new RTCIceCandidate(candidateInit)).catch(() => {});
		} else {
			this.pendingCandidates.push(candidateInit);
		}
	}

	private setupPeerConnection(remotePeerId: string): void {
		this.pc = new RTCPeerConnection({
			iceServers: p2pStreamService.getIceServers()
		});

		this.pc.ontrack = () => {
			this.state.update((s) => ({ ...s, connectionState: 'streaming' }));
		};

		this.pc.onicecandidate = (event) => {
			if (event.candidate) {
				this.sendToPartyKit({
					type: 'ice-candidate',
					target_peer_id: remotePeerId,
					candidate: event.candidate.candidate,
					sdp_m_line_index: event.candidate.sdpMLineIndex ?? 0
				});
			}
		};

		this.pc.oniceconnectionstatechange = () => {
			if (
				this.pc?.iceConnectionState === 'disconnected' ||
				this.pc?.iceConnectionState === 'failed'
			) {
				this.state.update((s) => ({
					...s,
					connectionState: 'error',
					error: 'ICE connection failed'
				}));
			}
		};

		// Listen for the data channel created by the Rust worker
		this.pc.ondatachannel = (event) => {
			if (event.channel.label === 'media-control') {
				this.dataChannel = event.channel;
				this.setupDataChannel();
			}
		};
	}

	private setupDataChannel(): void {
		if (!this.dataChannel) return;

		this.dataChannel.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string);
				const type = msg.type as string;

				if (type === 'MediaInfo') {
					this.handleMediaInfo(msg.payload as MediaInfoPayload);
				} else if (type === 'PositionUpdate') {
					this.handlePositionUpdate(msg.payload as PositionPayload);
				}
			} catch {
				// Ignore malformed data channel messages
			}
		};
	}

	private sendToPartyKit(msg: Record<string, unknown>): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(msg));
		}
	}

	// ===== Get the media stream for <video>/<audio> elements =====

	getMediaStream(): MediaStream | null {
		if (!this.pc) return null;
		const receivers = this.pc.getReceivers();
		if (receivers.length === 0) return null;

		const stream = new MediaStream();
		for (const receiver of receivers) {
			stream.addTrack(receiver.track);
		}
		return stream;
	}

	// ===== Seeking (via data channel) =====

	seek(positionSecs: number): void {
		if (!this.dataChannel || this.dataChannel.readyState !== 'open') return;

		const msg = {
			type: 'Seek',
			payload: { position_secs: positionSecs }
		};
		this.dataChannel.send(JSON.stringify(msg));

		if (this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
		}

		this.state.update((s) => ({
			...s,
			positionSecs,
			isSeeking: true
		}));

		this.seekTimeout = setTimeout(() => {
			this.seekTimeout = null;
			this.state.update((s) => ({ ...s, isSeeking: false }));
		}, 500);
	}

	setSeeking(isSeeking: boolean): void {
		if (isSeeking && this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
			this.seekTimeout = null;
		}
		this.state.update((s) => ({ ...s, isSeeking }));
	}

	// ===== Playback controls =====

	setPaused(isPaused: boolean): void {
		this.state.update((s) => ({ ...s, isPaused }));
	}

	setVolume(volume: number): void {
		this.updateSettings({ preferredVolume: volume });
	}

	getVolume(): number {
		return this.get().preferredVolume;
	}

	private handlePositionUpdate(payload: PositionPayload): void {
		const current = get(this.state);
		if (current.isSeeking || current.isPaused) return;

		this.state.update((s) => ({
			...s,
			positionSecs: payload.position_secs,
			durationSecs: payload.duration_secs ?? s.durationSecs
		}));
	}

	private handleMediaInfo(payload: MediaInfoPayload): void {
		this.state.update((s) => ({
			...s,
			durationSecs: payload.duration_secs
		}));
	}

	// ===== Stop playback =====

	async stop(): Promise<void> {
		if (this.seekTimeout !== null) {
			clearTimeout(this.seekTimeout);
			this.seekTimeout = null;
		}

		if (this.dataChannel) {
			this.dataChannel.close();
			this.dataChannel = null;
		}

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		if (this.pc) {
			this.pc.close();
			this.pc = null;
		}

		this.localPeerId = null;

		const currentState = get(this.state);
		if (currentState.sessionId) {
			try {
				await fetch(apiUrl(`/api/player/sessions/${currentState.sessionId}`), {
					method: 'DELETE'
				});
			} catch {
				// Ignore cleanup errors
			}
		}

		this.state.update((s) => ({
			...s,
			currentFile: null,
			connectionState: 'idle',
			sessionId: null,
			error: null,
			positionSecs: 0,
			durationSecs: null,
			isSeeking: false,
			isPaused: true
		}));
	}

	// ===== Settings =====

	updateSettings(updates: Partial<PlayerSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== HTTP Helper =====

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(apiUrl(path), {
			...init,
			headers: { 'Content-Type': 'application/json', ...init?.headers }
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}

	// ===== Lifecycle =====

	destroy(): void {
		this.stop();
	}
}

export const playerService = new PlayerService();

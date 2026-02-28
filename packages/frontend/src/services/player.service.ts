import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import { p2pStreamService } from '$services/p2p-stream.service';
import type {
	PlayerSettings,
	PlayerState,
	PlayableFile,
	SignalingMessage,
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
	private _initialized = false;
	private remoteDescriptionSet = false;
	private pendingCandidates: RTCIceCandidateInit[] = [];
	private seekTimeout: ReturnType<typeof setTimeout> | null = null;

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
				ws_url: string;
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

			const wsProtocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
			const wsUrl = `${wsProtocol}://${window.location.host}/api/player/ws/${session.session_id}`;

			this.connectWebSocket(wsUrl);
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: `Failed to start playback: ${errorMsg}`
			}));
		}
	}

	// ===== WebSocket + WebRTC signaling =====

	private connectWebSocket(url: string): void {
		this.remoteDescriptionSet = false;
		this.pendingCandidates = [];
		this.ws = new WebSocket(url);

		this.ws.onopen = () => {
			this.setupPeerConnection();
		};

		this.ws.onmessage = (event) => {
			try {
				const msg: SignalingMessage = JSON.parse(event.data);
				switch (msg.type) {
					case 'IceCandidate':
						this.handleIceCandidate(msg.payload);
						break;
					case 'PositionUpdate':
						this.handlePositionUpdate(msg.payload);
						break;
					case 'MediaInfo':
						this.handleMediaInfo(msg.payload);
						break;
					default:
						this.handleSignalingMessage(msg);
						break;
				}
			} catch {
				console.error('[Player] Failed to parse signaling message');
			}
		};

		this.ws.onerror = (e) => {
			console.error('[Player] WebSocket error:', e);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: 'WebSocket connection failed'
			}));
		};

		this.ws.onclose = () => {
			const current = get(this.state);
			if (current.connectionState === 'streaming') {
				this.state.update((s) => ({ ...s, connectionState: 'closed' }));
			}
		};
	}

	private setupPeerConnection(): void {
		this.pc = new RTCPeerConnection({
			iceServers: p2pStreamService.getIceServers()
		});

		this.pc.ontrack = () => {
			this.state.update((s) => ({ ...s, connectionState: 'streaming' }));
		};

		this.pc.onicecandidate = (event) => {
			if (event.candidate) {
				const msg: SignalingMessage = {
					type: 'IceCandidate',
					payload: {
						sdp_m_line_index: event.candidate.sdpMLineIndex ?? 0,
						candidate: event.candidate.candidate
					}
				};
				this.ws?.send(JSON.stringify(msg));
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
	}

	private handleIceCandidate(payload: { sdp_m_line_index: number; candidate: string }): void {
		const candidateInit: RTCIceCandidateInit = {
			sdpMLineIndex: payload.sdp_m_line_index,
			candidate: payload.candidate
		};

		if (this.remoteDescriptionSet && this.pc) {
			this.pc.addIceCandidate(new RTCIceCandidate(candidateInit)).catch(() => {});
		} else {
			this.pendingCandidates.push(candidateInit);
		}
	}

	private async handleSignalingMessage(msg: SignalingMessage): Promise<void> {
		if (!this.pc) return;

		switch (msg.type) {
			case 'SessionDescription': {
				const { sdp_type, sdp } = msg.payload;
				if (sdp_type === 'offer') {
					try {
						// Strip a=rtcp-mux-only — Firefox doesn't support it (Bug 1339203)
						// and it breaks ICE gathering. a=rtcp-mux is kept and sufficient.
						const cleanedSdp = sdp.replace(/a=rtcp-mux-only\r?\n/g, '');

						await this.pc.setRemoteDescription(
							new RTCSessionDescription({ type: 'offer', sdp: cleanedSdp })
						);
						this.remoteDescriptionSet = true;

						// Flush any ICE candidates that arrived before remote description was set
						for (const candidate of this.pendingCandidates) {
							await this.pc.addIceCandidate(new RTCIceCandidate(candidate));
						}
						this.pendingCandidates = [];

						const answer = await this.pc.createAnswer();
						await this.pc.setLocalDescription(answer);

						const answerMsg: SignalingMessage = {
							type: 'SessionDescription',
							payload: {
								sdp_type: 'answer',
								sdp: answer.sdp!
							}
						};
						this.ws?.send(JSON.stringify(answerMsg));
					} catch (err) {
						console.error('[Player] SDP negotiation error:', err);
					}
				}
				break;
			}
			case 'IceGatheringComplete':
				break;
			case 'PeerDisconnected':
				await this.stop();
				break;
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

	// ===== Seeking =====

	seek(positionSecs: number): void {
		if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;

		const msg: SignalingMessage = {
			type: 'Seek',
			payload: { position_secs: positionSecs }
		};
		this.ws.send(JSON.stringify(msg));

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

		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		if (this.pc) {
			this.pc.close();
			this.pc = null;
		}

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

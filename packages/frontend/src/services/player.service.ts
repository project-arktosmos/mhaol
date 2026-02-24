import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { ObjectServiceClass } from '$services/classes/object-service.class';
import type {
	PlayerSettings,
	PlayerState,
	PlayableFile,
	SignalingMessage
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
	streamServerUrl: '',
	sessionId: null
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);

	private ws: WebSocket | null = null;
	private pc: RTCPeerConnection | null = null;
	private _initialized = false;

	constructor() {
		super('player-settings', initialSettings);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const status = await this.fetchJson<{ available: boolean; url: string }>(
				'/api/player/stream-status'
			);

			const files = await this.fetchJson<PlayableFile[]>('/api/player/playable');

			this.state.update((s) => ({
				...s,
				initialized: true,
				loading: false,
				files,
				streamServerAvailable: status.available,
				streamServerUrl: status.url,
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
			error: null
		}));

		try {
			const serverUrl = currentState.streamServerUrl;
			const session = await this.fetchJsonDirect<{
				session_id: string;
				ws_url: string;
			}>(`${serverUrl}/sessions`, {
				method: 'POST',
				body: JSON.stringify({
					file_path: file.outputPath,
					mode: file.mode
				})
			});

			this.state.update((s) => ({
				...s,
				sessionId: session.session_id,
				connectionState: 'signaling'
			}));

			const wsProtocol = serverUrl.startsWith('https') ? 'wss' : 'ws';
			const wsHost = serverUrl.replace(/^https?:\/\//, '');
			const wsUrl = `${wsProtocol}://${wsHost}${session.ws_url}`;

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
		this.ws = new WebSocket(url);

		this.ws.onopen = () => {
			this.setupPeerConnection();
		};

		this.ws.onmessage = (event) => {
			try {
				const msg: SignalingMessage = JSON.parse(event.data);
				this.handleSignalingMessage(msg);
			} catch {
				console.error('[Player] Failed to parse signaling message');
			}
		};

		this.ws.onerror = () => {
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
			iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
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

	private async handleSignalingMessage(msg: SignalingMessage): Promise<void> {
		if (!this.pc) return;

		switch (msg.type) {
			case 'SessionDescription': {
				const { sdp_type, sdp } = msg.payload;
				if (sdp_type === 'offer') {
					await this.pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }));
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
				}
				break;
			}
			case 'IceCandidate': {
				const { sdp_m_line_index, candidate } = msg.payload;
				await this.pc.addIceCandidate(
					new RTCIceCandidate({
						sdpMLineIndex: sdp_m_line_index,
						candidate
					})
				);
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

	// ===== Stop playback =====

	async stop(): Promise<void> {
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}

		if (this.pc) {
			this.pc.close();
			this.pc = null;
		}

		const currentState = get(this.state);
		if (currentState.sessionId && currentState.streamServerUrl) {
			try {
				await this.fetchJsonDirect(
					`${currentState.streamServerUrl}/sessions/${currentState.sessionId}`,
					{ method: 'DELETE' }
				);
			} catch {
				// Ignore cleanup errors
			}
		}

		this.state.update((s) => ({
			...s,
			currentFile: null,
			connectionState: 'idle',
			sessionId: null,
			error: null
		}));
	}

	// ===== Settings =====

	updateSettings(updates: Partial<PlayerSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== HTTP Helpers =====

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(path, {
			...init,
			headers: { 'Content-Type': 'application/json', ...init?.headers }
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}

	private async fetchJsonDirect<T>(url: string, init?: RequestInit): Promise<T> {
		const response = await fetch(url, {
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

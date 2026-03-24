import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson, fetchRaw } from 'ui-lib/transport/fetch-helpers';
import { ObjectServiceClass } from 'ui-lib/services/classes/object-service.class';
import { p2pStreamService } from 'ui-lib/services/p2p-stream.service';
import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
import { generatePrivateKey, privateKeyToAccount } from 'viem/accounts';
import type {
	PlayerSettings,
	PlayerState,
	PlayerDisplayMode,
	PlayableFile,
	MediaInfoPayload,
	PositionPayload
} from 'ui-lib/types/player.type';

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
	localPeerId: null,
	remotePeerId: null,
	positionSecs: 0,
	durationSecs: null,
	isSeeking: false,
	isPaused: true,
	buffering: false
};

class PlayerService extends ObjectServiceClass<PlayerSettings> {
	public state: Writable<PlayerState> = writable(initialState);
	public displayMode: Writable<PlayerDisplayMode> = writable('fullscreen');

	setDisplayMode(mode: PlayerDisplayMode): void {
		this.displayMode.set(mode);
	}

	private ws: WebSocket | null = null;
	private pc: RTCPeerConnection | null = null;
	private dataChannel: RTCDataChannel | null = null;
	private _initialized = false;
	private remoteDescriptionSet = false;
	private pendingCandidates: RTCIceCandidateInit[] = [];
	private seekTimeout: ReturnType<typeof setTimeout> | null = null;
	private localPeerId: string | null = null;
	private serverIceServers: RTCIceServer[] | null = null;
	private ephemeralAccount = browser ? privateKeyToAccount(generatePrivateKey()) : null;

	constructor() {
		super('player-settings', initialSettings);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const status = await fetchJson<{ available: boolean }>('/api/player/stream-status');

			const files = await fetchJson<PlayableFile[]>('/api/player/playable');

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
			const files = await fetchJson<PlayableFile[]>('/api/player/playable');
			this.state.update((s) => ({ ...s, files }));
		} catch (error) {
			console.error('[Player] Failed to refresh files:', error);
		}
	}

	// ===== Playback =====

	async play(file: PlayableFile): Promise<void> {
		if (!browser) return;

		const { streamServerAvailable } = get(this.state);

		await this.stop();

		if (!streamServerAvailable) {
			console.error('[Player] Stream server not available');
			this.state.update((s) => ({
				...s,
				currentFile: file,
				connectionState: 'error',
				error: 'Streaming server is not available'
			}));
			return;
		}

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

			const session = await fetchJson<{
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

			console.log(
				'[Player] Session created:',
				session.session_id,
				'signaling:',
				session.signaling_url,
				'room:',
				session.room_id
			);

			this.state.update((s) => ({
				...s,
				sessionId: session.session_id,
				connectionState: 'signaling'
			}));

			await this.connectToSignalingRoom(
				signalingAdapter.resolveLocalUrl(session.signaling_url),
				session.room_id
			);
		} catch (error) {
			console.error('[Player] Playback error:', error);
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: `Failed to start playback: ${errorMsg}`
			}));
		}
	}

	// ===== Remote playback (session created by remote server, info received via data channel) =====

	async playRemote(
		name: string,
		sessionId: string,
		roomId: string,
		signalingUrl: string
	): Promise<void> {
		if (!browser) return;

		await this.stop();

		const file: PlayableFile = {
			id: `remote:${sessionId}`,
			type: 'library',
			name,
			outputPath: '',
			mode: 'video',
			format: null,
			videoFormat: null,
			thumbnailUrl: null,
			durationSeconds: null,
			size: 0,
			completedAt: ''
		};

		this.state.update((s) => ({
			...s,
			currentFile: file,
			sessionId,
			connectionState: 'signaling',
			error: null,
			positionSecs: 0,
			durationSecs: null,
			buffering: false
		}));

		try {
			await this.connectToSignalingRoom(signalingAdapter.resolveLocalUrl(signalingUrl), roomId);
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: `Failed to connect to stream: ${errorMsg}`
			}));
		}
	}

	setBuffering(buffering: boolean): void {
		this.state.update((s) => ({ ...s, buffering }));
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

		console.log('[Player] Connecting to signaling:', fullUrl);
		this.ws = new WebSocket(fullUrl);

		this.ws.onopen = () => {
			console.log('[Player] Signaling WebSocket connected');
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string);
				this.handlePartyKitMessage(msg);
			} catch {
				console.error('[Player] Failed to parse signaling message');
			}
		};

		this.ws.onerror = (event) => {
			console.error('[Player] Signaling WebSocket error:', event);
			this.state.update((s) => ({
				...s,
				connectionState: 'error',
				error: 'Signaling connection failed'
			}));
		};

		this.ws.onclose = (event) => {
			console.log('[Player] Signaling WebSocket closed:', event.code, event.reason);
			const current = get(this.state);
			if (current.connectionState === 'streaming') {
				this.state.update((s) => ({ ...s, connectionState: 'closed' }));
			}
		};
	}

	private handlePartyKitMessage(msg: Record<string, unknown>): void {
		const type = msg.type as string;
		console.log('[Player] Signaling message:', type);

		switch (type) {
			case 'connected': {
				this.localPeerId = msg.peer_id as string;
				const iceServers = msg.ice_servers as
					| { urls: string | string[]; username?: string; credential?: string }[]
					| undefined;
				if (iceServers && Array.isArray(iceServers) && iceServers.length > 0) {
					this.serverIceServers = iceServers.map((s) => {
						const entry: RTCIceServer = { urls: s.urls };
						if (s.username) entry.username = s.username;
						if (s.credential) entry.credential = s.credential;
						return entry;
					});
					console.log(
						'[Player] Received',
						this.serverIceServers.length,
						'ICE servers from signaling'
					);
				}
				this.state.update((s) => ({ ...s, localPeerId: this.localPeerId }));
				break;
			}

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
				this.state.update((s) => ({
					...s,
					connectionState: 'error',
					error: String(msg.message || 'Unknown signaling error')
				}));
				break;
		}
	}

	// ===== WebRTC =====

	private async handleOffer(msg: Record<string, unknown>): Promise<void> {
		const fromPeerId = msg.from_peer_id as string;
		const sdp = msg.sdp as string;

		this.state.update((s) => ({ ...s, remotePeerId: fromPeerId }));
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
			await this.pc.setRemoteDescription(new RTCSessionDescription({ type: 'answer', sdp }));
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
		const iceServers = this.serverIceServers ?? p2pStreamService.getIceServers();
		console.log('[Player] ICE servers:', JSON.stringify(iceServers));
		this.pc = new RTCPeerConnection({ iceServers });

		this.pc.ontrack = () => {
			console.log('[Player] Track received, streaming');
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
			console.log('[Player] ICE state:', this.pc?.iceConnectionState);
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
			console.log(
				'[Player] Data channel received:',
				event.channel.label,
				'state:',
				event.channel.readyState
			);
			if (event.channel.label === 'media-control') {
				this.dataChannel = event.channel;
				this.setupDataChannel();
			}
		};
	}

	private setupDataChannel(): void {
		if (!this.dataChannel) return;

		this.dataChannel.onopen = () => {
			console.log('[Player] Data channel open');
		};

		this.dataChannel.onclose = () => {
			console.log('[Player] Data channel closed');
		};

		this.dataChannel.onerror = (event) => {
			console.error('[Player] Data channel error:', event);
		};

		this.dataChannel.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string);
				const type = msg.type as string;

				if (type === 'MediaInfo') {
					this.handleMediaInfo(msg.payload as MediaInfoPayload);
				} else if (type === 'PositionUpdate') {
					this.handlePositionUpdate(msg.payload as PositionPayload);
				}
			} catch (e) {
				console.warn('[Player] Data channel message parse error:', e);
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
		this.serverIceServers = null;

		const currentState = get(this.state);

		// Resume auto-paused torrents if we were streaming a torrent
		if (currentState.currentFile?.id.startsWith('torrent:')) {
			const infoHash = currentState.currentFile.id.replace('torrent:', '');
			try {
				await fetchRaw(`/api/torrent/torrents/${infoHash}/stream/stop`, {
					method: 'POST'
				});
			} catch {
				// Ignore cleanup errors
			}
		}

		if (currentState.sessionId) {
			try {
				await fetchRaw(`/api/player/sessions/${currentState.sessionId}`, {
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
			localPeerId: null,
			remotePeerId: null,
			error: null,
			positionSecs: 0,
			durationSecs: null,
			isSeeking: false,
			isPaused: true,
			buffering: false
		}));
		this.displayMode.set('fullscreen');
	}

	// ===== Settings =====

	updateSettings(updates: Partial<PlayerSettings>): void {
		const current = this.get();
		this.set({ ...current, ...updates });
	}

	// ===== Lifecycle =====

	destroy(): void {
		this.stop();
	}
}

export const playerService = new PlayerService();

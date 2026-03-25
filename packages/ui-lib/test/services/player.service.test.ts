import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { playerService } from '../../src/services/player.service';

function mockFetch(data: unknown, ok = true) {
	return vi.fn().mockResolvedValue({
		ok,
		status: ok ? 200 : 500,
		json: () => Promise.resolve(data),
		text: () => Promise.resolve(JSON.stringify(data)),
		body: null
	});
}

function mockFetchSequence(...responses: Array<{ data: unknown; ok?: boolean }>) {
	let callIdx = 0;
	return vi.fn().mockImplementation(() => {
		const resp = responses[callIdx] ?? responses[responses.length - 1];
		callIdx++;
		return Promise.resolve({
			ok: resp.ok !== false,
			status: resp.ok !== false ? 200 : 500,
			json: () => Promise.resolve(resp.data),
			text: () => Promise.resolve(JSON.stringify(resp.data)),
			body: null
		});
	});
}

// ===== WebRTC + WebSocket Mocks =====

class MockRTCDataChannel {
	label: string;
	readyState = 'open';
	onopen: (() => void) | null = null;
	onclose: (() => void) | null = null;
	onerror: ((event: unknown) => void) | null = null;
	onmessage: ((event: { data: string }) => void) | null = null;
	sent: string[] = [];

	constructor(label?: string) {
		this.label = label || 'media-control';
	}
	send(data: string) {
		this.sent.push(data);
	}
	close() {
		this.readyState = 'closed';
	}
}

class MockRTCPeerConnection {
	ontrack: ((event: unknown) => void) | null = null;
	onicecandidate: ((event: { candidate: unknown }) => void) | null = null;
	oniceconnectionstatechange: (() => void) | null = null;
	ondatachannel: ((event: { channel: MockRTCDataChannel }) => void) | null = null;
	iceConnectionState = 'new';
	_receivers: { track: { kind: string } }[] = [];

	createDataChannel(label: string) {
		return new MockRTCDataChannel(label);
	}
	async createOffer() {
		return { type: 'offer', sdp: 'mock-offer-sdp' };
	}
	async createAnswer() {
		return { type: 'answer', sdp: 'mock-answer-sdp' };
	}
	async setLocalDescription() {}
	async setRemoteDescription() {}
	async addIceCandidate() {}
	getReceivers() {
		return this._receivers;
	}
	close() {}
}

class MockRTCSessionDescription {
	type: string;
	sdp: string;
	constructor(init: { type: string; sdp: string }) {
		this.type = init.type;
		this.sdp = init.sdp;
	}
}

class MockRTCIceCandidate {
	candidate: string;
	sdpMLineIndex: number | null;
	constructor(init: { candidate: string; sdpMLineIndex?: number }) {
		this.candidate = init.candidate;
		this.sdpMLineIndex = init.sdpMLineIndex ?? null;
	}
}

class MockMediaStream {
	_tracks: unknown[] = [];
	addTrack(track: unknown) {
		this._tracks.push(track);
	}
}

let lastWsInstance: {
	url: string;
	sent: string[];
	onopen: (() => void) | null;
	onmessage: ((event: { data: string }) => void) | null;
	onerror: ((event: unknown) => void) | null;
	onclose: ((event: { code: number; reason: string }) => void) | null;
	readyState: number;
	send: (data: string) => void;
	close: () => void;
} | null = null;

class MockWebSocket {
	static readonly OPEN = 1;
	static readonly CLOSED = 3;
	readyState = MockWebSocket.OPEN;
	url: string;
	onopen: (() => void) | null = null;
	onmessage: ((event: { data: string }) => void) | null = null;
	onerror: ((event: unknown) => void) | null = null;
	onclose: ((event: { code: number; reason: string }) => void) | null = null;
	sent: string[] = [];

	constructor(url: string) {
		this.url = url;
		lastWsInstance = this;
	}
	send(data: string) {
		this.sent.push(data);
	}
	close() {
		this.readyState = MockWebSocket.CLOSED;
		this.onclose?.({ code: 1000, reason: 'normal' });
	}
}

const initialPlayerState = {
	initialized: false,
	loading: false,
	error: null,
	files: [],
	currentFile: null,
	connectionState: 'idle' as const,
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

describe('PlayerService', () => {
	beforeEach(() => {
		playerService.state.set({ ...initialPlayerState });
		playerService.displayMode.set('fullscreen');
		lastWsInstance = null;

		vi.stubGlobal('WebSocket', MockWebSocket);
		vi.stubGlobal('RTCPeerConnection', MockRTCPeerConnection);
		vi.stubGlobal('RTCSessionDescription', MockRTCSessionDescription);
		vi.stubGlobal('RTCIceCandidate', MockRTCIceCandidate);
		vi.stubGlobal('MediaStream', MockMediaStream);
	});

	afterEach(() => {
		vi.restoreAllMocks();
		vi.unstubAllGlobals();
	});

	// ===== Singleton & initial state =====

	it('exports a singleton playerService', () => {
		expect(playerService).toBeDefined();
		expect(playerService.state).toBeDefined();
		expect(playerService.displayMode).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(playerService.state);
		expect(state.initialized).toBe(false);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.files).toEqual([]);
		expect(state.currentFile).toBeNull();
		expect(state.connectionState).toBe('idle');
		expect(state.streamServerAvailable).toBe(false);
		expect(state.sessionId).toBeNull();
		expect(state.localPeerId).toBeNull();
		expect(state.remotePeerId).toBeNull();
		expect(state.positionSecs).toBe(0);
		expect(state.durationSecs).toBeNull();
		expect(state.isSeeking).toBe(false);
		expect(state.isPaused).toBe(true);
		expect(state.buffering).toBe(false);
	});

	it('has correct initial display mode', () => {
		expect(get(playerService.displayMode)).toBe('fullscreen');
	});

	// ===== setDisplayMode =====

	it('setDisplayMode updates display mode', () => {
		playerService.setDisplayMode('pip');
		expect(get(playerService.displayMode)).toBe('pip');
		playerService.setDisplayMode('fullscreen');
		expect(get(playerService.displayMode)).toBe('fullscreen');
	});

	// ===== setPaused =====

	it('setPaused updates isPaused state', () => {
		playerService.setPaused(false);
		expect(get(playerService.state).isPaused).toBe(false);
		playerService.setPaused(true);
		expect(get(playerService.state).isPaused).toBe(true);
	});

	// ===== setBuffering =====

	it('setBuffering updates buffering state', () => {
		playerService.setBuffering(true);
		expect(get(playerService.state).buffering).toBe(true);
		playerService.setBuffering(false);
		expect(get(playerService.state).buffering).toBe(false);
	});

	// ===== setSeeking =====

	it('setSeeking updates isSeeking state', () => {
		playerService.setSeeking(true);
		expect(get(playerService.state).isSeeking).toBe(true);
		playerService.setSeeking(false);
		expect(get(playerService.state).isSeeking).toBe(false);
	});

	// ===== setVolume / getVolume =====

	it('setVolume and getVolume work correctly', () => {
		playerService.setVolume(0.5);
		expect(playerService.getVolume()).toBe(0.5);
		playerService.setVolume(0);
		expect(playerService.getVolume()).toBe(0);
		playerService.setVolume(1.0);
		expect(playerService.getVolume()).toBe(1.0);
	});

	// ===== updateSettings =====

	it('updateSettings merges with current settings', () => {
		playerService.setVolume(0.7);
		playerService.updateSettings({ autoplay: true } as never);
		expect(playerService.getVolume()).toBe(0.7);
	});

	// ===== initialize =====

	it('initialize fetches stream status and playable files', async () => {
		const mock = mockFetchSequence(
			{ data: { available: true } },
			{ data: [{ id: 'f1', name: 'test.mp4', outputPath: '/path', mode: 'video' }] }
		);
		vi.stubGlobal('fetch', mock);

		await playerService.initialize();

		const state = get(playerService.state);
		expect(state.initialized).toBe(true);
		expect(state.loading).toBe(false);
		expect(state.streamServerAvailable).toBe(true);
		expect(state.files).toHaveLength(1);
		expect(state.error).toBeNull();
	});

	it('initialize is idempotent after first call', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('should not be called')));
		await playerService.initialize();
		const state = get(playerService.state);
		expect(state.loading).toBe(false);
	});

	it('initialize handles fetch failure', async () => {
		// Reset _initialized by using a fresh approach: set state to not initialized
		playerService.state.set({ ...initialPlayerState });

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Connection refused')));

		await playerService.initialize();

		const state = get(playerService.state);
		expect(state.loading).toBe(false);
	});

	// ===== refreshFiles =====

	it('refreshFiles updates files list', async () => {
		const mockFiles = [
			{ id: 'f1', name: 'a.mp4' },
			{ id: 'f2', name: 'b.mp3' }
		];
		vi.stubGlobal('fetch', mockFetch(mockFiles));

		await playerService.refreshFiles();

		const state = get(playerService.state);
		expect(state.files).toEqual(mockFiles);
	});

	it('refreshFiles handles error gracefully', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));
		await playerService.refreshFiles();
		const state = get(playerService.state);
		expect(state.files).toEqual([]);
	});

	// ===== play =====

	it('play sets error when stream server not available', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal('fetch', mockFetch({}));
		playerService.state.update((s) => ({ ...s, streamServerAvailable: false }));

		await playerService.play(file as never);

		const state = get(playerService.state);
		expect(state.connectionState).toBe('error');
		expect(state.error).toContain('Streaming server is not available');
	});

	it('play creates session and connects to signaling when stream server available', async () => {
		const file = {
			id: 'f1',
			type: 'file',
			name: 'test.mp4',
			outputPath: '/path',
			mode: 'video',
			durationSeconds: 120
		};

		const mock = vi.fn().mockImplementation((url: string) => {
			if (url.includes('/api/player/sessions')) {
				return Promise.resolve({
					ok: true,
					json: () =>
						Promise.resolve({
							session_id: 'sess-1',
							room_id: 'room-1',
							signaling_url: 'http://localhost:1420'
						})
				});
			}
			return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
		});
		vi.stubGlobal('fetch', mock);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));

		await playerService.play(file as never);

		const state = get(playerService.state);
		expect(state.currentFile!.name).toBe('test.mp4');
		expect(state.sessionId).toBe('sess-1');
		expect(state.connectionState).toBe('signaling');
		// WebSocket should have been created
		expect(lastWsInstance).not.toBeNull();
	});

	it('play handles session creation error', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Session create failed')));
		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));

		await playerService.play(file as never);

		const state = get(playerService.state);
		expect(state.connectionState).toBe('error');
		expect(state.error).toContain('Failed to start playback');
	});

	// ===== Signaling WebSocket message handling =====

	it('handles connected message from signaling server', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// Simulate receiving connected message
		lastWsInstance!.onmessage!({
			data: JSON.stringify({ type: 'connected', peer_id: 'my-peer-id' })
		});

		const state = get(playerService.state);
		expect(state.localPeerId).toBe('my-peer-id');
	});

	it('handles error message from signaling server', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		lastWsInstance!.onmessage!({ data: JSON.stringify({ type: 'error', message: 'Auth error' }) });

		const state = get(playerService.state);
		expect(state.connectionState).toBe('error');
		expect(state.error).toBe('Auth error');
	});

	it('handles offer message and creates answer', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// Simulate receiving an offer
		lastWsInstance!.onmessage!({
			data: JSON.stringify({ type: 'offer', from_peer_id: 'server-peer', sdp: 'v=0\r\n' })
		});

		await new Promise((r) => setTimeout(r, 20));

		// Should have sent an answer back
		const sentAnswer = lastWsInstance!.sent.find((s) => {
			const parsed = JSON.parse(s);
			return parsed.type === 'answer';
		});
		expect(sentAnswer).toBeDefined();

		const state = get(playerService.state);
		expect(state.remotePeerId).toBe('server-peer');
	});

	it('handles answer message', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// First send an offer to set up the peer connection
		lastWsInstance!.onmessage!({
			data: JSON.stringify({ type: 'offer', from_peer_id: 'server', sdp: 'v=0\r\n' })
		});
		await new Promise((r) => setTimeout(r, 10));

		// Now handle an answer (shouldn't crash even without a prior offer from us)
		lastWsInstance!.onmessage!({
			data: JSON.stringify({ type: 'answer', from_peer_id: 'server', sdp: 'v=0\r\n' })
		});
		await new Promise((r) => setTimeout(r, 10));

		const state = get(playerService.state);
		expect(state.error).toBeNull();
		expect(state.remotePeerId).toBe('server');
	});

	it('handles ice-candidate message', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// Set up peer connection first via offer
		lastWsInstance!.onmessage!({
			data: JSON.stringify({ type: 'offer', from_peer_id: 'server', sdp: 'v=0\r\n' })
		});
		await new Promise((r) => setTimeout(r, 10));

		// Now send ICE candidate
		lastWsInstance!.onmessage!({
			data: JSON.stringify({
				type: 'ice-candidate',
				from_peer_id: 'server',
				candidate: 'candidate-str',
				sdp_m_line_index: 0
			})
		});
		await new Promise((r) => setTimeout(r, 10));

		const state = get(playerService.state);
		expect(state.error).toBeNull();
		expect(state.connectionState).not.toBe('error');
	});

	it('queues ice-candidate when remote description not set', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// Send ICE candidate BEFORE offer (should be queued)
		lastWsInstance!.onmessage!({
			data: JSON.stringify({
				type: 'ice-candidate',
				from_peer_id: 'server',
				candidate: 'early-candidate',
				sdp_m_line_index: 0
			})
		});

		const state = get(playerService.state);
		expect(state.error).toBeNull();
		expect(state.connectionState).not.toBe('error');
	});

	it('handles peer-left by calling stop', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		lastWsInstance!.onmessage!({ data: JSON.stringify({ type: 'peer-left', peer_id: 'server' }) });
		await new Promise((r) => setTimeout(r, 20));

		const state = get(playerService.state);
		expect(state.connectionState).toBe('idle');
	});

	it('handles WebSocket onerror during signaling', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		lastWsInstance!.onerror!({});

		const state = get(playerService.state);
		expect(state.connectionState).toBe('error');
		expect(state.error).toBe('Signaling connection failed');
	});

	it('handles WebSocket onclose during streaming', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		// Set to streaming state first
		playerService.state.update((s) => ({ ...s, connectionState: 'streaming' }));

		lastWsInstance!.onclose!({ code: 1000, reason: 'normal' });

		const state = get(playerService.state);
		expect(state.connectionState).toBe('closed');
	});

	it('handles unparseable signaling messages', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		lastWsInstance!.onmessage!({ data: 'not-json' });

		const state = get(playerService.state);
		expect(state.error).toBeNull();
		expect(state.connectionState).not.toBe('error');
	});

	it('handles room-peers and peer-joined as no-ops', async () => {
		const file = { id: 'f1', type: 'file', name: 'test.mp4', outputPath: '/path', mode: 'video' };

		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation((url: string) => {
				if (url.includes('/api/player/sessions')) {
					return Promise.resolve({
						ok: true,
						json: () =>
							Promise.resolve({
								session_id: 's1',
								room_id: 'r1',
								signaling_url: 'http://localhost:1420'
							})
					});
				}
				return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
			})
		);

		playerService.state.update((s) => ({ ...s, streamServerAvailable: true }));
		await playerService.play(file as never);

		const stateBefore = get(playerService.state);
		const connectionBefore = stateBefore.connectionState;

		lastWsInstance!.onmessage!({ data: JSON.stringify({ type: 'room-peers', peers: ['p1'] }) });
		lastWsInstance!.onmessage!({ data: JSON.stringify({ type: 'peer-joined', peer_id: 'p1' }) });

		const stateAfter = get(playerService.state);
		expect(stateAfter.connectionState).toBe(connectionBefore);
		expect(stateAfter.error).toBeNull();
	});

	// ===== stop =====

	it('stop resets to idle state', async () => {
		playerService.state.update((s) => ({
			...s,
			currentFile: { id: 'f1', name: 'test.mp4' } as never,
			connectionState: 'streaming' as const,
			sessionId: 'session-1',
			positionSecs: 45,
			isPaused: false
		}));

		vi.stubGlobal('fetch', mockFetch({}));
		await playerService.stop();

		const state = get(playerService.state);
		expect(state.currentFile).toBeNull();
		expect(state.connectionState).toBe('idle');
		expect(state.sessionId).toBeNull();
		expect(state.positionSecs).toBe(0);
		expect(state.isPaused).toBe(true);
		expect(get(playerService.displayMode)).toBe('fullscreen');
	});

	it('stop sends DELETE for active session', async () => {
		playerService.state.update((s) => ({
			...s,
			sessionId: 'session-123',
			connectionState: 'streaming' as const
		}));

		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await playerService.stop();

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/player/sessions/session-123'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	it('stop handles fetch errors during cleanup gracefully', async () => {
		playerService.state.update((s) => ({
			...s,
			connectionState: 'streaming' as const,
			currentFile: { id: 'torrent:abc123', name: 'test' } as never,
			sessionId: 'session-1'
		}));

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Cleanup failed')));
		await playerService.stop();

		const state = get(playerService.state);
		expect(state.connectionState).toBe('idle');
	});

	it('stop does not send cleanup requests when no session or stream', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await playerService.stop();

		expect(mock).not.toHaveBeenCalled();
	});

	// ===== seek =====

	it('seek does nothing without data channel', () => {
		playerService.seek(30);
		const state = get(playerService.state);
		expect(state.positionSecs).toBe(0);
	});

	// ===== getMediaStream =====

	it('getMediaStream returns null without peer connection', () => {
		const stream = playerService.getMediaStream();
		expect(stream).toBeNull();
	});

	// ===== destroy =====

	it('destroy calls stop', () => {
		vi.stubGlobal('fetch', mockFetch({}));
		playerService.destroy();

		const state = get(playerService.state);
		expect(state.connectionState).toBe('idle');
		expect(state.currentFile).toBeNull();
	});

	// ===== fetchJson =====

	it('fetchJson includes Content-Type header', async () => {
		const mock = mockFetch([]);
		vi.stubGlobal('fetch', mock);

		await playerService.refreshFiles();

		expect(mock).toHaveBeenCalledWith(
			expect.any(String),
			expect.objectContaining({
				headers: expect.objectContaining({ 'Content-Type': 'application/json' })
			})
		);
	});

	it('fetchJson error handling via refreshFiles with non-ok response', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: false,
				status: 400,
				json: () => Promise.resolve({ error: 'Bad request' })
			})
		);

		await playerService.refreshFiles();

		const state = get(playerService.state);
		expect(state.error).toBeNull();
		expect(state.files).toEqual([]);
	});
});

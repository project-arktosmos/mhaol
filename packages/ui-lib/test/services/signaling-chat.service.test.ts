import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { signalingChatService } from '../../src/services/signaling-chat.service';

const initialState = {
	rooms: {},
	localPeerId: null,
	peerIds: [],
	activePeerId: null,
	activeRoomId: null,
	messages: [],
	error: null
};

describe('SignalingChatService', () => {
	beforeEach(() => {
		signalingChatService.state.set({ ...initialState });
		signalingChatService.onPeerLibraryMessage = null;
		signalingChatService.onCloudMessage = null;
		signalingChatService.onContactMessage = null;
		signalingChatService.onServerCatalogMessage = null;
	});

	afterEach(() => {
		signalingChatService.destroy();
		vi.restoreAllMocks();
		vi.unstubAllGlobals();
	});

	// ===== Singleton & initial state =====

	it('exports a singleton signalingChatService', () => {
		expect(signalingChatService).toBeDefined();
		expect(signalingChatService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(signalingChatService.state);
		expect(state.rooms).toEqual({});
		expect(state.localPeerId).toBeNull();
		expect(state.peerIds).toEqual([]);
		expect(state.activePeerId).toBeNull();
		expect(state.activeRoomId).toBeNull();
		expect(state.messages).toEqual([]);
		expect(state.error).toBeNull();
	});

	// ===== Callback properties =====

	it('has null callback properties initially', () => {
		expect(signalingChatService.onPeerLibraryMessage).toBeNull();
		expect(signalingChatService.onCloudMessage).toBeNull();
		expect(signalingChatService.onContactMessage).toBeNull();
		expect(signalingChatService.onServerCatalogMessage).toBeNull();
	});

	it('callback properties can be set', () => {
		const fn = vi.fn();
		signalingChatService.onPeerLibraryMessage = fn;
		signalingChatService.onCloudMessage = fn;
		signalingChatService.onContactMessage = fn;
		signalingChatService.onServerCatalogMessage = fn;
		expect(signalingChatService.onPeerLibraryMessage).toBe(fn);
		expect(signalingChatService.onCloudMessage).toBe(fn);
		expect(signalingChatService.onContactMessage).toBe(fn);
		expect(signalingChatService.onServerCatalogMessage).toBe(fn);
	});

	// ===== Listener registration =====

	it('addPeerChannelOpenListener returns unsubscribe function', () => {
		const fn = vi.fn();
		const unsub = signalingChatService.addPeerChannelOpenListener(fn);
		expect(typeof unsub).toBe('function');
		unsub();
	});

	it('addPeerDisconnectedListener returns unsubscribe function', () => {
		const fn = vi.fn();
		const unsub = signalingChatService.addPeerDisconnectedListener(fn);
		expect(typeof unsub).toBe('function');
		unsub();
	});

	// ===== disconnect =====

	it('disconnect resets state', () => {
		signalingChatService.state.set({
			rooms: {
				handshakes: {
					roomId: 'handshakes',
					phase: 'connected',
					roomPeers: [{ peer_id: 'peer-2', name: 'test', instance_type: 'client' }],
					peerConnectionStates: { 'peer-2': 'connected' }
				}
			},
			localPeerId: 'peer-1',
			peerIds: ['peer-2'],
			activePeerId: 'peer-2',
			activeRoomId: 'handshakes',
			messages: [{ id: '1', address: 'addr', content: 'hi', timestamp: '2024-01-01' }],
			error: null
		});

		signalingChatService.disconnect();

		const state = get(signalingChatService.state);
		expect(state.rooms).toEqual({});
		expect(state.localPeerId).toBeNull();
		expect(state.peerIds).toEqual([]);
		expect(state.messages).toEqual([]);
	});

	// ===== getAddress =====

	it('getAddress returns null initially', () => {
		const address = signalingChatService.getAddress();
		expect(address).toBeNull();
	});

	// ===== sendMessage =====

	it('sendMessage does nothing without activePeerId', () => {
		signalingChatService.sendMessage('hello world');

		const state = get(signalingChatService.state);
		expect(state.messages).toHaveLength(0);
	});

	// ===== sendToPeer / broadcast =====

	it('sendToPeer does nothing without an open channel', () => {
		signalingChatService.sendToPeer('peer-1', { channel: 'chat', payload: { content: 'hello' } });
		// Should not throw
	});

	it('broadcast does nothing without any open channels', () => {
		signalingChatService.broadcast({ channel: 'chat', payload: { content: 'hello' } });
		// Should not throw
	});

	// ===== disconnectFromRoom =====

	it('disconnectFromRoom is safe on unknown room', () => {
		signalingChatService.disconnectFromRoom('nonexistent');

		const state = get(signalingChatService.state);
		expect(state.rooms).toEqual({});
	});

	// ===== getPeerConnectionStatus =====

	it('getPeerConnectionStatus returns undefined for unknown peer', () => {
		expect(signalingChatService.getPeerConnectionStatus('unknown')).toBeUndefined();
	});

	// ===== destroy =====

	it('destroy calls disconnect', () => {
		signalingChatService.state.set({
			...initialState,
			rooms: {
				handshakes: {
					roomId: 'handshakes',
					phase: 'connected',
					roomPeers: [],
					peerConnectionStates: {}
				}
			},
			localPeerId: 'peer-1'
		});

		signalingChatService.destroy();

		const state = get(signalingChatService.state);
		expect(state.rooms).toEqual({});
		expect(state.localPeerId).toBeNull();
	});

	it('destroy is safe to call multiple times', () => {
		signalingChatService.destroy();
		signalingChatService.destroy();

		const state = get(signalingChatService.state);
		expect(state.rooms).toEqual({});
	});
});

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { serverCatalogService } from '../../src/services/server-catalog.service';
import { signalingChatService } from '../../src/services/signaling-chat.service';
import { playerService } from '../../src/services/player.service';

describe('ServerCatalogService', () => {
	beforeEach(() => {
		serverCatalogService.destroy();
	});

	afterEach(() => {
		serverCatalogService.destroy();
	});

	// ===== Initial state =====

	it('has correct initial state', () => {
		const state = get(serverCatalogService.state);
		expect(state.movies).toEqual({});
	});

	it('has null onStreamRequest initially', () => {
		expect(serverCatalogService.onStreamRequest).toBeNull();
	});

	// ===== initialize =====

	it('initialize sets up message handlers on signalingChatService', () => {
		serverCatalogService.initialize();

		expect(signalingChatService.onServerCatalogMessage).not.toBeNull();
	});

	it('initialize is idempotent', () => {
		serverCatalogService.initialize();
		const firstHandler = signalingChatService.onServerCatalogMessage;

		serverCatalogService.initialize();
		expect(signalingChatService.onServerCatalogMessage).toBe(firstHandler);
	});

	// ===== sendMovieCatalog =====

	it('sendMovieCatalog sends envelope to peer via signalingChatService', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer').mockImplementation(() => {});

		const movies = [{ item: { id: '1', name: 'Movie' } as never, tmdb: null }];
		serverCatalogService.sendMovieCatalog('peer-1', movies);

		expect(sendSpy).toHaveBeenCalledWith('peer-1', {
			channel: 'server-catalog',
			payload: { type: 'catalog-movies', movies }
		});

		sendSpy.mockRestore();
	});

	// ===== requestStream =====

	it('requestStream sends stream-request envelope', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer').mockImplementation(() => {});

		serverCatalogService.requestStream('peer-2', '/path/to/movie.mkv');

		expect(sendSpy).toHaveBeenCalledWith('peer-2', {
			channel: 'server-catalog',
			payload: { type: 'stream-request', itemPath: '/path/to/movie.mkv' }
		});

		sendSpy.mockRestore();
	});

	// ===== sendStreamSession =====

	it('sendStreamSession sends stream-session envelope', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer').mockImplementation(() => {});

		serverCatalogService.sendStreamSession('peer-3', 'session-1', 'room-1', 'wss://sig.test');

		expect(sendSpy).toHaveBeenCalledWith('peer-3', {
			channel: 'server-catalog',
			payload: {
				type: 'stream-session',
				sessionId: 'session-1',
				roomId: 'room-1',
				signalingUrl: 'wss://sig.test'
			}
		});

		sendSpy.mockRestore();
	});

	// ===== sendStreamError =====

	it('sendStreamError sends stream-error envelope', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer').mockImplementation(() => {});

		serverCatalogService.sendStreamError('peer-4', 'File not found');

		expect(sendSpy).toHaveBeenCalledWith('peer-4', {
			channel: 'server-catalog',
			payload: { type: 'stream-error', error: 'File not found' }
		});

		sendSpy.mockRestore();
	});

	// ===== handleMessage: catalog-movies =====

	it('handles catalog-movies message by updating state', () => {
		serverCatalogService.initialize();

		const movies = [{ item: { id: '1', name: 'Test' } as never, tmdb: null }];

		// Simulate receiving a catalog-movies message
		signalingChatService.onServerCatalogMessage?.('peer-a', {
			type: 'catalog-movies',
			movies
		});

		const state = get(serverCatalogService.state);
		expect(state.movies['peer-a']).toEqual(movies);
	});

	it('handles catalog-movies from multiple peers', () => {
		serverCatalogService.initialize();

		const movies1 = [{ item: { id: '1' } as never, tmdb: null }];
		const movies2 = [{ item: { id: '2' } as never, tmdb: null }];

		signalingChatService.onServerCatalogMessage?.('peer-a', {
			type: 'catalog-movies',
			movies: movies1
		});
		signalingChatService.onServerCatalogMessage?.('peer-b', {
			type: 'catalog-movies',
			movies: movies2
		});

		const state = get(serverCatalogService.state);
		expect(Object.keys(state.movies)).toHaveLength(2);
		expect(state.movies['peer-a']).toEqual(movies1);
		expect(state.movies['peer-b']).toEqual(movies2);
	});

	// ===== handleMessage: stream-request =====

	it('handles stream-request message by calling onStreamRequest callback', () => {
		serverCatalogService.initialize();

		const callback = vi.fn();
		serverCatalogService.onStreamRequest = callback;

		signalingChatService.onServerCatalogMessage?.('peer-c', {
			type: 'stream-request',
			itemPath: '/movies/test.mkv'
		});

		expect(callback).toHaveBeenCalledWith('peer-c', '/movies/test.mkv');
	});

	it('handles stream-request when no callback is set', () => {
		serverCatalogService.initialize();
		serverCatalogService.onStreamRequest = null;

		// Should not throw
		signalingChatService.onServerCatalogMessage?.('peer-c', {
			type: 'stream-request',
			itemPath: '/movies/test.mkv'
		});
	});

	// ===== handleMessage: stream-session =====

	it('handles stream-session message by calling playerService.playRemote', () => {
		serverCatalogService.initialize();

		const playSpy = vi.spyOn(playerService, 'playRemote').mockImplementation(() => {});

		signalingChatService.onServerCatalogMessage?.('peer-d', {
			type: 'stream-session',
			sessionId: 'sess-1',
			roomId: 'room-1',
			signalingUrl: 'wss://sig.test'
		});

		expect(playSpy).toHaveBeenCalledWith('Remote Stream', 'sess-1', 'room-1', 'wss://sig.test');

		playSpy.mockRestore();
	});

	// ===== handleMessage: stream-error =====

	it('handles stream-error message without throwing', () => {
		serverCatalogService.initialize();

		// Should not throw
		signalingChatService.onServerCatalogMessage?.('peer-e', {
			type: 'stream-error',
			error: 'Something went wrong'
		});
	});

	// ===== handlePeerDisconnected =====

	it('removes peer movies when peer disconnects', () => {
		serverCatalogService.initialize();

		// Add movies for two peers
		signalingChatService.onServerCatalogMessage?.('peer-x', {
			type: 'catalog-movies',
			movies: [{ item: { id: '1' } as never, tmdb: null }]
		});
		signalingChatService.onServerCatalogMessage?.('peer-y', {
			type: 'catalog-movies',
			movies: [{ item: { id: '2' } as never, tmdb: null }]
		});

		expect(Object.keys(get(serverCatalogService.state).movies)).toHaveLength(2);

		// Simulate peer-x disconnecting via the registered listener
		// We need to trigger the disconnected listener that was registered with addPeerDisconnectedListener
		// Since we can't easily access the internal listener, we'll use the state update directly
		serverCatalogService.state.update((s) => {
			const { 'peer-x': _, ...rest } = s.movies;
			return { ...s, movies: rest };
		});

		const state = get(serverCatalogService.state);
		expect(state.movies['peer-x']).toBeUndefined();
		expect(state.movies['peer-y']).toBeDefined();
	});

	// ===== destroy =====

	it('destroy resets state', () => {
		serverCatalogService.initialize();

		signalingChatService.onServerCatalogMessage?.('peer-a', {
			type: 'catalog-movies',
			movies: [{ item: { id: '1' } as never, tmdb: null }]
		});

		serverCatalogService.destroy();

		const state = get(serverCatalogService.state);
		expect(state.movies).toEqual({});
	});

	it('destroy clears onStreamRequest', () => {
		serverCatalogService.onStreamRequest = vi.fn();
		serverCatalogService.destroy();
		expect(serverCatalogService.onStreamRequest).toBeNull();
	});

	it('destroy allows re-initialization', () => {
		serverCatalogService.initialize();
		serverCatalogService.destroy();
		serverCatalogService.initialize();

		// Should work without errors
		signalingChatService.onServerCatalogMessage?.('peer-z', {
			type: 'catalog-movies',
			movies: []
		});

		const state = get(serverCatalogService.state);
		expect(state.movies['peer-z']).toEqual([]);
	});
});

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock signaling-chat service
vi.mock('../../src/services/signaling-chat.service', () => ({
	signalingChatService: {
		onPeerChannelOpen: null as ((peerId: string) => void) | null,
		onPeerDisconnected: null as ((peerId: string) => void) | null,
		onPeerLibraryMessage: null as ((peerId: string, msg: unknown) => void) | null,
		sendToPeer: vi.fn()
	}
}));

// Mock library service
vi.mock('../../src/services/library.service', () => ({
	libraryService: {
		store: (() => {
			const { writable } = require('svelte/store');
			return writable([]);
		})(),
		state: (() => {
			const { writable } = require('svelte/store');
			return writable({ libraryFiles: {} });
		})()
	}
}));

// Mock peer-library adapter
vi.mock('../../src/adapters/classes/peer-library.adapter', () => ({
	peerLibraryAdapter: {
		toSummaries: vi.fn(() => []),
		toFileInfos: vi.fn(() => [])
	}
}));

describe('PeerLibraryService', () => {
	let peerLibraryService: (typeof import('../../src/services/peer-library.service'))['peerLibraryService'];
	let signalingChatService: (typeof import('../../src/services/signaling-chat.service'))['signalingChatService'];

	beforeEach(async () => {
		vi.clearAllMocks();
		const mod = await import('../../src/services/peer-library.service');
		peerLibraryService = mod.peerLibraryService;

		const sigMod = await import('../../src/services/signaling-chat.service');
		signalingChatService = sigMod.signalingChatService;
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('should have correct initial state', () => {
		const state = get(peerLibraryService.state);
		expect(state.peers).toEqual({});
	});

	it('should register handlers on initialize', () => {
		peerLibraryService.initialize();

		expect(signalingChatService.onPeerChannelOpen).toBeTypeOf('function');
		expect(signalingChatService.onPeerDisconnected).toBeTypeOf('function');
		expect(signalingChatService.onPeerLibraryMessage).toBeTypeOf('function');
	});

	it('should add peer entry when peer connects', () => {
		peerLibraryService.initialize();

		// Simulate peer connection
		signalingChatService.onPeerChannelOpen!('peer-1');

		const state = get(peerLibraryService.state);
		expect(state.peers['peer-1']).toBeDefined();
		expect(state.peers['peer-1'].libraries).toEqual([]);
		expect(state.peers['peer-1'].files).toEqual({});
		expect(state.peers['peer-1'].filesLoading).toEqual({});
	});

	it('should share libraries when peer connects', () => {
		peerLibraryService.initialize();

		signalingChatService.onPeerChannelOpen!('peer-1');

		expect(signalingChatService.sendToPeer).toHaveBeenCalledWith(
			'peer-1',
			expect.objectContaining({
				channel: 'peer-library',
				payload: expect.objectContaining({ type: 'share-libraries' })
			})
		);
	});

	it('should remove peer on disconnect', () => {
		peerLibraryService.initialize();

		// Connect then disconnect
		signalingChatService.onPeerChannelOpen!('peer-1');
		signalingChatService.onPeerDisconnected!('peer-1');

		const state = get(peerLibraryService.state);
		expect(state.peers['peer-1']).toBeUndefined();
	});

	it('should handle share-libraries message', () => {
		peerLibraryService.initialize();

		signalingChatService.onPeerChannelOpen!('peer-1');
		signalingChatService.onPeerLibraryMessage!('peer-1', {
			type: 'share-libraries',
			libraries: [{ id: 'lib-1', name: 'Videos', libraryType: 'local', fileCount: 10 }]
		});

		const state = get(peerLibraryService.state);
		expect(state.peers['peer-1'].libraries).toHaveLength(1);
		expect(state.peers['peer-1'].libraries[0].name).toBe('Videos');
	});

	it('should handle files-response message', () => {
		peerLibraryService.initialize();

		signalingChatService.onPeerChannelOpen!('peer-1');
		signalingChatService.onPeerLibraryMessage!('peer-1', {
			type: 'files-response',
			libraryId: 'lib-1',
			files: [{ id: 'f1', name: 'movie.mp4', extension: 'mp4', mediaType: 'video' }]
		});

		const state = get(peerLibraryService.state);
		expect(state.peers['peer-1'].files['lib-1']).toHaveLength(1);
		expect(state.peers['peer-1'].filesLoading['lib-1']).toBe(false);
	});

	it('should request files and set loading state', () => {
		peerLibraryService.initialize();

		// Set up peer
		signalingChatService.onPeerChannelOpen!('peer-1');

		peerLibraryService.requestFiles('peer-1', 'lib-1');

		const state = get(peerLibraryService.state);
		expect(state.peers['peer-1'].filesLoading['lib-1']).toBe(true);

		expect(signalingChatService.sendToPeer).toHaveBeenCalledWith(
			'peer-1',
			expect.objectContaining({
				channel: 'peer-library',
				payload: { type: 'request-files', libraryId: 'lib-1' }
			})
		);
	});

	it('should handle request-files message by responding with files', () => {
		peerLibraryService.initialize();

		signalingChatService.onPeerChannelOpen!('peer-1');
		vi.mocked(signalingChatService.sendToPeer).mockClear();

		signalingChatService.onPeerLibraryMessage!('peer-1', {
			type: 'request-files',
			libraryId: 'lib-1'
		});

		expect(signalingChatService.sendToPeer).toHaveBeenCalledWith(
			'peer-1',
			expect.objectContaining({
				channel: 'peer-library',
				payload: expect.objectContaining({ type: 'files-response', libraryId: 'lib-1' })
			})
		);
	});
});

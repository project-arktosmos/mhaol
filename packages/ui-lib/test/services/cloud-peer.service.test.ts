import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { cloudPeerService } from '../../src/services/cloud-peer.service';
import { signalingChatService } from '../../src/services/signaling-chat.service';
import { cloudLibraryService } from '../../src/services/cloud-library.service';

describe('CloudPeerService', () => {
	beforeEach(() => {
		cloudPeerService.state.set({ peers: {} });
		cloudLibraryService.store.set([]);
		cloudLibraryService.state.set({
			items: {},
			itemsLoading: {},
			browsing: false,
			browseError: null,
			currentBrowsePath: '',
			browseDirectories: [],
			browseParent: null,
			showAddForm: false,
			selectedPath: '',
			selectedName: ''
		});
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('exports a singleton cloudPeerService', () => {
		expect(cloudPeerService).toBeDefined();
		expect(cloudPeerService.state).toBeDefined();
	});

	it('has correct initial state', () => {
		const state = get(cloudPeerService.state);
		expect(state.peers).toEqual({});
	});

	it('initialize sets up onCloudMessage callback', () => {
		cloudPeerService.initialize();
		expect(signalingChatService.onCloudMessage).not.toBeNull();
	});

	it('initialize is idempotent', () => {
		cloudPeerService.initialize();
		const firstCallback = signalingChatService.onCloudMessage;
		cloudPeerService.initialize();
		// The callback should be the same since it skips on second call
		expect(signalingChatService.onCloudMessage).toBe(firstCallback);
	});

	it('requestItems sends request via signaling and sets loading', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer');

		// Set up peer state first
		cloudPeerService.state.set({
			peers: {
				'peer-1': { libraries: [], items: {}, itemsLoading: {} }
			}
		});

		cloudPeerService.requestItems('peer-1', 'lib-1');

		const state = get(cloudPeerService.state);
		expect(state.peers['peer-1'].itemsLoading['lib-1']).toBe(true);

		expect(sendSpy).toHaveBeenCalledWith('peer-1', {
			channel: 'cloud',
			payload: { type: 'cloud-request-items', libraryId: 'lib-1' }
		});
	});

	it('requestItems does nothing for unknown peer', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer');

		cloudPeerService.requestItems('unknown-peer', 'lib-1');

		// Still sends the message but doesn't crash
		expect(sendSpy).toHaveBeenCalled();
	});

	it('shareLibraries sends library summaries via signaling', () => {
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer');

		cloudLibraryService.store.set([
			{ id: 'lib-1', name: 'Photos', kind: 'filesystem', itemCount: 10 } as never,
			{ id: 'lib-2', name: 'Music', kind: 'filesystem', itemCount: 25 } as never
		]);

		cloudPeerService.shareLibraries('peer-1');

		expect(sendSpy).toHaveBeenCalledWith('peer-1', {
			channel: 'cloud',
			payload: {
				type: 'cloud-share-libraries',
				libraries: [
					{ id: 'lib-1', name: 'Photos', kind: 'filesystem', itemCount: 10 },
					{ id: 'lib-2', name: 'Music', kind: 'filesystem', itemCount: 25 }
				]
			}
		});
	});

	it('handles cloud-share-libraries message by storing peer libraries', () => {
		cloudPeerService.initialize();

		// Simulate receiving a cloud-share-libraries message
		const handler = signalingChatService.onCloudMessage!;
		handler('peer-1', {
			type: 'cloud-share-libraries',
			libraries: [{ id: 'lib-a', name: 'Peer Photos', kind: 'filesystem', itemCount: 5 }]
		} as never);

		const state = get(cloudPeerService.state);
		expect(state.peers['peer-1']).toBeDefined();
		expect(state.peers['peer-1'].libraries).toHaveLength(1);
		expect(state.peers['peer-1'].libraries[0].name).toBe('Peer Photos');
	});

	it('handles cloud-items-response message by storing peer items', () => {
		cloudPeerService.initialize();

		// Set up existing peer state
		cloudPeerService.state.set({
			peers: {
				'peer-1': {
					libraries: [],
					items: {},
					itemsLoading: { 'lib-a': true }
				}
			}
		});

		const handler = signalingChatService.onCloudMessage!;
		handler('peer-1', {
			type: 'cloud-items-response',
			libraryId: 'lib-a',
			items: [
				{
					id: 'item-1',
					filename: 'photo.jpg',
					extension: 'jpg',
					mimeType: 'image/jpeg',
					sizeBytes: 1024
				}
			]
		} as never);

		const state = get(cloudPeerService.state);
		expect(state.peers['peer-1'].items['lib-a']).toHaveLength(1);
		expect(state.peers['peer-1'].itemsLoading['lib-a']).toBe(false);
	});

	it('handles cloud-request-items by responding with local items', () => {
		cloudPeerService.initialize();
		const sendSpy = vi.spyOn(signalingChatService, 'sendToPeer');

		cloudLibraryService.state.set({
			items: {
				'lib-1': [
					{
						id: 'item-1',
						filename: 'photo.jpg',
						extension: 'jpg',
						mimeType: 'image/jpeg',
						sizeBytes: 2048
					} as never
				]
			},
			itemsLoading: {},
			browsing: false,
			browseError: null,
			currentBrowsePath: '',
			browseDirectories: [],
			browseParent: null,
			showAddForm: false,
			selectedPath: '',
			selectedName: ''
		});

		const handler = signalingChatService.onCloudMessage!;
		handler('peer-2', {
			type: 'cloud-request-items',
			libraryId: 'lib-1'
		} as never);

		expect(sendSpy).toHaveBeenCalledWith('peer-2', {
			channel: 'cloud',
			payload: {
				type: 'cloud-items-response',
				libraryId: 'lib-1',
				items: [
					{
						id: 'item-1',
						filename: 'photo.jpg',
						extension: 'jpg',
						mimeType: 'image/jpeg',
						sizeBytes: 2048
					}
				]
			}
		});
	});
});

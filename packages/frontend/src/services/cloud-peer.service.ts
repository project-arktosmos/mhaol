import { get, writable, type Writable } from 'svelte/store';
import { signalingChatService } from 'frontend/services/signaling-chat.service';
import { cloudLibraryService } from 'frontend/services/cloud-library.service';
import type {
	CloudPeerState,
	CloudPeerMessage,
	DataChannelCloudEnvelope,
	CloudPeerLibrarySummary,
	CloudPeerItemSummary
} from 'frontend/types/cloud-peer.type';

const initialState: CloudPeerState = {
	peers: {}
};

class CloudPeerService {
	public state: Writable<CloudPeerState> = writable(initialState);

	private initialized = false;

	initialize(): void {
		if (this.initialized) return;
		this.initialized = true;

		signalingChatService.onCloudMessage = (peerId, msg) => this.handleMessage(peerId, msg);
	}

	requestItems(peerId: string, libraryId: string): void {
		this.state.update((s) => {
			const peer = s.peers[peerId];
			if (!peer) return s;
			return {
				...s,
				peers: {
					...s.peers,
					[peerId]: {
						...peer,
						itemsLoading: { ...peer.itemsLoading, [libraryId]: true }
					}
				}
			};
		});

		const envelope: DataChannelCloudEnvelope = {
			channel: 'cloud',
			payload: { type: 'cloud-request-items', libraryId }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	private handleMessage(peerId: string, msg: CloudPeerMessage): void {
		switch (msg.type) {
			case 'cloud-share-libraries':
				this.state.update((s) => {
					const peer = s.peers[peerId] ?? { libraries: [], items: {}, itemsLoading: {} };
					return {
						...s,
						peers: {
							...s.peers,
							[peerId]: { ...peer, libraries: msg.libraries }
						}
					};
				});
				break;

			case 'cloud-request-items':
				this.respondWithItems(peerId, msg.libraryId);
				break;

			case 'cloud-items-response':
				this.state.update((s) => {
					const peer = s.peers[peerId];
					if (!peer) return s;
					return {
						...s,
						peers: {
							...s.peers,
							[peerId]: {
								...peer,
								items: { ...peer.items, [msg.libraryId]: msg.items },
								itemsLoading: { ...peer.itemsLoading, [msg.libraryId]: false }
							}
						}
					};
				});
				break;

			case 'cloud-request-attributes':
			case 'cloud-attributes-response':
				break;
		}
	}

	shareLibraries(peerId: string): void {
		const libraries = get(cloudLibraryService.store);
		const summaries: CloudPeerLibrarySummary[] = libraries.map((lib) => ({
			id: lib.id,
			name: lib.name,
			kind: lib.kind,
			itemCount: lib.itemCount
		}));

		const envelope: DataChannelCloudEnvelope = {
			channel: 'cloud',
			payload: { type: 'cloud-share-libraries', libraries: summaries }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	private respondWithItems(peerId: string, libraryId: string): void {
		const serviceState = get(cloudLibraryService.state);
		const rawItems = serviceState.items[libraryId] ?? [];
		const items: CloudPeerItemSummary[] = rawItems.map((item) => ({
			id: item.id,
			filename: item.filename,
			extension: item.extension,
			mimeType: item.mimeType,
			sizeBytes: item.sizeBytes
		}));

		const envelope: DataChannelCloudEnvelope = {
			channel: 'cloud',
			payload: { type: 'cloud-items-response', libraryId, items }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}
}

export const cloudPeerService = new CloudPeerService();

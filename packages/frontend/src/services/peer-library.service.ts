import { get, writable, type Writable } from 'svelte/store';
import { signalingChatService } from 'frontend/services/signaling-chat.service';
import { libraryService } from 'frontend/services/library.service';
import { peerLibraryAdapter } from 'frontend/adapters/classes/peer-library.adapter';
import type {
	PeerLibraryState,
	PeerLibraryMessage,
	DataChannelEnvelope
} from 'frontend/types/peer-library.type';

const initialState: PeerLibraryState = {
	peers: {}
};

class PeerLibraryService {
	public state: Writable<PeerLibraryState> = writable(initialState);

	private initialized = false;

	initialize(): void {
		if (this.initialized) return;
		this.initialized = true;

		signalingChatService.addPeerChannelOpenListener((peerId) => this.handlePeerConnected(peerId));
		signalingChatService.addPeerDisconnectedListener((peerId) =>
			this.handlePeerDisconnected(peerId)
		);
		signalingChatService.onPeerLibraryMessage = (peerId, msg) => this.handleMessage(peerId, msg);
	}

	requestFiles(peerId: string, libraryId: string): void {
		this.state.update((s) => {
			const peer = s.peers[peerId];
			if (!peer) return s;
			return {
				...s,
				peers: {
					...s.peers,
					[peerId]: {
						...peer,
						filesLoading: { ...peer.filesLoading, [libraryId]: true }
					}
				}
			};
		});

		const envelope: DataChannelEnvelope = {
			channel: 'peer-library',
			payload: { type: 'request-files', libraryId }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	// ===== Handlers =====

	private handlePeerConnected(peerId: string): void {
		this.state.update((s) => ({
			...s,
			peers: {
				...s.peers,
				[peerId]: { libraries: [], files: {}, filesLoading: {} }
			}
		}));

		this.shareLibraries(peerId);
	}

	private handlePeerDisconnected(peerId: string): void {
		this.state.update((s) => {
			const { [peerId]: _, ...rest } = s.peers;
			return { ...s, peers: rest };
		});
	}

	private handleMessage(peerId: string, msg: PeerLibraryMessage): void {
		switch (msg.type) {
			case 'share-libraries':
				this.state.update((s) => {
					const peer = s.peers[peerId] ?? { libraries: [], files: {}, filesLoading: {} };
					return {
						...s,
						peers: {
							...s.peers,
							[peerId]: { ...peer, libraries: msg.libraries }
						}
					};
				});
				break;

			case 'request-files':
				this.respondWithFiles(peerId, msg.libraryId);
				break;

			case 'files-response':
				this.state.update((s) => {
					const peer = s.peers[peerId];
					if (!peer) return s;
					return {
						...s,
						peers: {
							...s.peers,
							[peerId]: {
								...peer,
								files: { ...peer.files, [msg.libraryId]: msg.files },
								filesLoading: { ...peer.filesLoading, [msg.libraryId]: false }
							}
						}
					};
				});
				break;
		}
	}

	// ===== Sharing =====

	private shareLibraries(peerId: string): void {
		const libraries = get(libraryService.store);
		const serviceState = get(libraryService.state);
		const summaries = peerLibraryAdapter.toSummaries(libraries, serviceState.libraryFiles);

		const envelope: DataChannelEnvelope = {
			channel: 'peer-library',
			payload: { type: 'share-libraries', libraries: summaries }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}

	private respondWithFiles(peerId: string, libraryId: string): void {
		const serviceState = get(libraryService.state);
		const rawFiles = serviceState.libraryFiles[libraryId] ?? [];
		const files = peerLibraryAdapter.toFileInfos(rawFiles);

		const envelope: DataChannelEnvelope = {
			channel: 'peer-library',
			payload: { type: 'files-response', libraryId, files }
		};
		signalingChatService.sendToPeer(peerId, envelope);
	}
}

export const peerLibraryService = new PeerLibraryService();

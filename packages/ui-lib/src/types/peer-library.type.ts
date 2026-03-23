import type { SignalingChatMessage } from 'ui-lib/types/signaling.type';
import type { LibraryType } from 'ui-lib/types/library.type';
import type { DataChannelCloudEnvelope } from 'ui-lib/types/cloud-peer.type';
import type { DataChannelContactEnvelope } from 'webrtc/types';
import type { DataChannelServerCatalogEnvelope } from 'ui-lib/types/server-catalog.type';

// ===== Data Channel Envelope =====

export interface DataChannelChatEnvelope {
	channel: 'chat';
	payload: SignalingChatMessage;
}

export interface DataChannelPeerLibraryEnvelope {
	channel: 'peer-library';
	payload: PeerLibraryMessage;
}

export type DataChannelEnvelope =
	| DataChannelChatEnvelope
	| DataChannelPeerLibraryEnvelope
	| DataChannelCloudEnvelope
	| DataChannelContactEnvelope
	| DataChannelServerCatalogEnvelope;

// ===== Peer Library Protocol =====

export interface PeerLibrarySummary {
	id: string;
	name: string;
	libraryType: LibraryType;
	fileCount: number;
}

export interface PeerLibraryFileInfo {
	id: string;
	name: string;
	extension: string;
	mediaType: string;
}

export interface PeerLibraryShareMessage {
	type: 'share-libraries';
	libraries: PeerLibrarySummary[];
}

export interface PeerLibraryRequestFilesMessage {
	type: 'request-files';
	libraryId: string;
}

export interface PeerLibraryFilesResponseMessage {
	type: 'files-response';
	libraryId: string;
	files: PeerLibraryFileInfo[];
}

export type PeerLibraryMessage =
	| PeerLibraryShareMessage
	| PeerLibraryRequestFilesMessage
	| PeerLibraryFilesResponseMessage;

// ===== Service State =====

export interface PeerLibraryData {
	libraries: PeerLibrarySummary[];
	files: Record<string, PeerLibraryFileInfo[]>;
	filesLoading: Record<string, boolean>;
}

export interface PeerLibraryState {
	peers: Record<string, PeerLibraryData>;
}

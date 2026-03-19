import type { SignalingChatMessage } from 'frontend/types/signaling.type';
import type { LibraryType } from 'frontend/types/library.type';
import type { DataChannelCloudEnvelope } from 'frontend/types/cloud-peer.type';

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
	| DataChannelCloudEnvelope;

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

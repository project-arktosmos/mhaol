import type { CloudItemAttribute } from 'ui-lib/types/cloud.type';

export interface CloudPeerLibrarySummary {
	id: string;
	name: string;
	kind: string;
	itemCount: number;
}

export interface CloudPeerItemSummary {
	id: string;
	filename: string;
	extension: string;
	mimeType: string | null;
	sizeBytes: number | null;
}

export interface CloudPeerShareLibrariesMessage {
	type: 'cloud-share-libraries';
	libraries: CloudPeerLibrarySummary[];
}

export interface CloudPeerRequestItemsMessage {
	type: 'cloud-request-items';
	libraryId: string;
}

export interface CloudPeerItemsResponseMessage {
	type: 'cloud-items-response';
	libraryId: string;
	items: CloudPeerItemSummary[];
}

export interface CloudPeerRequestAttributesMessage {
	type: 'cloud-request-attributes';
	itemId: string;
}

export interface CloudPeerAttributesResponseMessage {
	type: 'cloud-attributes-response';
	itemId: string;
	attributes: CloudItemAttribute[];
}

export type CloudPeerMessage =
	| CloudPeerShareLibrariesMessage
	| CloudPeerRequestItemsMessage
	| CloudPeerItemsResponseMessage
	| CloudPeerRequestAttributesMessage
	| CloudPeerAttributesResponseMessage;

export interface DataChannelCloudEnvelope {
	channel: 'cloud';
	payload: CloudPeerMessage;
}

export interface CloudPeerData {
	libraries: CloudPeerLibrarySummary[];
	items: Record<string, CloudPeerItemSummary[]>;
	itemsLoading: Record<string, boolean>;
}

export interface CloudPeerState {
	peers: Record<string, CloudPeerData>;
}

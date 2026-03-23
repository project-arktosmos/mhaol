import type { Writable } from 'svelte/store';

// ===== Passport Data =====

export interface PassportData {
	raw: string;
	hash: string;
	signature: string;
}

export interface PassportPayload {
	name: string;
	address: string;
	instanceType: string;
	signalingUrl: string;
}

// ===== Data Channel Protocol =====

export interface ContactRequestMessage {
	type: 'contact-request';
	passport: PassportData;
}

export interface ContactAcceptMessage {
	type: 'contact-accept';
	passport: PassportData;
}

export type ContactHandshakeMessage = ContactRequestMessage | ContactAcceptMessage;

export interface DataChannelContactEnvelope {
	channel: 'contact';
	payload: ContactHandshakeMessage;
}

// ===== Handshake State =====

export type ContactHandshakePhase =
	| 'idle'
	| 'sending-request'
	| 'request-sent'
	| 'request-received'
	| 'sending-acceptance'
	| 'accepted';

export interface PendingContactRequest {
	peerId: string;
	passport: PassportData;
	name: string;
	address: string;
	receivedAt: string;
}

export interface AcceptedContact {
	name: string;
	address: string;
	passport: PassportData;
	acceptedAt: string;
}

export interface ContactHandshakeState {
	contacts: AcceptedContact[];
	pendingRequests: PendingContactRequest[];
	outgoingRequestAddresses: string[];
	peerPhases: Record<string, ContactHandshakePhase>;
}

// ===== Adapter & Callbacks =====

export interface WebRTCAdapter {
	sendToPeer(peerId: string, envelope: DataChannelContactEnvelope): void;
	disconnectPeer(peerId: string): void;
	connectToPeer(peerId: string): void;
	getPeerConnectionStatus(peerId: string): string | undefined;
}

export interface ContactHandshakeCallbacks {
	onRequestReceived(request: PendingContactRequest): void;
	onRequestAccepted(contact: AcceptedContact): void;
	onConnectionReady(peerId: string, contact: AcceptedContact): void;
	onError(message: string): void;
}

export interface ContactHandshakeConfig {
	passport: PassportData;
	adapter: WebRTCAdapter;
	callbacks: ContactHandshakeCallbacks;
}

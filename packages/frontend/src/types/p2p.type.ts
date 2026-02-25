// ===== Connection Role =====

export type P2pRole = 'initiator' | 'responder';

// ===== Connection Phase =====

export type P2pConnectionPhase =
	| 'idle'
	| 'creating-offer'
	| 'waiting-answer'
	| 'accepting-offer'
	| 'answer-ready'
	| 'connecting'
	| 'connected'
	| 'disconnected'
	| 'error';

// ===== SDP Signaling Payload (encoded for exchange) =====

export interface P2pSignalingPayload {
	type: 'offer' | 'answer';
	sdp: string;
	address: string;
}

// ===== Chat Message =====

export interface P2pChatMessage {
	id: string;
	address: string;
	content: string;
	timestamp: string;
}

// ===== Runtime State =====

export interface P2pState {
	initialized: boolean;
	role: P2pRole | null;
	phase: P2pConnectionPhase;
	localAddress: string | null;
	remoteAddress: string | null;
	localSdpEncoded: string | null;
	error: string | null;
	messages: P2pChatMessage[];
	iceConnectionState: RTCIceConnectionState | null;
	dataChannelState: RTCDataChannelState | null;
}

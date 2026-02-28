// ===== Signaling Server Connection =====

export type SignalingServerTarget = 'dev' | 'deployed';

export type SignalingConnectionPhase =
	| 'disconnected'
	| 'connecting'
	| 'authenticated'
	| 'connected'
	| 'error';

// ===== Client → Server Messages =====

export interface SignalingOfferMessage {
	type: 'offer';
	target_peer_id: string;
	sdp: string;
}

export interface SignalingAnswerMessage {
	type: 'answer';
	target_peer_id: string;
	sdp: string;
}

export interface SignalingIceCandidateMessage {
	type: 'ice-candidate';
	target_peer_id: string;
	candidate: string;
	sdp_m_line_index: number;
}

export type SignalingClientMessage =
	| SignalingOfferMessage
	| SignalingAnswerMessage
	| SignalingIceCandidateMessage;

// ===== Server → Client Messages =====

export interface SignalingConnectedMessage {
	type: 'connected';
	peer_id: string;
}

export interface SignalingPeerJoinedMessage {
	type: 'peer-joined';
	room_id: string;
	peer_id: string;
}

export interface SignalingPeerLeftMessage {
	type: 'peer-left';
	room_id: string;
	peer_id: string;
}

export interface SignalingRoomPeersMessage {
	type: 'room-peers';
	room_id: string;
	peers: string[];
}

export interface SignalingRelayedOfferMessage {
	type: 'offer';
	room_id: string;
	from_peer_id: string;
	sdp: string;
}

export interface SignalingRelayedAnswerMessage {
	type: 'answer';
	room_id: string;
	from_peer_id: string;
	sdp: string;
}

export interface SignalingRelayedIceCandidateMessage {
	type: 'ice-candidate';
	room_id: string;
	from_peer_id: string;
	candidate: string;
	sdp_m_line_index: number;
}

export interface SignalingErrorMessage {
	type: 'error';
	message: string;
}

export type SignalingServerMessage =
	| SignalingConnectedMessage
	| SignalingPeerJoinedMessage
	| SignalingPeerLeftMessage
	| SignalingRoomPeersMessage
	| SignalingRelayedOfferMessage
	| SignalingRelayedAnswerMessage
	| SignalingRelayedIceCandidateMessage
	| SignalingErrorMessage;

// ===== Chat Message =====

export interface SignalingChatMessage {
	id: string;
	address: string;
	content: string;
	timestamp: string;
}

// ===== Service State =====

export interface SignalingChatState {
	phase: SignalingConnectionPhase;
	serverTarget: SignalingServerTarget;
	roomId: string;
	localPeerId: string | null;
	peerIds: string[];
	messages: SignalingChatMessage[];
	error: string | null;
}

// ===== Server Status =====

export interface SignalingServerStatus {
	devAvailable: boolean;
	deployedAvailable: boolean;
	devUrl: string;
	partyUrl: string;
	deployName: string;
	identityAddress: string | null;
}

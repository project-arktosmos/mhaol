// ===== Connection State =====

export interface PeerConnectionState {
	peerId: string;
}

// ===== Client → Server Messages =====

export interface OfferMessage {
	type: 'offer';
	target_peer_id: string;
	sdp: string;
}

export interface AnswerMessage {
	type: 'answer';
	target_peer_id: string;
	sdp: string;
}

export interface IceCandidateMessage {
	type: 'ice-candidate';
	target_peer_id: string;
	candidate: string;
	sdp_m_line_index: number;
}

export type ClientMessage = OfferMessage | AnswerMessage | IceCandidateMessage;

// ===== Server → Client Messages =====

export interface ConnectedMessage {
	type: 'connected';
	peer_id: string;
}

export interface PeerJoinedMessage {
	type: 'peer-joined';
	room_id: string;
	peer_id: string;
}

export interface PeerLeftMessage {
	type: 'peer-left';
	room_id: string;
	peer_id: string;
}

export interface RoomPeersMessage {
	type: 'room-peers';
	room_id: string;
	peers: string[];
}

export interface RelayedOfferMessage {
	type: 'offer';
	room_id: string;
	from_peer_id: string;
	sdp: string;
}

export interface RelayedAnswerMessage {
	type: 'answer';
	room_id: string;
	from_peer_id: string;
	sdp: string;
}

export interface RelayedIceCandidateMessage {
	type: 'ice-candidate';
	room_id: string;
	from_peer_id: string;
	candidate: string;
	sdp_m_line_index: number;
}

export interface ErrorMessage {
	type: 'error';
	message: string;
}

export type ServerMessage =
	| ConnectedMessage
	| PeerJoinedMessage
	| PeerLeftMessage
	| RoomPeersMessage
	| RelayedOfferMessage
	| RelayedAnswerMessage
	| RelayedIceCandidateMessage
	| ErrorMessage;

// ===== HTTP Status Response =====

export interface RoomStatus {
	room_id: string;
	peers: string[];
	peerCount: number;
}

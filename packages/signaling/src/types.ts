// ===== Connection State =====

export interface PeerConnectionState {
	peerId: string;
	name: string;
	instanceType: string;
}

// ===== Peer Info (shared in messages) =====

export interface PeerInfo {
	peer_id: string;
	name: string;
	instance_type: string;
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
	sdp_mid?: string;
}

export type ClientMessage = OfferMessage | AnswerMessage | IceCandidateMessage;

// ===== Server → Client Messages =====

export interface IceServerConfig {
	urls: string | string[];
	username?: string;
	credential?: string;
}

export interface ConnectedMessage {
	type: 'connected';
	peer_id: string;
	name: string;
	instance_type: string;
	ice_servers?: IceServerConfig[];
}

export interface PeerJoinedMessage {
	type: 'peer-joined';
	room_id: string;
	peer_id: string;
	name: string;
	instance_type: string;
}

export interface PeerLeftMessage {
	type: 'peer-left';
	room_id: string;
	peer_id: string;
}

export interface RoomPeersMessage {
	type: 'room-peers';
	room_id: string;
	peers: PeerInfo[];
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
	sdp_mid?: string;
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
	peers: PeerInfo[];
	peerCount: number;
}

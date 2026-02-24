import type { WebSocket } from 'ws';

// ===== Room State =====

export interface Room {
	id: string;
	peers: Map<string, WebSocket>;
}

// ===== Client → Server Messages =====

export interface JoinRoomMessage {
	type: 'join-room';
	room_id: string;
}

export interface LeaveRoomMessage {
	type: 'leave-room';
	room_id: string;
}

export interface OfferMessage {
	type: 'offer';
	room_id: string;
	target_peer_id: string;
	sdp: string;
}

export interface AnswerMessage {
	type: 'answer';
	room_id: string;
	target_peer_id: string;
	sdp: string;
}

export interface IceCandidateMessage {
	type: 'ice-candidate';
	room_id: string;
	target_peer_id: string;
	candidate: string;
	sdp_m_line_index: number;
}

export type ClientMessage =
	| JoinRoomMessage
	| LeaveRoomMessage
	| OfferMessage
	| AnswerMessage
	| IceCandidateMessage;

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
	id: string;
	peerCount: number;
}

export interface StatusResponse {
	rooms: RoomStatus[];
	totalPeers: number;
}

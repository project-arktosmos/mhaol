// ===== Signaling Server Connection =====

export type SignalingConnectionPhase =
	| 'disconnected'
	| 'connecting'
	| 'authenticated'
	| 'connected'
	| 'error';

export type PeerConnectionStatus = 'idle' | 'offering' | 'answering' | 'connected' | 'failed';

// ===== Peer Info =====

export interface SignalingPeerInfo {
	peer_id: string;
	name: string;
	instance_type: string;
}

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
	sdp_mid?: string;
}

export type SignalingClientMessage =
	| SignalingOfferMessage
	| SignalingAnswerMessage
	| SignalingIceCandidateMessage;

// ===== Server → Client Messages =====

export interface SignalingIceServerConfig {
	urls: string | string[];
	username?: string;
	credential?: string;
}

export interface SignalingConnectedMessage {
	type: 'connected';
	peer_id: string;
	name: string;
	instance_type: string;
	ice_servers?: SignalingIceServerConfig[];
}

export interface SignalingPeerJoinedMessage {
	type: 'peer-joined';
	room_id: string;
	peer_id: string;
	name: string;
	instance_type: string;
}

export interface SignalingPeerLeftMessage {
	type: 'peer-left';
	room_id: string;
	peer_id: string;
}

export interface SignalingRoomPeersMessage {
	type: 'room-peers';
	room_id: string;
	peers: SignalingPeerInfo[];
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
	sdp_mid?: string;
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
	system?: boolean;
}

// ===== Room State =====

export interface RoomState {
	roomId: string;
	phase: SignalingConnectionPhase;
	roomPeers: SignalingPeerInfo[];
	peerConnectionStates: Record<string, PeerConnectionStatus>;
}

// ===== Service State =====

export interface SignalingChatState {
	rooms: Record<string, RoomState>;
	localPeerId: string | null;
	peerIds: string[];
	activePeerId: string | null;
	activeRoomId: string | null;
	messages: SignalingChatMessage[];
	error: string | null;
}

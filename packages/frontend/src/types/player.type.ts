import type { ID } from '$types/core.type';

// ===== Playable File =====

export interface PlayableFile {
	id: string;
	type: 'youtube' | 'torrent' | 'library';
	name: string;
	outputPath: string;
	mode: 'audio' | 'video';
	format: string | null;
	videoFormat: string | null;
	thumbnailUrl: string | null;
	durationSeconds: number | null;
	size: number;
	completedAt: string;
}

// ===== Player State =====

export type PlayerConnectionState =
	| 'idle'
	| 'connecting'
	| 'signaling'
	| 'streaming'
	| 'error'
	| 'closed';

export interface PlayerState {
	initialized: boolean;
	loading: boolean;
	error: string | null;
	files: PlayableFile[];
	currentFile: PlayableFile | null;
	connectionState: PlayerConnectionState;
	streamServerAvailable: boolean;
	sessionId: string | null;
	positionSecs: number;
	durationSecs: number | null;
	isSeeking: boolean;
	isPaused: boolean;
}

// ===== Player Settings (localStorage) =====

export interface PlayerSettings {
	id: ID;
	preferredVolume: number;
	autoplay: boolean;
}

// ===== Signaling Messages (mirrors Rust's SignalingMessage enum) =====
// Rust uses #[serde(tag = "type", content = "payload")] adjacently tagged format

export type SdpType = 'offer' | 'answer';

export interface SessionDescription {
	sdp_type: SdpType;
	sdp: string;
}

export interface IceCandidate {
	sdp_m_line_index: number;
	candidate: string;
}

export interface SeekCommand {
	position_secs: number;
}

export interface MediaInfoPayload {
	duration_secs: number | null;
}

export interface PositionPayload {
	position_secs: number;
	duration_secs: number | null;
}

export type SignalingMessage =
	| { type: 'SessionDescription'; payload: SessionDescription }
	| { type: 'IceCandidate'; payload: IceCandidate }
	| { type: 'IceGatheringComplete' }
	| { type: 'PeerDisconnected'; payload: { peer_id: string } }
	| { type: 'Seek'; payload: SeekCommand }
	| { type: 'MediaInfo'; payload: MediaInfoPayload }
	| { type: 'PositionUpdate'; payload: PositionPayload };

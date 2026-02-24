import type { ID } from '$types/core.type';

// ===== Persisted (localStorage) =====

export interface SignalingServer {
	id: ID;
	name: string;
	url: string;
	addedAt: string;
}

// ===== Runtime State =====

export interface ServerStatus {
	online: boolean;
	totalPeers: number;
	rooms: { id: string; peerCount: number }[];
	checking: boolean;
	lastChecked: string | null;
	error: string | null;
}

export interface SignalingState {
	initialized: boolean;
	showAddForm: boolean;
	serverStatuses: Record<string, ServerStatus>;
}

// ===== HTTP Status Response (mirrors packages/signaling/src/types.ts) =====

export interface SignalingStatusResponse {
	rooms: { id: string; peerCount: number }[];
	totalPeers: number;
}

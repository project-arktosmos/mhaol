export type RosterPeerStatus = 'online' | 'checking' | 'offline';

export interface RosterEntry {
	name: string;
	address: string;
	status: RosterPeerStatus;
}

export interface RosterState {
	loading: boolean;
	entries: RosterEntry[];
	signalingServerUrl: string;
	signalingRoomId: string;
	error: string | null;
}

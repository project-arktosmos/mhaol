export type RosterPeerStatus = 'online' | 'checking' | 'offline';

export type RosterStorageMode = 'local' | 'api';

export interface RosterEntry {
	name: string;
	address: string;
	status: RosterPeerStatus;
	passport?: string;
	instanceType?: string;
	endorsement?: string;
}

export interface RosterState {
	loading: boolean;
	entries: RosterEntry[];
	signalingServerUrl: string;
	signalingRoomId: string;
	error: string | null;
}

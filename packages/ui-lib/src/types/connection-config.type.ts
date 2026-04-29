export type TransportMode = 'ws';

export interface ConnectionConfig {
	transportMode: TransportMode;
	serverUrl: string;
	signalingUrl: string;
}

export type TransportMode = 'http' | 'webrtc' | 'ws';

export interface ConnectionConfig {
	transportMode: TransportMode;
	serverUrl: string;
	serverAddress: string;
	signalingUrl: string;
}

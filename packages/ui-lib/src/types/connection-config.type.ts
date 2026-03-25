export type TransportMode = 'http' | 'webrtc';

export interface ConnectionConfig {
	transportMode: TransportMode;
	serverUrl: string;
	serverAddress: string;
	signalingUrl: string;
}

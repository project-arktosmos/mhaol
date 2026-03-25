import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
import type { ConnectionConfig, TransportMode } from 'ui-lib/types/connection-config.type';

export function buildConnectUrl(config: ConnectionConfig): string {
	const url = new URL(window.location.href);
	// Clear any existing connect params
	url.searchParams.delete('connect');
	url.searchParams.delete('serverUrl');
	url.searchParams.delete('serverAddress');
	url.searchParams.delete('signalingUrl');

	url.searchParams.set('connect', config.transportMode);

	if (config.transportMode === 'http') {
		url.searchParams.set('serverUrl', config.serverUrl);
	} else {
		url.searchParams.set('serverAddress', config.serverAddress);
		if (config.signalingUrl && config.signalingUrl !== DEFAULT_SIGNALING_URL) {
			url.searchParams.set('signalingUrl', config.signalingUrl);
		}
	}

	return url.toString();
}

export function parseConnectUrl(): ConnectionConfig | null {
	if (typeof window === 'undefined') return null;

	const params = new URLSearchParams(window.location.search);
	const mode = params.get('connect') as TransportMode | null;
	if (!mode || (mode !== 'http' && mode !== 'webrtc')) return null;

	const config: ConnectionConfig = {
		transportMode: mode,
		serverUrl: params.get('serverUrl') ?? '',
		serverAddress: params.get('serverAddress') ?? '',
		signalingUrl: params.get('signalingUrl') ?? DEFAULT_SIGNALING_URL
	};

	// Validate that the required fields are present
	if (mode === 'http' && !config.serverUrl) return null;
	if (mode === 'webrtc' && !config.serverAddress) return null;

	return config;
}

export function clearConnectParams(): void {
	const url = new URL(window.location.href);
	url.searchParams.delete('connect');
	url.searchParams.delete('serverUrl');
	url.searchParams.delete('serverAddress');
	url.searchParams.delete('signalingUrl');
	window.history.replaceState({}, '', url.toString());
}

import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
import type { ConnectionConfig, TransportMode } from 'ui-lib/types/connection-config.type';

interface InviteHttp {
	transport: 'http' | 'ws';
	serverUrl: string;
}

interface InviteWebRtc {
	transport: 'webrtc';
	serverAddress: string;
	signalingUrl?: string;
}

type Invite = InviteHttp | InviteWebRtc;

const VALID_TRANSPORTS: TransportMode[] = ['http', 'ws', 'webrtc'];

export function buildInvite(config: ConnectionConfig): string {
	let invite: Invite;

	if (config.transportMode === 'http' || config.transportMode === 'ws') {
		invite = {
			transport: config.transportMode,
			serverUrl: config.serverUrl
		};
	} else {
		invite = {
			transport: 'webrtc',
			serverAddress: config.serverAddress,
			...(config.signalingUrl &&
				config.signalingUrl !== DEFAULT_SIGNALING_URL && {
					signalingUrl: config.signalingUrl
				})
		};
	}

	return JSON.stringify(invite);
}

export function parseInvite(json: string): ConnectionConfig | null {
	try {
		const data = JSON.parse(json);
		if (!data || typeof data !== 'object') return null;

		const transport = data.transport as TransportMode;
		if (!VALID_TRANSPORTS.includes(transport)) return null;

		if (transport === 'http' || transport === 'ws') {
			if (!data.serverUrl || typeof data.serverUrl !== 'string') return null;
			return {
				transportMode: transport,
				serverUrl: data.serverUrl,
				serverAddress: '',
				signalingUrl: DEFAULT_SIGNALING_URL
			};
		}

		if (transport === 'webrtc') {
			if (!data.serverAddress || typeof data.serverAddress !== 'string') return null;
			return {
				transportMode: 'webrtc',
				serverUrl: '',
				serverAddress: data.serverAddress,
				signalingUrl:
					typeof data.signalingUrl === 'string' ? data.signalingUrl : DEFAULT_SIGNALING_URL
			};
		}

		return null;
	} catch {
		return null;
	}
}

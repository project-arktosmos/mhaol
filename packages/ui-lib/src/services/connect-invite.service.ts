import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
import type { ConnectionConfig } from 'ui-lib/types/connection-config.type';

interface Invite {
	transport: 'ws';
	serverUrl: string;
}

export function buildInvite(config: ConnectionConfig): string {
	const invite: Invite = {
		transport: 'ws',
		serverUrl: config.serverUrl
	};
	return JSON.stringify(invite);
}

export function extractInviteFromUrl(): string | null {
	try {
		const params = new URLSearchParams(window.location.search);
		const encoded = params.get('invite');
		if (!encoded) return null;
		return atob(encoded);
	} catch {
		return null;
	}
}

export function clearInviteFromUrl(): void {
	const url = new URL(window.location.href);
	if (!url.searchParams.has('invite')) return;
	url.searchParams.delete('invite');
	history.replaceState(history.state, '', url.toString());
}

export function parseInvite(json: string): ConnectionConfig | null {
	try {
		const data = JSON.parse(json);
		if (!data || typeof data !== 'object') return null;
		if (data.transport !== 'ws') return null;
		if (!data.serverUrl || typeof data.serverUrl !== 'string') return null;
		return {
			transportMode: 'ws',
			serverUrl: data.serverUrl,
			signalingUrl: DEFAULT_SIGNALING_URL
		};
	} catch {
		return null;
	}
}

import { AdapterClass } from 'ui-lib/adapters/classes/adapter.class';
import type {
	PeerConnectionStatus,
	SignalingChatMessage,
	SignalingConnectionPhase
} from 'ui-lib/types/signaling.type';
import type { PlayerConnectionState } from 'ui-lib/types/player.type';

export class SignalingAdapter extends AdapterClass {
	constructor() {
		super('signaling');
	}

	shortAddress(address: string): string {
		if (!address) return '';
		if (!address.startsWith('0x') || address.length < 10) return address;
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	}

	formatTimestamp(iso: string): string {
		return new Date(iso).toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	phaseLabel(phase: SignalingConnectionPhase): string {
		const labels: Record<SignalingConnectionPhase, string> = {
			disconnected: 'Disconnected',
			connecting: 'Connecting...',
			authenticated: 'Authenticated',
			connected: 'Connected',
			error: 'Error'
		};
		return labels[phase];
	}

	phaseBadgeClass(phase: SignalingConnectionPhase): string {
		const map: Record<SignalingConnectionPhase, string> = {
			disconnected: 'badge-ghost',
			connecting: 'badge-info',
			authenticated: 'badge-info',
			connected: 'badge-success',
			error: 'badge-error'
		};
		return map[phase];
	}

	createMessage(address: string, content: string): SignalingChatMessage {
		return {
			id:
				typeof crypto.randomUUID === 'function'
					? crypto.randomUUID()
					: Math.random().toString(36).slice(2) + Date.now().toString(36),
			address,
			content,
			timestamp: new Date().toISOString()
		};
	}

	playerConnectionLabel(state: PlayerConnectionState): string {
		const labels: Record<PlayerConnectionState, string> = {
			idle: 'Idle',
			connecting: 'Connecting...',
			signaling: 'Signaling...',
			streaming: 'Streaming',
			error: 'Error',
			closed: 'Closed'
		};
		return labels[state];
	}

	playerConnectionBadgeClass(state: PlayerConnectionState): string {
		const map: Record<PlayerConnectionState, string> = {
			idle: 'badge-ghost',
			connecting: 'badge-info',
			signaling: 'badge-info',
			streaming: 'badge-success',
			error: 'badge-error',
			closed: 'badge-ghost'
		};
		return map[state];
	}

	peerConnectionStatusLabel(status: PeerConnectionStatus): string {
		const labels: Record<PeerConnectionStatus, string> = {
			idle: 'Idle',
			offering: 'Offering...',
			answering: 'Answering...',
			connected: 'Connected',
			failed: 'Failed'
		};
		return labels[status];
	}

	peerConnectionStatusBadgeClass(status: PeerConnectionStatus): string {
		const map: Record<PeerConnectionStatus, string> = {
			idle: 'badge-ghost',
			offering: 'badge-info',
			answering: 'badge-info',
			connected: 'badge-success',
			failed: 'badge-error'
		};
		return map[status];
	}

	resolveLocalUrl(url: string): string {
		try {
			const parsed = new URL(url);
			const local = ['127.0.0.1', 'localhost', '0.0.0.0'];
			if (local.includes(parsed.hostname)) {
				parsed.hostname = window.location.hostname;
				parsed.port = window.location.port;
				parsed.protocol = window.location.protocol;
			}
			return parsed.toString().replace(/\/$/, '');
		} catch {
			return url;
		}
	}

	buildWsUrl(baseUrl: string, roomId: string): string {
		const url = new URL(baseUrl);
		const wsProtocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
		return `${wsProtocol}//${url.host}/party/${roomId}`;
	}
}

export const signalingAdapter = new SignalingAdapter();

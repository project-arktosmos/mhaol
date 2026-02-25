import { AdapterClass } from '$adapters/classes/adapter.class';
import type { P2pChatMessage, P2pConnectionPhase } from '$types/p2p.type';

export class P2pAdapter extends AdapterClass {
	constructor() {
		super('p2p');
	}

	shortAddress(address: string): string {
		if (!address.startsWith('0x') || address.length < 10) return address;
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	}

	formatTimestamp(iso: string): string {
		return new Date(iso).toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	phaseLabel(phase: P2pConnectionPhase): string {
		const labels: Record<P2pConnectionPhase, string> = {
			idle: 'Not connected',
			'creating-offer': 'Creating offer...',
			'waiting-answer': 'Waiting for answer',
			'accepting-offer': 'Processing offer...',
			'answer-ready': 'Answer ready - share it',
			connecting: 'Connecting...',
			connected: 'Connected',
			disconnected: 'Disconnected',
			error: 'Error'
		};
		return labels[phase];
	}

	phaseBadgeClass(phase: P2pConnectionPhase): string {
		const map: Record<P2pConnectionPhase, string> = {
			idle: 'badge-ghost',
			'creating-offer': 'badge-info',
			'waiting-answer': 'badge-warning',
			'accepting-offer': 'badge-info',
			'answer-ready': 'badge-warning',
			connecting: 'badge-info',
			connected: 'badge-success',
			disconnected: 'badge-ghost',
			error: 'badge-error'
		};
		return map[phase];
	}

	createMessage(address: string, content: string): P2pChatMessage {
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
}

export const p2pAdapter = new P2pAdapter();

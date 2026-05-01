import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { setApiBase } from '$lib/api-base';
import { clearTransport, setTransport } from '$transport/transport-context';
import { WsTransport } from '$transport/ws-transport';
import { clientIdentityService } from '$services/client-identity.service';
import type { ConnectionConfig } from '$types/connection-config.type';

export type NodeConnectionPhase = 'idle' | 'connecting' | 'authenticating' | 'ready' | 'error';

export interface NodeConnectionState {
	phase: NodeConnectionPhase;
	error: string | null;
}

const initialState: NodeConnectionState = {
	phase: 'idle',
	error: null
};

class NodeConnectionService {
	state: Writable<NodeConnectionState> = writable(initialState);

	private wsTransport: WsTransport | null = null;

	async connectWs(config: ConnectionConfig): Promise<void> {
		if (!browser) return;

		this.state.set({ phase: 'connecting', error: null });

		try {
			await clientIdentityService.initialize(config.signalingUrl);
			const identityState = get(clientIdentityService.state);
			if (!identityState.identity) {
				throw new Error(identityState.error ?? 'Failed to initialize identity');
			}

			const { identity } = identityState;

			this.state.set({ phase: 'authenticating', error: null });
			const timestamp = String(Date.now());
			const message = `mhaol-rpc-auth:${timestamp}`;
			const signature = await clientIdentityService.signMessage(message);

			const params = new URLSearchParams({
				address: identity.address,
				signature,
				timestamp
			});

			const wsUrl = config.serverUrl.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:');

			const ws = new WebSocket(`${wsUrl}/api/rpc?${params.toString()}`);

			await new Promise<void>((resolve, reject) => {
				const timeout = setTimeout(() => {
					ws.close();
					reject(new Error('WebSocket connection timed out'));
				}, 30_000);

				ws.onopen = () => {
					clearTimeout(timeout);
					resolve();
				};
				ws.onerror = () => {
					clearTimeout(timeout);
					reject(new Error('WebSocket connection failed'));
				};
			});

			setApiBase(config.serverUrl);
			const transport = new WsTransport(ws, config.serverUrl);
			this.wsTransport = transport;
			setTransport(transport);

			this.state.set({ phase: 'ready', error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Connection failed';
			this.state.set({ phase: 'error', error: message });
			throw err;
		}
	}

	disconnect(): void {
		if (this.wsTransport) {
			this.wsTransport.destroy();
			this.wsTransport = null;
		}

		clearTransport();
		this.state.set(initialState);
	}
}

export const nodeConnectionService = new NodeConnectionService();

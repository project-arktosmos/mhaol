import { get } from 'svelte/store';
import localStorageWritableStore from 'ui-lib/utils/localStorageWritableStore';
import type { ConnectionConfig } from 'ui-lib/types/connection-config.type';
import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
import { isTauri, isMobile } from 'ui-lib/lib/platform';

const STORE_KEY = 'connection-config';

export interface NodeDefaults {
	serverUrl: string;
	serverAddress: string;
	signalingUrl: string;
	port: number;
}

class ConnectionConfigService {
	store = localStorageWritableStore<ConnectionConfig | null>(STORE_KEY, null);

	private _nodeDefaults: NodeDefaults | null = null;

	isConfigured(): boolean {
		return get(this.store) !== null;
	}

	get(): ConnectionConfig | null {
		return get(this.store);
	}

	save(config: ConnectionConfig): void {
		this.store.set(config);

		if (config.transportMode === 'http') {
			localStorage.setItem('api-server-url', config.serverUrl);
		} else {
			localStorage.setItem('signaling-url', config.signalingUrl);
		}
	}

	clear(): void {
		this.store.set(null);
		localStorage.removeItem('api-server-url');
		localStorage.removeItem('signaling-url');
	}

	defaults() {
		if (this._nodeDefaults && this._nodeDefaults.serverUrl) {
			return {
				serverUrl: this._nodeDefaults.serverUrl,
				serverAddress: this._nodeDefaults.serverAddress,
				signalingUrl: this._nodeDefaults.signalingUrl || DEFAULT_SIGNALING_URL
			};
		}

		// Mobile Tauri: dev via ADB (hostname=localhost) defaults to localhost
		// (ADB reverse forwards to host). Production leaves empty for LAN IP entry.
		if (isTauri && isMobile) {
			const host = typeof window !== 'undefined' ? window.location.hostname : '';
			return {
				serverUrl: host === 'localhost' ? 'http://localhost:1530' : '',
				serverAddress: '',
				signalingUrl: DEFAULT_SIGNALING_URL
			};
		}
		const host =
			typeof window !== 'undefined' ? window.location.hostname : '127.0.0.1';
		return {
			serverUrl: `http://${host}:1530`,
			serverAddress: '',
			signalingUrl: DEFAULT_SIGNALING_URL
		};
	}

	async loadNodeDefaults(): Promise<NodeDefaults | null> {
		if (typeof window === 'undefined') return null;

		try {
			const res = await globalThis.fetch('/node-defaults.json', {
				signal: AbortSignal.timeout(3000)
			});
			if (!res.ok) return null;
			const data: NodeDefaults = await res.json();
			if (data.serverUrl || data.serverAddress) {
				this._nodeDefaults = data;
				return data;
			}
		} catch {
			// File not available
		}
		return null;
	}
}

export const connectionConfigService = new ConnectionConfigService();

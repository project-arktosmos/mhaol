import { browser } from '$app/environment';
import { ArrayServiceClass } from 'ui-lib/services/classes/array-service.class';
import { connectionConfigService } from 'ui-lib/services/connection-config.service';
import { nodeConnectionService } from 'ui-lib/services/node-connection.service';
import type { CloudServer } from 'ui-lib/types/cloud-server.type';
import type { ConnectionConfig } from 'ui-lib/types/connection-config.type';

function cloudId(config: ConnectionConfig): string {
	const key =
		config.transportMode === 'webrtc'
			? config.serverAddress.toLowerCase()
			: config.serverUrl.toLowerCase();
	return `${config.transportMode}:${key}`;
}

function defaultName(config: ConnectionConfig): string {
	if (config.transportMode === 'webrtc') {
		const addr = config.serverAddress;
		return addr ? `${addr.slice(0, 6)}…${addr.slice(-4)}` : 'WebRTC cloud';
	}
	try {
		return new URL(config.serverUrl).host;
	} catch {
		return config.serverUrl || 'Cloud';
	}
}

class CloudsService extends ArrayServiceClass<CloudServer> {
	constructor() {
		super('clouds', []);
		if (browser) {
			nodeConnectionService.state.subscribe((s) => {
				if (s.phase !== 'ready') return;
				const config = connectionConfigService.get();
				if (config) this.upsertFromConfig(config);
			});
		}
	}

	upsertFromConfig(config: ConnectionConfig): CloudServer {
		const id = cloudId(config);
		const existing = this.exists(id);
		const now = Date.now();
		if (existing) {
			const updated: CloudServer = {
				...existing,
				transportMode: config.transportMode,
				serverUrl: config.serverUrl,
				serverAddress: config.serverAddress,
				signalingUrl: config.signalingUrl,
				lastConnectedAt: now
			};
			this.update(updated);
			return updated;
		}
		const created: CloudServer = {
			id,
			name: defaultName(config),
			transportMode: config.transportMode,
			serverUrl: config.serverUrl,
			serverAddress: config.serverAddress,
			signalingUrl: config.signalingUrl,
			lastConnectedAt: now
		};
		this.add(created);
		return created;
	}
}

export const cloudsService = new CloudsService();

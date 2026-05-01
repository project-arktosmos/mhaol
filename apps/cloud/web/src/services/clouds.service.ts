import { browser } from '$app/environment';
import { ArrayServiceClass } from '$services/classes/array-service.class';
import { connectionConfigService } from '$services/connection-config.service';
import { nodeConnectionService } from '$services/node-connection.service';
import type { CloudServer } from '$types/cloud-server.type';
import type { ConnectionConfig } from '$types/connection-config.type';

function cloudId(config: ConnectionConfig): string {
	return `ws:${config.serverUrl.toLowerCase()}`;
}

function defaultName(config: ConnectionConfig): string {
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
			signalingUrl: config.signalingUrl,
			lastConnectedAt: now
		};
		this.add(created);
		return created;
	}
}

export const cloudsService = new CloudsService();
